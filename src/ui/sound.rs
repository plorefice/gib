use gib_core::io::MixerOut;

use failure::format_err;
use failure::Error;

use std::sync::mpsc;

/// Component responsible for audio playback.
pub struct SoundEngine {
    sample_updates: mpsc::Sender<MixerOut>,
}

impl SoundEngine {
    /// Creates and starts a new sound engine.
    ///
    /// An error is returned if an output device cannot be found or opened.
    pub fn start() -> Result<SoundEngine, Error> {
        // Open the system's default output device
        let device =
            cpal::default_output_device().ok_or_else(|| format_err!("no output device found"))?;
        let format = device.default_output_format()?;

        // Create and start a new stream
        let event_loop = cpal::EventLoop::new();
        let stream_id = event_loop.build_output_stream(&device, &format)?;
        event_loop.play_stream(stream_id.clone());

        let (sender, receiver) = mpsc::channel();

        // Run the stream's blocking event loop in a separate thread
        std::thread::spawn(move || {
            let sample_rate = format.sample_rate.0 as f32;

            let mut sample_clock = 0f32;
            let mut sample_frequency = 0f32;
            let mut sample_volume = 0f32;

            event_loop.run(move |_, data| {
                // Before a new sample is produced, see if a new one has been received
                if let Ok(MixerOut { frequency, volume }) = receiver.try_recv() {
                    // IMPORTANT: this prevents popping, leave it here!
                    sample_clock *= sample_frequency / f32::from(frequency);
                    sample_frequency = f32::from(frequency);
                    sample_volume = f32::from(volume) / 100.0;
                }

                // TODO right now, a 50% square wave is produced. This should be configurable
                // to the supported intervals of 12.5%, 25%, 50% and 75%.
                let mut next_value = || {
                    sample_clock = (sample_clock + 1.0) % sample_rate;
                    (sample_clock * sample_frequency * 2.0 * std::f32::consts::PI / sample_rate)
                        .sin()
                        .signum()
                        * sample_volume
                };

                // Push the new sample to the stream in all possible formats
                match data {
                    cpal::StreamData::Output {
                        buffer: cpal::UnknownTypeOutputBuffer::U16(mut buffer),
                    } => {
                        for sample in buffer.chunks_mut(format.channels as usize) {
                            let value =
                                ((next_value() * 0.5 + 0.5) * f32::from(std::u16::MAX)) as u16;
                            for out in sample.iter_mut() {
                                *out = value;
                            }
                        }
                    }
                    cpal::StreamData::Output {
                        buffer: cpal::UnknownTypeOutputBuffer::I16(mut buffer),
                    } => {
                        for sample in buffer.chunks_mut(format.channels as usize) {
                            let value = (next_value() * f32::from(std::i16::MAX)) as i16;
                            for out in sample.iter_mut() {
                                *out = value;
                            }
                        }
                    }
                    cpal::StreamData::Output {
                        buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer),
                    } => {
                        for sample in buffer.chunks_mut(format.channels as usize) {
                            let value = next_value();
                            for out in sample.iter_mut() {
                                *out = value;
                            }
                        }
                    }
                    _ => (),
                }
            });
        });

        Ok(SoundEngine {
            sample_updates: sender,
        })
    }

    /// Pushes a new audio sample to the engine for playback.
    pub fn push_new_sample(&mut self, sample: MixerOut) -> Result<(), Error> {
        self.sample_updates.send(sample)?;
        Ok(())
    }
}
