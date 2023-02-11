//! Audio player that provides a thin wrapper over `SoundMaker`.

use crate::{errors::AudioError, instruments::Instrument, note::Note, primitives::SoundMaker};
use cpal::traits::{DeviceTrait, HostTrait};
use std::{
    any::TypeId,
    sync::{Arc, Mutex},
};

/// An audio player that provides a thin wrapper over `SoundMaker` to enable users to pass in sound data in real time.
/// Like `SoundMaker`, the audio will stop playing after you drop `Player`.
pub struct Player {
    notes: Arc<Mutex<Vec<Note>>>,
    sound_maker: SoundMaker,
}

impl Player {
    /// Creates a new player instance. This spawns an audio thread in the background. Therefore, we are able to add or remove
    /// notes as the audio plays concurrently. You can optionally specify an `amplitude_limit` to avoid blowing
    /// out your speakers while testing.
    pub fn new(amplitude_limit: Option<f64>) -> Result<Self, AudioError> {
        let notes: Arc<Mutex<Vec<Note>>> = Arc::new(Mutex::new(Vec::new()));
        let notes_vec_clone = Arc::clone(&notes);
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or(AudioError::UnknownDevice)?;
        let config = device.default_output_config()?;
        let mut sound_maker = SoundMaker::new(device, config);
        sound_maker.set_callback(move |time| {
            let mut notes_lock = notes_vec_clone.lock().unwrap();
            let mut mixed_output = 0.0;
            for n in &mut *notes_lock {
                let mut note_finished = false;
                let sound = n.channel.sound(time, n.on, n.off, n.id, &mut note_finished);
                mixed_output += sound;
                if note_finished {
                    n.active = false;
                }
            }
            notes_lock.retain(|n| n.active);
            let mut res = mixed_output * 0.2;
            if let Some(limit) = amplitude_limit {
                res = res.min(limit);
            }
            res
        })?;
        Ok(Self { notes, sound_maker })
    }

    /// Adds a note to the queue.
    pub fn add_note(&self, voice: &Voice) {
        self.add_notes(vec![voice]);
    }

    /// Removes a note from the queue.
    pub fn remove_note(&self, voices: &Voice) {
        self.remove_notes(vec![voices]);
    }

    /// Adds multiple notes to the queue in bulk.
    pub fn add_notes(&self, voices: Vec<&Voice>) {
        let mut notes = self.notes.lock().unwrap();
        for voice in voices {
            let note_found = notes
                .iter_mut()
                .find(|n| n.id == voice.note_id && n.instrument_id == voice.instrument_id);
            if let Some(found_note) = note_found {
                if found_note.off >= found_note.on {
                    found_note.on = self.sound_maker.get_time();
                    found_note.active = true;
                }
            } else {
                let new_note = self.sound_maker.create_note(
                    voice.note_id,
                    voice.instrument_id,
                    dyn_clone::clone_box(&*voice.instrument),
                );
                notes.push(new_note);
            }
        }
    }

    /// Removes multiple notes from the queue in bulk.
    pub fn remove_notes(&self, voices: Vec<&Voice>) {
        let mut notes = self.notes.lock().unwrap();
        for voice in voices {
            let note_found = notes
                .iter_mut()
                .find(|n| n.id == voice.note_id + 64 && n.instrument_id == voice.instrument_id);
            if let Some(found_note) = note_found {
                if found_note.off <= found_note.on {
                    found_note.off = self.sound_maker.get_time();
                }
            }
        }
    }

    /// Gets the number of notes currently in the queue.
    pub fn get_simultaneous_notes(&self) -> usize {
        self.notes.lock().unwrap().len()
    }
}

/// Signifies a note to be passed into `Player`.
#[derive(Clone)]
pub struct Voice {
    instrument: Box<dyn Instrument>,
    instrument_id: TypeId,
    note_id: u8,
    instrument_name: &'static str,
}

impl Voice {
    /// Create a new `Voice` instance.
    pub fn new<I>(instrument: I, note_id: u8) -> Self
    where
        I: Instrument + 'static,
    {
        let instrument_name = I::get_name();
        let instrument_id = TypeId::of::<I>();
        Self {
            instrument: Box::new(instrument),
            instrument_id,
            note_id,
            instrument_name,
        }
    }

    /// Gets the instrument name of a `Voice`.
    pub fn get_instrument_name(&self) -> &'static str {
        self.instrument_name
    }

    pub(crate) fn new_inner(
        instrument: Box<dyn Instrument>,
        instrument_id: TypeId,
        note_id: u8,
        instrument_name: &'static str,
    ) -> Self {
        Self {
            instrument,
            instrument_id,
            note_id,
            instrument_name,
        }
    }
}
