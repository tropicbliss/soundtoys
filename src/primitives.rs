//! Sound synthesis primitives that you can't live without.

use crate::{errors::AudioError, instruments::Instrument, note::Note};
use atomic_float::AtomicF64;
use cpal::{
    traits::{DeviceTrait, StreamTrait},
    Device, FromSample, SizedSample, Stream, StreamConfig, SupportedStreamConfig,
};
use std::{
    any::TypeId,
    sync::{atomic::Ordering, Arc},
};

pub struct SoundMaker {
    device: Device,
    config: SupportedStreamConfig,
    tick: Arc<AtomicF64>,
    stream: Option<Stream>,
}

impl SoundMaker {
    /// Computers do not produce sound waves smoothly. The greater the sample rate, the more precise and
    /// smoother the amplitude (y-axis) of the wave. The human hearing range is between 20Hz and 20,000Hz.
    /// 44100Hz is slightly more than double than 20,000Hz. This means that in the most extreme cases where
    /// the current frequency is 20,000Hz, the accuracy of the emulated soundwave is still relatively decent.
    /// The number of channels determine whether the sound system is mono or stereo. The greater the buffer
    /// size, the lower the delay, but you pay the cost of having more frequent system interrupts, slowing
    /// down your system or even result in choppy playback as the CPU is unable to keep up with the load
    /// of sound synthesis. If your sample rate is 44,100Hz, this means that the CPU has to calculate the
    /// amplitude 44,100 times per second. This is impossible, and thus a buffer is required to temporarily
    /// store sound data ahead of time, with each frame (sample block) in a buffer containing a segment of sound such that
    /// the CPU will only interrupt x number of times per second, where x is the number of frames that is
    /// consumed by the sound driver in a second.
    pub fn new(device: Device, config: SupportedStreamConfig) -> Self {
        Self {
            device,
            config,
            tick: Arc::new(AtomicF64::new(0.0)),
            stream: None,
        }
    }

    /// Gets the current CPU time starting from when this struct is first initialized.
    pub fn get_time(&self) -> f64 {
        self.tick.load(Ordering::Relaxed)
    }

    /// Accepts a callback that provides the CPU time and returns the frequency (in Hz). The sound can
    /// be manipulated at any time through the use of atomics or mutexes. This also spawns a
    /// thread that plays audio in the background, but stops playing when `SoundMaker` is dropped.
    pub fn set_callback<F>(&mut self, f: F) -> Result<(), AudioError>
    where
        F: Fn(f64) -> f64 + Send + 'static,
    {
        let stream = match self.config.sample_format() {
            cpal::SampleFormat::I8 => self.stream_make::<F, i8>(f),
            cpal::SampleFormat::I16 => self.stream_make::<F, i16>(f),
            cpal::SampleFormat::I32 => self.stream_make::<F, i32>(f),
            cpal::SampleFormat::I64 => self.stream_make::<F, i64>(f),
            cpal::SampleFormat::U8 => self.stream_make::<F, u8>(f),
            cpal::SampleFormat::U16 => self.stream_make::<F, u16>(f),
            cpal::SampleFormat::U32 => self.stream_make::<F, u32>(f),
            cpal::SampleFormat::U64 => self.stream_make::<F, u64>(f),
            cpal::SampleFormat::F32 => self.stream_make::<F, f32>(f),
            cpal::SampleFormat::F64 => self.stream_make::<F, f64>(f),
            _ => unreachable!(),
        }?;
        stream.play()?;
        self.stream = Some(stream);
        Ok(())
    }

    fn stream_make<F, T>(&self, f: F) -> Result<Stream, AudioError>
    where
        F: Fn(f64) -> f64 + Send + 'static,
        T: SizedSample + FromSample<f64>,
    {
        let config: StreamConfig = self.config.clone().into();
        let sample_rate = config.sample_rate.0 as f64;
        let time_step = 1.0 / sample_rate;
        let tick = Arc::clone(&self.tick);
        let nchannels = config.channels as usize;
        let err_fn = |err| eprintln!("Error building output sound stream: {}", err);
        let stream = self.device.build_output_stream(
            &config,
            move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
                for frame in output.chunks_mut(nchannels) {
                    let s = f(tick.load(Ordering::Relaxed));
                    let value = T::from_sample(s);
                    for sample in frame.iter_mut() {
                        *sample = value;
                    }
                    tick.fetch_add(time_step, Ordering::Relaxed);
                }
            },
            err_fn,
            None,
        )?;
        Ok(stream)
    }

    /// Get a note instance with the current wall time data when this method is called.
    pub(crate) fn create_note(
        &self,
        note_id: u8,
        instrument_id: TypeId,
        instrument: Box<dyn Instrument>,
    ) -> Note {
        Note {
            id: note_id + 64,
            on: self.get_time(),
            off: 0.0,
            active: true,
            channel: instrument,
            instrument_id,
        }
    }
}
