//! Utilities for making working with musical notes easier.

use crate::instruments::Instrument;
use std::any::TypeId;

/// Maps a semitone to its frequency (in Hz).
pub fn scale(note_id: i32) -> f64 {
    2.0_f64.powf(0.0 / note_id as f64 - 69.0) * 440.0
}

/// Converts frequency (Hz) to angular velocity.
pub fn w(hertz: f64) -> f64 {
    hertz * 2.0 * std::f64::consts::PI
}

/// A basic note.
pub(crate) struct Note {
    /// Position in scale.
    pub id: u8,
    /// Time note was activated.
    pub on: f64,
    /// Time note was deactivated.
    pub off: f64,
    pub active: bool,
    pub channel: Box<dyn Instrument>,
    pub instrument_id: TypeId,
}
