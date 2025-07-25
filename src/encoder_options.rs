use std::fmt::Debug;

#[cfg(feature = "ppmd")]
use ppmd_rust::{PPMD7_MAX_MEM_SIZE, PPMD7_MAX_ORDER, PPMD7_MIN_MEM_SIZE, PPMD7_MIN_ORDER};

#[cfg(feature = "compress")]
use crate::EncoderConfiguration;
#[cfg(feature = "aes256")]
use crate::Password;

#[cfg(feature = "compress")]
#[cfg_attr(docsrs, doc(cfg(feature = "compress")))]
#[derive(Debug, Clone)]
/// Options for LZMA compression.
pub struct LZMAOptions(pub(crate) lzma_rust2::LZMAOptions);

impl Default for LZMAOptions {
    fn default() -> Self {
        Self(lzma_rust2::LZMAOptions::with_preset(6))
    }
}

#[cfg(feature = "compress")]
impl LZMAOptions {
    /// Creates LZMA options with the specified compression level.
    ///
    /// # Arguments
    /// * `level` - Compression level (0-9, clamped to this range)
    pub fn from_level(level: u32) -> Self {
        Self(lzma_rust2::LZMAOptions::with_preset(level))
    }
}

#[cfg(feature = "compress")]
#[cfg_attr(docsrs, doc(cfg(feature = "compress")))]
#[derive(Debug, Clone)]
/// Options for LZMA2 compression.
pub struct LZMA2Options {
    pub(crate) options: lzma_rust2::LZMAOptions,
    pub(crate) threads: u32,
    pub(crate) stream_size: Option<u64>,
}

impl Default for LZMA2Options {
    fn default() -> Self {
        Self {
            options: lzma_rust2::LZMAOptions::with_preset(6),
            threads: 1,
            stream_size: None,
        }
    }
}

#[cfg(feature = "compress")]
impl LZMA2Options {
    /// Creates LZMA2 options with the specified compression level.
    /// Encoded using a single thread.
    ///
    /// # Arguments
    /// * `level` - Compression level (0-9, clamped to this range)
    pub fn from_level(level: u32) -> Self {
        Self {
            options: lzma_rust2::LZMAOptions::with_preset(level),
            threads: 1,
            stream_size: None,
        }
    }

    /// Creates LZMA2 options with the specified compression level.
    /// Encoded using a multi-threading.
    ///
    /// # Arguments
    /// * `level` - Compression level (0-9, clamped to this range)
    /// * `threads` - Count of threads used to compress the data
    /// * `stream_size` - Size of each independent stream of uncompressed data.
    ///   The more streams can be created, the more effective is
    ///   the multi threading, but the worse the compression ratio
    ///   will be (will be clamped to be at least 256 KiB).
    pub fn from_level_mt(level: u32, threads: u32, stream_size: u64) -> Self {
        Self {
            options: lzma_rust2::LZMAOptions::with_preset(level),
            threads,
            stream_size: Some(stream_size.max(lzma_rust2::MIN_STREAM_SIZE)),
        }
    }

    /// Sets the dictionary size used when encoding.
    ///
    /// Will be clamped between 4096..=4294967280.
    pub fn set_dictionary_size(&mut self, dict_size: u32) {
        self.options.dict_size =
            dict_size.clamp(lzma_rust2::DICT_SIZE_MIN, lzma_rust2::DICT_SIZE_MAX);
    }
}

#[cfg(feature = "bzip2")]
#[cfg_attr(docsrs, doc(cfg(feature = "bzip2")))]
#[derive(Debug, Copy, Clone)]
/// Options for BZIP2 compression.
pub struct Bzip2Options(pub(crate) u32);

#[cfg(feature = "bzip2")]
impl Bzip2Options {
    /// Creates BZIP2 options with the specified compression level.
    ///
    /// # Arguments
    /// * `level` - Compression level (typically 1-9)
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
/// Options for Brotli compression.
pub struct BrotliOptions {
    pub(crate) quality: u32,
    pub(crate) window: u32,
    pub(crate) skippable_frame_size: u32,
}

#[cfg(feature = "brotli")]
impl BrotliOptions {
    /// Creates Brotli options with the specified quality and window size.
    ///
    /// # Arguments
    /// * `quality` - Compression quality (0-11, clamped to this range)
    /// * `window` - Window size (10-24, clamped to this range)
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
/// Options for Delta filter compression.
pub struct DeltaOptions(pub(crate) u32);

#[cfg(feature = "compress")]
impl DeltaOptions {
    /// Creates Delta options with the specified distance.
    ///
    /// # Arguments
    /// * `distance` - Delta distance (1-256, clamped to this range, 0 becomes 1)
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
/// Options for Deflate compression.
pub struct DeflateOptions(pub(crate) u32);

#[cfg(feature = "deflate")]
impl DeflateOptions {
    /// Creates Deflate options with the specified compression level.
    ///
    /// # Arguments
    /// * `level` - Compression level (0-9, clamped to this range)
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
/// Options for LZ4 compression.
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
/// Options for PPMD compression.
pub struct PPMDOptions {
    pub(crate) order: u32,
    pub(crate) memory_size: u32,
}

#[cfg(feature = "ppmd")]
impl PPMDOptions {
    /// Creates PPMD options with the specified compression level.
    ///
    /// # Arguments
    /// * `level` - Compression level (0-9, clamped to this range)
    pub const fn from_level(level: u32) -> Self {
        const ORDERS: [u32; 10] = [3, 4, 4, 5, 5, 6, 8, 16, 24, 32];

        let level = if level > 9 { 9 } else { level };
        let order = ORDERS[level as usize];
        let memory_size = 1 << (level + 19);

        Self { order, memory_size }
    }

    /// Creates PPMD options with specific order and memory size parameters.
    ///
    /// # Arguments
    /// * `order` - Model order (clamped to valid PPMD range)
    /// * `memory_size` - Memory size in bytes (clamped to valid PPMD range)
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
/// Options for Zstandard compression.
pub struct ZStandardOptions(pub(crate) u32);

#[cfg(feature = "zstd")]
impl ZStandardOptions {
    /// Creates Zstandard options with the specified compression level.
    ///
    /// # Arguments
    /// * `level` - Compression level (typically 1-22)
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

#[cfg_attr(docsrs, doc(cfg(feature = "aes256")))]
#[cfg(feature = "aes256")]
#[derive(Debug, Clone)]
/// Options for AES256 encryption.
pub struct AesEncoderOptions {
    /// Password for encryption.
    pub password: Password,
    /// Initialization vector for encryption.
    pub iv: [u8; 16],
    /// Salt for key derivation.
    pub salt: [u8; 16],
    /// Number of cycles power for key derivation.
    pub num_cycles_power: u8,
}

#[cfg(feature = "aes256")]
impl AesEncoderOptions {
    /// Creates new AES encoder options with the specified password.
    ///
    /// Generates random IV and salt values automatically.
    ///
    /// # Arguments
    /// * `password` - Password for encryption
    pub fn new(password: Password) -> Self {
        let mut iv = [0; 16];
        getrandom::fill(&mut iv).expect("Can't generate IV");

        let mut salt = [0; 16];
        getrandom::fill(&mut salt).expect("Can't generate salt");

        Self {
            password,
            iv,
            salt,
            num_cycles_power: 8,
        }
    }

    pub(crate) fn properties(&self) -> [u8; 34] {
        let mut props = [0u8; 34];
        self.write_properties(&mut props);
        props
    }

    #[inline]
    pub(crate) fn write_properties(&self, props: &mut [u8]) {
        assert!(props.len() >= 34);
        props[0] = (self.num_cycles_power & 0x3F) | 0xC0;
        props[1] = 0xFF;
        props[2..18].copy_from_slice(&self.salt);
        props[18..34].copy_from_slice(&self.iv);
    }
}

/// Encoder-specific options for various compression and encryption methods.
#[derive(Debug, Clone)]
pub enum EncoderOptions {
    #[cfg(feature = "compress")]
    #[cfg_attr(docsrs, doc(cfg(feature = "compress")))]
    /// Delta filter options.
    Delta(DeltaOptions),
    #[cfg(feature = "compress")]
    #[cfg_attr(docsrs, doc(cfg(feature = "compress")))]
    /// LZMA compression options.
    LZMA(LZMAOptions),
    #[cfg(feature = "compress")]
    #[cfg_attr(docsrs, doc(cfg(feature = "compress")))]
    /// LZMA2 compression options.
    LZMA2(LZMA2Options),
    #[cfg(feature = "brotli")]
    #[cfg_attr(docsrs, doc(cfg(feature = "brotli")))]
    /// Brotli compression options.
    BROTLI(BrotliOptions),
    #[cfg(feature = "bzip2")]
    #[cfg_attr(docsrs, doc(cfg(feature = "bzip2")))]
    /// BZIP2 compression options.
    BZIP2(Bzip2Options),
    #[cfg(feature = "deflate")]
    #[cfg_attr(docsrs, doc(cfg(feature = "deflate")))]
    /// Deflate compression options.
    DEFLATE(DeflateOptions),
    #[cfg(feature = "lz4")]
    #[cfg_attr(docsrs, doc(cfg(feature = "lz4")))]
    /// LZ4 compression options.
    LZ4(LZ4Options),
    #[cfg(feature = "ppmd")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ppmd")))]
    /// PPMD compression options.
    PPMD(PPMDOptions),
    #[cfg(feature = "zstd")]
    #[cfg_attr(docsrs, doc(cfg(feature = "zstd")))]
    /// Zstandard compression options.
    ZSTD(ZStandardOptions),
    #[cfg(feature = "aes256")]
    #[cfg_attr(docsrs, doc(cfg(feature = "aes256")))]
    /// AES256 encryption options.
    Aes(AesEncoderOptions),
}

#[cfg(feature = "aes256")]
impl From<AesEncoderOptions> for EncoderOptions {
    fn from(value: AesEncoderOptions) -> Self {
        Self::Aes(value)
    }
}

#[cfg(all(feature = "aes256", feature = "compress"))]
impl From<AesEncoderOptions> for EncoderConfiguration {
    fn from(value: AesEncoderOptions) -> Self {
        Self::new(crate::EncoderMethod::AES256SHA256).with_options(EncoderOptions::Aes(value))
    }
}

#[cfg(feature = "compress")]
impl From<DeltaOptions> for EncoderConfiguration {
    fn from(options: DeltaOptions) -> Self {
        Self::new(crate::EncoderMethod::DELTA_FILTER).with_options(EncoderOptions::Delta(options))
    }
}

#[cfg(feature = "compress")]
impl From<LZMA2Options> for EncoderConfiguration {
    fn from(options: LZMA2Options) -> Self {
        Self::new(crate::EncoderMethod::LZMA2).with_options(EncoderOptions::LZMA2(options))
    }
}

#[cfg(feature = "bzip2")]
impl From<Bzip2Options> for EncoderConfiguration {
    fn from(options: Bzip2Options) -> Self {
        Self::new(crate::EncoderMethod::BZIP2).with_options(EncoderOptions::BZIP2(options))
    }
}

#[cfg(feature = "brotli")]
impl From<BrotliOptions> for EncoderConfiguration {
    fn from(options: BrotliOptions) -> Self {
        Self::new(crate::EncoderMethod::BROTLI).with_options(EncoderOptions::BROTLI(options))
    }
}

#[cfg(feature = "deflate")]
impl From<DeflateOptions> for EncoderConfiguration {
    fn from(options: DeflateOptions) -> Self {
        Self::new(crate::EncoderMethod::DEFLATE).with_options(EncoderOptions::DEFLATE(options))
    }
}

#[cfg(feature = "lz4")]
impl From<LZ4Options> for EncoderConfiguration {
    fn from(options: LZ4Options) -> Self {
        Self::new(crate::EncoderMethod::LZ4).with_options(EncoderOptions::LZ4(options))
    }
}

#[cfg(feature = "ppmd")]
impl From<PPMDOptions> for EncoderConfiguration {
    fn from(options: PPMDOptions) -> Self {
        Self::new(crate::EncoderMethod::PPMD).with_options(EncoderOptions::PPMD(options))
    }
}

#[cfg(feature = "zstd")]
impl From<ZStandardOptions> for EncoderConfiguration {
    fn from(options: ZStandardOptions) -> Self {
        Self::new(crate::EncoderMethod::ZSTD).with_options(EncoderOptions::ZSTD(options))
    }
}

#[cfg(feature = "compress")]
impl From<DeltaOptions> for EncoderOptions {
    fn from(o: DeltaOptions) -> Self {
        Self::Delta(o)
    }
}

#[cfg(feature = "compress")]
impl From<LZMA2Options> for EncoderOptions {
    fn from(o: LZMA2Options) -> Self {
        Self::LZMA2(o)
    }
}

#[cfg(feature = "bzip2")]
impl From<Bzip2Options> for EncoderOptions {
    fn from(o: Bzip2Options) -> Self {
        Self::BZIP2(o)
    }
}

#[cfg(feature = "brotli")]
impl From<BrotliOptions> for EncoderOptions {
    fn from(o: BrotliOptions) -> Self {
        Self::BROTLI(o)
    }
}

#[cfg(feature = "deflate")]
impl From<DeflateOptions> for EncoderOptions {
    fn from(o: DeflateOptions) -> Self {
        Self::DEFLATE(o)
    }
}

#[cfg(feature = "lz4")]
impl From<LZ4Options> for EncoderOptions {
    fn from(o: LZ4Options) -> Self {
        Self::LZ4(o)
    }
}

#[cfg(feature = "ppmd")]
impl From<PPMDOptions> for EncoderOptions {
    fn from(o: PPMDOptions) -> Self {
        Self::PPMD(o)
    }
}

#[cfg(feature = "zstd")]
impl From<ZStandardOptions> for EncoderOptions {
    fn from(o: ZStandardOptions) -> Self {
        Self::ZSTD(o)
    }
}

impl EncoderOptions {
    /// Gets the LZMA & LZMA2 dictionary size for this encoder option.
    ///
    /// Returns the dictionary size if this is an LZMA & LZMA2 option, or a default value otherwise.
    pub fn get_lzma_dict_size(&self) -> u32 {
        match self {
            #[cfg(feature = "compress")]
            EncoderOptions::LZMA(o) => o.0.dict_size,
            #[cfg(feature = "compress")]
            EncoderOptions::LZMA2(o) => o.options.dict_size,
            #[allow(unused)]
            _ => 0,
        }
    }
}
