//! Utilities for making working with musical notes easier.

use crate::instruments::Instrument;
use std::any::TypeId;

/// Maps a semitone to its frequency (in Hz).
pub fn scale(note_id: i32) -> f64 {
    8.0 * 1.0594630943592952645618252949463_f64.powi(note_id)
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
