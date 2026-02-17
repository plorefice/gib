use bitflags::bitflags;

use crate::{
    dbg,
    mem::{MemR, MemRW, MemW},
};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct JoypadState: u8 {
        const DOWN   = 0b_1000_0000;
        const UP     = 0b_0100_0000;
        const LEFT   = 0b_0010_0000;
        const RIGHT  = 0b_0001_0000;
        const START  = 0b_0000_1000;
        const SELECT = 0b_0000_0100;
        const B      = 0b_0000_0010;
        const A      = 0b_0000_0001;

        const DEFAULT = 0b_1111_1111;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct JoyP: u8 {
        const SEL_BTNS = 0b_0010_0000;
        const SEL_DIRS = 0b_0001_0000;
        const BTN_MASK = 0b_0000_1111;

        const DEFAULT = 0b_0000_1111;
    }
}

mem_rw!(JoyP, 0xC0);

pub struct Joypad {
    joyp: JoyP,

    state: JoypadState,
}

impl Default for Joypad {
    fn default() -> Joypad {
        Joypad {
            joyp: JoyP::DEFAULT,
            state: JoypadState::DEFAULT,
        }
    }
}

impl Joypad {
    pub fn new() -> Joypad {
        Joypad::default()
    }

    pub fn set_pressed_keys(&mut self, pressed: JoypadState) {
        self.state &= !pressed;
    }

    pub fn set_release_keys(&mut self, released: JoypadState) {
        self.state |= released;
    }
}

impl MemR for Joypad {
    fn read(&self, _addr: u16) -> Result<u8, dbg::TraceEvent> {
        // Assign upper, lower or no half of state depending on the selection bits
        let res = if !self.joyp.contains(JoyP::SEL_BTNS) {
            self.state.bits()
        } else if !self.joyp.contains(JoyP::SEL_DIRS) {
            self.state.bits() >> 4
        } else {
            0x0F
        };

        let joyp = (self.joyp | JoyP::BTN_MASK) & JoyP::from_bits_truncate(res | 0xF0);

        (&joyp).read(0)
    }
}

impl MemW for Joypad {
    fn write(&mut self, _addr: u16, val: u8) -> Result<(), dbg::TraceEvent> {
        (&mut self.joyp).write(0, val)
    }
}

impl MemRW for Joypad {}
