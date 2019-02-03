use super::dbg;

pub trait MemSize: Default {
    fn byte_size() -> u8;

    /* IO operations */
    fn read_le(buf: &[u8]) -> Result<Self, dbg::TraceEvent>;
    fn write_le(buf: &mut [u8], v: Self) -> Result<(), dbg::TraceEvent>;
    fn write_mut_le(buf: &mut [&mut u8], v: Self) -> Result<(), dbg::TraceEvent>;

    /* Byte manipulation */
    fn low(&self) -> u8;
    fn high(&self) -> u8;
}

impl MemSize for u8 {
    fn byte_size() -> u8 {
        1
    }

    fn read_le(buf: &[u8]) -> Result<u8, dbg::TraceEvent> {
        Ok(buf[0])
    }

    fn write_le(buf: &mut [u8], v: u8) -> Result<(), dbg::TraceEvent> {
        buf[0] = v;
        Ok(())
    }

    fn write_mut_le(buf: &mut [&mut u8], v: u8) -> Result<(), dbg::TraceEvent> {
        *buf[0] = v;
        Ok(())
    }

    fn low(&self) -> u8 {
        *self
    }

    fn high(&self) -> u8 {
        *self
    }
}

impl MemSize for i8 {
    fn byte_size() -> u8 {
        1
    }

    fn read_le(buf: &[u8]) -> Result<i8, dbg::TraceEvent> {
        Ok(buf[0] as i8)
    }

    fn write_le(buf: &mut [u8], v: i8) -> Result<(), dbg::TraceEvent> {
        buf[0] = v as u8;
        Ok(())
    }

    fn write_mut_le(buf: &mut [&mut u8], v: i8) -> Result<(), dbg::TraceEvent> {
        *buf[0] = v as u8;
        Ok(())
    }

    fn low(&self) -> u8 {
        *self as u8
    }

    fn high(&self) -> u8 {
        *self as u8
    }
}

impl MemSize for u16 {
    fn byte_size() -> u8 {
        2
    }

    fn read_le(buf: &[u8]) -> Result<u16, dbg::TraceEvent> {
        if buf.len() < u16::byte_size() as usize {
            Err(dbg::TraceEvent::AccessFault)
        } else {
            Ok((u16::from(buf[1]) << 8) | u16::from(buf[0]))
        }
    }

    fn write_le(buf: &mut [u8], v: u16) -> Result<(), dbg::TraceEvent> {
        if buf.len() < u16::byte_size() as usize {
            Err(dbg::TraceEvent::AccessFault)
        } else {
            buf[0] = (v & 0xFF) as u8;
            buf[1] = (v >> 8) as u8;
            Ok(())
        }
    }

    fn write_mut_le(buf: &mut [&mut u8], v: u16) -> Result<(), dbg::TraceEvent> {
        if buf.len() < u16::byte_size() as usize {
            Err(dbg::TraceEvent::AccessFault)
        } else {
            *buf[0] = (v & 0xFF) as u8;
            *buf[1] = (v >> 8) as u8;
            Ok(())
        }
    }

    fn low(&self) -> u8 {
        (*self & 0xFF) as u8
    }

    fn high(&self) -> u8 {
        (*self >> 8) as u8
    }
}

pub trait MemR {
    fn read<T: MemSize>(&self, addr: u16) -> Result<T, dbg::TraceEvent>;
}

pub trait MemW {
    fn write<T: MemSize>(&mut self, addr: u16, val: T) -> Result<(), dbg::TraceEvent>;
}

pub trait MemRW: MemR + MemW {}
