use std::{fmt, ops::RangeInclusive};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl Default for MemoryType {
    fn default() -> Self {
        MemoryType::RomBank(0)
    }
}

pub struct Iter(Option<MemoryType>);

impl Iterator for Iter {
    type Item = MemoryType;

    fn next(&mut self) -> Option<Self::Item> {
        use MemoryType::*;

        let ret = self.0;

        self.0 = match self.0 {
            None => Some(RomBank(0)),
            Some(m) => match m {
                RomBank(0) => Some(RomBank(1)),
                RomBank(_) => Some(VideoRam),
                VideoRam => Some(ExternalRam),
                ExternalRam => Some(WorkRamBank(0)),
                WorkRamBank(0) => Some(WorkRamBank(1)),
                WorkRamBank(_) => Some(EchoRam(0)),
                EchoRam(0) => Some(EchoRam(1)),
                EchoRam(_) => Some(SpriteMemory),
                SpriteMemory => Some(NotUsable),
                NotUsable => Some(IoSpace),
                IoSpace => Some(HighRam),
                HighRam => None,
            },
        };

        ret
    }
}

impl MemoryType {
    pub fn iter(self) -> Iter {
        Iter(Some(self))
    }

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

#[derive(Debug, Clone, Copy)]
pub enum McbOp {
    RomBank,
    RamBank,
}

impl fmt::Display for McbOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            McbOp::RomBank => write!(f, "ROM bank select"),
            McbOp::RamBank => write!(f, "RAM bank select"),
        }
    }
}

#[derive(thiserror::Error, Debug, Clone, Copy)]
#[allow(unused)]
pub enum TraceEvent {
    #[error("Breakpoint reached: 0x{0:04X}")]
    Breakpoint(u16),
    #[error("Illegal opcode: {0:02X}")]
    IllegalInstructionFault(u8),
    #[error("Bus fault accessing 0x{0:04X}")]
    BusFault(u16),
    #[error("Memory fault accessing 0x{0:04X}")]
    MemFault(u16),
    #[error("Unsupported MBC: {0:02X}")]
    UnsupportedMbcType(u8),
    #[error("Invalid MBC operation: {0}@{1:02X}")]
    InvalidMbcOp(McbOp, u8),
    #[error("CGB speed switch request")]
    CgbSpeedSwitchReq,
    #[error("Unsupported CGB operation: {0:04X}")]
    UnsupportedCgbOp(u16),
    #[error("CGB mode not supported")]
    CgbNotSupported,
}
