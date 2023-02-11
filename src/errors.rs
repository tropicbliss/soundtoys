use cpal::{BuildStreamError, DefaultStreamConfigError, PlayStreamError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AudioError {
    #[error(transparent)]
    PlayStreamError(#[from] PlayStreamError),

    #[error(transparent)]
    BuildStreamError(#[from] BuildStreamError),

    #[error("cannot find output device")]
    UnknownDevice,

    #[error(transparent)]
    DefaultStreamConfigError(#[from] DefaultStreamConfigError),
}
