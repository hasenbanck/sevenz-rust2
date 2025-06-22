mod decoder_7;
mod encoder_7;

mod byte_reader;
mod byte_writer;
mod decoder_8;
mod encoder_8;
mod internal;
mod memory;

pub use decoder_7::Ppmd7Decoder;
pub use decoder_8::Ppmd8Decoder;
pub use encoder_7::Ppmd7Encoder;
pub use encoder_8::Ppmd8Encoder;

pub const PPMD7_MIN_ORDER: u32 = 2;

pub const PPMD7_MAX_ORDER: u32 = 64;

pub const PPMD7_MIN_MEM_SIZE: u32 = 2048;

pub const PPMD7_MAX_MEM_SIZE: u32 = 4294967259;

pub const PPMD8_MIN_ORDER: u32 = 2;

pub const PPMD8_MAX_ORDER: u32 = 16;

const SYM_END: i32 = -1;
const SYM_ERROR: i32 = -2;

pub type Result<T> = core::result::Result<T, Error>;

/// The restore method used in PPMd8.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum RestoreMethod {
    Restart = 0 as _,
    CutOff = 1 as _,
}

/// Crate error type.
pub enum Error {
    RangeDecoderInitialization,
    InvalidParameter,
    IoError(std::io::Error),
    InternalError(&'static str),
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::RangeDecoderInitialization => {
                write!(f, "Could not initialize the range decoder")
            }
            Error::InvalidParameter => write!(f, "Wrong PPMd parameter"),
            Error::IoError(err) => write!(f, "Io error: {err}"),
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
