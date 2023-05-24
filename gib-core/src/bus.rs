use std::{convert::TryFrom, mem};

use dbg::{McbOp, TraceEvent};

use crate::{
    dbg,
    io::{InterruptSource, IrqController, Joypad, Serial, Timer, APU, PPU},
    mem::{MemR, MemRW, MemW, Memory},
};

// Specifies which Memory Bank Controller (if any) is used in the cartridge.
#[derive(Debug)]
pub enum MbcType {
    None,
    Mbc1,
    Mbc3,
}

// The error type returned when parsing the MBC type code fails.
#[derive(Debug)]
pub struct McbTypeError(u8);

impl TryFrom<u8> for MbcType {
    type Error = McbTypeError;

    fn try_from(n: u8) -> Result<Self, Self::Error> {
        match n {
            0x00 => Ok(MbcType::None),
            0x01..=0x03 => Ok(MbcType::Mbc1),
            0x0f..=0x13 => Ok(MbcType::Mbc3),
            _ => Err(McbTypeError(n)),
        }
    }
}

// Specifies the ROM size of the cartridge in 16KB banks.
#[derive(Debug)]
pub struct RomBanks(usize);

// The error type returned when a parsing a ROM size code fails.
#[derive(Debug)]
pub struct RomSizeError(u8);

impl TryFrom<u8> for RomBanks {
    type Error = RomSizeError;

    fn try_from(n: u8) -> Result<Self, Self::Error> {
        match n {
            0x00 => Ok(RomBanks(2)),   //  32KByte (no ROM banking)
            0x01 => Ok(RomBanks(4)),   //  64KByte (4 banks)
            0x02 => Ok(RomBanks(8)),   // 128KByte (8 banks)
            0x03 => Ok(RomBanks(16)),  // 256KByte (16 banks)
            0x04 => Ok(RomBanks(32)),  // 512KByte (32 banks)
            0x05 => Ok(RomBanks(64)),  //   1MByte (64 banks)  - only 63 banks used by MBC1
            0x06 => Ok(RomBanks(128)), //   2MByte (128 banks) - only 125 banks used by MBC1
            0x07 => Ok(RomBanks(256)), //   4MByte (256 banks)
            0x08 => Ok(RomBanks(512)), //   8MByte (512 banks)
            0x52 => Ok(RomBanks(72)),  // 1.1MByte (72 banks)
            0x53 => Ok(RomBanks(82)),  // 1.2MByte (80 banks)
            0x54 => Ok(RomBanks(92)),  // 1.5MByte (96 banks)
            _ => Err(RomSizeError(n)),
        }
    }
}

// Specifies the size of the external RAM in the cartridge in 8KB banks.
#[derive(Debug)]
pub struct RamBanks(usize);

// The error type returned when a parsing a RAM size code fails.
#[derive(Debug)]
pub struct RamSizeError(u8);

impl TryFrom<u8> for RamBanks {
    type Error = RamSizeError;

    fn try_from(n: u8) -> Result<Self, Self::Error> {
        match n {
            0x00 => Ok(RamBanks(0)),  // 00h - None
            0x01 => Ok(RamBanks(1)),  // 01h - 2 KBytes
            0x02 => Ok(RamBanks(1)),  // 02h - 8 Kbytes
            0x03 => Ok(RamBanks(4)),  // 03h - 32 KBytes (4 banks of 8KBytes each)
            0x04 => Ok(RamBanks(16)), // 04h - 128 KBytes (16 banks of 8KBytes each)
            0x05 => Ok(RamBanks(8)),  // 05h - 64 KBytes (8 banks of 8KBytes each)
            _ => Err(RamSizeError(n)),
        }
    }
}

pub struct Bus {
    rom_banks: Vec<Memory>,
    pub rom_nn: usize,

    ram_banks: Vec<Memory>,
    pub ram_nn: usize,

    pub hram: Memory,
    pub wram_00: Memory,
    pub wram_nn: Memory,

    pub apu: APU,
    pub ppu: PPU,
    pub tim: Timer,
    pub sdt: Serial,
    pub joy: Joypad,
    pub itr: IrqController,

    mbc: MbcType,
}

impl Default for Bus {
    fn default() -> Bus {
        Bus {
            rom_banks: vec![Memory::new(0x4000); 512],
            rom_nn: 1,

            ram_banks: vec![Memory::new(0x2000); 16],
            ram_nn: 0,

            hram: Memory::new(127),
            wram_00: Memory::new(0x1000),
            wram_nn: Memory::new(0x1000),

            apu: APU::default(),
            ppu: PPU::new(),
            tim: Timer::new(),
            sdt: Serial::new(),
            joy: Joypad::new(),
            itr: IrqController::new(),

            mbc: MbcType::None,
        }
    }
}

impl Bus {
    pub fn new() -> Bus {
        Bus::default()
    }

    /// Resets the internal bus to its power-up state.
    ///
    /// This includes resetting all the connected peripherals and clearning RAM contents.
    /// The contents of the whole ROM are preserved.
    pub fn reset(&mut self) {
        // Preserve ROM contents
        let rom_banks = mem::take(&mut self.rom_banks);

        // Reset the APU to keep sample rate and audio channel intact, the rest can be recreated
        let mut apu = mem::take(&mut self.apu);
        apu.reset();

        *self = Self {
            rom_banks,
            apu,
            ..Default::default()
        };
    }

    pub fn load_rom(&mut self, rom: &[u8]) -> Result<(), TraceEvent> {
        // Filter out ROMs using unsupported emulator features (eg. CGB-only mode)
        if rom[0x143] == 0xC0 {
            return Err(TraceEvent::CgbNotSupported);
        }

        // Check MBC type in the ROM header
        self.mbc = MbcType::try_from(rom[0x147])
            .map_err(|McbTypeError(n)| TraceEvent::UnsupportedMbcType(n))?;

        // Load ROM into its allocated banks
        for (n, chunk) in rom.chunks(0x4000).enumerate() {
            for (i, b) in chunk.iter().enumerate() {
                self.rom_banks[n].write(i as u16, *b)?;
            }
        }

        Ok(())
    }

    /// Advances the system peripheral/memory bus by a single M-cycle.
    pub fn tick(&mut self) -> Result<(), TraceEvent> {
        if let Some((src, dst)) = self.ppu.advance_dma_xfer() {
            let b = self.read(src)?;
            self.ppu.write_to_oam(dst, b)?;
        }

        self.ppu.tick();
        self.apu.tick();
        self.tim.tick();

        // Fetch interrupt requests from interrupt sources
        if let Some(irq) = self.ppu.get_and_clear_irq() {
            self.itr.set_irq(irq.into());
        }
        if let Some(irq) = self.tim.get_and_clear_irq() {
            self.itr.set_irq(irq.into());
        }
        if let Some(irq) = self.apu.get_and_clear_irq() {
            self.itr.set_irq(irq.into());
        }
        if let Some(irq) = self.sdt.get_and_clear_irq() {
            self.itr.set_irq(irq.into());
        }

        Ok(())
    }

    fn ram_enable(&mut self, _val: u8) -> Result<(), TraceEvent> {
        // TODO handle this just in case some ROMs rely on uncorrect behavior
        Ok(())
    }

    fn rom_select(&mut self, val: u8) -> Result<(), TraceEvent> {
        self.rom_nn = if val == 0 {
            1
        } else {
            // TODO is this remainder here the correct way of handling bank number overflow?
            // Some ROMs (eg. blargg's dmg_sound-2) seem to rely on this behavior.
            usize::from(val) % self.rom_banks.len()
        };
        Ok(())
    }

    fn ram_rom_select(&mut self, addr: u16, val: u8) -> Result<(), TraceEvent> {
        match val {
            0x00..=0x03 => self.ram_nn = val.into(),
            _ => return Err(TraceEvent::InvalidMbcOp(McbOp::Write(addr), val)),
        };
        Ok(())
    }

    fn mbc_write_op(&mut self, addr: u16, val: u8) -> Result<(), TraceEvent> {
        match self.mbc {
            MbcType::Mbc3 => {
                // TODO latch RTC register value
                Ok(())
            }
            _ => Err(TraceEvent::InvalidMbcOp(McbOp::Write(addr), val)),
        }
    }

    fn write_to_cgb_functions(&mut self, addr: u16, _val: u8) -> Result<(), TraceEvent> {
        match addr {
            0xFF4D => Err(TraceEvent::CgbSpeedSwitchReq),
            _ => Ok(()),
        }
    }
}

impl MemR for Bus {
    fn read(&self, addr: u16) -> Result<u8, TraceEvent> {
        match addr {
            0x0000..=0x3FFF => self.rom_banks[0].read(addr),
            0x4000..=0x7FFF => self.rom_banks[self.rom_nn].read(addr - 0x4000),
            0x8000..=0x9FFF => self.ppu.read(addr),
            0xA000..=0xBFFF => self.ram_banks[self.ram_nn].read(addr - 0xA000),
            0xC000..=0xCFFF => self.wram_00.read(addr - 0xC000),
            0xD000..=0xDFFF => self.wram_nn.read(addr - 0xD000),
            0xE000..=0xEFFF => self.wram_00.read(addr - 0xE000),
            0xF000..=0xFDFF => self.wram_nn.read(addr - 0xF000),
            0xFE00..=0xFE9F => self.ppu.read(addr),
            0xFF00..=0xFF00 => self.joy.read(addr),
            0xFF01..=0xFF02 => self.sdt.read(addr),
            0xFF04..=0xFF07 => self.tim.read(addr),
            0xFF10..=0xFF3F => self.apu.read(addr),
            0xFF40..=0xFF4B => self.ppu.read(addr),
            0xFF80..=0xFFFE => self.hram.read(addr - 0xFF80),
            0xFF0F | 0xFFFF => self.itr.read(addr),
            _ => Ok(0xFF),
        }
    }
}

impl MemW for Bus {
    fn write(&mut self, addr: u16, val: u8) -> Result<(), TraceEvent> {
        match addr {
            0x0000..=0x1FFF => self.ram_enable(val),
            0x2000..=0x3FFF => self.rom_select(val),
            0x4000..=0x5FFF => self.ram_rom_select(addr, val),
            0x6000..=0x7FFF => self.mbc_write_op(addr, val),
            0x8000..=0x9FFF => self.ppu.write(addr, val),
            0xA000..=0xBFFF => self
                .ram_banks
                .get_mut(self.ram_nn)
                .map_or(Ok(()), |bank| bank.write(addr - 0xA000, val)),
            0xC000..=0xCFFF => self.wram_00.write(addr - 0xC000, val),
            0xD000..=0xDFFF => self.wram_nn.write(addr - 0xD000, val),
            0xE000..=0xEFFF => self.wram_00.write(addr - 0xE000, val),
            0xF000..=0xFDFF => self.wram_nn.write(addr - 0xF000, val),
            0xFE00..=0xFE9F => self.ppu.write(addr, val),
            0xFF00..=0xFF00 => self.joy.write(addr, val),
            0xFF01..=0xFF02 => self.sdt.write(addr, val),
            0xFF04..=0xFF07 => self.tim.write(addr, val),
            0xFF10..=0xFF3F => self.apu.write(addr, val),
            0xFF40..=0xFF4B => self.ppu.write(addr, val),
            0xFF4C..=0xFF4F => self.write_to_cgb_functions(addr, val),
            0xFF51..=0xFF7F => self.write_to_cgb_functions(addr, val),
            0xFF80..=0xFFFE => self.hram.write(addr - 0xFF80, val),
            0xFF0F | 0xFFFF => self.itr.write(addr, val),
            _ => Ok(()),
        }
    }
}

impl MemRW for Bus {}
