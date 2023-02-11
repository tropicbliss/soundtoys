//! A bunch of ready-made utilities to get you started on digital sound synthesis.

use crate::note::w;
use rand::Rng;
use std::f64::consts::PI;

#[non_exhaustive]
/// Represents the various general purpose oscillator types.
#[derive(Clone, Debug)]
pub enum Oscillator {
    /// Sine wave
    Sine,
    /// Square wave
    Square,
    /// Triangle wave
    Triangle,
    /// Saw wave (analogue, warm, slow) (you can optionally specify the resolution of the wave)
    SawAna(Option<usize>),
    /// Saw wave (optimised, harsh, fast)
    SawDig,
    /// Pseudo-random noise
    Noise,
}

/// Carries information about LFO, which can be used to model vibratos, etc.
pub struct LowFrequencyOscillator {
    pub hertz: f64,
    pub amplitude: f64,
}

impl Default for LowFrequencyOscillator {
    fn default() -> Self {
        Self {
            hertz: 0.0,
            amplitude: 0.0,
        }
    }
}

/// Oscillates between -1.0 and 1.0 depending on the oscillator provided. Time is representative of the x-axis and
/// the output representative of the y-axis.
pub fn osc(
    time: f64,
    hertz: f64,
    osc_type: Oscillator,
    lfo: Option<LowFrequencyOscillator>,
) -> f64 {
    let lfo = lfo.unwrap_or_default();
    let freq: f64 = w(hertz) * time + lfo.amplitude * hertz * (w(lfo.hertz) * time).sin();

    match osc_type {
        Oscillator::Sine => freq.sin(),
        Oscillator::Square => {
            if freq.sin() > 0.0 {
                1.0
            } else {
                -1.0
            }
        }
        Oscillator::Triangle => freq.sin().asin() * (2.0 / PI),
        Oscillator::SawAna(res) => {
            let res = res.unwrap_or(50);
            (1..res)
                .into_iter()
                .map(|n| (n as f64 * freq).sin() / n as f64)
                .sum::<f64>()
                * (2.0 / PI)
        }
        Oscillator::SawDig => (2.0 / PI) * (hertz * PI * (time % (1.0 / hertz)) - (PI / 2.0)),
        Oscillator::Noise => rand::thread_rng().gen_range(-1.0..=1.0),
    }
}

/// Attack: Initial rise in amplitude.
///
/// Decay: The minute decrease in amplitude from the peak as it approaches the equilibrium amplitude.
///
/// Sustain: The moment after decay where the amplitude remains constant at equilibrium.
///
/// Release: The drop-off in amplitude towards the end.
///
/// Certain string instruments when played in pizzicato has no decay and sustain time.
#[derive(Clone, Debug)]
pub struct EnvelopeADSR {
    pub attack_time: f64,
    pub decay_time: f64,
    pub release_time: f64,
    pub sustain_amplitude: f64,
    pub start_amplitude: f64,
}

impl Default for EnvelopeADSR {
    fn default() -> Self {
        Self {
            attack_time: 0.1,
            decay_time: 0.1,
            release_time: 0.2,
            sustain_amplitude: 1.0,
            start_amplitude: 1.0,
        }
    }
}

impl EnvelopeADSR {
    /// Return the amplitude of this envelope for a given time.
    pub fn amplitude(&self, time: f64, time_on: f64, time_off: f64) -> f64 {
        let mut amplitude = if time_on > time_off {
            let life_time = time - time_on;
            if life_time <= self.attack_time {
                (life_time / self.attack_time) * self.start_amplitude
            } else if life_time > self.attack_time
                && life_time <= (self.attack_time + self.decay_time)
            {
                ((life_time - self.attack_time) / self.decay_time)
                    * (self.sustain_amplitude - self.start_amplitude)
                    + self.start_amplitude
            } else {
                self.sustain_amplitude
            }
        } else {
            let life_time = time_off - time_on;
            let release_amplitude = if life_time <= self.attack_time {
                (life_time / self.attack_time) * self.start_amplitude
            } else if life_time > self.attack_time
                && life_time <= (self.attack_time + self.decay_time)
            {
                ((life_time - self.attack_time) / self.decay_time)
                    * (self.sustain_amplitude - self.start_amplitude)
                    + self.start_amplitude
            } else {
                self.sustain_amplitude
            };
            ((time - time_off) / self.release_time) * (-release_amplitude) + release_amplitude
        };
        if amplitude <= f64::EPSILON {
            amplitude = 0.0;
        }
        amplitude
    }
}
