mod decoder_7;
mod encoder_7;

mod internal;

pub use decoder_7::Ppmd7Decoder;
pub use encoder_7::Ppmd7Encoder;

pub const PPMD7_MIN_ORDER: u32 = 2;

pub const PPMD7_MAX_ORDER: u32 = 64;

pub const PPMD7_MIN_MEM_SIZE: u32 = 2048;

pub const PPMD7_MAX_MEM_SIZE: u32 = 4294967259;

const PPMD7_SYM_END: i32 = -1;
const PPMD8_SYM_ERROR: i32 = -2;

pub type Result<T> = core::result::Result<T, Error>;

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
