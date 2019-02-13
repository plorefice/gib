use crossbeam::queue::ArrayQueue;
use failure::format_err;
use failure::Error;

use std::sync::Arc;

/// Component responsible for audio playback.
pub struct SoundEngine {
    device: cpal::Device,
    format: cpal::Format,
}

impl SoundEngine {
    /// Creates a new instance of the sound engine using the system's default output device.
    pub fn new() -> Result<SoundEngine, Error> {
        // Open the system's default output device
        let device =
            cpal::default_output_device().ok_or_else(|| format_err!("no output device found"))?;
        let format = device.default_output_format()?;

        Ok(SoundEngine { device, format })
    }

    /// Returns the engine's current sample rate.
    pub fn get_sample_rate(&self) -> f32 {
        self.format.sample_rate.0 as f32
    }

    /// Starts the sound engine. The audio playback happens in a seprate thread,
    /// with audio samples being received from the provided sample queue.
    ///
    /// An error is returned if a new audio stream cannot be created.
    pub fn start(&mut self, sample_queue: Arc<ArrayQueue<i16>>) -> Result<(), Error> {
        // Create and start a new stream
        let event_loop = cpal::EventLoop::new();
        let stream_id = event_loop.build_output_stream(&self.device, &self.format)?;
        let format = self.format.clone();

        event_loop.play_stream(stream_id.clone());

        // Run the stream's blocking event loop in a separate thread
        std::thread::spawn(move || {
            let mut last_sample = 0f32;

            event_loop.run(move |_, data| {
                let mut next_value = || {
                    if let Ok(sample) = sample_queue.pop() {
                        last_sample = f32::from(sample) * 0.001;
                    }
                    last_sample
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

        Ok(())
    }
}
