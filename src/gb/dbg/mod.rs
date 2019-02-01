use std::fmt;
use std::ops::RangeInclusive;

#[derive(Debug, Clone, Copy)]
pub enum Peripheral {
    VPU,
}

#[derive(Debug, Clone, Copy)]
pub enum MemoryType {
    RomBank(u8),
    VideoRam,
    ExternalRam,
    WorkRamBank(u8),
    EchoRam(u8),
    SpriteMemory,
    IoSpace,
    HighRam,
    NotUsable,
}

impl MemoryType {
    pub fn range(self) -> RangeInclusive<u16> {
        use MemoryType::*;

        match self {
            RomBank(0) => 0x0000..=0x3FFF,
            RomBank(_) => 0x4000..=0x7FFF,
            VideoRam => 0x8000..=0x9FFF,
            ExternalRam => 0xA000..=0xBFFF,
            WorkRamBank(0) => 0xC000..=0xCFFF,
            WorkRamBank(_) => 0xD000..=0xDFFF,
            EchoRam(0) => 0xE000..=0xEFFF,
            EchoRam(_) => 0xF000..=0xFDFF,
            SpriteMemory => 0xFE00..=0xFE9F,
            NotUsable => 0xFEA0..=0xFEFF,
            IoSpace => 0xFF00..=0xFF7F,
            HighRam => 0xFF80..=0xFFFE,
        }
    }

    pub fn at(addr: u16) -> MemoryType {
        use MemoryType::*;

        match addr {
            0x0000..=0x3FFF => RomBank(0),
            0x4000..=0x7FFF => RomBank(0xFF),
            0x8000..=0x9FFF => VideoRam,
            0xA000..=0xBFFF => ExternalRam,
            0xC000..=0xCFFF => WorkRamBank(0),
            0xD000..=0xDFFF => WorkRamBank(0xFF),
            0xE000..=0xEFFF => EchoRam(0),
            0xF000..=0xFDFF => EchoRam(0xFF),
            0xFE00..=0xFE9F => SpriteMemory,
            0xFF00..=0xFF7F => IoSpace,
            0xFF80..=0xFFFE => HighRam,
            _ => NotUsable,
        }
    }
}

impl fmt::Display for MemoryType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use MemoryType::*;

        match self {
            RomBank(n) => write!(f, "ROM{:02X}", n),
            VideoRam => write!(f, "VRAM"),
            ExternalRam => write!(f, "ERAM"),
            WorkRamBank(n) => write!(f, "WRAM{:02X}", n),
            EchoRam(n) => write!(f, "ECHO{:02X}", n),
            SpriteMemory => write!(f, "Sprite memory"),
            IoSpace => write!(f, "IO space"),
            HighRam => write!(f, "HRAM"),
            NotUsable => write!(f, "Not usable"),
        }
    }
}

#[derive(Debug, Fail, Clone, Copy)]
pub enum TraceEvent {
    #[fail(display = "Breakpoint reached: 0x{:04X}", _0)]
    Breakpoint(u16),
    #[fail(display = "Illegal opcode: {:02X}", _0)]
    IllegalInstructionFault(u8),
    #[fail(display = "Bus fault accessing 0x{:04X}", _0)]
    BusFault(u16),
    #[fail(display = "Memory fault accessing 0x{:04X}", _0)]
    MemFault(u16),
    #[fail(display = "IO fault accessing {:?}@{:04X}", _0, _1)]
    IoFault(Peripheral, u16),
}
