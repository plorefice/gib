use anyhow::{anyhow, Error};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, OutputCallbackInfo, Stream, StreamConfig,
};
use gib_core::AudioSink;

/// Component responsible for audio playback.
pub struct SoundEngine {
    device: Device,
    config: StreamConfig,
    stream: Option<Stream>,
}

impl SoundEngine {
    /// Creates a new instance of the sound engine using the system's default output device.
    pub fn new() -> Result<SoundEngine, Error> {
        // Open the system's default output device
        let device = cpal::default_host()
            .default_output_device()
            .ok_or_else(|| anyhow!("no output device found"))?;

        let config = device.default_output_config()?.into();

        Ok(SoundEngine {
            device,
            config,
            stream: None,
        })
    }

    /// Returns the engine's current sample rate.
    pub fn get_sample_rate(&self) -> f32 {
        self.config.sample_rate as f32
    }

    /// Starts the sound engine. The audio playback happens in a seprate thread,
    /// with audio samples being received from the provided channel.
    ///
    /// An error is returned if a new audio stream cannot be created.
    pub fn start(&mut self, mut sink: AudioSink) -> Result<(), Error> {
        // This closure will fetch the next sample from the stream, or replicate the last sample
        // if no new sample is available.
        let mut last_sample = 0f32;
        let mut next_sample = move || {
            if let Some(sample) = sink.pop() {
                last_sample = sample as f32 * 0.001;
            }
            last_sample
        };

        self.stream = {
            let channels = self.config.channels as usize;
            let stream = self.device.build_output_stream(
                &self.config,
                move |output: &mut [f32], _: &OutputCallbackInfo| {
                    // Push the new sample to the stream
                    for sample in output.chunks_mut(channels) {
                        let value = next_sample();
                        for out in sample.iter_mut() {
                            *out = value;
                        }
                    }
                },
                move |err| println!("Sound error: {}", err),
                None,
            )?;

            stream.play()?;

            // We need to keep the stream alive, otherwise the spawned thread will get dropped!
            Some(stream)
        };

        Ok(())
    }
}
