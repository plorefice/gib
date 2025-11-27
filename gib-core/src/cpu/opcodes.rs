use crate::{
    cpu::{Cpu, MemoryAddressing::*, OpcodeInfo, OperandLocation::*, WritebackOp},
    dbg,
};

macro_rules! jp {
    ($cpu:ident, $cond:expr, $abs:expr) => {{
        if $cond {
            $cpu.pc = $abs;
            $cpu.branch_taken = true;
        }
    }};
}

macro_rules! jr {
    ($cpu:ident, $cond:expr, $offset:expr) => {
        jp!(
            $cpu,
            $cond,
            (i32::from($cpu.pc) + i32::from($offset)) as u16
        )
    };
}

macro_rules! call {
    ($cpu:ident, $cond:expr, $to:expr) => {{
        if $cond {
            $cpu.write_op = Some(WritebackOp::Push($cpu.pc));
            $cpu.pc = $to;
            $cpu.branch_taken = true;
            $cpu.call_stack.push($cpu.pc);
        }
    }};
}

macro_rules! ret {
    ($cpu:ident, $cond:expr) => {{
        if $cond {
            $cpu.call_stack.pop();
            $cpu.write_op = Some(WritebackOp::Return);
            $cpu.branch_taken = true;
        }
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

macro_rules! and { ($cpu:ident, $rhs:expr) => { logical!($cpu, &, $rhs, 0, 1, 0) }; }
macro_rules! xor { ($cpu:ident, $rhs:expr) => { logical!($cpu, ^, $rhs, 0, 0, 0) }; }
macro_rules! or  { ($cpu:ident, $rhs:expr) => { logical!($cpu, |, $rhs, 0, 0, 0) }; }

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
        let res = $v.rotate_left(4);

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

impl Cpu {
    #[rustfmt::skip]
    pub fn op(&mut self) -> Result<(), dbg::TraceEvent> {
        match self.opcode {
            /*
             * Misc/control instructions
             */
            0x00 => (),

            0x10 | 0x76 => self.halted.load(true),

            0xF3 => self.intr_enabled.reset(false),
            0xFB => self.intr_enabled.load(true),

            /*
             * Jump/calls
             */
            0x20 => jr!(self, !self.zf(), self.operand as i8),
            0x30 => jr!(self, !self.cy(), self.operand as i8),
            0x28 => jr!(self, self.zf(),  self.operand as i8),
            0x38 => jr!(self, self.cy(),  self.operand as i8),
            0x18 => jr!(self, true,       self.operand as i8),

            0xC2 => jp!(self, !self.zf(), self.operand),
            0xD2 => jp!(self, !self.cy(), self.operand),
            0xCA => jp!(self, self.zf(),  self.operand),
            0xDA => jp!(self, self.cy(),  self.operand),
            0xC3 => jp!(self, true,       self.operand),

            0xE9 => jp!(self, true, self.hl),

            0xC4 => call!(self, !self.zf(), self.operand),
            0xD4 => call!(self, !self.cy(), self.operand),
            0xCC => call!(self, self.zf(),  self.operand),
            0xDC => call!(self, self.cy(),  self.operand),
            0xCD => call!(self, true,       self.operand),

            0xC0 => ret!(self, !self.zf()),
            0xD0 => ret!(self, !self.cy()),
            0xC8 => ret!(self, self.zf()),
            0xD8 => ret!(self, self.cy()),

            0xC9 => ret!(self, true),
            0xD9 => { ret!(self, true); self.intr_enabled.reset(true); }

            0xC7 => call!(self, true, 0x00),
            0xCF => call!(self, true, 0x08),
            0xD7 => call!(self, true, 0x10),
            0xDF => call!(self, true, 0x18),
            0xE7 => call!(self, true, 0x20),
            0xEF => call!(self, true, 0x28),
            0xF7 => call!(self, true, 0x30),
            0xFF => call!(self, true, 0x38),

            /*
             * 8bit load/store/move instructions
             */
            0x02 => self.write_op = Some(WritebackOp::Write8(self.bc, self.a())),
            0x12 => self.write_op = Some(WritebackOp::Write8(self.de, self.a())),
            0x22 => { self.write_op = Some(WritebackOp::Write8(self.hl, self.a())); self.hl += 1; }
            0x32 => { self.write_op = Some(WritebackOp::Write8(self.hl, self.a())); self.hl -= 1; }

            0x0A => self.set_a(self.operand as u8),
            0x1A => self.set_a(self.operand as u8),
            0x2A => { self.set_a(self.operand as u8); self.hl += 1; }
            0x3A => { self.set_a(self.operand as u8); self.hl -= 1; }

            0x06 => self.set_b(self.operand as u8),
            0x16 => self.set_d(self.operand as u8),
            0x26 => self.set_h(self.operand as u8),
            0x36 => self.write_op = Some(WritebackOp::Write8(self.hl, self.operand as u8)),
            0x0E => self.set_c(self.operand as u8),
            0x1E => self.set_e(self.operand as u8),
            0x2E => self.set_l(self.operand as u8),
            0x3E => self.set_a(self.operand as u8),

            0x40 => self.set_b(self.b()),
            0x41 => self.set_b(self.c()),
            0x42 => self.set_b(self.d()),
            0x43 => self.set_b(self.e()),
            0x44 => self.set_b(self.h()),
            0x45 => self.set_b(self.l()),
            0x46 => self.set_b(self.operand as u8),
            0x47 => self.set_b(self.a()),
            0x48 => self.set_c(self.b()),
            0x49 => self.set_c(self.c()),
            0x4A => self.set_c(self.d()),
            0x4B => self.set_c(self.e()),
            0x4C => self.set_c(self.h()),
            0x4D => self.set_c(self.l()),
            0x4E => self.set_c(self.operand as u8),
            0x4F => self.set_c(self.a()),
            0x50 => self.set_d(self.b()),
            0x51 => self.set_d(self.c()),
            0x52 => self.set_d(self.d()),
            0x53 => self.set_d(self.e()),
            0x54 => self.set_d(self.h()),
            0x55 => self.set_d(self.l()),
            0x56 => self.set_d(self.operand as u8),
            0x57 => self.set_d(self.a()),
            0x58 => self.set_e(self.b()),
            0x59 => self.set_e(self.c()),
            0x5A => self.set_e(self.d()),
            0x5B => self.set_e(self.e()),
            0x5C => self.set_e(self.h()),
            0x5D => self.set_e(self.l()),
            0x5E => self.set_e(self.operand as u8),
            0x5F => self.set_e(self.a()),
            0x60 => self.set_h(self.b()),
            0x61 => self.set_h(self.c()),
            0x62 => self.set_h(self.d()),
            0x63 => self.set_h(self.e()),
            0x64 => self.set_h(self.h()),
            0x65 => self.set_h(self.l()),
            0x66 => self.set_h(self.operand as u8),
            0x67 => self.set_h(self.a()),
            0x68 => self.set_l(self.b()),
            0x69 => self.set_l(self.c()),
            0x6A => self.set_l(self.d()),
            0x6B => self.set_l(self.e()),
            0x6C => self.set_l(self.h()),
            0x6D => self.set_l(self.l()),
            0x6E => self.set_l(self.operand as u8),
            0x6F => self.set_l(self.a()),
            0x78 => self.set_a(self.b()),
            0x79 => self.set_a(self.c()),
            0x7A => self.set_a(self.d()),
            0x7B => self.set_a(self.e()),
            0x7C => self.set_a(self.h()),
            0x7D => self.set_a(self.l()),
            0x7E => self.set_a(self.operand as u8),
            0x7F => self.set_a(self.a()),

            0x70 => self.write_op = Some(WritebackOp::Write8(self.hl, self.b())),
            0x71 => self.write_op = Some(WritebackOp::Write8(self.hl, self.c())),
            0x72 => self.write_op = Some(WritebackOp::Write8(self.hl, self.d())),
            0x73 => self.write_op = Some(WritebackOp::Write8(self.hl, self.e())),
            0x74 => self.write_op = Some(WritebackOp::Write8(self.hl, self.h())),
            0x75 => self.write_op = Some(WritebackOp::Write8(self.hl, self.l())),
            0x77 => self.write_op = Some(WritebackOp::Write8(self.hl, self.a())),

            0xE0 => self.write_op = Some(WritebackOp::Write8(0xFF00 + self.operand, self.a())),
            0xE2 => self.write_op = Some(WritebackOp::Write8(0xFF00 + u16::from(self.c()), self.a())),
            0xEA => self.write_op = Some(WritebackOp::Write8(self.operand, self.a())),

            0xF0 | 0xF2 | 0xFA => self.set_a(self.operand as u8),

            /*
             * 16bit load/store/move instructions
             */
            0x01 => self.bc = self.operand,
            0x11 => self.de = self.operand,
            0x21 => self.hl = self.operand,
            0x31 => self.sp = self.operand,

            0xC1 => self.bc = self.operand,
            0xD1 => self.de = self.operand,
            0xE1 => self.hl = self.operand,
            0xF1 => self.af = self.operand & 0xFFF0,

            0xC5 => self.write_op = Some(WritebackOp::Push(self.bc)),
            0xD5 => self.write_op = Some(WritebackOp::Push(self.de)),
            0xE5 => self.write_op = Some(WritebackOp::Push(self.hl)),
            0xF5 => self.write_op = Some(WritebackOp::Push(self.af)),

            0x08 => self.write_op = Some(WritebackOp::Write16(self.operand, self.sp)),
            0xF9 => self.sp = self.hl,

            0xF8 => self.hl = addi16!(self, self.sp, self.operand as i8),

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
                let v = inc!(self, self.operand as u8);
                self.write_op = Some(WritebackOp::Write8(self.hl, v));
            }

            0x05 => { let v = dec!(self, self.b()); self.set_b(v); }
            0x15 => { let v = dec!(self, self.d()); self.set_d(v); }
            0x25 => { let v = dec!(self, self.h()); self.set_h(v); }
            0x0D => { let v = dec!(self, self.c()); self.set_c(v); }
            0x1D => { let v = dec!(self, self.e()); self.set_e(v); }
            0x2D => { let v = dec!(self, self.l()); self.set_l(v); }
            0x3D => { let v = dec!(self, self.a()); self.set_a(v); }
            0x35 => {
                let v = dec!(self, self.operand as u8);
                self.write_op = Some(WritebackOp::Write8(self.hl, v));
            }

            0x80 => add!(self, self.b(), 0u8),
            0x81 => add!(self, self.c(), 0u8),
            0x82 => add!(self, self.d(), 0u8),
            0x83 => add!(self, self.e(), 0u8),
            0x84 => add!(self, self.h(), 0u8),
            0x85 => add!(self, self.l(), 0u8),
            0x87 => add!(self, self.a(), 0u8),
            0x86 | 0xC6 => add!(self, self.operand as u8, 0u8),

            0x88 => add!(self, self.b(), self.cy() as u8),
            0x89 => add!(self, self.c(), self.cy() as u8),
            0x8A => add!(self, self.d(), self.cy() as u8),
            0x8B => add!(self, self.e(), self.cy() as u8),
            0x8C => add!(self, self.h(), self.cy() as u8),
            0x8D => add!(self, self.l(), self.cy() as u8),
            0x8F => add!(self, self.a(), self.cy() as u8),
            0x8E | 0xCE => add!(self, self.operand as u8, self.cy() as u8),

            0x90 => sub!(self, self.b(), 0u8),
            0x91 => sub!(self, self.c(), 0u8),
            0x92 => sub!(self, self.d(), 0u8),
            0x93 => sub!(self, self.e(), 0u8),
            0x94 => sub!(self, self.h(), 0u8),
            0x95 => sub!(self, self.l(), 0u8),
            0x97 => sub!(self, self.a(), 0u8),
            0x96 | 0xD6 => sub!(self, self.operand as u8, 0u8),

            0x98 => sub!(self, self.b(), self.cy() as u8),
            0x99 => sub!(self, self.c(), self.cy() as u8),
            0x9A => sub!(self, self.d(), self.cy() as u8),
            0x9B => sub!(self, self.e(), self.cy() as u8),
            0x9C => sub!(self, self.h(), self.cy() as u8),
            0x9D => sub!(self, self.l(), self.cy() as u8),
            0x9F => sub!(self, self.a(), self.cy() as u8),
            0x9E | 0xDE => sub!(self, self.operand as u8, self.cy() as u8),

            0xA0 => and!(self, self.b()),
            0xA1 => and!(self, self.c()),
            0xA2 => and!(self, self.d()),
            0xA3 => and!(self, self.e()),
            0xA4 => and!(self, self.h()),
            0xA5 => and!(self, self.l()),
            0xA7 => and!(self, self.a()),
            0xA6 | 0xE6 => and!(self, self.operand as u8),

            0xA8 => xor!(self, self.b()),
            0xA9 => xor!(self, self.c()),
            0xAA => xor!(self, self.d()),
            0xAB => xor!(self, self.e()),
            0xAC => xor!(self, self.h()),
            0xAD => xor!(self, self.l()),
            0xAF => xor!(self, self.a()),
            0xAE | 0xEE => xor!(self, self.operand as u8),

            0xB0 => or!(self, self.b()),
            0xB1 => or!(self, self.c()),
            0xB2 => or!(self, self.d()),
            0xB3 => or!(self, self.e()),
            0xB4 => or!(self, self.h()),
            0xB5 => or!(self, self.l()),
            0xB7 => or!(self, self.a()),
            0xB6 | 0xF6 => or!(self, self.operand as u8),

            0xB8 => cmp!(self, self.a(), self.b()),
            0xB9 => cmp!(self, self.a(), self.c()),
            0xBA => cmp!(self, self.a(), self.d()),
            0xBB => cmp!(self, self.a(), self.e()),
            0xBC => cmp!(self, self.a(), self.h()),
            0xBD => cmp!(self, self.a(), self.l()),
            0xBF => cmp!(self, self.a(), self.a()),
            0xBE | 0xFE => cmp!(self, self.a(), self.operand as u8),

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
            0x03 => self.bc += 1,
            0x13 => self.de += 1,
            0x23 => self.hl += 1,
            0x33 => self.sp += 1,

            0x0B => self.bc -= 1,
            0x1B => self.de -= 1,
            0x2B => self.hl -= 1,
            0x3B => self.sp -= 1,

            0x09 => add16!(self, self.hl, self.bc),
            0x19 => add16!(self, self.hl, self.de),
            0x29 => add16!(self, self.hl, self.hl),
            0x39 => add16!(self, self.hl, self.sp),
            0xE8 => self.sp = addi16!(self, self.sp, self.operand as i8),

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
            0xCB | 0xD3 | 0xDB | 0xDD | 0xE3 | 0xE4 | 0xEB | 0xEC | 0xED | 0xF4 | 0xFC | 0xFD => {
                return Err(dbg::TraceEvent::IllegalInstructionFault(self.opcode));
            }
        };

        Ok(())
    }

    #[rustfmt::skip]
    pub fn op_cb(&mut self) -> Result<(), dbg::TraceEvent> {
        match self.opcode {
            0x00 => { let v = rl!(self, true, self.b()); self.set_b(v); }
            0x01 => { let v = rl!(self, true, self.c()); self.set_c(v); }
            0x02 => { let v = rl!(self, true, self.d()); self.set_d(v); }
            0x03 => { let v = rl!(self, true, self.e()); self.set_e(v); }
            0x04 => { let v = rl!(self, true, self.h()); self.set_h(v); }
            0x05 => { let v = rl!(self, true, self.l()); self.set_l(v); }
            0x07 => { let v = rl!(self, true, self.a()); self.set_a(v); }
            0x06 => {
                let v = rl!(self, true, self.operand as u8);
                self.write_op = Some(WritebackOp::Write8(self.hl, v));
            }

            0x08 => { let v = rr!(self, true, self.b()); self.set_b(v); }
            0x09 => { let v = rr!(self, true, self.c()); self.set_c(v); }
            0x0A => { let v = rr!(self, true, self.d()); self.set_d(v); }
            0x0B => { let v = rr!(self, true, self.e()); self.set_e(v); }
            0x0C => { let v = rr!(self, true, self.h()); self.set_h(v); }
            0x0D => { let v = rr!(self, true, self.l()); self.set_l(v); }
            0x0F => { let v = rr!(self, true, self.a()); self.set_a(v); }
            0x0E => {
                let v = rr!(self, true, self.operand as u8);
                self.write_op = Some(WritebackOp::Write8(self.hl, v));
            }

            0x10 => { let v = rl!(self, false, self.b()); self.set_b(v); }
            0x11 => { let v = rl!(self, false, self.c()); self.set_c(v); }
            0x12 => { let v = rl!(self, false, self.d()); self.set_d(v); }
            0x13 => { let v = rl!(self, false, self.e()); self.set_e(v); }
            0x14 => { let v = rl!(self, false, self.h()); self.set_h(v); }
            0x15 => { let v = rl!(self, false, self.l()); self.set_l(v); }
            0x17 => { let v = rl!(self, false, self.a()); self.set_a(v); }
            0x16 => {
                let v = rl!(self, false, self.operand as u8);
                self.write_op = Some(WritebackOp::Write8(self.hl, v));
            }

            0x18 => { let v = rr!(self, false, self.b()); self.set_b(v); }
            0x19 => { let v = rr!(self, false, self.c()); self.set_c(v); }
            0x1A => { let v = rr!(self, false, self.d()); self.set_d(v); }
            0x1B => { let v = rr!(self, false, self.e()); self.set_e(v); }
            0x1C => { let v = rr!(self, false, self.h()); self.set_h(v); }
            0x1D => { let v = rr!(self, false, self.l()); self.set_l(v); }
            0x1F => { let v = rr!(self, false, self.a()); self.set_a(v); }
            0x1E => {
                let v = rr!(self, false, self.operand as u8);
                self.write_op = Some(WritebackOp::Write8(self.hl, v));
            }

            0x20 => { let v = sla!(self, self.b()); self.set_b(v); }
            0x21 => { let v = sla!(self, self.c()); self.set_c(v); }
            0x22 => { let v = sla!(self, self.d()); self.set_d(v); }
            0x23 => { let v = sla!(self, self.e()); self.set_e(v); }
            0x24 => { let v = sla!(self, self.h()); self.set_h(v); }
            0x25 => { let v = sla!(self, self.l()); self.set_l(v); }
            0x27 => { let v = sla!(self, self.a()); self.set_a(v); }
            0x26 => {
                let v = sla!(self, self.operand as u8);
                self.write_op = Some(WritebackOp::Write8(self.hl, v));
            }

            0x28 => { let v = sra!(self, self.b()); self.set_b(v); }
            0x29 => { let v = sra!(self, self.c()); self.set_c(v); }
            0x2A => { let v = sra!(self, self.d()); self.set_d(v); }
            0x2B => { let v = sra!(self, self.e()); self.set_e(v); }
            0x2C => { let v = sra!(self, self.h()); self.set_h(v); }
            0x2D => { let v = sra!(self, self.l()); self.set_l(v); }
            0x2F => { let v = sra!(self, self.a()); self.set_a(v); }
            0x2E => {
                let v = sra!(self, self.operand as u8);
                self.write_op = Some(WritebackOp::Write8(self.hl, v));
            }

            0x30 => { let v = swap!(self, self.b()); self.set_b(v); }
            0x31 => { let v = swap!(self, self.c()); self.set_c(v); }
            0x32 => { let v = swap!(self, self.d()); self.set_d(v); }
            0x33 => { let v = swap!(self, self.e()); self.set_e(v); }
            0x34 => { let v = swap!(self, self.h()); self.set_h(v); }
            0x35 => { let v = swap!(self, self.l()); self.set_l(v); }
            0x37 => { let v = swap!(self, self.a()); self.set_a(v); }
            0x36 => {
                let v = swap!(self, self.operand as u8);
                self.write_op = Some(WritebackOp::Write8(self.hl, v));
            }

            0x38 => { let v = srl!(self, self.b()); self.set_b(v); }
            0x39 => { let v = srl!(self, self.c()); self.set_c(v); }
            0x3A => { let v = srl!(self, self.d()); self.set_d(v); }
            0x3B => { let v = srl!(self, self.e()); self.set_e(v); }
            0x3C => { let v = srl!(self, self.h()); self.set_h(v); }
            0x3D => { let v = srl!(self, self.l()); self.set_l(v); }
            0x3F => { let v = srl!(self, self.a()); self.set_a(v); }
            0x3E => {
                let v = srl!(self, self.operand as u8);
                self.write_op = Some(WritebackOp::Write8(self.hl, v));
            }

            0x40 => bit!(self, 0, self.b()),
            0x41 => bit!(self, 0, self.c()),
            0x42 => bit!(self, 0, self.d()),
            0x43 => bit!(self, 0, self.e()),
            0x44 => bit!(self, 0, self.h()),
            0x45 => bit!(self, 0, self.l()),
            0x47 => bit!(self, 0, self.a()),
            0x46 => bit!(self, 0, self.operand as u8),

            0x48 => bit!(self, 1, self.b()),
            0x49 => bit!(self, 1, self.c()),
            0x4A => bit!(self, 1, self.d()),
            0x4B => bit!(self, 1, self.e()),
            0x4C => bit!(self, 1, self.h()),
            0x4D => bit!(self, 1, self.l()),
            0x4F => bit!(self, 1, self.a()),
            0x4E => bit!(self, 1, self.operand as u8),

            0x50 => bit!(self, 2, self.b()),
            0x51 => bit!(self, 2, self.c()),
            0x52 => bit!(self, 2, self.d()),
            0x53 => bit!(self, 2, self.e()),
            0x54 => bit!(self, 2, self.h()),
            0x55 => bit!(self, 2, self.l()),
            0x57 => bit!(self, 2, self.a()),
            0x56 => bit!(self, 2, self.operand as u8),

            0x58 => bit!(self, 3, self.b()),
            0x59 => bit!(self, 3, self.c()),
            0x5A => bit!(self, 3, self.d()),
            0x5B => bit!(self, 3, self.e()),
            0x5C => bit!(self, 3, self.h()),
            0x5D => bit!(self, 3, self.l()),
            0x5F => bit!(self, 3, self.a()),
            0x5E => bit!(self, 3, self.operand as u8),

            0x60 => bit!(self, 4, self.b()),
            0x61 => bit!(self, 4, self.c()),
            0x62 => bit!(self, 4, self.d()),
            0x63 => bit!(self, 4, self.e()),
            0x64 => bit!(self, 4, self.h()),
            0x65 => bit!(self, 4, self.l()),
            0x67 => bit!(self, 4, self.a()),
            0x66 => bit!(self, 4, self.operand as u8),

            0x68 => bit!(self, 5, self.b()),
            0x69 => bit!(self, 5, self.c()),
            0x6A => bit!(self, 5, self.d()),
            0x6B => bit!(self, 5, self.e()),
            0x6C => bit!(self, 5, self.h()),
            0x6D => bit!(self, 5, self.l()),
            0x6F => bit!(self, 5, self.a()),
            0x6E => bit!(self, 5, self.operand as u8),

            0x70 => bit!(self, 6, self.b()),
            0x71 => bit!(self, 6, self.c()),
            0x72 => bit!(self, 6, self.d()),
            0x73 => bit!(self, 6, self.e()),
            0x74 => bit!(self, 6, self.h()),
            0x75 => bit!(self, 6, self.l()),
            0x77 => bit!(self, 6, self.a()),
            0x76 => bit!(self, 6, self.operand as u8),

            0x78 => bit!(self, 7, self.b()),
            0x79 => bit!(self, 7, self.c()),
            0x7A => bit!(self, 7, self.d()),
            0x7B => bit!(self, 7, self.e()),
            0x7C => bit!(self, 7, self.h()),
            0x7D => bit!(self, 7, self.l()),
            0x7F => bit!(self, 7, self.a()),
            0x7E => bit!(self, 7, self.operand as u8),

            0x80 => self.set_b(res!(0, self.b())),
            0x81 => self.set_c(res!(0, self.c())),
            0x82 => self.set_d(res!(0, self.d())),
            0x83 => self.set_e(res!(0, self.e())),
            0x84 => self.set_h(res!(0, self.h())),
            0x85 => self.set_l(res!(0, self.l())),
            0x87 => self.set_a(res!(0, self.a())),
            0x86 => {
                let v = res!(0, self.operand as u8);
                self.write_op = Some(WritebackOp::Write8(self.hl, v));
            }

            0x88 => self.set_b(res!(1, self.b())),
            0x89 => self.set_c(res!(1, self.c())),
            0x8A => self.set_d(res!(1, self.d())),
            0x8B => self.set_e(res!(1, self.e())),
            0x8C => self.set_h(res!(1, self.h())),
            0x8D => self.set_l(res!(1, self.l())),
            0x8F => self.set_a(res!(1, self.a())),
            0x8E => {
                let v = res!(1, self.operand as u8);
                self.write_op = Some(WritebackOp::Write8(self.hl, v));
            }

            0x90 => self.set_b(res!(2, self.b())),
            0x91 => self.set_c(res!(2, self.c())),
            0x92 => self.set_d(res!(2, self.d())),
            0x93 => self.set_e(res!(2, self.e())),
            0x94 => self.set_h(res!(2, self.h())),
            0x95 => self.set_l(res!(2, self.l())),
            0x97 => self.set_a(res!(2, self.a())),
            0x96 => {
                let v = res!(2, self.operand as u8);
                self.write_op = Some(WritebackOp::Write8(self.hl, v));
            }

            0x98 => self.set_b(res!(3, self.b())),
            0x99 => self.set_c(res!(3, self.c())),
            0x9A => self.set_d(res!(3, self.d())),
            0x9B => self.set_e(res!(3, self.e())),
            0x9C => self.set_h(res!(3, self.h())),
            0x9D => self.set_l(res!(3, self.l())),
            0x9F => self.set_a(res!(3, self.a())),
            0x9E => {
                let v = res!(3, self.operand as u8);
                self.write_op = Some(WritebackOp::Write8(self.hl, v));
            }

            0xA0 => self.set_b(res!(4, self.b())),
            0xA1 => self.set_c(res!(4, self.c())),
            0xA2 => self.set_d(res!(4, self.d())),
            0xA3 => self.set_e(res!(4, self.e())),
            0xA4 => self.set_h(res!(4, self.h())),
            0xA5 => self.set_l(res!(4, self.l())),
            0xA7 => self.set_a(res!(4, self.a())),
            0xA6 => {
                let v = res!(4, self.operand as u8);
                self.write_op = Some(WritebackOp::Write8(self.hl, v));
            }

            0xA8 => self.set_b(res!(5, self.b())),
            0xA9 => self.set_c(res!(5, self.c())),
            0xAA => self.set_d(res!(5, self.d())),
            0xAB => self.set_e(res!(5, self.e())),
            0xAC => self.set_h(res!(5, self.h())),
            0xAD => self.set_l(res!(5, self.l())),
            0xAF => self.set_a(res!(5, self.a())),
            0xAE => {
                let v = res!(5, self.operand as u8);
                self.write_op = Some(WritebackOp::Write8(self.hl, v));
            }

            0xB0 => self.set_b(res!(6, self.b())),
            0xB1 => self.set_c(res!(6, self.c())),
            0xB2 => self.set_d(res!(6, self.d())),
            0xB3 => self.set_e(res!(6, self.e())),
            0xB4 => self.set_h(res!(6, self.h())),
            0xB5 => self.set_l(res!(6, self.l())),
            0xB7 => self.set_a(res!(6, self.a())),
            0xB6 => {
                let v = res!(6, self.operand as u8);
                self.write_op = Some(WritebackOp::Write8(self.hl, v));
            }

            0xB8 => self.set_b(res!(7, self.b())),
            0xB9 => self.set_c(res!(7, self.c())),
            0xBA => self.set_d(res!(7, self.d())),
            0xBB => self.set_e(res!(7, self.e())),
            0xBC => self.set_h(res!(7, self.h())),
            0xBD => self.set_l(res!(7, self.l())),
            0xBF => self.set_a(res!(7, self.a())),
            0xBE => {
                let v = res!(7, self.operand as u8);
                self.write_op = Some(WritebackOp::Write8(self.hl, v));
            }

            0xC0 => self.set_b(set!(0, self.b())),
            0xC1 => self.set_c(set!(0, self.c())),
            0xC2 => self.set_d(set!(0, self.d())),
            0xC3 => self.set_e(set!(0, self.e())),
            0xC4 => self.set_h(set!(0, self.h())),
            0xC5 => self.set_l(set!(0, self.l())),
            0xC7 => self.set_a(set!(0, self.a())),
            0xC6 => {
                let v = set!(0, self.operand as u8);
                self.write_op = Some(WritebackOp::Write8(self.hl, v));
            }

            0xC8 => self.set_b(set!(1, self.b())),
            0xC9 => self.set_c(set!(1, self.c())),
            0xCA => self.set_d(set!(1, self.d())),
            0xCB => self.set_e(set!(1, self.e())),
            0xCC => self.set_h(set!(1, self.h())),
            0xCD => self.set_l(set!(1, self.l())),
            0xCF => self.set_a(set!(1, self.a())),
            0xCE => {
                let v = set!(1, self.operand as u8);
                self.write_op = Some(WritebackOp::Write8(self.hl, v));
            }

            0xD0 => self.set_b(set!(2, self.b())),
            0xD1 => self.set_c(set!(2, self.c())),
            0xD2 => self.set_d(set!(2, self.d())),
            0xD3 => self.set_e(set!(2, self.e())),
            0xD4 => self.set_h(set!(2, self.h())),
            0xD5 => self.set_l(set!(2, self.l())),
            0xD7 => self.set_a(set!(2, self.a())),
            0xD6 => {
                let v = set!(2, self.operand as u8);
                self.write_op = Some(WritebackOp::Write8(self.hl, v));
            }

            0xD8 => self.set_b(set!(3, self.b())),
            0xD9 => self.set_c(set!(3, self.c())),
            0xDA => self.set_d(set!(3, self.d())),
            0xDB => self.set_e(set!(3, self.e())),
            0xDC => self.set_h(set!(3, self.h())),
            0xDD => self.set_l(set!(3, self.l())),
            0xDF => self.set_a(set!(3, self.a())),
            0xDE => {
                let v = set!(3, self.operand as u8);
                self.write_op = Some(WritebackOp::Write8(self.hl, v));
            }

            0xE0 => self.set_b(set!(4, self.b())),
            0xE1 => self.set_c(set!(4, self.c())),
            0xE2 => self.set_d(set!(4, self.d())),
            0xE3 => self.set_e(set!(4, self.e())),
            0xE4 => self.set_h(set!(4, self.h())),
            0xE5 => self.set_l(set!(4, self.l())),
            0xE7 => self.set_a(set!(4, self.a())),
            0xE6 => {
                let v = set!(4, self.operand as u8);
                self.write_op = Some(WritebackOp::Write8(self.hl, v));
            }

            0xE8 => self.set_b(set!(5, self.b())),
            0xE9 => self.set_c(set!(5, self.c())),
            0xEA => self.set_d(set!(5, self.d())),
            0xEB => self.set_e(set!(5, self.e())),
            0xEC => self.set_h(set!(5, self.h())),
            0xED => self.set_l(set!(5, self.l())),
            0xEF => self.set_a(set!(5, self.a())),
            0xEE => {
                let v = set!(5, self.operand as u8);
                self.write_op = Some(WritebackOp::Write8(self.hl, v));
            }

            0xF0 => self.set_b(set!(6, self.b())),
            0xF1 => self.set_c(set!(6, self.c())),
            0xF2 => self.set_d(set!(6, self.d())),
            0xF3 => self.set_e(set!(6, self.e())),
            0xF4 => self.set_h(set!(6, self.h())),
            0xF5 => self.set_l(set!(6, self.l())),
            0xF7 => self.set_a(set!(6, self.a())),
            0xF6 => {
                let v = set!(6, self.operand as u8);
                self.write_op = Some(WritebackOp::Write8(self.hl, v));
            }

            0xF8 => self.set_b(set!(7, self.b())),
            0xF9 => self.set_c(set!(7, self.c())),
            0xFA => self.set_d(set!(7, self.d())),
            0xFB => self.set_e(set!(7, self.e())),
            0xFC => self.set_h(set!(7, self.h())),
            0xFD => self.set_l(set!(7, self.l())),
            0xFF => self.set_a(set!(7, self.a())),
            0xFE => {
                let v = set!(7, self.operand as u8);
                self.write_op = Some(WritebackOp::Write8(self.hl, v));
            }
        };

        Ok(())
    }
}

#[rustfmt::skip]
pub const OPCODES: [OpcodeInfo; 256] = [
    OpcodeInfo("NOP",         Register,    Register,     1, 4,  4),
    OpcodeInfo("LD BC,d16",   Register,    Immediate,    3, 12, 12),
    OpcodeInfo("LD (BC),A",   Memory(BC),  Register,     1, 8,  8),
    OpcodeInfo("INC BC",      Register,    Register,     1, 8,  8),
    OpcodeInfo("INC B",       Register,    Register,     1, 4,  4),
    OpcodeInfo("DEC B",       Register,    Register,     1, 4,  4),
    OpcodeInfo("LD B,d8",     Register,    Immediate,    2, 8,  8),
    OpcodeInfo("RLCA",        Register,    Register,     1, 4,  4),
    OpcodeInfo("LD (a16),SP", Memory(A16), Register,     3, 20, 20),
    OpcodeInfo("ADD HL,BC",   Register,    Register,     1, 8,  8),
    OpcodeInfo("LD A,(BC)",   Register,    Memory(BC),   1, 8,  8),
    OpcodeInfo("DEC BC",      Register,    Register,     1, 8,  8),
    OpcodeInfo("INC C",       Register,    Register,     1, 4,  4),
    OpcodeInfo("DEC C",       Register,    Register,     1, 4,  4),
    OpcodeInfo("LD C,d8",     Register,    Immediate,    2, 8,  8),
    OpcodeInfo("RRCA",        Register,    Register,     1, 4,  4),
    OpcodeInfo("STOP 0",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD DE,d16",   Register,    Immediate,    3, 12, 12),
    OpcodeInfo("LD (DE),A",   Memory(DE),  Register,     1, 8,  8),
    OpcodeInfo("INC DE",      Register,    Register,     1, 8,  8),
    OpcodeInfo("INC D",       Register,    Register,     1, 4,  4),
    OpcodeInfo("DEC D",       Register,    Register,     1, 4,  4),
    OpcodeInfo("LD D,d8",     Register,    Immediate,    2, 8,  8),
    OpcodeInfo("RLA",         Register,    Register,     1, 4,  4),
    OpcodeInfo("JR r8",       Register,    Immediate,    2, 12, 12),
    OpcodeInfo("ADD HL,DE",   Register,    Register,     1, 8,  8),
    OpcodeInfo("LD A,(DE)",   Register,    Memory(DE),   1, 8,  8),
    OpcodeInfo("DEC DE",      Register,    Register,     1, 8,  8),
    OpcodeInfo("INC E",       Register,    Register,     1, 4,  4),
    OpcodeInfo("DEC E",       Register,    Register,     1, 4,  4),
    OpcodeInfo("LD E,d8",     Register,    Immediate,    2, 8,  8),
    OpcodeInfo("RRA",         Register,    Register,     1, 4,  4),
    OpcodeInfo("JR NZ,r8",    Register,    Immediate,    2, 12, 8),
    OpcodeInfo("LD HL,d16",   Register,    Immediate,    3, 12, 12),
    OpcodeInfo("LD (HL+),A",  Memory(HL),  Register,     1, 8,  8),
    OpcodeInfo("INC HL",      Register,    Register,     1, 8,  8),
    OpcodeInfo("INC H",       Register,    Register,     1, 4,  4),
    OpcodeInfo("DEC H",       Register,    Register,     1, 4,  4),
    OpcodeInfo("LD H,d8",     Register,    Immediate,    2, 8,  8),
    OpcodeInfo("DAA",         Register,    Register,     1, 4,  4),
    OpcodeInfo("JR Z,r8",     Register,    Immediate,    2, 12, 8),
    OpcodeInfo("ADD HL,HL",   Register,    Register,     1, 8,  8),
    OpcodeInfo("LD A,(HL+)",  Register,    Memory(HL),   1, 8,  8),
    OpcodeInfo("DEC HL",      Register,    Register,     1, 8,  8),
    OpcodeInfo("INC L",       Register,    Register,     1, 4,  4),
    OpcodeInfo("DEC L",       Register,    Register,     1, 4,  4),
    OpcodeInfo("LD L,d8",     Register,    Immediate,    2, 8,  8),
    OpcodeInfo("CPL",         Register,    Register,     1, 4,  4),
    OpcodeInfo("JR NC,r8",    Register,    Immediate,    2, 12, 8),
    OpcodeInfo("LD SP,d16",   Register,    Immediate,    3, 12, 12),
    OpcodeInfo("LD (HL-),A",  Memory(HL),  Register,     1, 8,  8),
    OpcodeInfo("INC SP",      Register,    Register,     1, 8,  8),
    OpcodeInfo("INC (HL)",    Memory(HL),  Memory(HL),   1, 12, 12),
    OpcodeInfo("DEC (HL)",    Memory(HL),  Memory(HL),   1, 12, 12),
    OpcodeInfo("LD (HL),d8",  Memory(HL),  Immediate,    2, 12, 12),
    OpcodeInfo("SCF",         Register,    Register,     1, 4,  4),
    OpcodeInfo("JR C,r8",     Register,    Immediate,    2, 12, 8),
    OpcodeInfo("ADD HL,SP",   Register,    Register,     1, 8,  8),
    OpcodeInfo("LD A,(HL-)",  Register,    Memory(HL),   1, 8,  8),
    OpcodeInfo("DEC SP",      Register,    Register,     1, 8,  8),
    OpcodeInfo("INC A",       Register,    Register,     1, 4,  4),
    OpcodeInfo("DEC A",       Register,    Register,     1, 4,  4),
    OpcodeInfo("LD A,d8",     Register,    Immediate,    2, 8,  8),
    OpcodeInfo("CCF",         Register,    Register,     1, 4,  4),
    OpcodeInfo("LD B,B",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD B,C",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD B,D",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD B,E",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD B,H",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD B,L",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD B,(HL)",   Register,    Memory(HL),   1, 8,  8),
    OpcodeInfo("LD B,A",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD C,B",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD C,C",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD C,D",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD C,E",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD C,H",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD C,L",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD C,(HL)",   Register,    Memory(HL),   1, 8,  8),
    OpcodeInfo("LD C,A",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD D,B",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD D,C",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD D,D",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD D,E",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD D,H",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD D,L",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD D,(HL)",   Register,    Memory(HL),   1, 8,  8),
    OpcodeInfo("LD D,A",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD E,B",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD E,C",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD E,D",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD E,E",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD E,H",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD E,L",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD E,(HL)",   Register,    Memory(HL),   1, 8,  8),
    OpcodeInfo("LD E,A",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD H,B",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD H,C",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD H,D",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD H,E",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD H,H",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD H,L",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD H,(HL)",   Register,    Memory(HL),   1, 8,  8),
    OpcodeInfo("LD H,A",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD L,B",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD L,C",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD L,D",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD L,E",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD L,H",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD L,L",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD L,(HL)",   Register,    Memory(HL),   1, 8,  8),
    OpcodeInfo("LD L,A",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD (HL),B",   Memory(HL),  Register,     1, 8,  8),
    OpcodeInfo("LD (HL),C",   Memory(HL),  Register,     1, 8,  8),
    OpcodeInfo("LD (HL),D",   Memory(HL),  Register,     1, 8,  8),
    OpcodeInfo("LD (HL),E",   Memory(HL),  Register,     1, 8,  8),
    OpcodeInfo("LD (HL),H",   Memory(HL),  Register,     1, 8,  8),
    OpcodeInfo("LD (HL),L",   Memory(HL),  Register,     1, 8,  8),
    OpcodeInfo("HALT",        Register,    Register,     1, 4,  4),
    OpcodeInfo("LD (HL),A",   Memory(HL),  Register,     1, 8,  8),
    OpcodeInfo("LD A,B",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD A,C",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD A,D",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD A,E",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD A,H",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD A,L",      Register,    Register,     1, 4,  4),
    OpcodeInfo("LD A,(HL)",   Register,    Memory(HL),   1, 8,  8),
    OpcodeInfo("LD A,A",      Register,    Register,     1, 4,  4),
    OpcodeInfo("ADD A,B",     Register,    Register,     1, 4,  4),
    OpcodeInfo("ADD A,C",     Register,    Register,     1, 4,  4),
    OpcodeInfo("ADD A,D",     Register,    Register,     1, 4,  4),
    OpcodeInfo("ADD A,E",     Register,    Register,     1, 4,  4),
    OpcodeInfo("ADD A,H",     Register,    Register,     1, 4,  4),
    OpcodeInfo("ADD A,L",     Register,    Register,     1, 4,  4),
    OpcodeInfo("ADD A,(HL)",  Register,    Memory(HL),   1, 8,  8),
    OpcodeInfo("ADD A,A",     Register,    Register,     1, 4,  4),
    OpcodeInfo("ADC A,B",     Register,    Register,     1, 4,  4),
    OpcodeInfo("ADC A,C",     Register,    Register,     1, 4,  4),
    OpcodeInfo("ADC A,D",     Register,    Register,     1, 4,  4),
    OpcodeInfo("ADC A,E",     Register,    Register,     1, 4,  4),
    OpcodeInfo("ADC A,H",     Register,    Register,     1, 4,  4),
    OpcodeInfo("ADC A,L",     Register,    Register,     1, 4,  4),
    OpcodeInfo("ADC A,(HL)",  Register,    Memory(HL),   1, 8,  8),
    OpcodeInfo("ADC A,A",     Register,    Register,     1, 4,  4),
    OpcodeInfo("SUB B",       Register,    Register,     1, 4,  4),
    OpcodeInfo("SUB C",       Register,    Register,     1, 4,  4),
    OpcodeInfo("SUB D",       Register,    Register,     1, 4,  4),
    OpcodeInfo("SUB E",       Register,    Register,     1, 4,  4),
    OpcodeInfo("SUB H",       Register,    Register,     1, 4,  4),
    OpcodeInfo("SUB L",       Register,    Register,     1, 4,  4),
    OpcodeInfo("SUB (HL)",    Register,    Memory(HL),   1, 8,  8),
    OpcodeInfo("SUB A",       Register,    Register,     1, 4,  4),
    OpcodeInfo("SBC A,B",     Register,    Register,     1, 4,  4),
    OpcodeInfo("SBC A,C",     Register,    Register,     1, 4,  4),
    OpcodeInfo("SBC A,D",     Register,    Register,     1, 4,  4),
    OpcodeInfo("SBC A,E",     Register,    Register,     1, 4,  4),
    OpcodeInfo("SBC A,H",     Register,    Register,     1, 4,  4),
    OpcodeInfo("SBC A,L",     Register,    Register,     1, 4,  4),
    OpcodeInfo("SBC A,(HL)",  Register,    Memory(HL),   1, 8,  8),
    OpcodeInfo("SBC A,A",     Register,    Register,     1, 4,  4),
    OpcodeInfo("AND B",       Register,    Register,     1, 4,  4),
    OpcodeInfo("AND C",       Register,    Register,     1, 4,  4),
    OpcodeInfo("AND D",       Register,    Register,     1, 4,  4),
    OpcodeInfo("AND E",       Register,    Register,     1, 4,  4),
    OpcodeInfo("AND H",       Register,    Register,     1, 4,  4),
    OpcodeInfo("AND L",       Register,    Register,     1, 4,  4),
    OpcodeInfo("AND (HL)",    Register,    Memory(HL),   1, 8,  8),
    OpcodeInfo("AND A",       Register,    Register,     1, 4,  4),
    OpcodeInfo("XOR B",       Register,    Register,     1, 4,  4),
    OpcodeInfo("XOR C",       Register,    Register,     1, 4,  4),
    OpcodeInfo("XOR D",       Register,    Register,     1, 4,  4),
    OpcodeInfo("XOR E",       Register,    Register,     1, 4,  4),
    OpcodeInfo("XOR H",       Register,    Register,     1, 4,  4),
    OpcodeInfo("XOR L",       Register,    Register,     1, 4,  4),
    OpcodeInfo("XOR (HL)",    Register,    Memory(HL),   1, 8,  8),
    OpcodeInfo("XOR A",       Register,    Register,     1, 4,  4),
    OpcodeInfo("OR B",        Register,    Register,     1, 4,  4),
    OpcodeInfo("OR C",        Register,    Register,     1, 4,  4),
    OpcodeInfo("OR D",        Register,    Register,     1, 4,  4),
    OpcodeInfo("OR E",        Register,    Register,     1, 4,  4),
    OpcodeInfo("OR H",        Register,    Register,     1, 4,  4),
    OpcodeInfo("OR L",        Register,    Register,     1, 4,  4),
    OpcodeInfo("OR (HL)",     Register,    Memory(HL),   1, 8,  8),
    OpcodeInfo("OR A",        Register,    Register,     1, 4,  4),
    OpcodeInfo("CP B",        Register,    Register,     1, 4,  4),
    OpcodeInfo("CP C",        Register,    Register,     1, 4,  4),
    OpcodeInfo("CP D",        Register,    Register,     1, 4,  4),
    OpcodeInfo("CP E",        Register,    Register,     1, 4,  4),
    OpcodeInfo("CP H",        Register,    Register,     1, 4,  4),
    OpcodeInfo("CP L",        Register,    Register,     1, 4,  4),
    OpcodeInfo("CP (HL)",     Register,    Memory(HL),   1, 8,  8),
    OpcodeInfo("CP A",        Register,    Register,     1, 4,  4),
    OpcodeInfo("RET NZ",      Register,    Register,     1, 20, 8),
    OpcodeInfo("POP BC",      Register,    Memory(SP),   1, 12, 12),
    OpcodeInfo("JP NZ,a16",   Register,    Immediate,    3, 16, 12),
    OpcodeInfo("JP a16",      Register,    Immediate,    3, 16, 16),
    OpcodeInfo("CALL NZ,a16", Register,    Immediate,    3, 24, 12),
    OpcodeInfo("PUSH BC",     Memory(SP),  Register,     1, 16, 16),
    OpcodeInfo("ADD A,d8",    Register,    Immediate,    2, 8,  8),
    OpcodeInfo("RST 00H",     Register,    Register,     1, 16, 16),
    OpcodeInfo("RET Z",       Register,    Register,     1, 20, 8),
    OpcodeInfo("RET",         Register,    Register,     1, 16, 16),
    OpcodeInfo("JP Z,a16",    Register,    Immediate,    3, 16, 12),
    OpcodeInfo("PREFIX CB",   Register,    Immediate,    2, 8,  8),
    OpcodeInfo("CALL Z,a16",  Register,    Immediate,    3, 24, 12),
    OpcodeInfo("CALL a16",    Register,    Immediate,    3, 24, 24),
    OpcodeInfo("ADC A,d8",    Register,    Immediate,    2, 8,  8),
    OpcodeInfo("RST 08H",     Register,    Register,     1, 16, 16),
    OpcodeInfo("RET NC",      Register,    Register,     1, 20, 8),
    OpcodeInfo("POP DE",      Register,    Memory(SP),   1, 12, 12),
    OpcodeInfo("JP NC,a16",   Register,    Immediate,    3, 16, 12),
    OpcodeInfo("-",           Register,    Register,     1, 0,  0),
    OpcodeInfo("CALL NC,a16", Register,    Immediate,    3, 24, 12),
    OpcodeInfo("PUSH DE",     Memory(SP),  Register,     1, 16, 16),
    OpcodeInfo("SUB d8",      Register,    Immediate,    2, 8,  8),
    OpcodeInfo("RST 10H",     Register,    Register,     1, 16, 16),
    OpcodeInfo("RET C",       Register,    Register,     1, 20, 8),
    OpcodeInfo("RETI",        Register,    Register,     1, 16, 16),
    OpcodeInfo("JP C,a16",    Register,    Immediate,    3, 16, 12),
    OpcodeInfo("-",           Register,    Register,     1, 0,  0),
    OpcodeInfo("CALL C,a16",  Register,    Immediate,    3, 24, 12),
    OpcodeInfo("-",           Register,    Register,     1, 0,  0),
    OpcodeInfo("SBC A,d8",    Register,    Immediate,    2, 8,  8),
    OpcodeInfo("RST 18H",     Register,    Register,     1, 16, 16),
    OpcodeInfo("LDH (a8),A",  Memory(IO),  Register,     2, 12, 12),
    OpcodeInfo("POP HL",      Register,    Memory(SP),   1, 12, 12),
    OpcodeInfo("LD (C),A",    Memory(C),   Register,     1, 8,  8),
    OpcodeInfo("-",           Register,    Register,     1, 0,  0),
    OpcodeInfo("-",           Register,    Register,     1, 0,  0),
    OpcodeInfo("PUSH HL",     Memory(HL),  Register,     1, 16, 16),
    OpcodeInfo("AND d8",      Register,    Immediate,    2, 8,  8),
    OpcodeInfo("RST 20H",     Register,    Register,     1, 16, 16),
    OpcodeInfo("ADD SP,r8",   Register,    Immediate,    2, 16, 16),
    OpcodeInfo("JP HL",       Register,    Register,     1, 4,  4),
    OpcodeInfo("LD (a16),A",  Memory(A16), Register,     3, 16, 16),
    OpcodeInfo("-",           Register,    Register,     1, 0,  0),
    OpcodeInfo("-",           Register,    Register,     1, 0,  0),
    OpcodeInfo("-",           Register,    Register,     1, 0,  0),
    OpcodeInfo("XOR d8",      Register,    Immediate,    2, 8,  8),
    OpcodeInfo("RST 28H",     Register,    Register,     1, 16, 16),
    OpcodeInfo("LDH A,(a8)",  Register,    Memory(IO),   2, 12, 12),
    OpcodeInfo("POP AF",      Register,    Memory(SP),   1, 12, 12),
    OpcodeInfo("LD A,(C)",    Register,    Memory(C),    1, 8,  8),
    OpcodeInfo("DI",          Register,    Register,     1, 4,  4),
    OpcodeInfo("-",           Register,    Register,     1, 0,  0),
    OpcodeInfo("PUSH AF",     Memory(SP),  Register,     1, 16, 16),
    OpcodeInfo("OR d8",       Register,    Immediate,    2, 8,  8),
    OpcodeInfo("RST 30H",     Register,    Register,     1, 16, 16),
    OpcodeInfo("LD HL,SP+r8", Register,    Immediate,    2, 12, 12),
    OpcodeInfo("LD SP,HL",    Register,    Register,     1, 8,  8),
    OpcodeInfo("LD A,(a16)",  Register,    Memory(A16),  3, 16, 16),
    OpcodeInfo("EI",          Register,    Register,     1, 4,  4),
    OpcodeInfo("-",           Register,    Register,     1, 0,  0),
    OpcodeInfo("-",           Register,    Register,     1, 0,  0),
    OpcodeInfo("CP d8",       Register,    Immediate,    2, 8,  8),
    OpcodeInfo("RST 38H",     Register,    Register,     1, 16, 16),
];

#[cfg(test)]
mod test {
    use super::*;

    use crate::{
        cpu::{CpuState, CpuState::*},
        dbg,
        mem::{MemR, MemRW, MemW},
    };

    impl MemR for &mut [u8] {
        fn read(&self, addr: u16) -> Result<u8, dbg::TraceEvent> {
            Ok(self[addr as usize])
        }
    }

    impl MemW for &mut [u8] {
        fn write(&mut self, addr: u16, val: u8) -> Result<(), dbg::TraceEvent> {
            self[addr as usize] = val;
            Ok(())
        }
    }

    impl MemRW for &mut [u8] {}

    struct CpuTest {
        ticks: usize,
        memory: Vec<u8>,
        setup_fn: Box<dyn FnMut(&mut Cpu)>,
        target_states: Option<Vec<CpuState>>,
        target_memory: Option<Vec<u8>>,
    }

    impl CpuTest {
        fn new(ticks: usize, memory: Vec<u8>) -> CpuTest {
            CpuTest {
                ticks,
                memory,
                setup_fn: Box::new(|_| {}),
                target_states: None,
                target_memory: None,
            }
        }

        fn match_states(mut self, states: Vec<CpuState>) -> CpuTest {
            self.target_states = Some(states);
            self
        }

        fn match_memory(mut self, mem: Vec<u8>) -> CpuTest {
            self.target_memory = Some(mem);
            self
        }

        fn setup(mut self, setup: impl FnMut(&mut Cpu) + 'static) -> CpuTest {
            self.setup_fn = Box::new(setup);
            self
        }

        fn run<F>(mut self, mut verify: F)
        where
            F: FnMut(&mut Cpu, &[u8]),
        {
            let mut cpu = Cpu::new();

            // Reset CPU state for test purposes
            cpu.af = 0;
            cpu.bc = 0;
            cpu.de = 0;
            cpu.hl = 0;
            cpu.sp = 0;
            cpu.pc = 0;

            (self.setup_fn)(&mut cpu);

            for t in 0..self.ticks {
                cpu.tick(&mut (&mut self.memory[..])).unwrap();

                if let Some(ref states) = self.target_states {
                    assert_eq!(cpu.state, states[t]);
                }
            }

            if let Some(ref tgt_mem) = self.target_memory {
                assert_eq!(self.memory, *tgt_mem);
            }

            verify(&mut cpu, &self.memory[..]);
        }
    }

    #[test]
    fn nop_works() {
        CpuTest::new(1, vec![0x00])
            .match_states(vec![FetchOpcode])
            .run(|cpu, _| {
                assert_eq!(cpu.pc, 1);
            });
    }

    #[test]
    fn misc_opcodes_work() {
        // STOP/HALT
        CpuTest::new(1, vec![0x10])
            .match_states(vec![FetchOpcode])
            .run(|cpu, _| {
                assert!(*cpu.halted.loaded());
            });

        CpuTest::new(2, vec![0x10, 0x00])
            .match_states(vec![FetchOpcode, FetchOpcode])
            .run(|cpu, _| {
                assert!(*cpu.halted.value());
            });

        // EI
        CpuTest::new(1, vec![0xFB])
            .match_states(vec![FetchOpcode])
            .run(|cpu, _| {
                assert!(!*cpu.intr_enabled.value());
            });

        CpuTest::new(2, vec![0xFB, 0x00])
            .match_states(vec![FetchOpcode, FetchOpcode])
            .run(|cpu, _| {
                assert!(*cpu.intr_enabled.value());
            });

        // DI
        CpuTest::new(2, vec![0xFB, 0xF3])
            .match_states(vec![FetchOpcode, FetchOpcode])
            .run(|cpu, _| {
                assert!(!*cpu.intr_enabled.value());
            });
    }

    #[test]
    fn branch_opcodes_work() {}

    #[test]
    fn ld16_opcodes_work() {
        // LD rr,d16
        CpuTest::new(3, vec![0x01, 0xAA, 0x55])
            .match_states(vec![FetchByte0, FetchByte1, FetchOpcode])
            .run(|cpu, _| {
                assert_eq!(cpu.bc, 0x55AA);
            });

        // LD (a16),SP
        CpuTest::new(5, vec![0x08, 0x03, 0x00, 0x00, 0x00])
            .match_states(vec![
                FetchByte0,
                FetchByte1,
                Writeback,
                Delay(0),
                FetchOpcode,
            ])
            .match_memory(vec![0x08, 0x03, 0x00, 0xC0, 0xBE])
            .setup(|cpu| {
                cpu.sp = 0xBEC0;
            })
            .run(|cpu, _| {
                assert_eq!(cpu.sp, 0xBEC0);
            });

        // PUSH rr
        CpuTest::new(4, vec![0xD5, 0x00, 0x00, 0x22, 0x11])
            .match_states(vec![Writeback, Delay(1), Delay(0), FetchOpcode])
            .match_memory(vec![0xD5, 0x00, 0x00, 0xBB, 0xAA])
            .setup(|cpu| {
                cpu.sp = 0x0005;
                cpu.de = 0xAABB;
            })
            .run(|cpu, _| {
                assert_eq!(cpu.sp, 0x0003);
                assert_eq!(cpu.de, 0xAABB);
            });

        // POP rr
        CpuTest::new(3, vec![0xE1, 0x00, 0x00, 0x22, 0x11])
            .match_states(vec![FetchMemory0, FetchMemory1, FetchOpcode])
            .match_memory(vec![0xE1, 0x00, 0x00, 0x22, 0x11])
            .setup(|cpu| {
                cpu.sp = 0x0003;
                cpu.hl = 0x0000;
            })
            .run(|cpu, _| {
                assert_eq!(cpu.sp, 0x0005);
                assert_eq!(cpu.hl, 0x1122);
            });

        // LD SP,HL
        CpuTest::new(2, vec![0xF9])
            .match_states(vec![Delay(0), FetchOpcode])
            .setup(|cpu| {
                cpu.sp = 0x0000;
                cpu.hl = 0x1234;
            })
            .run(|cpu, _| {
                assert_eq!(cpu.sp, 0x1234);
                assert_eq!(cpu.hl, 0x1234);
            });

        // LD HL,SP+r8
        CpuTest::new(3, vec![0xF8, 0x15])
            .match_states(vec![FetchByte0, Delay(0), FetchOpcode])
            .setup(|cpu| {
                cpu.sp = 0x2500;
                cpu.hl = 0x1234;
            })
            .run(|cpu, _| {
                assert_eq!(cpu.sp, 0x2500);
                assert_eq!(cpu.hl, 0x2515);
            });

        CpuTest::new(3, vec![0xF8, 0xFE])
            .match_states(vec![FetchByte0, Delay(0), FetchOpcode])
            .setup(|cpu| {
                cpu.sp = 0x2500;
                cpu.hl = 0x1234;
            })
            .run(|cpu, _| {
                assert_eq!(cpu.sp, 0x2500);
                assert_eq!(cpu.hl, 0x24FE);
            });
    }

    #[test]
    fn ld8_opcodes_work() {
        // LD m,A
        CpuTest::new(2, vec![0x02, 0x00])
            .match_states(vec![Writeback, FetchOpcode])
            .match_memory(vec![0x02, 0xAA])
            .setup(|cpu| {
                cpu.bc = 0x0001;
                cpu.set_a(0xAA);
            })
            .run(|_, _| {});

        // LD r,d8
        CpuTest::new(2, vec![0x16, 0xAB])
            .match_states(vec![FetchByte0, FetchOpcode])
            .run(|cpu, _| {
                assert_eq!(cpu.d(), 0xAB);
            });

        // LD m,d8
        CpuTest::new(3, vec![0x36, 0xAB, 0x00, 0xFF])
            .match_states(vec![FetchByte0, Writeback, FetchOpcode])
            .match_memory(vec![0x36, 0xAB, 0xAB, 0xFF])
            .setup(|cpu| {
                cpu.hl = 0x2;
            })
            .run(|_, _| {});

        // LD A,m
        CpuTest::new(2, vec![0x1A, 0xFE, 0x00])
            .match_states(vec![FetchMemory0, FetchOpcode])
            .setup(|cpu| {
                cpu.de = 0x1;
            })
            .run(|cpu, _| {
                assert_eq!(cpu.a(), 0xFE);
            });

        // LD A,m+
        CpuTest::new(2, vec![0x2A, 0xFE, 0x00])
            .match_states(vec![FetchMemory0, FetchOpcode])
            .setup(|cpu| {
                cpu.hl = 0x1;
            })
            .run(|cpu, _| {
                assert_eq!(cpu.a(), 0xFE);
                assert_eq!(cpu.hl, 0x2);
            });

        // LD r,r
        CpuTest::new(1, vec![0x51])
            .match_states(vec![FetchOpcode])
            .setup(|cpu| {
                cpu.set_d(0x23);
                cpu.set_c(0x12);
            })
            .run(|cpu, _| {
                assert_eq!(cpu.d(), 0x12);
                assert_eq!(cpu.c(), 0x12);
            });

        // LD A,m
        CpuTest::new(2, vec![0x5E, 0xFE, 0x00])
            .match_states(vec![FetchMemory0, FetchOpcode])
            .setup(|cpu| {
                cpu.hl = 0x1;
            })
            .run(|cpu, _| {
                assert_eq!(cpu.e(), 0xFE);
            });

        // LD m,A
        CpuTest::new(2, vec![0x71, 0x00])
            .match_states(vec![Writeback, FetchOpcode])
            .match_memory(vec![0x71, 0xAA])
            .setup(|cpu| {
                cpu.hl = 0x1;
                cpu.set_c(0xAA);
            })
            .run(|_, _| {});

        // LDH m,A
        CpuTest::new(3, vec![0xE0; 0x10000])
            .match_states(vec![FetchByte0, Writeback, FetchOpcode])
            .setup(|cpu| {
                cpu.set_a(0xAB);
            })
            .run(|_, mem| {
                assert_eq!(mem[0xFFE0], 0xAB);
            });

        CpuTest::new(2, vec![0xE2; 0x10000])
            .match_states(vec![Writeback, FetchOpcode])
            .setup(|cpu| {
                cpu.set_a(0xAB);
                cpu.set_c(0x10);
            })
            .run(|_, mem| {
                assert_eq!(mem[0xFF10], 0xAB);
            });

        // LDH A,m
        CpuTest::new(3, vec![0xF0; 0x10000])
            .match_states(vec![FetchByte0, FetchMemory0, FetchOpcode])
            .run(|cpu, _| {
                assert_eq!(cpu.a(), 0xF0);
            });
    }

    #[test]
    fn arith16_opcodes_work() {
        // INC rr
        CpuTest::new(2, vec![0x23])
            .match_states(vec![Delay(0), FetchOpcode])
            .run(|cpu, _| {
                assert_eq!(cpu.hl, 0x1);
            });

        // DEC rr
        CpuTest::new(2, vec![0x0B])
            .match_states(vec![Delay(0), FetchOpcode])
            .run(|cpu, _| {
                assert_eq!(cpu.bc, 0xFFFF);
            });

        // ADD HL,rr
        CpuTest::new(2, vec![0x39])
            .match_states(vec![Delay(0), FetchOpcode])
            .setup(|cpu| {
                cpu.hl = 0xFA;
                cpu.sp = 0xF120;
            })
            .run(|cpu, _| {
                assert_eq!(cpu.hl, 0xF21A);
            });

        // ADD SP,d8
        CpuTest::new(4, vec![0xE8, 0x05])
            .match_states(vec![FetchByte0, Delay(1), Delay(0), FetchOpcode])
            .setup(|cpu| {
                cpu.sp = 0xA890;
            })
            .run(|cpu, _| {
                assert_eq!(cpu.sp, 0xA895);
            });

        CpuTest::new(4, vec![0xE8, 0xFE])
            .match_states(vec![FetchByte0, Delay(1), Delay(0), FetchOpcode])
            .setup(|cpu| {
                cpu.sp = 0xA890;
            })
            .run(|cpu, _| {
                assert_eq!(cpu.sp, 0xA88E);
            });
    }

    #[test]
    fn arith8_opcodes_work() {
        // INC r
        CpuTest::new(1, vec![0x14])
            .match_states(vec![FetchOpcode])
            .run(|cpu, _| {
                assert_eq!(cpu.d(), 0x1);
            });

        // DEC r
        CpuTest::new(1, vec![0x1D])
            .match_states(vec![FetchOpcode])
            .run(|cpu, _| {
                assert_eq!(cpu.e(), 0xFF);
            });

        // INC m
        CpuTest::new(3, vec![0x34, 0x04])
            .match_states(vec![FetchMemory0, Writeback, FetchOpcode])
            .match_memory(vec![0x34, 0x05])
            .setup(|cpu| {
                cpu.hl = 0x1;
            })
            .run(|_, _| {});

        // DEC m
        CpuTest::new(3, vec![0x35, 0x04])
            .match_states(vec![FetchMemory0, Writeback, FetchOpcode])
            .match_memory(vec![0x35, 0x03])
            .setup(|cpu| {
                cpu.hl = 0x1;
            })
            .run(|_, _| {});

        // ADD/ADC/SUB/SBC r,r
        CpuTest::new(1, vec![0x83])
            .match_states(vec![FetchOpcode])
            .setup(|cpu| {
                cpu.set_a(0xA5);
                cpu.set_e(0x05);
            })
            .run(|cpu, _| {
                assert_eq!(cpu.a(), 0xAA);
            });

        // ADD/ADC/SUB/SBC r,m
        CpuTest::new(2, vec![0x96, 0x0E])
            .match_states(vec![FetchMemory0, FetchOpcode])
            .setup(|cpu| {
                cpu.hl = 0x1;
                cpu.set_a(0xA5);
            })
            .run(|cpu, _| {
                assert_eq!(cpu.a(), 0x97);
            });

        // AND/OR/XOR r,r
        CpuTest::new(1, vec![0xB0])
            .match_states(vec![FetchOpcode])
            .setup(|cpu| {
                cpu.set_a(0b_1000_1001);
                cpu.set_b(0b_1101_0000);
            })
            .run(|cpu, _| {
                assert_eq!(cpu.a(), 0b_1101_1001);
            });

        // AND/OR/XOR r,m
        CpuTest::new(2, vec![0xA6, 0b_0010_1001])
            .match_states(vec![FetchMemory0, FetchOpcode])
            .setup(|cpu| {
                cpu.hl = 0x1;
                cpu.set_a(0b_0101_1010);
            })
            .run(|cpu, _| {
                assert_eq!(cpu.a(), 0b_0000_1000);
            });

        // ADD/ADC/SUB/SBC/AND/OR/XOR r,d8
        CpuTest::new(2, vec![0xEE, 0b_1011_0010])
            .match_states(vec![FetchByte0, FetchOpcode])
            .setup(|cpu| {
                cpu.set_a(0b_1000_1110);
            })
            .run(|cpu, _| {
                assert_eq!(cpu.a(), 0b_0011_1100);
            });

        // RLCA/RLA/RRCA/RRA
        for op in [0x07, 0x0F, 0x17, 0x1F].iter() {
            CpuTest::new(1, vec![*op])
                .match_states(vec![FetchOpcode])
                .run(|_, _| {});
        }
    }

    #[test]
    fn prefix_cb_opcodes_work() {
        // CB r
        CpuTest::new(2, vec![0xCB, 0xDB])
            .match_states(vec![FetchByte0, FetchOpcode])
            .run(|cpu, _| {
                assert_eq!(cpu.e(), 0b_0000_1000);
            });

        // CB m
        CpuTest::new(4, vec![0xCB, 0x96, 0b_0110_1110])
            .match_states(vec![FetchByte0, FetchMemory0, Writeback, FetchOpcode])
            .match_memory(vec![0xCB, 0x96, 0b_0110_1010])
            .setup(|cpu| {
                cpu.hl = 0x2;
            })
            .run(|_, _| {});
    }

    #[test]
    fn opcode_timings_are_correct() {
        for op in 0_u8..=255 {
            let info = OPCODES[op as usize];

            let res_on_branch_taken = std::panic::catch_unwind(|| {
                CpuTest::new(info.4 as usize, vec![op; 0x10000]).run(|cpu, _| {
                    assert_eq!(cpu.state, FetchOpcode);
                });
            });

            let res_on_branch_not_taken = std::panic::catch_unwind(|| {
                CpuTest::new(info.5 as usize, vec![op; 0x10000]).run(|cpu, _| {
                    assert_eq!(cpu.state, FetchOpcode);
                });
            });

            if res_on_branch_taken.is_err() && res_on_branch_not_taken.is_err() {
                panic!("both timings are wrong for opcode {:02X}", op);
            }
        }
    }
}
