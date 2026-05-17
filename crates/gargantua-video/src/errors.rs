// FILE: crates/gargantua-video/src/errors.rs

use thiserror::Error;

#[derive(Debug, Error)]
pub enum VideoError {
    #[error("video error: {0}")]
    Message(String),
}

pub type VideoResult<T> = Result<T, VideoError>;
