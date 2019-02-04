use super::dbg;
use super::mem::MemR;
use super::opcodes::OPCODES;
use super::CPU;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Immediate {
    Imm8(u8),
    Imm16(u16),
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Instruction {
    pub opcode: u8,
    pub mnemonic: &'static str,
    pub imm: Option<Immediate>,
    pub size: u8,
}

impl CPU {
    pub fn disasm(&self, mem: &impl MemR, addr: u16) -> Result<Instruction, dbg::TraceEvent> {
        let opcode = mem.read::<u8>(addr)?;
        let info = &OPCODES[opcode as usize];

        let imm: Option<Immediate> = match info.3 {
            1 => None,
            2 => Some(Immediate::Imm8(mem.read::<u8>(addr + 1)?)),
            3 => Some(Immediate::Imm16(mem.read::<u16>(addr + 1)?)),
            _ => unreachable!(),
        };

        Ok(Instruction {
            opcode,
            mnemonic: info.0,
            imm,
            size: info.3,
        })
    }
}
