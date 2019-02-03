use super::dbg;
use super::mem::MemRW;
use super::CPU;

macro_rules! pop {
    ($cpu:ident, $bus:ident, $reg:ident) => {{
        $cpu.$reg = $cpu.fetch($bus, $cpu.sp)?;
        $cpu.sp += 2;
    }};
}

macro_rules! push {
    ($cpu:ident, $bus:ident, $reg:ident) => {{
        $cpu.sp -= 2;
        $cpu.clk += 4;
        $cpu.store($bus, $cpu.sp, $cpu.$reg)?;
    }};
}

macro_rules! jp {
    ($cpu:ident, $cond:expr, $abs:expr) => {{
        if $cond {
            $cpu.pc = $abs;
            $cpu.clk += 4;
        }
    }};
}

macro_rules! jr {
    ($cpu:ident, $cond:expr, $offset:expr) => {
        jp!(
            $cpu,
            $cond,
            (i32::from($cpu.pc) + i32::from($offset)) as u16
        );
    };
}

macro_rules! call {
    ($cpu:ident, $bus:ident, $cond:expr, $to:expr) => {{
        if $cond {
            push!($cpu, $bus, pc);
            $cpu.pc = $to;
        }
    }};
}

macro_rules! ret {
    ($cpu:ident, $bus:ident, $cond:expr) => {{
        if $cond {
            pop!($cpu, $bus, pc);
            $cpu.clk += 4;
        }
        $cpu.clk += 4;
    }};
}

macro_rules! logical {
    ($cpu:ident, $op:tt, $rhs:expr, $sf:expr, $hc: expr, $cy:expr) => {{
        $cpu.set_a($cpu.a() $op $rhs);

        $cpu.set_zf($cpu.a() == 0);
        $cpu.set_sf($sf != 0);
        $cpu.set_hc($hc != 0);
        $cpu.set_cy($cy != 0);
    }};
}

macro_rules! and { ($cpu:ident, $rhs:expr) => { logical!($cpu, &, $rhs, 0, 1, 0); }; }
macro_rules! xor { ($cpu:ident, $rhs:expr) => { logical!($cpu, ^, $rhs, 0, 0, 0); }; }
macro_rules! or  { ($cpu:ident, $rhs:expr) => { logical!($cpu, |, $rhs, 0, 0, 0); }; }

macro_rules! inc {
    ($cpu:ident, $v:expr) => {{
        $cpu.set_zf(($v + 1) == 0);
        $cpu.set_sf(false);
        $cpu.set_hc(($v & 0xF) == 0xF);
        $v + 1
    }};
}

macro_rules! dec {
    ($cpu:ident, $v:expr) => {{
        $cpu.set_zf(($v - 1) == 0);
        $cpu.set_sf(true);
        $cpu.set_hc($v.trailing_zeros() >= 4);
        $v - 1
    }};
}

macro_rules! add {
    ($cpu:ident, $v:expr, $cy:expr) => {{
        let x = u16::from($cpu.a());
        let y = u16::from($v);
        let c = u16::from($cy);

        let r = x + y + c;
        $cpu.set_a(r as u8);

        $cpu.set_zf($cpu.a() == 0);
        $cpu.set_sf(false);
        $cpu.set_hc((x & 0xF) + (y & 0xF) + c >= 0x10);
        $cpu.set_cy(r >= 0x100);
    }};
}

macro_rules! sub {
    ($cpu:ident, $v:expr, $cy:expr) => {{
        let x = u16::from($cpu.a());
        let y = u16::from($v);
        let c = u16::from($cy);

        let r = x - y - c;
        $cpu.set_a(r as u8);

        $cpu.set_zf($cpu.a() == 0);
        $cpu.set_sf(true);
        $cpu.set_hc((y & 0xF) + c > (x & 0xF));
        $cpu.set_cy(r >= 0x100);
    }};
}

macro_rules! add16 {
    ($cpu:ident, $dst: expr, $v:expr) => {{
        let old = $dst;
        $dst += $v;

        $cpu.set_sf(false);
        $cpu.set_hc((old & 0x0FFF) + ($v & 0x0FFF) >= 0x1000);
        $cpu.set_cy($dst < old);
        $cpu.clk += 4;
    }};
}

macro_rules! addi16 {
    ($cpu:ident, $a:expr, $b:expr) => {{
        let b = u16::from($b as u8);
        let n = (i32::from($a) + i32::from($b)) as u16;
        let r = ($a & 0xFF) + b;

        $cpu.set_zf(false);
        $cpu.set_sf(false);
        $cpu.set_hc(($a & 0xF) + (b & 0xF) >= 0x10);
        $cpu.set_cy(r > 0xFF);
        $cpu.clk += 4;

        n
    }};
}

macro_rules! cmp {
    ($cpu:ident, $a:expr, $b:expr) => {{
        $cpu.set_zf($a == $b);
        $cpu.set_sf(true);
        $cpu.set_hc(($b & 0xF) > ($a & 0xF));
        $cpu.set_cy($b > $a);
    }};
}

macro_rules! rl {
    ($cpu:ident, $cy:expr, $v:expr) => {{
        let cy = $v >> 7;
        let res = ($v << 1) | if $cy { cy } else { u8::from($cpu.cy()) };

        $cpu.set_f(0);
        $cpu.set_zf(res == 0);
        $cpu.set_cy(cy != 0);
        res
    }};
}

macro_rules! rr {
    ($cpu:ident, $cy:expr, $v:expr) => {{
        let cy = $v & 0x1;
        let res = ($v >> 1) | (if $cy { cy } else { u8::from($cpu.cy()) } << 7);

        $cpu.set_f(0);
        $cpu.set_zf(res == 0);
        $cpu.set_cy(cy != 0);
        res
    }};
}

macro_rules! sla {
    ($cpu:ident, $v:expr) => {{
        let cy = $v >> 7;
        let res = $v << 1;

        $cpu.set_f(0);
        $cpu.set_zf(res == 0);
        $cpu.set_cy(cy != 0);
        res
    }};
}

macro_rules! sra {
    ($cpu:ident, $v:expr) => {{
        let cy = $v & 0x1;
        let res = ($v >> 1) | ($v & 0x80);

        $cpu.set_f(0);
        $cpu.set_zf(res == 0);
        $cpu.set_cy(cy != 0);
        res
    }};
}

macro_rules! srl {
    ($cpu:ident, $v:expr) => {{
        let cy = $v & 0x1;
        let res = $v >> 1;

        $cpu.set_f(0);
        $cpu.set_zf(res == 0);
        $cpu.set_cy(cy != 0);
        res
    }};
}

macro_rules! swap {
    ($cpu:ident, $v:expr) => {{
        let res = ($v >> 4) | ($v << 4);

        $cpu.set_f(0);
        $cpu.set_zf(res == 0);
        res
    }};
}

macro_rules! bit {
    ($cpu:ident, $n:expr, $v:expr) => {{
        $cpu.set_zf(($v & (1 << $n)) == 0);
        $cpu.set_sf(false);
        $cpu.set_hc(true);
    }};
}

macro_rules! res {
    ($n:expr, $v:expr) => {
        $v & (!(1 << $n))
    };
}

macro_rules! set {
    ($n:expr, $v:expr) => {
        $v | (1 << $n)
    };
}

impl CPU {
    #[rustfmt::skip]
    #[allow(clippy::cyclomatic_complexity)]
    pub fn op(&mut self, bus: &mut impl MemRW, opcode: u8) -> Result<bool, dbg::TraceEvent> {
        match opcode {
            /*
             * Misc/control instructions
             */
            0x00 => (),

            0x10 | 0x76 => return Ok(true),

            0xF3 => self.intr_enabled = false,
            0xFB => self.intr_enabled = true,

            0xCB => {
                let cb: u8 = self.fetch_pc(bus)?;
                self.op_cb(bus, cb)?;
            }

            /*
             * Jump/calls
             */
            0x20 => { let off: i8 = self.fetch_pc(bus)?; jr!(self, !self.zf(), off); }
            0x30 => { let off: i8 = self.fetch_pc(bus)?; jr!(self, !self.cy(), off); }
            0x28 => { let off: i8 = self.fetch_pc(bus)?; jr!(self, self.zf(),  off); }
            0x38 => { let off: i8 = self.fetch_pc(bus)?; jr!(self, self.cy(),  off); }
            0x18 => { let off: i8 = self.fetch_pc(bus)?; jr!(self, true,       off); }

            0xC2 => { let abs: u16 = self.fetch_pc(bus)?; jp!(self, !self.zf(), abs); }
            0xD2 => { let abs: u16 = self.fetch_pc(bus)?; jp!(self, !self.cy(), abs); }
            0xCA => { let abs: u16 = self.fetch_pc(bus)?; jp!(self, self.zf(),  abs); }
            0xDA => { let abs: u16 = self.fetch_pc(bus)?; jp!(self, self.cy(),  abs); }
            0xC3 => { let abs: u16 = self.fetch_pc(bus)?; jp!(self, true,       abs); }

            0xE9 => { jp!(self, true, self.hl); self.clk -= 4; }

            0xC4 => { let abs: u16 = self.fetch_pc(bus)?; call!(self, bus, !self.zf(), abs); }
            0xD4 => { let abs: u16 = self.fetch_pc(bus)?; call!(self, bus, !self.cy(), abs); }
            0xCC => { let abs: u16 = self.fetch_pc(bus)?; call!(self, bus, self.zf(),  abs); }
            0xDC => { let abs: u16 = self.fetch_pc(bus)?; call!(self, bus, self.cy(),  abs); }
            0xCD => { let abs: u16 = self.fetch_pc(bus)?; call!(self, bus, true,       abs); }

            0xC0 => ret!(self, bus, !self.zf()),
            0xD0 => ret!(self, bus, !self.cy()),
            0xC8 => ret!(self, bus, self.zf()),
            0xD8 => ret!(self, bus, self.cy()),

            0xC9 => { ret!(self, bus, true); self.clk -= 4; }
            0xD9 => { ret!(self, bus, true); self.clk -= 4; self.intr_enabled = true; }

            0xC7 => call!(self, bus, true, 0x00),
            0xCF => call!(self, bus, true, 0x08),
            0xD7 => call!(self, bus, true, 0x10),
            0xDF => call!(self, bus, true, 0x18),
            0xE7 => call!(self, bus, true, 0x20),
            0xEF => call!(self, bus, true, 0x28),
            0xF7 => call!(self, bus, true, 0x30),
            0xFF => call!(self, bus, true, 0x38),

            /*
             * 8bit load/store/move instructions
             */
            0x02 => self.store(bus, self.bc, self.a())?,
            0x12 => self.store(bus, self.de, self.a())?,

            0x22 => { self.store(bus, self.hl, self.a())?; self.hl += 1; }
            0x32 => { self.store(bus, self.hl, self.a())?; self.hl -= 1; }

            0x0A => { let d8: u8 = self.fetch(bus, self.bc)?; self.set_a(d8); }
            0x1A => { let d8: u8 = self.fetch(bus, self.de)?; self.set_a(d8); }

            0x2A => { let d8: u8 = self.fetch(bus, self.hl)?; self.set_a(d8); self.hl += 1; }
            0x3A => { let d8: u8 = self.fetch(bus, self.hl)?; self.set_a(d8); self.hl -= 1; }

            0x06 => { let d8: u8 = self.fetch_pc(bus)?; self.set_b(d8);                }
            0x16 => { let d8: u8 = self.fetch_pc(bus)?; self.set_d(d8);                }
            0x26 => { let d8: u8 = self.fetch_pc(bus)?; self.set_h(d8);                }
            0x36 => { let d8: u8 = self.fetch_pc(bus)?; self.store(bus, self.hl, d8)?; }
            0x0E => { let d8: u8 = self.fetch_pc(bus)?; self.set_c(d8);                }
            0x1E => { let d8: u8 = self.fetch_pc(bus)?; self.set_e(d8);                }
            0x2E => { let d8: u8 = self.fetch_pc(bus)?; self.set_l(d8);                }
            0x3E => { let d8: u8 = self.fetch_pc(bus)?; self.set_a(d8);                }

            0x40 => self.set_b(self.b()),
            0x41 => self.set_b(self.c()),
            0x42 => self.set_b(self.d()),
            0x43 => self.set_b(self.e()),
            0x44 => self.set_b(self.h()),
            0x45 => self.set_b(self.l()),
            0x46 => { let b = self.fetch(bus, self.hl)?; self.set_b(b); }
            0x47 => self.set_b(self.a()),
            0x48 => self.set_c(self.b()),
            0x49 => self.set_c(self.c()),
            0x4A => self.set_c(self.d()),
            0x4B => self.set_c(self.e()),
            0x4C => self.set_c(self.h()),
            0x4D => self.set_c(self.l()),
            0x4E => { let d8 = self.fetch(bus, self.hl)?; self.set_c(d8); }
            0x4F => self.set_c(self.a()),
            0x50 => self.set_d(self.b()),
            0x51 => self.set_d(self.c()),
            0x52 => self.set_d(self.d()),
            0x53 => self.set_d(self.e()),
            0x54 => self.set_d(self.h()),
            0x55 => self.set_d(self.l()),
            0x56 => { let d8 = self.fetch(bus, self.hl)?; self.set_d(d8); }
            0x57 => self.set_d(self.a()),
            0x58 => self.set_e(self.b()),
            0x59 => self.set_e(self.c()),
            0x5A => self.set_e(self.d()),
            0x5B => self.set_e(self.e()),
            0x5C => self.set_e(self.h()),
            0x5D => self.set_e(self.l()),
            0x5E => { let d8 = self.fetch(bus, self.hl)?; self.set_e(d8); }
            0x5F => self.set_e(self.a()),
            0x60 => self.set_h(self.b()),
            0x61 => self.set_h(self.c()),
            0x62 => self.set_h(self.d()),
            0x63 => self.set_h(self.e()),
            0x64 => self.set_h(self.h()),
            0x65 => self.set_h(self.l()),
            0x66 => { let d8 = self.fetch(bus, self.hl)?; self.set_h(d8); }
            0x67 => self.set_h(self.a()),
            0x68 => self.set_l(self.b()),
            0x69 => self.set_l(self.c()),
            0x6A => self.set_l(self.d()),
            0x6B => self.set_l(self.e()),
            0x6C => self.set_l(self.h()),
            0x6D => self.set_l(self.l()),
            0x6E => { let d8 = self.fetch(bus, self.hl)?; self.set_l(d8); }
            0x6F => self.set_l(self.a()),
            0x78 => self.set_a(self.b()),
            0x79 => self.set_a(self.c()),
            0x7A => self.set_a(self.d()),
            0x7B => self.set_a(self.e()),
            0x7C => self.set_a(self.h()),
            0x7D => self.set_a(self.l()),
            0x7E => { let d8 = self.fetch(bus, self.hl)?; self.set_a(d8); }
            0x7F => self.set_a(self.a()),

            0x70 => self.store(bus, self.hl, self.b())?,
            0x71 => self.store(bus, self.hl, self.c())?,
            0x72 => self.store(bus, self.hl, self.d())?,
            0x73 => self.store(bus, self.hl, self.e())?,
            0x74 => self.store(bus, self.hl, self.h())?,
            0x75 => self.store(bus, self.hl, self.l())?,
            0x77 => self.store(bus, self.hl, self.a())?,

            0xE0 => {
                let d8: u8 = self.fetch_pc(bus)?;
                self.store(bus, 0xFF00 + u16::from(d8), self.a())?;
            }
            0xF0 => {
                let d8: u8 = self.fetch_pc(bus)?;
                let a: u8 = self.fetch(bus, 0xFF00 + u16::from(d8))?;
                self.set_a(a);
            }

            0xE2 => self.store(bus, 0xFF00 + u16::from(self.c()), self.a())?,
            0xF2 => {
                let d8: u8 = self.fetch(bus, 0xFF00 + u16::from(self.c()))?;
                self.set_a(d8);
            }

            0xEA => { let d16: u16 = self.fetch_pc(bus)?; self.store(bus, d16, self.a())?; }
            0xFA => {
                let d16: u16 = self.fetch_pc(bus)?;
                let a: u8 = self.fetch(bus, d16)?;
                self.set_a(a);
            }

            /*
             * 16bit load/store/move instructions
             */
            0x01 => self.bc = self.fetch_pc(bus)?,
            0x11 => self.de = self.fetch_pc(bus)?,
            0x21 => self.hl = self.fetch_pc(bus)?,
            0x31 => self.sp = self.fetch_pc(bus)?,

            0xC1 => pop!(self, bus, bc),
            0xD1 => pop!(self, bus, de),
            0xE1 => pop!(self, bus, hl),
            0xF1 => {
                pop!(self, bus, af);
                self.af &= 0xFFF0;
            }

            0xC5 => push!(self, bus, bc),
            0xD5 => push!(self, bus, de),
            0xE5 => push!(self, bus, hl),
            0xF5 => push!(self, bus, af),

            0x08 => { let a16: u16 = self.fetch_pc(bus)?; self.store(bus, a16, self.sp)?; }
            0xF9 => { self.sp = self.hl; self.clk += 4; }

            0xF8 => {
                let d8: i8 = self.fetch_pc(bus)?;
                self.hl = addi16!(self, self.sp, d8);
            }

            /*
             * 8bit arithmetic/logical instructions
             */
            0x04 => { let v = inc!(self, self.b()); self.set_b(v); }
            0x14 => { let v = inc!(self, self.d()); self.set_d(v); }
            0x24 => { let v = inc!(self, self.h()); self.set_h(v); }
            0x0C => { let v = inc!(self, self.c()); self.set_c(v); }
            0x1C => { let v = inc!(self, self.e()); self.set_e(v); }
            0x2C => { let v = inc!(self, self.l()); self.set_l(v); }
            0x3C => { let v = inc!(self, self.a()); self.set_a(v); }
            0x34 => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                let v = inc!(self, d8);
                self.store(bus, self.hl, v)?;
            }

            0x05 => { let v = dec!(self, self.b()); self.set_b(v); }
            0x15 => { let v = dec!(self, self.d()); self.set_d(v); }
            0x25 => { let v = dec!(self, self.h()); self.set_h(v); }
            0x0D => { let v = dec!(self, self.c()); self.set_c(v); }
            0x1D => { let v = dec!(self, self.e()); self.set_e(v); }
            0x2D => { let v = dec!(self, self.l()); self.set_l(v); }
            0x3D => { let v = dec!(self, self.a()); self.set_a(v); }
            0x35 => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                let v = dec!(self, d8);
                self.store(bus, self.hl, v)?;
            }

            0x80 => add!(self, self.b(), 0u8),
            0x81 => add!(self, self.c(), 0u8),
            0x82 => add!(self, self.d(), 0u8),
            0x83 => add!(self, self.e(), 0u8),
            0x84 => add!(self, self.h(), 0u8),
            0x85 => add!(self, self.l(), 0u8),
            0x87 => add!(self, self.a(), 0u8),
            0x86 => { let d8: u8 = self.fetch(bus, self.hl)?; add!(self, d8, 0u8); }
            0xC6 => { let d8: u8 = self.fetch_pc(bus)?; add!(self, d8, 0u8); }

            0x88 => add!(self, self.b(), self.cy() as u8),
            0x89 => add!(self, self.c(), self.cy() as u8),
            0x8A => add!(self, self.d(), self.cy() as u8),
            0x8B => add!(self, self.e(), self.cy() as u8),
            0x8C => add!(self, self.h(), self.cy() as u8),
            0x8D => add!(self, self.l(), self.cy() as u8),
            0x8F => add!(self, self.a(), self.cy() as u8),
            0x8E => { let d8: u8 = self.fetch(bus, self.hl)?; add!(self, d8, self.cy() as u8); }
            0xCE => { let d8: u8 = self.fetch_pc(bus)?; add!(self, d8, self.cy() as u8); }

            0x90 => sub!(self, self.b(), 0u8),
            0x91 => sub!(self, self.c(), 0u8),
            0x92 => sub!(self, self.d(), 0u8),
            0x93 => sub!(self, self.e(), 0u8),
            0x94 => sub!(self, self.h(), 0u8),
            0x95 => sub!(self, self.l(), 0u8),
            0x97 => sub!(self, self.a(), 0u8),
            0x96 => { let d8: u8 = self.fetch(bus, self.hl)?; sub!(self, d8, 0u8); }
            0xD6 => { let d8: u8 = self.fetch_pc(bus)?; sub!(self, d8, 0u8); }

            0x98 => sub!(self, self.b(), self.cy() as u8),
            0x99 => sub!(self, self.c(), self.cy() as u8),
            0x9A => sub!(self, self.d(), self.cy() as u8),
            0x9B => sub!(self, self.e(), self.cy() as u8),
            0x9C => sub!(self, self.h(), self.cy() as u8),
            0x9D => sub!(self, self.l(), self.cy() as u8),
            0x9F => sub!(self, self.a(), self.cy() as u8),
            0x9E => { let d8: u8 = self.fetch(bus, self.hl)?; sub!(self, d8, self.cy() as u8); }
            0xDE => { let d8: u8 = self.fetch_pc(bus)?; sub!(self, d8, self.cy() as u8); }

            0xA0 => and!(self, self.b()),
            0xA1 => and!(self, self.c()),
            0xA2 => and!(self, self.d()),
            0xA3 => and!(self, self.e()),
            0xA4 => and!(self, self.h()),
            0xA5 => and!(self, self.l()),
            0xA7 => and!(self, self.a()),
            0xA6 => { let d8: u8 = self.fetch(bus, self.hl)?; and!(self, d8); }
            0xE6 => { let d8: u8 = self.fetch_pc(bus)?; and!(self, d8); }

            0xA8 => xor!(self, self.b()),
            0xA9 => xor!(self, self.c()),
            0xAA => xor!(self, self.d()),
            0xAB => xor!(self, self.e()),
            0xAC => xor!(self, self.h()),
            0xAD => xor!(self, self.l()),
            0xAF => xor!(self, self.a()),
            0xAE => { let d8: u8 = self.fetch(bus, self.hl)?; xor!(self, d8); }
            0xEE => { let d8: u8 = self.fetch_pc(bus)?; xor!(self, d8); }

            0xB0 => or!(self, self.b()),
            0xB1 => or!(self, self.c()),
            0xB2 => or!(self, self.d()),
            0xB3 => or!(self, self.e()),
            0xB4 => or!(self, self.h()),
            0xB5 => or!(self, self.l()),
            0xB7 => or!(self, self.a()),
            0xB6 => { let d8: u8 = self.fetch(bus, self.hl)?; or!(self, d8); }
            0xF6 => { let d8: u8 = self.fetch_pc(bus)?; or!(self, d8); }

            0xB8 => cmp!(self, self.a(), self.b()),
            0xB9 => cmp!(self, self.a(), self.c()),
            0xBA => cmp!(self, self.a(), self.d()),
            0xBB => cmp!(self, self.a(), self.e()),
            0xBC => cmp!(self, self.a(), self.h()),
            0xBD => cmp!(self, self.a(), self.l()),
            0xBF => cmp!(self, self.a(), self.a()),
            0xBE => { let d8: u8 = self.fetch(bus, self.hl)?; cmp!(self, self.a(), d8); }
            0xFE => { let d8: u8 = self.fetch_pc(bus)?; cmp!(self, self.a(), d8); }

            0x2F => { self.set_a(!self.a()); self.set_sf(true); self.set_hc(true); }
            0x37 => { self.set_sf(false); self.set_hc(false); self.set_cy(true); }
            0x3F => { self.set_sf(false); self.set_hc(false); self.set_cy(!self.cy()); }

            0x27 => {
                if !self.sf() {
                    if self.cy() || self.a() > 0x99 {
                        self.set_a(self.a() + 0x60);
                        self.set_cy(true);
                    }
                    if self.hc() || (self.a() & 0x0f) > 0x09 {
                        self.set_a(self.a() + 0x06);
                    }
                } else {
                    if self.cy() {
                        self.set_a(self.a() - 0x60);
                    }
                    if self.hc() {
                        self.set_a(self.a() - 0x06);
                    }
                }

                self.set_zf(self.a() == 0);
                self.set_hc(false);
            }

            /*
             * 	16bit arithmetic/logical instructions
             */
            0x03 => { self.bc += 1; self.clk += 4; }
            0x13 => { self.de += 1; self.clk += 4; }
            0x23 => { self.hl += 1; self.clk += 4; }
            0x33 => { self.sp += 1; self.clk += 4; }

            0x0B => { self.bc -= 1; self.clk += 4; }
            0x1B => { self.de -= 1; self.clk += 4; }
            0x2B => { self.hl -= 1; self.clk += 4; }
            0x3B => { self.sp -= 1; self.clk += 4; }

            0x09 => add16!(self, self.hl, self.bc),
            0x19 => add16!(self, self.hl, self.de),
            0x29 => add16!(self, self.hl, self.hl),
            0x39 => add16!(self, self.hl, self.sp),
            0xE8 => {
                let d8: i8 = self.fetch_pc(bus)?;
                self.sp = addi16!(self, self.sp, d8);
                self.clk += 4;
            }

            /*
             * 8bit rotations/shifts and bit instructions
             */
            0x07 => { let v = rl!(self, true, self.a()); self.set_a(v); self.set_zf(false); }
            0x17 => { let v = rl!(self, false, self.a()); self.set_a(v); self.set_zf(false); }
            0x0F => { let v = rr!(self, true, self.a()); self.set_a(v); self.set_zf(false); }
            0x1F => { let v = rr!(self, false, self.a()); self.set_a(v); self.set_zf(false); }

            /*
             * Invalid opcodes
             */
            0xD3 | 0xDB | 0xDD | 0xE3 | 0xE4 | 0xEB | 0xEC | 0xED | 0xF4 | 0xFC | 0xFD => {
                return Err(dbg::TraceEvent::IllegalInstructionFault(opcode));
            }
        };

        Ok(false)
    }

    #[rustfmt::skip]
    #[allow(clippy::cyclomatic_complexity)]
    fn op_cb(&mut self, bus: &mut impl MemRW, opcode: u8) -> Result<(), dbg::TraceEvent> {
        match opcode {
            0x00 => { let v = rl!(self, true, self.b()); self.set_b(v); }
            0x01 => { let v = rl!(self, true, self.c()); self.set_c(v); }
            0x02 => { let v = rl!(self, true, self.d()); self.set_d(v); }
            0x03 => { let v = rl!(self, true, self.e()); self.set_e(v); }
            0x04 => { let v = rl!(self, true, self.h()); self.set_h(v); }
            0x05 => { let v = rl!(self, true, self.l()); self.set_l(v); }
            0x07 => { let v = rl!(self, true, self.a()); self.set_a(v); }
            0x06 => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                let v = rl!(self, true, d8);
                self.store(bus, self.hl, v)?;
            }

            0x08 => { let v = rr!(self, true, self.b()); self.set_b(v); }
            0x09 => { let v = rr!(self, true, self.c()); self.set_c(v); }
            0x0A => { let v = rr!(self, true, self.d()); self.set_d(v); }
            0x0B => { let v = rr!(self, true, self.e()); self.set_e(v); }
            0x0C => { let v = rr!(self, true, self.h()); self.set_h(v); }
            0x0D => { let v = rr!(self, true, self.l()); self.set_l(v); }
            0x0F => { let v = rr!(self, true, self.a()); self.set_a(v); }
            0x0E => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                let v = rr!(self, true, d8);
                self.store(bus, self.hl, v)?;
            }

            0x10 => { let v = rl!(self, false, self.b()); self.set_b(v); }
            0x11 => { let v = rl!(self, false, self.c()); self.set_c(v); }
            0x12 => { let v = rl!(self, false, self.d()); self.set_d(v); }
            0x13 => { let v = rl!(self, false, self.e()); self.set_e(v); }
            0x14 => { let v = rl!(self, false, self.h()); self.set_h(v); }
            0x15 => { let v = rl!(self, false, self.l()); self.set_l(v); }
            0x17 => { let v = rl!(self, false, self.a()); self.set_a(v); }
            0x16 => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                let v = rl!(self, false, d8);
                self.store(bus, self.hl, v)?;
            }

            0x18 => { let v = rr!(self, false, self.b()); self.set_b(v); }
            0x19 => { let v = rr!(self, false, self.c()); self.set_c(v); }
            0x1A => { let v = rr!(self, false, self.d()); self.set_d(v); }
            0x1B => { let v = rr!(self, false, self.e()); self.set_e(v); }
            0x1C => { let v = rr!(self, false, self.h()); self.set_h(v); }
            0x1D => { let v = rr!(self, false, self.l()); self.set_l(v); }
            0x1F => { let v = rr!(self, false, self.a()); self.set_a(v); }
            0x1E => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                let v = rr!(self, false, d8);
                self.store(bus, self.hl, v)?;
            }

            0x20 => { let v = sla!(self, self.b()); self.set_b(v); }
            0x21 => { let v = sla!(self, self.c()); self.set_c(v); }
            0x22 => { let v = sla!(self, self.d()); self.set_d(v); }
            0x23 => { let v = sla!(self, self.e()); self.set_e(v); }
            0x24 => { let v = sla!(self, self.h()); self.set_h(v); }
            0x25 => { let v = sla!(self, self.l()); self.set_l(v); }
            0x27 => { let v = sla!(self, self.a()); self.set_a(v); }
            0x26 => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                let v = sla!(self, d8);
                self.store(bus, self.hl, v)?;
            }

            0x28 => { let v = sra!(self, self.b()); self.set_b(v); }
            0x29 => { let v = sra!(self, self.c()); self.set_c(v); }
            0x2A => { let v = sra!(self, self.d()); self.set_d(v); }
            0x2B => { let v = sra!(self, self.e()); self.set_e(v); }
            0x2C => { let v = sra!(self, self.h()); self.set_h(v); }
            0x2D => { let v = sra!(self, self.l()); self.set_l(v); }
            0x2F => { let v = sra!(self, self.a()); self.set_a(v); }
            0x2E => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                let v = sra!(self, d8);
                self.store(bus, self.hl, v)?;
            }

            0x30 => { let v = swap!(self, self.b()); self.set_b(v); }
            0x31 => { let v = swap!(self, self.c()); self.set_c(v); }
            0x32 => { let v = swap!(self, self.d()); self.set_d(v); }
            0x33 => { let v = swap!(self, self.e()); self.set_e(v); }
            0x34 => { let v = swap!(self, self.h()); self.set_h(v); }
            0x35 => { let v = swap!(self, self.l()); self.set_l(v); }
            0x37 => { let v = swap!(self, self.a()); self.set_a(v); }
            0x36 => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                let v = swap!(self, d8);
                self.store(bus, self.hl, v)?;
            }

            0x38 => { let v = srl!(self, self.b()); self.set_b(v); }
            0x39 => { let v = srl!(self, self.c()); self.set_c(v); }
            0x3A => { let v = srl!(self, self.d()); self.set_d(v); }
            0x3B => { let v = srl!(self, self.e()); self.set_e(v); }
            0x3C => { let v = srl!(self, self.h()); self.set_h(v); }
            0x3D => { let v = srl!(self, self.l()); self.set_l(v); }
            0x3F => { let v = srl!(self, self.a()); self.set_a(v); }
            0x3E => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                let v = srl!(self, d8);
                self.store(bus, self.hl, v)?;
            }

            0x40 => bit!(self, 0, self.b()),
            0x41 => bit!(self, 0, self.c()),
            0x42 => bit!(self, 0, self.d()),
            0x43 => bit!(self, 0, self.e()),
            0x44 => bit!(self, 0, self.h()),
            0x45 => bit!(self, 0, self.l()),
            0x47 => bit!(self, 0, self.a()),
            0x46 => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                bit!(self, 0, d8);
            }

            0x48 => bit!(self, 1, self.b()),
            0x49 => bit!(self, 1, self.c()),
            0x4A => bit!(self, 1, self.d()),
            0x4B => bit!(self, 1, self.e()),
            0x4C => bit!(self, 1, self.h()),
            0x4D => bit!(self, 1, self.l()),
            0x4F => bit!(self, 1, self.a()),
            0x4E => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                bit!(self, 1, d8);
            }

            0x50 => bit!(self, 2, self.b()),
            0x51 => bit!(self, 2, self.c()),
            0x52 => bit!(self, 2, self.d()),
            0x53 => bit!(self, 2, self.e()),
            0x54 => bit!(self, 2, self.h()),
            0x55 => bit!(self, 2, self.l()),
            0x57 => bit!(self, 2, self.a()),
            0x56 => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                bit!(self, 2, d8);
            }

            0x58 => bit!(self, 3, self.b()),
            0x59 => bit!(self, 3, self.c()),
            0x5A => bit!(self, 3, self.d()),
            0x5B => bit!(self, 3, self.e()),
            0x5C => bit!(self, 3, self.h()),
            0x5D => bit!(self, 3, self.l()),
            0x5F => bit!(self, 3, self.a()),
            0x5E => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                bit!(self, 3, d8);
            }

            0x60 => bit!(self, 4, self.b()),
            0x61 => bit!(self, 4, self.c()),
            0x62 => bit!(self, 4, self.d()),
            0x63 => bit!(self, 4, self.e()),
            0x64 => bit!(self, 4, self.h()),
            0x65 => bit!(self, 4, self.l()),
            0x67 => bit!(self, 4, self.a()),
            0x66 => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                bit!(self, 4, d8);
            }

            0x68 => bit!(self, 5, self.b()),
            0x69 => bit!(self, 5, self.c()),
            0x6A => bit!(self, 5, self.d()),
            0x6B => bit!(self, 5, self.e()),
            0x6C => bit!(self, 5, self.h()),
            0x6D => bit!(self, 5, self.l()),
            0x6F => bit!(self, 5, self.a()),
            0x6E => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                bit!(self, 5, d8);
            }

            0x70 => bit!(self, 6, self.b()),
            0x71 => bit!(self, 6, self.c()),
            0x72 => bit!(self, 6, self.d()),
            0x73 => bit!(self, 6, self.e()),
            0x74 => bit!(self, 6, self.h()),
            0x75 => bit!(self, 6, self.l()),
            0x77 => bit!(self, 6, self.a()),
            0x76 => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                bit!(self, 6, d8);
            }

            0x78 => bit!(self, 7, self.b()),
            0x79 => bit!(self, 7, self.c()),
            0x7A => bit!(self, 7, self.d()),
            0x7B => bit!(self, 7, self.e()),
            0x7C => bit!(self, 7, self.h()),
            0x7D => bit!(self, 7, self.l()),
            0x7F => bit!(self, 7, self.a()),
            0x7E => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                bit!(self, 7, d8);
            }

            0x80 => self.set_b(res!(0, self.b())),
            0x81 => self.set_c(res!(0, self.c())),
            0x82 => self.set_d(res!(0, self.d())),
            0x83 => self.set_e(res!(0, self.e())),
            0x84 => self.set_h(res!(0, self.h())),
            0x85 => self.set_l(res!(0, self.l())),
            0x87 => self.set_a(res!(0, self.a())),
            0x86 => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                let v = res!(0, d8);
                self.store(bus, self.hl, v)?;
            }

            0x88 => self.set_b(res!(1, self.b())),
            0x89 => self.set_c(res!(1, self.c())),
            0x8A => self.set_d(res!(1, self.d())),
            0x8B => self.set_e(res!(1, self.e())),
            0x8C => self.set_h(res!(1, self.h())),
            0x8D => self.set_l(res!(1, self.l())),
            0x8F => self.set_a(res!(1, self.a())),
            0x8E => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                let v = res!(1, d8);
                self.store(bus, self.hl, v)?;
            }

            0x90 => self.set_b(res!(2, self.b())),
            0x91 => self.set_c(res!(2, self.c())),
            0x92 => self.set_d(res!(2, self.d())),
            0x93 => self.set_e(res!(2, self.e())),
            0x94 => self.set_h(res!(2, self.h())),
            0x95 => self.set_l(res!(2, self.l())),
            0x97 => self.set_a(res!(2, self.a())),
            0x96 => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                let v = res!(2, d8);
                self.store(bus, self.hl, v)?;
            }

            0x98 => self.set_b(res!(3, self.b())),
            0x99 => self.set_c(res!(3, self.c())),
            0x9A => self.set_d(res!(3, self.d())),
            0x9B => self.set_e(res!(3, self.e())),
            0x9C => self.set_h(res!(3, self.h())),
            0x9D => self.set_l(res!(3, self.l())),
            0x9F => self.set_a(res!(3, self.a())),
            0x9E => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                let v = res!(3, d8);
                self.store(bus, self.hl, v)?;
            }

            0xA0 => self.set_b(res!(4, self.b())),
            0xA1 => self.set_c(res!(4, self.c())),
            0xA2 => self.set_d(res!(4, self.d())),
            0xA3 => self.set_e(res!(4, self.e())),
            0xA4 => self.set_h(res!(4, self.h())),
            0xA5 => self.set_l(res!(4, self.l())),
            0xA7 => self.set_a(res!(4, self.a())),
            0xA6 => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                let v = res!(4, d8);
                self.store(bus, self.hl, v)?;
            }

            0xA8 => self.set_b(res!(5, self.b())),
            0xA9 => self.set_c(res!(5, self.c())),
            0xAA => self.set_d(res!(5, self.d())),
            0xAB => self.set_e(res!(5, self.e())),
            0xAC => self.set_h(res!(5, self.h())),
            0xAD => self.set_l(res!(5, self.l())),
            0xAF => self.set_a(res!(5, self.a())),
            0xAE => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                let v = res!(5, d8);
                self.store(bus, self.hl, v)?;
            }

            0xB0 => self.set_b(res!(6, self.b())),
            0xB1 => self.set_c(res!(6, self.c())),
            0xB2 => self.set_d(res!(6, self.d())),
            0xB3 => self.set_e(res!(6, self.e())),
            0xB4 => self.set_h(res!(6, self.h())),
            0xB5 => self.set_l(res!(6, self.l())),
            0xB7 => self.set_a(res!(6, self.a())),
            0xB6 => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                let v = res!(6, d8);
                self.store(bus, self.hl, v)?;
            }

            0xB8 => self.set_b(res!(7, self.b())),
            0xB9 => self.set_c(res!(7, self.c())),
            0xBA => self.set_d(res!(7, self.d())),
            0xBB => self.set_e(res!(7, self.e())),
            0xBC => self.set_h(res!(7, self.h())),
            0xBD => self.set_l(res!(7, self.l())),
            0xBF => self.set_a(res!(7, self.a())),
            0xBE => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                let v = res!(7, d8);
                self.store(bus, self.hl, v)?;
            }

            0xC0 => self.set_b(set!(0, self.b())),
            0xC1 => self.set_c(set!(0, self.c())),
            0xC2 => self.set_d(set!(0, self.d())),
            0xC3 => self.set_e(set!(0, self.e())),
            0xC4 => self.set_h(set!(0, self.h())),
            0xC5 => self.set_l(set!(0, self.l())),
            0xC7 => self.set_a(set!(0, self.a())),
            0xC6 => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                let v = set!(0, d8);
                self.store(bus, self.hl, v)?;
            }

            0xC8 => self.set_b(set!(1, self.b())),
            0xC9 => self.set_c(set!(1, self.c())),
            0xCA => self.set_d(set!(1, self.d())),
            0xCB => self.set_e(set!(1, self.e())),
            0xCC => self.set_h(set!(1, self.h())),
            0xCD => self.set_l(set!(1, self.l())),
            0xCF => self.set_a(set!(1, self.a())),
            0xCE => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                let v = set!(1, d8);
                self.store(bus, self.hl, v)?;
            }

            0xD0 => self.set_b(set!(2, self.b())),
            0xD1 => self.set_c(set!(2, self.c())),
            0xD2 => self.set_d(set!(2, self.d())),
            0xD3 => self.set_e(set!(2, self.e())),
            0xD4 => self.set_h(set!(2, self.h())),
            0xD5 => self.set_l(set!(2, self.l())),
            0xD7 => self.set_a(set!(2, self.a())),
            0xD6 => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                let v = set!(2, d8);
                self.store(bus, self.hl, v)?;
            }

            0xD8 => self.set_b(set!(3, self.b())),
            0xD9 => self.set_c(set!(3, self.c())),
            0xDA => self.set_d(set!(3, self.d())),
            0xDB => self.set_e(set!(3, self.e())),
            0xDC => self.set_h(set!(3, self.h())),
            0xDD => self.set_l(set!(3, self.l())),
            0xDF => self.set_a(set!(3, self.a())),
            0xDE => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                let v = set!(3, d8);
                self.store(bus, self.hl, v)?;
            }

            0xE0 => self.set_b(set!(4, self.b())),
            0xE1 => self.set_c(set!(4, self.c())),
            0xE2 => self.set_d(set!(4, self.d())),
            0xE3 => self.set_e(set!(4, self.e())),
            0xE4 => self.set_h(set!(4, self.h())),
            0xE5 => self.set_l(set!(4, self.l())),
            0xE7 => self.set_a(set!(4, self.a())),
            0xE6 => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                let v = set!(4, d8);
                self.store(bus, self.hl, v)?;
            }

            0xE8 => self.set_b(set!(5, self.b())),
            0xE9 => self.set_c(set!(5, self.c())),
            0xEA => self.set_d(set!(5, self.d())),
            0xEB => self.set_e(set!(5, self.e())),
            0xEC => self.set_h(set!(5, self.h())),
            0xED => self.set_l(set!(5, self.l())),
            0xEF => self.set_a(set!(5, self.a())),
            0xEE => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                let v = set!(5, d8);
                self.store(bus, self.hl, v)?;
            }

            0xF0 => self.set_b(set!(6, self.b())),
            0xF1 => self.set_c(set!(6, self.c())),
            0xF2 => self.set_d(set!(6, self.d())),
            0xF3 => self.set_e(set!(6, self.e())),
            0xF4 => self.set_h(set!(6, self.h())),
            0xF5 => self.set_l(set!(6, self.l())),
            0xF7 => self.set_a(set!(6, self.a())),
            0xF6 => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                let v = set!(6, d8);
                self.store(bus, self.hl, v)?;
            }

            0xF8 => self.set_b(set!(7, self.b())),
            0xF9 => self.set_c(set!(7, self.c())),
            0xFA => self.set_d(set!(7, self.d())),
            0xFB => self.set_e(set!(7, self.e())),
            0xFC => self.set_h(set!(7, self.h())),
            0xFD => self.set_l(set!(7, self.l())),
            0xFF => self.set_a(set!(7, self.a())),
            0xFE => {
                let d8: u8 = self.fetch(bus, self.hl)?;
                let v = set!(7, d8);
                self.store(bus, self.hl, v)?;
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::super::dbg;
    use super::super::mem::{MemR, MemSize, MemW};
    use super::*;

    impl<'a> MemR for &'a mut [u8] {
        fn read<T: MemSize>(&self, addr: u16) -> Result<T, dbg::TraceEvent> {
            T::read_le(&self[addr as usize..])
        }
    }

    impl<'a> MemW for &'a mut [u8] {
        fn write<T: MemSize>(&mut self, addr: u16, val: T) -> Result<(), dbg::TraceEvent> {
            T::write_le(&mut self[addr as usize..], val)
        }
    }

    impl<'a> MemRW for &'a mut [u8] {}

    fn check_opcode(cpu: Option<CPU>, opcode: u8, exp_pc: u16, exp_clk: u64) {
        let mut cpu = cpu.unwrap_or_else(CPU::new);
        cpu.exec(&mut (&mut [opcode; 0x10000][..]))
            .expect("unexpected trace event");

        assert!(
            cpu.pc == exp_pc,
            "wrong PC for {:02X}: {:04X} != {:04X}",
            opcode,
            cpu.pc,
            exp_pc
        );
        assert!(
            cpu.clk == exp_clk,
            "wrong clk for {:02X}: {} != {}",
            opcode,
            cpu.clk,
            exp_clk
        );
    }

    #[test]
    fn opcode_misc_timings() {
        [0x00u8, 0x10, 0x27, 0x76, 0xF3, 0xFB]
            .iter()
            .for_each(|&opc| check_opcode(None, opc, 1, 4));
    }

    #[test]
    fn opcode_jump_timings() {
        let cpu_with_zf = || {
            let mut cpu = CPU::new();
            cpu.set_zf(true);
            cpu
        };

        let cpu_with_cy = || {
            let mut cpu = CPU::new();
            cpu.set_cy(true);
            cpu
        };

        check_opcode(None, 0x18, 2 + 0x18, 12);
        check_opcode(None, 0xC3, 0xC3C3, 16);
        check_opcode(None, 0xCD, 0xCDCD, 24);
        check_opcode(None, 0xE9, 0, 4);

        check_opcode(None, 0x20, 2 + 0x20, 12);
        check_opcode(None, 0x28, 2, 8);
        check_opcode(Some(cpu_with_zf()), 0x20, 2, 8);
        check_opcode(Some(cpu_with_zf()), 0x28, 2 + 0x28, 12);

        check_opcode(None, 0x30, 2 + 0x30, 12);
        check_opcode(None, 0x38, 2, 8);
        check_opcode(Some(cpu_with_cy()), 0x30, 2, 8);
        check_opcode(Some(cpu_with_cy()), 0x38, 2 + 0x38, 12);

        check_opcode(None, 0xC2, 0xC2C2, 16);
        check_opcode(None, 0xCA, 3, 12);
        check_opcode(Some(cpu_with_zf()), 0xC2, 3, 12);
        check_opcode(Some(cpu_with_zf()), 0xCA, 0xCACA, 16);

        check_opcode(None, 0xD2, 0xD2D2, 16);
        check_opcode(None, 0xDA, 3, 12);
        check_opcode(Some(cpu_with_cy()), 0xD2, 3, 12);
        check_opcode(Some(cpu_with_cy()), 0xDA, 0xDADA, 16);

        check_opcode(None, 0xC4, 0xC4C4, 24);
        check_opcode(None, 0xCC, 3, 12);
        check_opcode(Some(cpu_with_zf()), 0xC4, 3, 12);
        check_opcode(Some(cpu_with_zf()), 0xCC, 0xCCCC, 24);

        check_opcode(None, 0xD4, 0xD4D4, 24);
        check_opcode(None, 0xDC, 3, 12);
        check_opcode(Some(cpu_with_cy()), 0xD4, 3, 12);
        check_opcode(Some(cpu_with_cy()), 0xDC, 0xDCDC, 24);

        check_opcode(None, 0xC7, 0x00, 16);
        check_opcode(None, 0xD7, 0x10, 16);
        check_opcode(None, 0xE7, 0x20, 16);
        check_opcode(None, 0xF7, 0x30, 16);

        check_opcode(None, 0xCF, 0x08, 16);
        check_opcode(None, 0xDF, 0x18, 16);
        check_opcode(None, 0xEF, 0x28, 16);
        check_opcode(None, 0xFF, 0x38, 16);

        check_opcode(None, 0xC0, 0xC0C0, 20);
        check_opcode(None, 0xC8, 1, 8);
        check_opcode(Some(cpu_with_zf()), 0xC0, 1, 8);
        check_opcode(Some(cpu_with_zf()), 0xC8, 0xC8C8, 20);

        check_opcode(None, 0xD0, 0xD0D0, 20);
        check_opcode(None, 0xD8, 1, 8);
        check_opcode(Some(cpu_with_cy()), 0xD0, 1, 8);
        check_opcode(Some(cpu_with_cy()), 0xD8, 0xD8D8, 20);

        check_opcode(None, 0xC9, 0xC9C9, 16);
        check_opcode(None, 0xD9, 0xD9D9, 16);
    }

    #[test]
    fn opcode_ld8_timings() {
        (0x40..=0x6F)
            .for_each(|opc| check_opcode(None, opc, 1, if (opc & 0x07) != 0x6 { 4 } else { 8 }));

        [0x70u8, 0x71, 0x72, 0x73, 0x74, 0x75, 0x77]
            .iter()
            .for_each(|&opc| check_opcode(None, opc, 1, 8));

        (0x78..=0x7F)
            .for_each(|opc| check_opcode(None, opc, 1, if (opc & 0x07) != 0x6 { 4 } else { 8 }));

        [0x02u8, 0x12, 0x22, 0x32, 0x0A, 0x1A, 0x2A, 0x3A]
            .iter()
            .for_each(|&opc| check_opcode(None, opc, 1, 8));

        [0x06u8, 0x16, 0x26, 0x0E, 0x1E, 0x2E, 0x3E]
            .iter()
            .for_each(|&opc| check_opcode(None, opc, 2, 8));

        [0x36u8, 0xE0, 0xF0]
            .iter()
            .for_each(|&opc| check_opcode(None, opc, 2, 12));

        check_opcode(None, 0xE2, 1, 8);
        check_opcode(None, 0xF2, 1, 8);
        check_opcode(None, 0xEA, 3, 16);
        check_opcode(None, 0xFA, 3, 16);
    }

    #[test]
    fn opcode_ld16_timings() {
        [0x01u8, 0x11, 0x21, 0x31]
            .iter()
            .for_each(|&opc| check_opcode(None, opc, 3, 12));

        [0xC1u8, 0xD1, 0xE1, 0xF1]
            .iter()
            .for_each(|&opc| check_opcode(None, opc, 1, 12));

        [0xC5u8, 0xD5, 0xE5, 0xF5]
            .iter()
            .for_each(|&opc| check_opcode(None, opc, 1, 16));

        check_opcode(None, 0x08, 3, 20);
        check_opcode(None, 0xF8, 2, 12);
        check_opcode(None, 0xF9, 1, 8);
    }

    #[test]
    fn opcode_alu8_timings() {
        [
            0x04u8, 0x05, 0x0C, 0x0D, 0x14u8, 0x15, 0x1C, 0x1D, 0x24u8, 0x25, 0x2C, 0x2D, 0x3C,
            0x3D, 0x37, 0x2F, 0x3F,
        ]
        .iter()
        .for_each(|&opc| check_opcode(None, opc, 1, 4));

        [0x34u8, 0x35]
            .iter()
            .for_each(|&opc| check_opcode(None, opc, 1, 12));

        (0x80..=0xBF)
            .for_each(|opc| check_opcode(None, opc, 1, if (opc & 0x07) != 0x6 { 4 } else { 8 }));

        [0xC6u8, 0xCE, 0xD6, 0xDE, 0xE6, 0xEE, 0xF6, 0xFE]
            .iter()
            .for_each(|&opc| check_opcode(None, opc, 2, 8));
    }

    #[test]
    fn opcode_alu16_timings() {
        [
            0x03u8, 0x13, 0x23, 0x33, 0x0B, 0x1B, 0x2B, 0x3B, 0x09, 0x19, 0x29, 0x39,
        ]
        .iter()
        .for_each(|&opc| check_opcode(None, opc, 1, 8));

        check_opcode(None, 0xE8, 2, 16);
    }

    #[test]
    fn opcode_bitwise8_timings() {
        let check_cb_opcode = |opcode, exp_pc, exp_clk| {
            let mut cpu = CPU::new();
            cpu.exec(&mut (&mut [0xCBu8, opcode][..]))
                .expect("unexpected trace event");

            assert!(
                cpu.pc == exp_pc,
                "[CB] wrong PC for {:02X}: {:04X} != {:04X}",
                opcode,
                cpu.pc,
                exp_pc
            );
            assert!(
                cpu.clk == exp_clk,
                "[CB] wrong clk for {:02X}: {} != {}",
                opcode,
                cpu.clk,
                exp_clk
            );
        };

        [0x07u8, 0x0F, 0x17, 0x1F]
            .iter()
            .for_each(|&opc| check_opcode(None, opc, 1, 4));

        (0x00u8..=0xFF).for_each(|opc| {
            check_cb_opcode(
                opc,
                2,
                if (opc & 0x7) != 0x6 {
                    8
                } else {
                    match opc & 0xF0 {
                        0x40..=0x70 => 12,
                        _ => 16,
                    }
                },
            )
        });
    }
}
