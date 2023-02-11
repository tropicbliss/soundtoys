//! A percussion instrument sequencer that outputs looped drum beats to be played at any given time.

use crate::{instruments::Instrument, player::Voice};
use std::{
    any::TypeId,
    collections::HashMap,
    hash::{Hash, Hasher},
    time::Instant,
};

/// Builds a `PercussionSequencer`.
#[derive(Clone)]
pub struct PercussionSequencerBuilder<const BEATS: usize> {
    beats: u32,
    sub_beats: u32,
    tempo: f64,
    channels: HashMap<InstrumentObj, [PercussiveState; BEATS]>,
}

impl<const BEATS: usize> PercussionSequencerBuilder<BEATS> {
    /// Constructs a new `PercussionSequencerBuilder`.
    pub fn new(tempo: f64, beats: u32, sub_beats: u32) -> Self {
        Self {
            beats,
            sub_beats,
            tempo,
            channels: HashMap::new(),
        }
    }

    /// Constructs a new `PercussionSequencerBuilder` with a certain tempo.
    pub fn new_with_tempo(tempo: f64) -> Self {
        Self {
            beats: 4,
            sub_beats: 4,
            tempo,
            channels: HashMap::new(),
        }
    }

    pub fn default() -> Self {
        Self {
            beats: 4,
            sub_beats: 4,
            tempo: 120.0,
            channels: HashMap::new(),
        }
    }

    /// Adds a track to the sequencer.
    pub fn add_track<I>(&mut self, instrument: I, notes: [PercussiveState; BEATS])
    where
        I: Instrument + 'static,
    {
        let instrument_id = TypeId::of::<I>();
        let instrument = Box::new(instrument);
        self.channels.insert(
            InstrumentObj {
                instrument,
                instrument_id,
                instrument_name: I::get_name(),
            },
            notes,
        );
    }

    /// Constructs a `PercussionSequencer`. The internal clock of the `PercussionSequencer` will start counting down as soon as this method
    /// is called.
    pub fn start(self) -> PercussionSequencer<BEATS> {
        PercussionSequencer {
            beat_time: (60.0 / self.tempo) / self.sub_beats as f64,
            current_beat: 0,
            total_beats: self.sub_beats as usize * self.beats as usize,
            accumulate: 0.0,
            previous: Instant::now(),
            channels: self.channels,
        }
    }
}

/// A percussion instrument sequencer that outputs looped drum beats to be played at any given time, designed to be used in conjunction with `Player`.
#[derive(Clone)]
pub struct PercussionSequencer<const BEATS: usize> {
    beat_time: f64,
    current_beat: usize,
    total_beats: usize,
    accumulate: f64,
    previous: Instant,
    channels: HashMap<InstrumentObj, [PercussiveState; BEATS]>,
}

impl<const N: usize> PercussionSequencer<N> {
    /// Outputs a vector of `Voice`s to be played by `Player` at a given time. It accounts for any previous calls
    /// to update and outputs new `Voice`s that has not been played yet.
    /// Since this is a toneless percussion sequencer, all `Voice`s this method outputs have its `note_id` set to 64.
    pub fn update(&mut self) -> Vec<Voice> {
        let elapsed_time = self.previous.elapsed().as_secs_f64();
        self.previous = Instant::now();
        let mut result = Vec::new();
        self.accumulate += elapsed_time;
        while self.accumulate >= self.beat_time {
            self.accumulate -= self.beat_time;
            self.current_beat += 1;
            if self.current_beat >= self.total_beats {
                self.current_beat = 0;
            }
            for channel in &self.channels {
                if channel.1[self.current_beat] == PercussiveState::Beat {
                    let voice = Voice::new_inner(
                        dyn_clone::clone_box(&*channel.0.instrument),
                        channel.0.instrument_id,
                        64,
                        channel.0.instrument_name,
                    );
                    result.push(voice);
                }
            }
        }
        result
    }
}

#[derive(Clone)]
struct InstrumentObj {
    instrument: Box<dyn Instrument>,
    instrument_id: TypeId,
    instrument_name: &'static str,
}

impl PartialEq for InstrumentObj {
    fn eq(&self, other: &Self) -> bool {
        self.instrument_id == other.instrument_id
    }
}

impl Eq for InstrumentObj {}

impl Hash for InstrumentObj {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.instrument_id.hash(state);
    }
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum PercussiveState {
    Rest,
    Beat,
}

impl Eq for PercussiveState {}
