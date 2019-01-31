use super::dbg;

pub trait MemSize: Default {
    fn byte_size() -> u8;

    fn read_le(buf: &[u8]) -> Self;
    fn write_le(buf: &mut [u8], v: Self);
}

impl MemSize for u8 {
    fn byte_size() -> u8 {
        1
    }

    fn read_le(buf: &[u8]) -> u8 {
        buf[0]
    }

    fn write_le(buf: &mut [u8], v: u8) {
        buf[0] = v;
    }
}

impl MemSize for i8 {
    fn byte_size() -> u8 {
        1
    }

    fn read_le(buf: &[u8]) -> i8 {
        buf[0] as i8
    }

    fn write_le(buf: &mut [u8], v: i8) {
        buf[0] = v as u8;
    }
}

impl MemSize for u16 {
    fn byte_size() -> u8 {
        2
    }

    fn read_le(buf: &[u8]) -> u16 {
        (u16::from(buf[1]) << 8) | u16::from(buf[0])
    }

    fn write_le(buf: &mut [u8], v: u16) {
        buf[0] = (v & 0xFF) as u8;
        buf[1] = (v >> 8) as u8;
    }
}

pub trait MemR {
    fn read<T: MemSize>(&self, addr: u16) -> Result<T, dbg::TraceEvent>;
}

pub trait MemW {
    fn write<T: MemSize>(&mut self, addr: u16, val: T) -> Result<(), dbg::TraceEvent>;
}

pub trait MemRW: MemR + MemW {}
