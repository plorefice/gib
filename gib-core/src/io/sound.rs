use bitflags::bitflags;

use super::dbg;
use super::IoReg;
use super::{InterruptSource, IrqSource};
use super::{MemR, MemW};

const CLK_64_RELOAD: u32 = 4_194_304 / 64;
const CLK_128_RELOAD: u32 = 4_194_304 / 128;
const CLK_256_RELOAD: u32 = 4_194_304 / 256;

bitflags! {
    // NRx1 - Channel x Sound Length/Wave Pattern Duty (R/W)
    struct NRx1: u8 {
        const WAVE_DUTY = 0b_1100_0000;
        const SOUND_LEN = 0b_0011_1111;
    }
}

bitflags! {
    // NRx2 - Channel x Volume Envelope (R/W)
    struct NRx2: u8 {
        const START_VOL  = 0b_1111_0000;
        const ENV_DIR    = 0b_0000_1000;
        const ENV_PERIOD = 0b_0000_0111;
    }
}

bitflags! {
    // NRx4 - Channel x Frequency hi data (R/W)
    struct NRx4: u8 {
        const TRIGGER = 0b_1000_0000;
        const LEN_EN  = 0b_0100_0000;
        const FREQ_HI = 0b_0000_0111;
    }
}

/// A sound channel able to produce quadrangular wave patterns
/// with optional sweep and envelope functions.
struct ToneChannel {
    // Channel registers
    nrx1: NRx1,
    nrx2: NRx2,
    nrx3: IoReg<u8>,
    nrx4: NRx4,

    volume: u8,
    vol_ctr: u8,
    vol_env_enabled: bool,

    enabled: bool,
}

impl ToneChannel {
    /// Advances the volume envelope by 1/64th of a second.
    fn tick_vol_env(&mut self) {
        let period = (self.nrx2 & NRx2::ENV_PERIOD).bits();

        // When the timer generates a clock and the envelope period is not zero,
        // a new volume is calculated by adding or subtracting 1 from the current volume.
        if self.vol_env_enabled && period > 0 {
            self.nrx2 = (self.nrx2 & !NRx2::ENV_PERIOD) | NRx2::from_bits_truncate(period - 1);

            let new_volume = if self.nrx2.contains(NRx2::ENV_DIR) {
                self.volume + 1
            } else {
                self.volume - 1
            };

            // If this new volume within the 0 to 15 range, the volume is updated,
            // otherwise it is left unchanged and no further automatic increments/decrements
            // are made to the volume until the channel is triggered again.
            if new_volume <= 15 {
                self.volume = new_volume;
            } else {
                self.vol_env_enabled = false;
            }
        }
    }

    /// Advances the length counter by 1/256th of a second.
    fn tick_len_ctr(&mut self) {
        let len = (self.nrx1 & NRx1::SOUND_LEN).bits();

        // When clocked while enabled by NRx4 and the counter is not zero, length is decremented
        if self.nrx4.contains(NRx4::LEN_EN) && len != 0 {
            let len = len - 1;

            self.nrx1 = (self.nrx1 & !NRx1::SOUND_LEN) | NRx1::from_bits_truncate(len);

            // If it becomes zero, the channel is disabled
            if len == 0 {
                self.enabled = false;
            }
        }
    }

    /// Returns the channel's current tone frequency.
    fn get_frequency(&self) -> u16 {
        let hi = u32::from((self.nrx4 & NRx4::FREQ_HI).bits());
        let lo = u32::from(self.nrx3.0);

        (131_072 / (2048 - ((hi << 8) | lo))) as u16
    }

    /// Returns the channel's current volume.
    fn get_volume(&self) -> u16 {
        u16::from(self.enabled) * u16::from(self.volume)
    }

    /// Handles a write to the NRx4 register.
    fn write_to_nr4(&mut self, val: u8) {
        self.nrx4 = NRx4::from_bits_truncate(val);

        // When a TRIGGER occurs, a number of things happen
        if self.nrx4.contains(NRx4::TRIGGER) {
            // Channel is enabled
            self.enabled = true;

            // If length counter is zero, it is set to 64 (256 for wave channel)
            if (self.nrx1 & NRx1::SOUND_LEN).bits() == 0 {
                self.nrx1 |= NRx1::SOUND_LEN;
            }

            // Volume envelope timer is reloaded with period and
            // channel volume is reloaded from NRx2.
            self.volume = (self.nrx2 & NRx2::START_VOL).bits() >> 4;
            self.vol_ctr = (self.nrx2 & NRx2::ENV_PERIOD).bits();
            self.vol_env_enabled = true;
        }
    }
}

impl MemR for ToneChannel {
    fn read(&self, addr: u16) -> Result<u8, dbg::TraceEvent> {
        Ok(match addr {
            0 => self.nrx1.bits() | 0x3F,
            1 => self.nrx2.bits(),
            2 => self.nrx3.0 | 0xFF,
            3 => self.nrx4.bits() | 0xBF,
            _ => unreachable!(),
        })
    }
}

impl MemW for ToneChannel {
    fn write(&mut self, addr: u16, val: u8) -> Result<(), dbg::TraceEvent> {
        match addr {
            0 => self.nrx1 = NRx1::from_bits_truncate(val),
            1 => self.nrx2 = NRx2::from_bits_truncate(val),
            2 => self.nrx3.0 = val,
            3 => self.write_to_nr4(val),
            _ => unreachable!(),
        };

        Ok(())
    }
}

pub struct APU {
    // Channel 1 registers
    ch1_swp_reg: IoReg<u8>,
    ch1_len_reg: IoReg<u8>,
    ch1_vol_reg: IoReg<u8>,
    ch1_flo_reg: IoReg<u8>,
    ch1_fhi_reg: IoReg<u8>,

    // Channel 2
    ch2: ToneChannel,

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

    // Frame sequencer clocks
    clk_64: u32,
    clk_128: u32,
    clk_256: u32,
}

impl Default for APU {
    fn default() -> APU {
        APU {
            ch1_swp_reg: IoReg(0x80),
            ch1_len_reg: IoReg(0x8F),
            ch1_vol_reg: IoReg(0xF3),
            ch1_flo_reg: IoReg(0x00),
            ch1_fhi_reg: IoReg(0xBF),

            ch2: ToneChannel {
                nrx1: NRx1::from_bits_truncate(0x3F),
                nrx2: NRx2::from_bits_truncate(0x00),
                nrx3: IoReg(0x00),
                nrx4: NRx4::from_bits_truncate(0xBF),

                volume: 0,
                vol_ctr: 0,
                vol_env_enabled: false,
                enabled: false,
            },

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

            // TODO according to [1] these clocks are slightly out of phase,
            // initialization and ticking should be fixed accordingly.
            // [1] http://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware#Frame_Sequencer
            clk_64: CLK_64_RELOAD,
            clk_128: CLK_128_RELOAD,
            clk_256: CLK_256_RELOAD,
        }
    }
}

impl APU {
    pub fn new() -> APU {
        APU::default()
    }

    /// Advances the sound controller state machine by a single M-cycle.
    pub fn tick(&mut self) {
        self.clk_64 -= 4;
        self.clk_128 -= 4;
        self.clk_256 -= 4;

        // Volume envelope clock tick
        if self.clk_64 == 0 {
            self.clk_64 = CLK_64_RELOAD;

            self.ch2.tick_vol_env();
        }

        // Sweep clock tick
        if self.clk_128 == 0 {
            self.clk_128 = CLK_128_RELOAD;
        }

        // Lenght counter clock tick
        if self.clk_256 == 0 {
            self.clk_256 = CLK_256_RELOAD;

            self.ch2.tick_len_ctr();
        }
    }

    /// Returns the output frequency of the sound mixer.
    pub fn get_mixer_output(&self) -> u16 {
        // TODO handle volume appropriately
        if self.ch2.get_volume() > 0 {
            self.ch2.get_frequency()
        } else {
            0
        }
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

            0xFF16..=0xFF19 => self.ch2.read(addr - 0xFF16)?,

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

            0xFF16..=0xFF19 => self.ch2.write(addr - 0xFF16, val)?,

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
