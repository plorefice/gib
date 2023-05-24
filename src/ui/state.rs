use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Error;
use crossbeam::queue::ArrayQueue;
use gib_core::{bus::Bus, cpu::CPU, dbg, GameBoy};

pub struct EmuState {
    gb: GameBoy,
    rom_file: Option<PathBuf>,

    // Sound-related fields
    snd_sink: Option<Arc<ArrayQueue<i16>>>,
    snd_sample_rate: f32,

    // Emulation-related fields
    turbo_mode: bool,
    step_to_next: bool,
    run_to_breakpoint: bool,
    trace_event: Option<dbg::TraceEvent>,
}

impl Default for EmuState {
    fn default() -> Self {
        Self {
            gb: GameBoy::new(),
            rom_file: None,

            snd_sink: None,
            snd_sample_rate: 0f32,

            turbo_mode: false,
            step_to_next: false,
            run_to_breakpoint: false,
            trace_event: None,
        }
    }
}

impl EmuState {
    pub fn load_rom<P: AsRef<Path>>(&mut self, rom: P) -> Result<(), Error> {
        self.rom_file = Some(rom.as_ref().to_path_buf());

        self.reset()
    }

    pub fn pause(&mut self) {
        self.turbo_mode = false;
        self.step_to_next = false;
        self.run_to_breakpoint = false;
        self.gb.cpu_mut().pause();
    }

    /// Performs a single emulation step, depending on the emulator's state:
    ///
    /// * if we are in step mode, execute a single instruction
    /// * if we are in run mode, run to audio sync (ie. audio queue full)
    ///
    /// In both cases, if an event happens, pause the emulator.
    pub fn do_step(&mut self) {
        if self.paused() {
            return;
        }

        self.trace_event = None;

        let res = if self.step_to_next {
            let r = self.gb.step();
            self.pause();
            r
        } else if self.turbo_mode {
            self.gb.run_for_vblank()
        } else if self.run_to_breakpoint {
            self.run_to_audio_sync()
        } else {
            Ok(())
        };

        if let Err(ref evt) = res {
            self.trace_event = Some(*evt);
            self.pause();
        };
    }

    /// Runs the emulator until the audio queue is full, to avoid dropping
    /// audio samples and cause skipping/popping.
    fn run_to_audio_sync(&mut self) -> Result<(), dbg::TraceEvent> {
        if let Some(ref sink) = self.snd_sink {
            while sink.len() < sink.capacity() {
                self.gb.step()?;
            }
        }
        Ok(())
    }

    /// Sets the emulator's audio sink and sample rate.
    pub fn set_audio_sink(&mut self, sink: Arc<ArrayQueue<i16>>, sample_rate: f32) {
        self.snd_sink = Some(sink.clone());
        self.snd_sample_rate = sample_rate;

        self.gb.set_audio_sink(sink, sample_rate);
    }

    pub fn last_event(&self) -> &Option<dbg::TraceEvent> {
        &self.trace_event
    }

    pub fn set_single_step(&mut self) {
        self.step_to_next = true;
    }

    pub fn set_running(&mut self) {
        self.run_to_breakpoint = true;
    }

    /// Sets or resets turbo mode.
    ///
    /// In turbo mode, the emulator runs to video-sync rather than audio-sync,
    /// likely dropping audio samples.
    pub fn set_turbo(&mut self, enable: bool) {
        self.turbo_mode = enable;
    }

    pub fn paused(&mut self) -> bool {
        self.gb.cpu().paused() && !(self.step_to_next || self.run_to_breakpoint)
    }

    /// Reset the emulator's sate.
    pub fn reset(&mut self) -> Result<(), Error> {
        let Some(rom_file) = self.rom_file.as_ref() else {
            return Ok(());
        };

        // Save breakpoints to restore after reset
        let bkps = self.cpu().breakpoints().clone();

        self.gb = GameBoy::new();
        self.gb.load_rom(&(fs::read(rom_file)?)[..])?;

        if let Some(ref sink) = self.snd_sink {
            self.gb.set_audio_sink(sink.clone(), self.snd_sample_rate);
        }

        for b in bkps.iter() {
            self.cpu_mut().set_breakpoint(*b);
        }

        // Default to running state
        self.set_running();

        Ok(())
    }

    pub fn gameboy(&self) -> &GameBoy {
        &self.gb
    }

    pub fn gameboy_mut(&mut self) -> &mut GameBoy {
        &mut self.gb
    }

    pub fn cpu(&self) -> &CPU {
        self.gb.cpu()
    }

    pub fn cpu_mut(&mut self) -> &mut CPU {
        self.gb.cpu_mut()
    }

    pub fn bus(&self) -> &Bus {
        self.gb.bus()
    }
}
