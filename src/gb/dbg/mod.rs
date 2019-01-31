#[derive(Debug)]
pub enum Peripheral {
    VPU,
}

#[derive(Debug, Fail)]
pub enum TraceEvent {
    #[fail(display = "Illegal opcode: {:02X}", _0)]
    IllegalInstructionFault(u8),
    #[fail(display = "Bus fault at 0x{:04X}", _0)]
    BusFault(u16),
    #[fail(display = "Memory fault at 0x{:04X}", _0)]
    MemFault(u16),
    #[fail(display = "IO fault while accessing {:?}@{:04X}", _0, _1)]
    IoFault(Peripheral, u16),
}
