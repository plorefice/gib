#[derive(Debug)]
pub enum TraceEvent {
    IllegalInstructionFault(u8),
    BusFault(u16),
    MemFault(u16),
    IoFault(u16),
}
