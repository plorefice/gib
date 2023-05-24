use crossbeam::channel::{Receiver, Sender};

use crate::{bus::Bus, cpu::Cpu, dbg, io::JoypadState};

pub const CPU_CLOCK: u64 = 4_194_304; // Hz
pub const HSYNC_CLOCK: u64 = 9_198; // Hz

const CYCLES_PER_HSYNC: u64 = CPU_CLOCK / HSYNC_CLOCK;

pub struct GameBoy {
    cpu: Cpu,
    bus: Bus,

    cycles: u64,
}

impl Default for GameBoy {
    fn default() -> GameBoy {
        GameBoy {
            cpu: Cpu::new(),
            bus: Bus::new(),

            cycles: 0x18FCC,
        }
    }
}

impl GameBoy {
    /// Create a new Game Boy instance.
    pub fn new() -> GameBoy {
        GameBoy::default()
    }

    /// Resets the Game Boy to its power-up state.
    ///
    /// The only things preserved by this operation are some debugging information related to the
    /// CPU (eg. breakpoints) and the audio channel for the APU, if one was configured.
    pub fn reset(&mut self) {
        self.cpu.reset();
        self.bus.reset();
        self.cycles = Self::default().cycles;
    }

    pub fn load_rom(&mut self, rom: &[u8]) -> Result<(), dbg::TraceEvent> {
        self.bus.load_rom(rom)
    }

    pub fn step(&mut self) -> Result<(), dbg::TraceEvent> {
        // The first tick fetches the opcode
        self.tick()?;

        // The others perform the instruction itself, if necessary
        while self.cpu.executing {
            self.tick()?;
        }

        // Finally, handle any interrupts that arised
        self.handle_irqs()?;

        Ok(())
    }

    fn tick(&mut self) -> Result<(), dbg::TraceEvent> {
        self.cpu.tick(&mut self.bus)?;

        // Section 4.10 of "The Cycle-Accurate GameBoy Docs"
        // =================================================
        // The HALT bug triggers if a HALT instruction is executed when IME = 0 && (IE & IF) != 0.
        // In this case, the CPU is NOT halted, and the HALT bug is triggered, causing the PC
        // to NOT be incremented when the next instruction is executed (ie. the next instruction
        // is executed twice).
        if *self.cpu.halted.loaded()
            && (!*self.cpu.intr_enabled.value() && self.bus.itr.pending_irqs())
        {
            self.cpu.halt_bug = true;
            self.cpu.halted.reset(false);
        }

        self.bus.tick()?;

        self.cycles += 4;

        Ok(())
    }

    fn handle_irqs(&mut self) -> Result<(), dbg::TraceEvent> {
        if let Some(id) = self.bus.itr.get_pending_irq() {
            let addr = (0x40 + 0x08 * id) as u16;

            self.cpu.halted.reset(false);

            // If IME = 1, disable HALT mode (if in it), set IME = 0,
            // clear IF and run the corresponding ISR.
            // If IME = 0, simply leave HALT mode.
            if *self.cpu.intr_enabled.value() {
                self.cpu.intr_enabled.reset(false);
                self.bus.itr.clear_irq(id);

                // Jump to interrupt service routing and wait 5 cycles until
                // the jump has been performed.
                self.cpu.jump_to_isr(&mut self.bus, addr)?;

                while self.cpu.executing {
                    self.tick()?;
                }
            }
        }
        Ok(())
    }

    pub fn run_for_vblank(&mut self) -> Result<(), dbg::TraceEvent> {
        let until = self.cycles + (CYCLES_PER_HSYNC * 154);

        while self.cycles < until {
            self.step()?;
        }
        Ok(())
    }

    /// Configures the audio channel for the sound peripheral, along with the required sample rate.
    pub fn configure_audio_channel(&mut self, source: AudioSource, sample_rate: f32) {
        self.bus.apu.set_sample_rate(sample_rate);
        self.bus.apu.set_audio_source(source);
    }

    /// Enables or disables "sync-by-audio" emulation.
    ///
    /// When enabled, the emulation will block until one or more audio samples are requested by
    /// the playback stream. Due to the precise timing of audio playback, this is the best way to
    /// synchronize emulation speed to real-world timings, provided that the playback sample rate
    /// matches the sample rate of the Game Boy's APU. It may however introduce some input latency,
    /// since the emulator can't handle key presses while blocked.
    ///
    /// When disabled, the APU will drop sound samples if the audio stream is already saturated.
    /// This is what's called "turbo speed" in some emulators: emulation will run as fast as
    /// possible, out-of-sync with wall-clock time, resulting in much higher frame skip and
    /// crackling audio.
    pub fn enable_audio_sync(&mut self, enable: bool) {
        if let Some(ref mut source) = self.bus.apu.audio_source_mut() {
            source.set_blocking(enable);
        }
    }

    /// Marks the given key as pressed.
    pub fn press_key(&mut self, key: JoypadState) {
        self.bus.joy.set_pressed_keys(key);
    }

    /// Marks the given key as not pressed.
    pub fn release_key(&mut self, key: JoypadState) {
        self.bus.joy.set_release_keys(key);
    }

    pub fn rasterize(&self, vbuf: &mut [u8]) {
        self.bus.ppu.rasterize(vbuf);
    }

    pub fn clock_cycles(&self) -> u64 {
        self.cycles
    }

    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    pub fn cpu_mut(&mut self) -> &mut Cpu {
        &mut self.cpu
    }

    pub fn bus(&self) -> &Bus {
        &self.bus
    }
}

/// The trasmitting end of an audio stream's channel.
pub struct AudioSource {
    channel: Sender<i16>,
    blocking: bool,
}

impl AudioSource {
    /// Sets the audio source's blocking behavior when pushing a new sample.
    ///
    /// If non-blocking, new samples are discarded when there's no space left in the channel.
    pub fn set_blocking(&mut self, blocking: bool) {
        self.blocking = blocking
    }

    /// Pushes a new audio sample to the audio stream.
    pub fn push(&mut self, sample: i16) {
        if self.blocking {
            self.channel.send(sample).ok();
        } else {
            self.channel.try_send(sample).ok();
        }
    }
}

/// The receiving end of an audio stream's channel.
pub struct AudioSink {
    channel: Receiver<i16>,
    blocking: bool,
}

impl AudioSink {
    /// Sets the audio source's blocking behavior when fetching a sample.
    ///
    /// In non-blocking mode, [`AudioSink::pop`] returns `None` if the channel is empty.
    pub fn set_blocking(&mut self, blocking: bool) {
        self.blocking = blocking
    }

    /// Returns the next audio sample in the channel, or `None` in case of errors.
    pub fn pop(&mut self) -> Option<i16> {
        if self.blocking {
            self.channel.recv().ok()
        } else {
            self.channel.try_recv().ok()
        }
    }
}

/// Returns both ends of a new audio channel with a given capacity.
///
/// Usually, the [`AudioSource`] will be passed to the emulator using
/// [`GameBoy::configure_audio_channel`], while the [`AudioSink`] will be used by the audio thread
/// for audio playback.
pub fn create_sound_channel(capacity: usize) -> (AudioSource, AudioSink) {
    let (sender, receiver) = crossbeam::channel::bounded(capacity);
    (
        AudioSource {
            channel: sender,
            blocking: true,
        },
        AudioSink {
            channel: receiver,
            blocking: true,
        },
    )
}
