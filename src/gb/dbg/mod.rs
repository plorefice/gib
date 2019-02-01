#[derive(Debug, Clone, Copy)]
pub enum Peripheral {
    VPU,
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
