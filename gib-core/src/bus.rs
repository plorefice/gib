use super::dbg;
use super::io::{IrqController, Joypad, Serial, Timer, APU, PPU};
use super::mem::{MemR, MemRW, MemW, Memory};

use std::convert::TryFrom;

pub enum MbcType {
    None,
    MBC1,
}

pub struct McbTypeError(u8);

impl TryFrom<u8> for MbcType {
    type Error = McbTypeError;

    fn try_from(n: u8) -> Result<Self, Self::Error> {
        match n {
            0x00 => Ok(MbcType::None),
            0x01..=0x03 => Ok(MbcType::MBC1),
            _ => Err(McbTypeError(n)),
        }
    }
}

pub struct Bus {
    rom_banks: Vec<Memory>,
    pub rom_nn: usize,

    pub eram: Memory,
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
            rom_banks: vec![],
            rom_nn: 1,

            eram: Memory::new(0x2000),
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

    pub fn load_rom(&mut self, rom: &[u8]) -> Result<(), dbg::TraceEvent> {
        for chunk in rom.chunks(0x4000) {
            let mut mem = Memory::new(0x4000);

            for (i, b) in chunk.iter().enumerate() {
                mem.write(i as u16, *b)?;
            }
            self.rom_banks.push(mem);
        }

        // Check MBC type in the ROM header
        self.mbc = MbcType::try_from(rom[0x147])
            .map_err(|McbTypeError(n)| dbg::TraceEvent::UnsupportedMbcType(n))?;

        Ok(())
    }

    /// Advances the system peripheral/memory bus by a single M-cycle.
    pub fn tick(&mut self) -> Result<(), dbg::TraceEvent> {
        if let Some((src, dst)) = self.ppu.advance_dma_xfer() {
            let b = self.read(src)?;
            self.ppu.write_to_oam(dst, b)?;
        }

        self.ppu.tick();
        self.apu.tick();
        self.tim.tick();

        Ok(())
    }

    fn ram_enable(&mut self, _val: u8) -> Result<(), dbg::TraceEvent> {
        // TODO handle this just in case some ROMs rely on uncorrect behavior
        Ok(())
    }

    fn rom_select(&mut self, val: u8) -> Result<(), dbg::TraceEvent> {
        self.rom_nn = match val {
            0x00 => 0x01,
            v @ 0x01..=0x1F => usize::from(v),
            v => return Err(dbg::TraceEvent::InvalidMbcOp(dbg::McbOp::RomBank, v)),
        };
        Ok(())
    }

    fn ram_rom_select(&mut self, val: u8) -> Result<(), dbg::TraceEvent> {
        Err(dbg::TraceEvent::InvalidMbcOp(dbg::McbOp::RamBank, val))
    }

    fn mode_select(&mut self, val: u8) -> Result<(), dbg::TraceEvent> {
        Err(dbg::TraceEvent::InvalidMbcOp(dbg::McbOp::RamBank, val))
    }

    fn write_to_cgb_functions(&mut self, addr: u16, _val: u8) -> Result<(), dbg::TraceEvent> {
        match addr {
            0xFF4D => Err(dbg::TraceEvent::CgbSpeedSwitchReq),
            _ => Ok(()),
        }
    }
}

impl MemR for Bus {
    fn read(&self, addr: u16) -> Result<u8, dbg::TraceEvent> {
        match addr {
            0x0000..=0x3FFF => self.rom_banks[0].read(addr),
            0x4000..=0x7FFF => self.rom_banks[self.rom_nn].read(addr - 0x4000),
            0x8000..=0x9FFF => self.ppu.read(addr),
            0xA000..=0xBFFF => self.eram.read(addr - 0xA000),
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
    fn write(&mut self, addr: u16, val: u8) -> Result<(), dbg::TraceEvent> {
        match addr {
            0x0000..=0x1FFF => self.ram_enable(val),
            0x2000..=0x3FFF => self.rom_select(val),
            0x4000..=0x5FFF => self.ram_rom_select(val),
            0x6000..=0x7FFF => self.mode_select(val),
            0x8000..=0x9FFF => self.ppu.write(addr, val),
            0xA000..=0xBFFF => self.eram.write(addr - 0xA000, val),
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
