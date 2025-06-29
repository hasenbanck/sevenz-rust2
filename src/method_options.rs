use std::fmt::Debug;

#[cfg(feature = "compress")]
pub use lzma_rust2::LZMA2Options;
#[cfg(feature = "ppmd")]
use ppmd_rust::{PPMD7_MAX_MEM_SIZE, PPMD7_MAX_ORDER, PPMD7_MIN_MEM_SIZE, PPMD7_MIN_ORDER};

#[cfg(feature = "aes256")]
use crate::aes256sha256::AesEncoderOptions;

#[cfg(feature = "bzip2")]
#[cfg_attr(docsrs, doc(cfg(feature = "bzip2")))]
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

#[cfg(any(feature = "brotli", feature = "lz4"))]
const MINIMAL_SKIPPABLE_FRAME_SIZE: u32 = 64 * 1024;
#[cfg(feature = "brotli")]
const DEFAULT_SKIPPABLE_FRAME_SIZE: u32 = 128 * 1024;

#[cfg(feature = "brotli")]
#[cfg_attr(docsrs, doc(cfg(feature = "brotli")))]
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
        } else {
            self.skippable_frame_size =
                u32::max(skippable_frame_size, MINIMAL_SKIPPABLE_FRAME_SIZE);
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

#[cfg(feature = "compress")]
#[cfg_attr(docsrs, doc(cfg(feature = "compress")))]
#[derive(Debug, Copy, Clone)]
pub struct DeltaOptions(pub(crate) u32);

#[cfg(feature = "compress")]
impl DeltaOptions {
    pub const fn from_distance(distance: u32) -> Self {
        let distance = if distance == 0 {
            1
        } else if distance > 256 {
            256
        } else {
            distance
        };
        Self(distance)
    }
}

#[cfg(feature = "compress")]
impl Default for DeltaOptions {
    fn default() -> Self {
        Self(1)
    }
}

#[cfg(feature = "deflate")]
#[cfg_attr(docsrs, doc(cfg(feature = "deflate")))]
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
#[cfg_attr(docsrs, doc(cfg(feature = "lz4")))]
#[derive(Debug, Copy, Clone, Default)]
pub struct LZ4Options {
    pub(crate) skippable_frame_size: u32,
}

#[cfg(feature = "lz4")]
impl LZ4Options {
    /// Set's the skippable frame size. The size is defined as the size of uncompressed data a frame
    /// contains. A value of 0 deactivates skippable frames and uses the native LZ4 bitstream.
    /// If a value is set, then the similar skippable frame format is used.
    ///
    /// Af value between 1..=64KiB will be set to 64KiB.
    ///
    /// This was first implemented by zstdmt.
    ///
    /// Defaults to not use the skippable frame format at all, since LZ4 is extremely fast and will
    /// most likely saturate IO even on a single thread.
    pub fn with_skippable_frame_size(mut self, skippable_frame_size: u32) -> Self {
        if skippable_frame_size == 0 {
            self.skippable_frame_size = 0;
        } else {
            self.skippable_frame_size =
                u32::max(skippable_frame_size, MINIMAL_SKIPPABLE_FRAME_SIZE);
        }

        self
    }
}

#[cfg(feature = "ppmd")]
#[cfg_attr(docsrs, doc(cfg(feature = "ppmd")))]
#[derive(Debug, Copy, Clone)]
pub struct PPMDOptions {
    pub(crate) order: u32,
    pub(crate) memory_size: u32,
}

#[cfg(feature = "ppmd")]
impl PPMDOptions {
    pub const fn from_level(level: u32) -> Self {
        const ORDERS: [u32; 10] = [3, 4, 4, 5, 5, 6, 8, 16, 24, 32];

        let level = if level > 9 { 9 } else { level };
        let order = ORDERS[level as usize];
        let memory_size = 1 << (level + 19);

        Self { order, memory_size }
    }

    pub const fn from_order_memory_size(order: u32, memory_size: u32) -> Self {
        let order = if order > PPMD7_MAX_ORDER {
            PPMD7_MAX_ORDER
        } else if order < PPMD7_MIN_ORDER {
            PPMD7_MIN_ORDER
        } else {
            order
        };
        let memory_size = if memory_size > PPMD7_MAX_MEM_SIZE {
            PPMD7_MAX_MEM_SIZE
        } else if memory_size < PPMD7_MIN_MEM_SIZE {
            PPMD7_MIN_MEM_SIZE
        } else {
            memory_size
        };
        Self { order, memory_size }
    }
}

#[cfg(feature = "ppmd")]
impl Default for PPMDOptions {
    fn default() -> Self {
        Self::from_level(6)
    }
}

#[cfg(feature = "zstd")]
#[cfg_attr(docsrs, doc(cfg(feature = "zstd")))]
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
    #[cfg_attr(docsrs, doc(cfg(feature = "compress")))]
    Delta(DeltaOptions),
    #[cfg(feature = "compress")]
    #[cfg_attr(docsrs, doc(cfg(feature = "compress")))]
    LZMA2(LZMA2Options),
    #[cfg(feature = "brotli")]
    #[cfg_attr(docsrs, doc(cfg(feature = "brotli")))]
    BROTLI(BrotliOptions),
    #[cfg(feature = "bzip2")]
    #[cfg_attr(docsrs, doc(cfg(feature = "bzip2")))]
    BZIP2(Bzip2Options),
    #[cfg(feature = "deflate")]
    #[cfg_attr(docsrs, doc(cfg(feature = "deflate")))]
    DEFLATE(DeflateOptions),
    #[cfg(feature = "lz4")]
    #[cfg_attr(docsrs, doc(cfg(feature = "lz4")))]
    LZ4(LZ4Options),
    #[cfg(feature = "ppmd")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ppmd")))]
    PPMD(PPMDOptions),
    #[cfg(feature = "zstd")]
    #[cfg_attr(docsrs, doc(cfg(feature = "zstd")))]
    ZSTD(ZStandardOptions),
    #[cfg(feature = "aes256")]
    #[cfg_attr(docsrs, doc(cfg(feature = "aes256")))]
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
impl From<DeltaOptions> for crate::SevenZMethodConfiguration {
    fn from(options: DeltaOptions) -> Self {
        Self::new(crate::SevenZMethod::DELTA_FILTER).with_options(MethodOptions::Delta(options))
    }
}

#[cfg(feature = "compress")]
impl From<LZMA2Options> for crate::SevenZMethodConfiguration {
    fn from(options: LZMA2Options) -> Self {
        Self::new(crate::SevenZMethod::LZMA2).with_options(MethodOptions::LZMA2(options))
    }
}

#[cfg(feature = "bzip2")]
impl From<Bzip2Options> for crate::SevenZMethodConfiguration {
    fn from(options: Bzip2Options) -> Self {
        Self::new(crate::SevenZMethod::BZIP2).with_options(MethodOptions::BZIP2(options))
    }
}

#[cfg(feature = "brotli")]
impl From<BrotliOptions> for crate::SevenZMethodConfiguration {
    fn from(options: BrotliOptions) -> Self {
        Self::new(crate::SevenZMethod::BROTLI).with_options(MethodOptions::BROTLI(options))
    }
}

#[cfg(feature = "deflate")]
impl From<DeflateOptions> for crate::SevenZMethodConfiguration {
    fn from(options: DeflateOptions) -> Self {
        Self::new(crate::SevenZMethod::DEFLATE).with_options(MethodOptions::DEFLATE(options))
    }
}

#[cfg(feature = "lz4")]
impl From<LZ4Options> for crate::SevenZMethodConfiguration {
    fn from(options: LZ4Options) -> Self {
        Self::new(crate::SevenZMethod::LZ4).with_options(MethodOptions::LZ4(options))
    }
}

#[cfg(feature = "ppmd")]
impl From<PPMDOptions> for crate::SevenZMethodConfiguration {
    fn from(options: PPMDOptions) -> Self {
        Self::new(crate::SevenZMethod::PPMD).with_options(MethodOptions::PPMD(options))
    }
}

#[cfg(feature = "zstd")]
impl From<ZStandardOptions> for crate::SevenZMethodConfiguration {
    fn from(options: ZStandardOptions) -> Self {
        Self::new(crate::SevenZMethod::ZSTD).with_options(MethodOptions::ZSTD(options))
    }
}

impl From<u32> for MethodOptions {
    fn from(n: u32) -> Self {
        Self::Num(n)
    }
}

#[cfg(feature = "compress")]
impl From<DeltaOptions> for MethodOptions {
    fn from(o: DeltaOptions) -> Self {
        Self::Delta(o)
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

#[cfg(feature = "ppmd")]
impl From<PPMDOptions> for MethodOptions {
    fn from(o: PPMDOptions) -> Self {
        Self::PPMD(o)
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
