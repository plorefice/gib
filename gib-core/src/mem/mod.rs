mod memory;

use super::dbg;

pub use memory::*;

pub trait MemR {
    fn read(&self, addr: u16) -> Result<u8, dbg::TraceEvent>;
}

pub trait MemW {
    fn write(&mut self, addr: u16, val: u8) -> Result<(), dbg::TraceEvent>;
}

pub trait MemRW: MemR + MemW {}
