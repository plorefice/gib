use super::dbg;
use super::mem::{MemRW, MemSize};

use std::collections::HashSet;

#[derive(Default, Clone)]
pub struct CPU {
    pub af: u16,
    pub bc: u16,
    pub de: u16,
    pub hl: u16,
    pub sp: u16,
    pub pc: u16,

    pub intr_enabled: bool,
    pub halted: bool,
    pub clk: u64,

    paused: bool,
    breakpoints: HashSet<u16>,
}

impl CPU {
    pub fn new() -> CPU {
        CPU::default()
    }

    pub fn exec(&mut self, bus: &mut impl MemRW) -> Result<(), dbg::TraceEvent> {
        if !self.paused() && self.breakpoints.contains(&self.pc) {
            self.pause();
            return Err(dbg::TraceEvent::Breakpoint(self.pc));
        } else {
            self.resume();
        }

        let saved_ctx = self.clone();

        let opc = self.fetch_pc(bus)?;
        let res = self.op(bus, opc);

        if res.is_err() {
            *self = saved_ctx;
        }
        res
    }

    pub fn fetch_pc<T: MemSize>(&mut self, bus: &mut impl MemRW) -> Result<T, dbg::TraceEvent> {
        let v = self.fetch(bus, self.pc)?;
        self.pc += u16::from(T::byte_size());
        Ok(v)
    }

    pub fn fetch<T: MemSize>(
        &mut self,
        bus: &mut impl MemRW,
        addr: u16,
    ) -> Result<T, dbg::TraceEvent> {
        let v = bus.read::<T>(addr)?;
        self.clk += u64::from(T::byte_size() * 4);
        Ok(v)
    }

    pub fn store<T: MemSize>(
        &mut self,
        bus: &mut impl MemRW,
        addr: u16,
        val: T,
    ) -> Result<(), dbg::TraceEvent> {
        bus.write::<T>(addr, val)?;
        self.clk += u64::from(T::byte_size() * 4);
        Ok(())
    }

    fn resume(&mut self) {
        self.paused = false;
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn paused(&self) -> bool {
        self.paused
    }

    pub fn set_breakpoint(&mut self, addr: u16) {
        self.breakpoints.insert(addr);
    }

    pub fn clear_breakpoint(&mut self, addr: u16) {
        self.breakpoints.remove(&addr);
    }

    pub fn breakpoint_at(&self, addr: u16) -> bool {
        self.breakpoints.contains(&addr)
    }

    pub fn breakpoints(&self) -> &HashSet<u16> {
        &self.breakpoints
    }
}

#[rustfmt::skip]
impl CPU {
    pub fn c(&self) -> u8 { self.bc as u8 }
    pub fn e(&self) -> u8 { self.de as u8 }
    pub fn l(&self) -> u8 { self.hl as u8 }
    pub fn a(&self) -> u8 { (self.af >> 8) as u8 }
    pub fn b(&self) -> u8 { (self.bc >> 8) as u8 }
    pub fn d(&self) -> u8 { (self.de >> 8) as u8 }
    pub fn h(&self) -> u8 { (self.hl >> 8) as u8 }
    pub fn f(&self) -> u8 { (self.af & 0x00F0) as u8 }

    pub fn set_c(&mut self, v: u8) { self.bc = (self.bc & 0xFF00) | u16::from(v); }
    pub fn set_e(&mut self, v: u8) { self.de = (self.de & 0xFF00) | u16::from(v); }
    pub fn set_l(&mut self, v: u8) { self.hl = (self.hl & 0xFF00) | u16::from(v); }
    pub fn set_f(&mut self, v: u8) { self.af = (self.af & 0xFF00) | u16::from(v & 0xF0); }
    pub fn set_a(&mut self, v: u8) { self.af = (self.af & 0x00FF) | (u16::from(v) << 8); }
    pub fn set_b(&mut self, v: u8) { self.bc = (self.bc & 0x00FF) | (u16::from(v) << 8); }
    pub fn set_d(&mut self, v: u8) { self.de = (self.de & 0x00FF) | (u16::from(v) << 8); }
    pub fn set_h(&mut self, v: u8) { self.hl = (self.hl & 0x00FF) | (u16::from(v) << 8); }

    pub fn zf(&self) -> bool { (self.f() & 0x80) != 0 }
    pub fn sf(&self) -> bool { (self.f() & 0x40) != 0 }
    pub fn hc(&self) -> bool { (self.f() & 0x20) != 0 }
    pub fn cy(&self) -> bool { (self.f() & 0x10) != 0 }

    pub fn set_zf(&mut self, v: bool) { self.set_f((self.f() & (!0x80)) | (u8::from(v) << 7)); }
    pub fn set_sf(&mut self, v: bool) { self.set_f((self.f() & (!0x40)) | (u8::from(v) << 6)); }
    pub fn set_hc(&mut self, v: bool) { self.set_f((self.f() & (!0x20)) | (u8::from(v) << 5)); }
    pub fn set_cy(&mut self, v: bool) { self.set_f((self.f() & (!0x10)) | (u8::from(v) << 4)); }
}
