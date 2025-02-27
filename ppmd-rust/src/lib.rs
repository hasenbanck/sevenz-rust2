mod byte_reader;
mod byte_writer;
mod decoder_7;
mod decoder_7a;
mod decoder_8;
mod encoder_7;
mod encoder_8;
mod memory;

use ppmd_sys::*;

pub use decoder_7::Ppmd7Decoder;
pub use decoder_7a::Ppmd7aDecoder;
pub use decoder_8::Ppmd8Decoder;
pub use encoder_7::Ppmd7Encoder;
pub use encoder_8::Ppmd8Encoder;

pub use ppmd_sys::{
    PPMD7_MAX_MEM_SIZE, PPMD7_MAX_ORDER, PPMD7_MIN_MEM_SIZE, PPMD7_MIN_ORDER, PPMD8_MAX_ORDER,
    PPMD8_MIN_ORDER,
};

pub type Result<T> = core::result::Result<T, Error>;

/// The restore method used in PPMd8.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum RestoreMethod {
    Restart = PPMD8_RESTORE_METHOD_RESTART as _,
    CutOff = PPMD8_RESTORE_METHOD_CUT_OFF as _,
}

/// Crate error type.
pub enum Error {
    InvalidParameter,
    InternalError(&'static str),
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidParameter => write!(f, "Wrong PPMd parameter"),
            Error::InternalError(err) => write!(f, "Internal error: {err}"),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl std::error::Error for Error {}
