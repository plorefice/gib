use super::dbg;
use super::IoReg;
use super::{InterruptSource, IrqSource};
use super::{MemR, MemW};

/// One of Game Boy's four sound channels.
#[derive(Debug, Copy, Clone)]
pub enum Channel {
    Ch1,
    Ch2,
    Ch3,
    Ch4,
}

pub struct APU {
    // Channel 1 registers
    ch1_swp_reg: IoReg<u8>,
    ch1_len_reg: IoReg<u8>,
    ch1_vol_reg: IoReg<u8>,
    ch1_flo_reg: IoReg<u8>,
    ch1_fhi_reg: IoReg<u8>,

    // Channel 2 registers
    ch2_len_reg: IoReg<u8>,
    ch2_vol_reg: IoReg<u8>,
    ch2_flo_reg: IoReg<u8>,
    ch2_fhi_reg: IoReg<u8>,

    // Channel 3 registers
    ch3_snd_reg: IoReg<u8>,
    ch3_len_reg: IoReg<u8>,
    ch3_vol_reg: IoReg<u8>,
    ch3_flo_reg: IoReg<u8>,
    ch3_fhi_reg: IoReg<u8>,

    // Channel 4 registers
    ch4_len_reg: IoReg<u8>,
    ch4_vol_reg: IoReg<u8>,
    ch4_cnt_reg: IoReg<u8>,
    ch4_ini_reg: IoReg<u8>,

    // Control registers
    ctrl_master_reg: IoReg<u8>,
    ctrl_output_reg: IoReg<u8>,
    ctrl_snd_en_reg: IoReg<u8>,

    wave_ram: [u8; 16],
}

impl Default for APU {
    fn default() -> APU {
        APU {
            ch1_swp_reg: IoReg(0x80),
            ch1_len_reg: IoReg(0x8F),
            ch1_vol_reg: IoReg(0xF3),
            ch1_flo_reg: IoReg(0x00),
            ch1_fhi_reg: IoReg(0xBF),

            ch2_len_reg: IoReg(0x3F),
            ch2_vol_reg: IoReg(0x00),
            ch2_flo_reg: IoReg(0x00),
            ch2_fhi_reg: IoReg(0xBF),

            ch3_snd_reg: IoReg(0x7F),
            ch3_len_reg: IoReg(0xFF),
            ch3_vol_reg: IoReg(0x9F),
            ch3_flo_reg: IoReg(0x00),
            ch3_fhi_reg: IoReg(0xBF),

            ch4_len_reg: IoReg(0xFF),
            ch4_vol_reg: IoReg(0x00),
            ch4_cnt_reg: IoReg(0x00),
            ch4_ini_reg: IoReg(0xBF),

            ctrl_master_reg: IoReg(0x77),
            ctrl_output_reg: IoReg(0xF3),
            ctrl_snd_en_reg: IoReg(0xF1),

            wave_ram: [0; 16],
        }
    }
}

impl APU {
    pub fn new() -> APU {
        APU::default()
    }

    /// Returns the current tone frequency of a sound channel.
    pub fn get_frequency(&self, ch: Channel) -> u16 {
        let f = match ch {
            Channel::Ch2 => {
                u32::from(self.ch2_fhi_reg.0 & 0x7) << 8 | u32::from(self.ch2_flo_reg.0)
            }
            _ => unimplemented!(),
        };

        (131_072 / (2048 - f)) as u16
    }
}

impl InterruptSource for APU {
    fn get_and_clear_irq(&mut self) -> Option<IrqSource> {
        None
    }
}

impl MemR for APU {
    fn read(&self, addr: u16) -> Result<u8, dbg::TraceEvent> {
        Ok(match addr {
            0xFF10 => self.ch1_swp_reg.0 | 0x80,
            0xFF11 => self.ch1_len_reg.0 | 0x3F,
            0xFF12 => self.ch1_vol_reg.0,
            0xFF13 => self.ch1_flo_reg.0 | 0xFF,
            0xFF14 => self.ch1_fhi_reg.0 | 0xBF,

            0xFF16 => self.ch2_len_reg.0 | 0x3F,
            0xFF17 => self.ch2_vol_reg.0,
            0xFF18 => self.ch2_flo_reg.0 | 0xFF,
            0xFF19 => self.ch2_fhi_reg.0 | 0xBF,

            0xFF1A => self.ch3_snd_reg.0 | 0x7F,
            0xFF1B => self.ch3_len_reg.0,
            0xFF1C => self.ch3_vol_reg.0 | 0x9F,
            0xFF1D => self.ch3_flo_reg.0 | 0xFF,
            0xFF1E => self.ch3_fhi_reg.0 | 0xBF,

            0xFF20 => self.ch4_len_reg.0 | 0xC0,
            0xFF21 => self.ch4_vol_reg.0,
            0xFF22 => self.ch4_cnt_reg.0,
            0xFF23 => self.ch4_ini_reg.0 | 0xBF,

            0xFF24 => self.ctrl_master_reg.0,
            0xFF25 => self.ctrl_output_reg.0,
            0xFF26 => self.ctrl_snd_en_reg.0 | 0x70,

            0xFF30..=0xFF3F => self.wave_ram[usize::from(addr) - 0xFF30],

            // Unused regs in this range: 0xFF15, 0xFF1F, 0xFF27..=0xFF2F
            _ => 0xFF,
        })
    }
}

impl MemW for APU {
    fn write(&mut self, addr: u16, val: u8) -> Result<(), dbg::TraceEvent> {
        match addr {
            0xFF10 => self.ch1_swp_reg.0 = val,
            0xFF11 => self.ch1_len_reg.0 = val,
            0xFF12 => self.ch1_vol_reg.0 = val,
            0xFF13 => self.ch1_flo_reg.0 = val,
            0xFF14 => self.ch1_fhi_reg.0 = val,

            0xFF16 => self.ch2_len_reg.0 = val,
            0xFF17 => self.ch2_vol_reg.0 = val,
            0xFF18 => self.ch2_flo_reg.0 = val,
            0xFF19 => self.ch2_fhi_reg.0 = val,

            0xFF1A => self.ch3_snd_reg.0 = val,
            0xFF1B => self.ch3_len_reg.0 = val,
            0xFF1C => self.ch3_vol_reg.0 = val,
            0xFF1D => self.ch3_flo_reg.0 = val,
            0xFF1E => self.ch3_fhi_reg.0 = val,

            0xFF20 => self.ch4_len_reg.0 = val,
            0xFF21 => self.ch4_vol_reg.0 = val,
            0xFF22 => self.ch4_cnt_reg.0 = val,
            0xFF23 => self.ch4_ini_reg.0 = val,

            0xFF24 => self.ctrl_master_reg.0 = val,
            0xFF25 => self.ctrl_output_reg.0 = val,
            0xFF26 => self.ctrl_snd_en_reg.0 = val,

            0xFF30..=0xFF3F => self.wave_ram[usize::from(addr) - 0xFF30] = val,

            // Unused regs in this range: 0xFF15, 0xFF1F, 0xFF27..=0xFF2F
            _ => (),
        };

        Ok(())
    }
}
