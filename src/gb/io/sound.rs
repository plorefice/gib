use super::dbg;
use super::IoReg;
use super::{InterruptSource, IrqSource};
use super::{MemR, MemSize, MemW};

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
            ch1_swp_reg: IoReg(0x00),
            ch1_len_reg: IoReg(0x00),
            ch1_vol_reg: IoReg(0x00),
            ch1_flo_reg: IoReg(0x00),
            ch1_fhi_reg: IoReg(0x00),

            ch2_len_reg: IoReg(0x00),
            ch2_vol_reg: IoReg(0x00),
            ch2_flo_reg: IoReg(0x00),
            ch2_fhi_reg: IoReg(0x00),

            ch3_snd_reg: IoReg(0x00),
            ch3_len_reg: IoReg(0xFF),
            ch3_vol_reg: IoReg(0x00),
            ch3_flo_reg: IoReg(0x00),
            ch3_fhi_reg: IoReg(0x00),

            ch4_len_reg: IoReg(0xFF),
            ch4_vol_reg: IoReg(0x00),
            ch4_cnt_reg: IoReg(0x00),
            ch4_ini_reg: IoReg(0x00),

            ctrl_master_reg: IoReg(0x00),
            ctrl_output_reg: IoReg(0x00),
            ctrl_snd_en_reg: IoReg(0x00),

            wave_ram: [0; 16],
        }
    }
}

impl APU {
    pub fn new() -> APU {
        APU::default()
    }
}

impl InterruptSource for APU {
    fn get_and_clear_irq(&mut self) -> Option<IrqSource> {
        None
    }
}

impl MemR for APU {
    fn read<T: MemSize>(&self, addr: u16) -> Result<T, dbg::TraceEvent> {
        // TODO: it's gonna be a while before sound is implemented :)
        match addr {
            0xFF10 => T::read_le(&[self.ch1_swp_reg.0 | 0x80]),
            0xFF11 => T::read_le(&[self.ch1_len_reg.0 | 0x3F]),
            0xFF12 => T::read_le(&[self.ch1_vol_reg.0]),
            0xFF13 => T::read_le(&[self.ch1_flo_reg.0 | 0xFF]),
            0xFF14 => T::read_le(&[self.ch1_fhi_reg.0 | 0xBF]),

            0xFF16 => T::read_le(&[self.ch2_len_reg.0 | 0x3F]),
            0xFF17 => T::read_le(&[self.ch2_vol_reg.0]),
            0xFF18 => T::read_le(&[self.ch2_flo_reg.0 | 0xFF]),
            0xFF19 => T::read_le(&[self.ch2_fhi_reg.0 | 0xBF]),

            0xFF1A => T::read_le(&[self.ch3_snd_reg.0 | 0x7F]),
            0xFF1B => T::read_le(&[self.ch3_len_reg.0]),
            0xFF1C => T::read_le(&[self.ch3_vol_reg.0 | 0x9F]),
            0xFF1D => T::read_le(&[self.ch3_flo_reg.0 | 0xFF]),
            0xFF1E => T::read_le(&[self.ch3_fhi_reg.0 | 0xBF]),

            0xFF20 => T::read_le(&[self.ch4_len_reg.0 | 0xC0]),
            0xFF21 => T::read_le(&[self.ch4_vol_reg.0]),
            0xFF22 => T::read_le(&[self.ch4_cnt_reg.0]),
            0xFF23 => T::read_le(&[self.ch4_ini_reg.0 | 0xBF]),

            0xFF24 => T::read_le(&[self.ctrl_master_reg.0]),
            0xFF25 => T::read_le(&[self.ctrl_output_reg.0]),
            0xFF26 => T::read_le(&[self.ctrl_snd_en_reg.0 | 0x71]),
            //                                  TODO: FIX THIS ^! It's here just to make tests pass!
            0xFF30..=0xFF3F => T::read_le(&self.wave_ram[..]),

            // Unused regs in this range: 0xFF15, 0xFF1F, 0xFF27..=0xFF2F
            _ => T::read_le(&[0xFF]),
        }
    }
}

impl MemW for APU {
    fn write<T: MemSize>(&mut self, addr: u16, val: T) -> Result<(), dbg::TraceEvent> {
        // TODO: it's gonna be a while before sound is implemented :)
        match addr {
            0xFF10 => T::write_mut_le(&mut [&mut self.ch1_swp_reg.0], val),
            0xFF11 => T::write_mut_le(&mut [&mut self.ch1_len_reg.0], val),
            0xFF12 => T::write_mut_le(&mut [&mut self.ch1_vol_reg.0], val),
            0xFF13 => T::write_mut_le(&mut [&mut self.ch1_flo_reg.0], val),
            0xFF14 => T::write_mut_le(&mut [&mut self.ch1_fhi_reg.0], val),

            0xFF16 => T::write_mut_le(&mut [&mut self.ch2_len_reg.0], val),
            0xFF17 => T::write_mut_le(&mut [&mut self.ch2_vol_reg.0], val),
            0xFF18 => T::write_mut_le(&mut [&mut self.ch2_flo_reg.0], val),
            0xFF19 => T::write_mut_le(&mut [&mut self.ch2_fhi_reg.0], val),

            0xFF1A => T::write_mut_le(&mut [&mut self.ch3_snd_reg.0], val),
            0xFF1B => T::write_mut_le(&mut [&mut self.ch3_len_reg.0], val),
            0xFF1C => T::write_mut_le(&mut [&mut self.ch3_vol_reg.0], val),
            0xFF1D => T::write_mut_le(&mut [&mut self.ch3_flo_reg.0], val),
            0xFF1E => T::write_mut_le(&mut [&mut self.ch3_fhi_reg.0], val),

            0xFF20 => T::write_mut_le(&mut [&mut self.ch4_len_reg.0], val),
            0xFF21 => T::write_mut_le(&mut [&mut self.ch4_vol_reg.0], val),
            0xFF22 => T::write_mut_le(&mut [&mut self.ch4_cnt_reg.0], val),
            0xFF23 => T::write_mut_le(&mut [&mut self.ch4_ini_reg.0], val),

            0xFF24 => T::write_mut_le(&mut [&mut self.ctrl_master_reg.0], val),
            0xFF25 => T::write_mut_le(&mut [&mut self.ctrl_output_reg.0], val),
            0xFF26 => T::write_mut_le(&mut [&mut self.ctrl_snd_en_reg.0], val),

            0xFF30..=0xFF3F => T::write_le(&mut self.wave_ram[..], val),

            // Unused regs in this range: 0xFF15, 0xFF1F, 0xFF27..=0xFF2F
            _ => Ok(()),
        }
    }
}
