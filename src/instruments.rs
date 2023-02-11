//! A bunch of ready-made instruments for your perusal. Feel free to read the docs and the source code
//! to guide you on implementing your own virtual instrument. Do not be afraid to do your own
//! experimentation, this is a result of trial and error.

use crate::{
    note::scale,
    oscillators::{osc, EnvelopeADSR, LowFrequencyOscillator, Oscillator},
};
use dyn_clone::DynClone;

pub trait Instrument: Send + DynClone {
    fn sound(
        &self,
        time: f64,
        time_on: f64,
        time_off: f64,
        note_id: u8,
        note_finished: &mut bool,
    ) -> f64;

    fn get_name() -> &'static str
    where
        Self: Sized;
}

dyn_clone::clone_trait_object!(Instrument);

/// This bell is akin to a glockenspiel. Since a bell is a crisp and clean instrument, we do not want to add
/// any noise. Since a bell has almost no sustain, we need to adjust the envelope to suit that as well. This
/// involves shortening the attack time, and increasing the decay time for the trailing off, and setting the
/// sustain amplitude to 0. We also double the input frequency to make the bell sound higher pitched.
#[derive(Clone)]
pub struct Bell {
    pub env: EnvelopeADSR,
    pub volume: f64,
}

impl Bell {
    pub fn new() -> Self {
        let mut env = EnvelopeADSR::default();
        env.attack_time = 0.01;
        env.decay_time = 1.0;
        env.sustain_amplitude = 0.0;
        env.release_time = 1.0;
        let volume = 1.0;
        Self { env, volume }
    }
}

impl Instrument for Bell {
    fn sound(
        &self,
        time: f64,
        time_on: f64,
        time_off: f64,
        note_id: u8,
        note_finished: &mut bool,
    ) -> f64 {
        let amplitude = self.env.amplitude(time, time_on, time_off);
        if amplitude <= 0.0 {
            *note_finished = true;
        }
        let sound =
            1.00 * osc(
                time - time_on,
                scale(note_id as i32 + 12),
                Oscillator::Sine,
                Some(LowFrequencyOscillator {
                    hertz: 5.0,
                    amplitude: 0.0,
                }),
            ) + 0.50
                * osc(
                    time - time_on,
                    scale(note_id as i32 + 24),
                    Oscillator::Sine,
                    None,
                )
                + 0.25
                    * osc(
                        time - time_on,
                        scale(note_id as i32 + 36),
                        Oscillator::Sine,
                        None,
                    );
        amplitude * sound * self.volume
    }

    fn get_name() -> &'static str
    where
        Self: Sized,
    {
        "Bell"
    }
}

/// 8-bit bell.
#[derive(Clone)]
pub struct Bell8 {
    pub env: EnvelopeADSR,
    pub volume: f64,
}

impl Bell8 {
    pub fn new() -> Self {
        let mut env = EnvelopeADSR::default();
        env.attack_time = 0.01;
        env.decay_time = 0.5;
        env.sustain_amplitude = 0.8;
        env.release_time = 1.0;
        let volume = 1.0;
        Self { env, volume }
    }
}

impl Instrument for Bell8 {
    fn sound(
        &self,
        time: f64,
        time_on: f64,
        time_off: f64,
        note_id: u8,
        note_finished: &mut bool,
    ) -> f64 {
        let amplitude = self.env.amplitude(time, time_on, time_off);
        if amplitude <= 0.0 {
            *note_finished = true;
        }
        let sound =
            1.00 * osc(
                time - time_on,
                scale(note_id as i32),
                Oscillator::Square,
                Some(LowFrequencyOscillator {
                    hertz: 5.0,
                    amplitude: 0.0,
                }),
            ) + 0.50
                * osc(
                    time - time_on,
                    scale(note_id as i32 + 12),
                    Oscillator::Sine,
                    None,
                )
                + 0.25
                    * osc(
                        time - time_on,
                        scale(note_id as i32 + 24),
                        Oscillator::Sine,
                        None,
                    );
        amplitude * sound * self.volume
    }

    fn get_name() -> &'static str
    where
        Self: Sized,
    {
        "8-Bit Bell"
    }
}

/// Since a harmonica is a reed instrument, you want to use a square wave. Since it sounds pretty breathy,
/// we also add some noise to it.
#[derive(Clone)]
pub struct Harmonica {
    pub env: EnvelopeADSR,
    pub volume: f64,
}

impl Harmonica {
    pub fn new() -> Self {
        let mut env = EnvelopeADSR::default();
        env.attack_time = 0.0;
        env.decay_time = 1.0;
        env.sustain_amplitude = 0.95;
        env.release_time = 0.1;
        let volume = 0.3;
        Self { env, volume }
    }
}

impl Instrument for Harmonica {
    fn sound(
        &self,
        time: f64,
        time_on: f64,
        time_off: f64,
        note_id: u8,
        note_finished: &mut bool,
    ) -> f64 {
        let amplitude = self.env.amplitude(time, time_on, time_off);
        if amplitude <= 0.0 {
            *note_finished = true;
        }
        let sound =
            1.00 * osc(
                time_on - time,
                scale(note_id as i32 - 12),
                Oscillator::SawAna(None),
                Some(LowFrequencyOscillator {
                    hertz: 5.0,
                    amplitude: 0.0,
                }),
            ) + 1.00
                * osc(
                    time - time_on,
                    scale(note_id as i32),
                    Oscillator::Square,
                    Some(LowFrequencyOscillator {
                        hertz: 5.0,
                        amplitude: 0.0,
                    }),
                )
                + 0.50
                    * osc(
                        time - time_on,
                        scale(note_id as i32 + 12),
                        Oscillator::Square,
                        None,
                    )
                + 0.05
                    * osc(
                        time - time_on,
                        scale(note_id as i32 + 24),
                        Oscillator::Noise,
                        None,
                    );
        amplitude * sound * self.volume
    }

    fn get_name() -> &'static str
    where
        Self: Sized,
    {
        "Harmonica"
    }
}

/// A lifetime is added for percussion instruments to ensure that the note is switched off once the `max_life_time`
/// expires.
#[derive(Clone)]
pub struct Drumkick {
    pub env: EnvelopeADSR,
    pub volume: f64,
    pub max_life_time: f64,
}

impl Drumkick {
    pub fn new() -> Self {
        let mut env = EnvelopeADSR::default();
        env.attack_time = 0.01;
        env.decay_time = 0.15;
        env.sustain_amplitude = 0.0;
        env.release_time = 0.0;
        let volume = 1.0;
        let max_life_time = 1.5;
        Self {
            env,
            volume,
            max_life_time,
        }
    }
}

impl Instrument for Drumkick {
    fn sound(
        &self,
        time: f64,
        time_on: f64,
        time_off: f64,
        note_id: u8,
        note_finished: &mut bool,
    ) -> f64 {
        let amplitude = self.env.amplitude(time, time_on, time_off);
        if self.max_life_time > 0.0 && time - time_on >= self.max_life_time {
            *note_finished = true;
        }
        let sound =
            0.99 * osc(
                time - time_on,
                scale(note_id as i32 - 36),
                Oscillator::Sine,
                Some(LowFrequencyOscillator {
                    hertz: 1.0,
                    amplitude: 1.0,
                }),
            ) + 0.01 * osc(time - time_on, 0.0, Oscillator::Noise, None);
        amplitude * sound * self.volume
    }

    fn get_name() -> &'static str
    where
        Self: Sized,
    {
        "Drum Kick"
    }
}

#[derive(Clone)]
pub struct Drumsnare {
    pub env: EnvelopeADSR,
    pub volume: f64,
    pub max_life_time: f64,
}

impl Drumsnare {
    pub fn new() -> Self {
        let mut env = EnvelopeADSR::default();
        env.attack_time = 0.0;
        env.decay_time = 0.2;
        env.sustain_amplitude = 0.0;
        env.release_time = 0.0;
        let volume = 1.0;
        let max_life_time = 1.0;
        Self {
            env,
            volume,
            max_life_time,
        }
    }
}

impl Instrument for Drumsnare {
    fn sound(
        &self,
        time: f64,
        time_on: f64,
        time_off: f64,
        note_id: u8,
        note_finished: &mut bool,
    ) -> f64 {
        let amplitude = self.env.amplitude(time, time_on, time_off);
        if self.max_life_time > 0.0 && time - time_on >= self.max_life_time {
            *note_finished = true;
        }
        let sound =
            0.5 * osc(
                time - time_on,
                scale(note_id as i32 - 24),
                Oscillator::Sine,
                Some(LowFrequencyOscillator {
                    hertz: 0.5,
                    amplitude: 1.0,
                }),
            ) + 0.5 * osc(time - time_on, 0.0, Oscillator::Noise, None);
        amplitude * sound * self.volume
    }

    fn get_name() -> &'static str
    where
        Self: Sized,
    {
        "Drum Snare"
    }
}

#[derive(Clone)]
pub struct DrumHiHat {
    pub env: EnvelopeADSR,
    pub volume: f64,
    pub max_life_time: f64,
}

impl DrumHiHat {
    pub fn new() -> Self {
        let mut env = EnvelopeADSR::default();
        env.attack_time = 0.01;
        env.decay_time = 0.05;
        env.sustain_amplitude = 0.0;
        env.release_time = 0.0;
        let volume = 0.5;
        let max_life_time = 1.0;
        Self {
            env,
            volume,
            max_life_time,
        }
    }
}

impl Instrument for DrumHiHat {
    fn sound(
        &self,
        time: f64,
        time_on: f64,
        time_off: f64,
        note_id: u8,
        note_finished: &mut bool,
    ) -> f64 {
        let amplitude = self.env.amplitude(time, time_on, time_off);
        if self.max_life_time > 0.0 && time - time_on >= self.max_life_time {
            *note_finished = true;
        }
        let sound =
            0.1 * osc(
                time - time_on,
                scale(note_id as i32 - 12),
                Oscillator::Square,
                Some(LowFrequencyOscillator {
                    hertz: 1.5,
                    amplitude: 1.0,
                }),
            ) + 0.9 * osc(time - time_on, 0.0, Oscillator::Noise, None);
        amplitude * sound * self.volume
    }

    fn get_name() -> &'static str
    where
        Self: Sized,
    {
        "Drum HiHat"
    }
}
