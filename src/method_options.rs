#[cfg(feature = "compress")]
use lzma_rust2::LZMA2Options;

#[cfg(feature = "aes256")]
use crate::aes256sha256::AesEncoderOptions;
use std::fmt::Debug;

#[cfg(feature = "bzip2")]
#[derive(Debug, Copy, Clone)]
pub struct Bzip2Options(pub(crate) u32);

#[cfg(feature = "bzip2")]
impl Bzip2Options {
    pub const fn from_level(level: u32) -> Self {
        Self(level)
    }
}

#[cfg(feature = "bzip2")]
impl Default for Bzip2Options {
    fn default() -> Self {
        Self(6)
    }
}

#[cfg(feature = "brotli")]
const MINIMAL_SKIPPABLE_FRAME_SIZE: u32 = 64 * 1024;
#[cfg(feature = "brotli")]
const DEFAULT_SKIPPABLE_FRAME_SIZE: u32 = 128 * 1024;

#[cfg(feature = "brotli")]
#[derive(Debug, Copy, Clone)]
pub struct BrotliOptions {
    pub(crate) quality: u32,
    pub(crate) window: u32,
    pub(crate) skippable_frame_size: u32,
}

#[cfg(feature = "brotli")]
impl BrotliOptions {
    pub const fn from_quality_window(quality: u32, window: u32) -> Self {
        let quality = if quality > 11 { 11 } else { quality };
        let window = if window > 24 { 24 } else { window };
        Self {
            quality,
            window,
            skippable_frame_size: DEFAULT_SKIPPABLE_FRAME_SIZE,
        }
    }

    /// Set's the skippable frame size. The size is defined as the size of uncompressed data a frame
    /// contains. A value of 0 deactivates skippable frames and uses the native brotli bitstream.
    /// If a value is set, then a similar skippable frame format used by LZ4 and ZSTD is used.
    ///
    /// Af value between 1..=64KiB will be set to 64KiB.
    ///
    /// This was first implemented by zstdmt. The default value is 128 KiB.
    pub fn with_skippable_frame_size(mut self, skippable_frame_size: u32) -> Self {
        if skippable_frame_size == 0 {
            self.skippable_frame_size = 0;
        } else if skippable_frame_size < MINIMAL_SKIPPABLE_FRAME_SIZE {
            self.skippable_frame_size = MINIMAL_SKIPPABLE_FRAME_SIZE;
        } else {
            self.skippable_frame_size = skippable_frame_size;
        }

        self
    }
}

#[cfg(feature = "brotli")]
impl Default for BrotliOptions {
    fn default() -> Self {
        Self {
            quality: 11,
            window: 22,
            skippable_frame_size: DEFAULT_SKIPPABLE_FRAME_SIZE,
        }
    }
}

#[cfg(feature = "deflate")]
#[derive(Debug, Copy, Clone)]
pub struct DeflateOptions(pub(crate) u32);

#[cfg(feature = "deflate")]
impl DeflateOptions {
    pub const fn from_level(level: u32) -> Self {
        let level = if level > 9 { 9 } else { level };
        Self(level)
    }
}

#[cfg(feature = "deflate")]
impl Default for DeflateOptions {
    fn default() -> Self {
        Self(6)
    }
}

#[cfg(feature = "lz4")]
#[derive(Debug, Copy, Clone)]
pub struct LZ4Options(pub(crate) u32);

#[cfg(feature = "lz4")]
impl LZ4Options {
    pub const fn from_level(level: u32) -> Self {
        let level = if level == 0 {
            1
        } else if level > 12 {
            12
        } else {
            level
        };
        Self(level)
    }
}

#[cfg(feature = "lz4")]
impl Default for LZ4Options {
    fn default() -> Self {
        Self(1)
    }
}

#[cfg(feature = "zstd")]
#[derive(Debug, Copy, Clone)]
pub struct ZStandardOptions(pub(crate) u32);

#[cfg(feature = "zstd")]
impl ZStandardOptions {
    pub const fn from_level(level: u32) -> Self {
        let level = if level > 22 { 22 } else { level };
        Self(level)
    }
}

#[cfg(feature = "zstd")]
impl Default for ZStandardOptions {
    fn default() -> Self {
        Self(3)
    }
}

#[derive(Debug, Clone)]
pub enum MethodOptions {
    Num(u32),
    #[cfg(feature = "compress")]
    LZMA2(LZMA2Options),
    #[cfg(feature = "brotli")]
    BROTLI(BrotliOptions),
    #[cfg(feature = "bzip2")]
    BZIP2(Bzip2Options),
    #[cfg(feature = "deflate")]
    DEFLATE(DeflateOptions),
    #[cfg(feature = "lz4")]
    LZ4(LZ4Options),
    #[cfg(feature = "zstd")]
    ZSTD(ZStandardOptions),
    #[cfg(feature = "aes256")]
    Aes(AesEncoderOptions),
}

#[cfg(feature = "aes256")]
impl From<AesEncoderOptions> for MethodOptions {
    fn from(value: AesEncoderOptions) -> Self {
        Self::Aes(value)
    }
}

#[cfg(feature = "aes256")]
impl From<AesEncoderOptions> for crate::SevenZMethodConfiguration {
    fn from(value: AesEncoderOptions) -> Self {
        Self::new(crate::SevenZMethod::AES256SHA256).with_options(MethodOptions::Aes(value))
    }
}

#[cfg(feature = "compress")]
impl From<LZMA2Options> for crate::SevenZMethodConfiguration {
    fn from(value: LZMA2Options) -> Self {
        Self::new(crate::SevenZMethod::LZMA2).with_options(MethodOptions::LZMA2(value))
    }
}

impl From<u32> for MethodOptions {
    fn from(n: u32) -> Self {
        Self::Num(n)
    }
}

#[cfg(feature = "compress")]
impl From<LZMA2Options> for MethodOptions {
    fn from(o: LZMA2Options) -> Self {
        Self::LZMA2(o)
    }
}

#[cfg(feature = "bzip2")]
impl From<Bzip2Options> for MethodOptions {
    fn from(o: Bzip2Options) -> Self {
        Self::BZIP2(o)
    }
}

#[cfg(feature = "brotli")]
impl From<BrotliOptions> for MethodOptions {
    fn from(o: BrotliOptions) -> Self {
        Self::BROTLI(o)
    }
}

#[cfg(feature = "deflate")]
impl From<DeflateOptions> for MethodOptions {
    fn from(o: DeflateOptions) -> Self {
        Self::DEFLATE(o)
    }
}

#[cfg(feature = "lz4")]
impl From<LZ4Options> for MethodOptions {
    fn from(o: LZ4Options) -> Self {
        Self::LZ4(o)
    }
}

#[cfg(feature = "zstd")]
impl From<ZStandardOptions> for MethodOptions {
    fn from(o: ZStandardOptions) -> Self {
        Self::ZSTD(o)
    }
}

impl MethodOptions {
    pub fn get_lzma2_dict_size(&self) -> u32 {
        match self {
            MethodOptions::Num(n) => *n,
            #[cfg(feature = "compress")]
            MethodOptions::LZMA2(o) => o.dict_size,
            #[allow(unused)]
            _ => 0,
        }
    }
}
