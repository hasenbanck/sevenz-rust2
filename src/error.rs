use std::{borrow::Cow, fmt::Display};

/// The error type of the crate.
#[derive(Debug)]
pub enum Error {
    BadSignature([u8; 6]),
    UnsupportedVersion { major: u8, minor: u8 },
    ChecksumVerificationFailed,
    NextHeaderCrcMismatch,
    Io(std::io::Error, Cow<'static, str>),
    FileOpen(std::io::Error, String),
    Other(Cow<'static, str>),
    BadTerminatedStreamsInfo(u8),
    BadTerminatedUnpackInfo,
    BadTerminatedPackInfo(u8),
    BadTerminatedSubStreamsInfo,
    BadTerminatedheader(u8),
    ExternalUnsupported,
    UnsupportedCompressionMethod(String),
    MaxMemLimited { max_kb: usize, actaul_kb: usize },
    PasswordRequired,
    Unsupported(Cow<'static, str>),
    MaybeBadPassword(std::io::Error),
    FileNotFound,
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::io(value)
    }
}

impl Error {
    #[inline]
    pub fn other<S: Into<Cow<'static, str>>>(s: S) -> Self {
        Self::Other(s.into())
    }

    #[inline]
    pub fn unsupported<S: Into<Cow<'static, str>>>(s: S) -> Self {
        Self::Unsupported(s.into())
    }

    #[inline]
    pub fn io(e: std::io::Error) -> Self {
        Self::io_msg(e, "")
    }

    #[inline]
    pub fn io_msg(e: std::io::Error, msg: impl Into<Cow<'static, str>>) -> Self {
        Self::Io(e, msg.into())
    }

    pub fn bad_password(e: std::io::Error, encryped: bool) -> Self {
        if encryped {
            Self::MaybeBadPassword(e)
        } else {
            Self::io(e)
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[inline]
    pub(crate) fn file_open(e: std::io::Error, filename: impl Into<Cow<'static, str>>) -> Self {
        Self::Io(e, filename.into())
    }

    pub(crate) fn maybe_bad_password(self, encryped: bool) -> Self {
        if !encryped {
            return self;
        }
        match self {
            Self::Io(e, s) if s.is_empty() => Self::MaybeBadPassword(e),
            _ => self,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

impl std::error::Error for Error {}
