#[cfg(feature = "compress")]
use crate::encoder_options::EncoderOptions;
use crate::{NtTime, bitset::BitSet, block::*};

pub(crate) const SIGNATURE_HEADER_SIZE: u64 = 32;
pub(crate) const SEVEN_Z_SIGNATURE: &[u8] = &[b'7', b'z', 0xBC, 0xAF, 0x27, 0x1C];

pub(crate) const K_END: u8 = 0x00;
pub(crate) const K_HEADER: u8 = 0x01;
pub(crate) const K_ARCHIVE_PROPERTIES: u8 = 0x02;
pub(crate) const K_ADDITIONAL_STREAMS_INFO: u8 = 0x03;
pub(crate) const K_MAIN_STREAMS_INFO: u8 = 0x04;
pub(crate) const K_FILES_INFO: u8 = 0x05;
pub(crate) const K_PACK_INFO: u8 = 0x06;
pub(crate) const K_UNPACK_INFO: u8 = 0x07;
pub(crate) const K_SUB_STREAMS_INFO: u8 = 0x08;
pub(crate) const K_SIZE: u8 = 0x09;
pub(crate) const K_CRC: u8 = 0x0A;
pub(crate) const K_FOLDER: u8 = 0x0B;
pub(crate) const K_CODERS_UNPACK_SIZE: u8 = 0x0C;
pub(crate) const K_NUM_UNPACK_STREAM: u8 = 0x0D;
pub(crate) const K_EMPTY_STREAM: u8 = 0x0E;
pub(crate) const K_EMPTY_FILE: u8 = 0x0F;
pub(crate) const K_ANTI: u8 = 0x10;
pub(crate) const K_NAME: u8 = 0x11;
pub(crate) const K_C_TIME: u8 = 0x12;
pub(crate) const K_A_TIME: u8 = 0x13;
pub(crate) const K_M_TIME: u8 = 0x14;
pub(crate) const K_WIN_ATTRIBUTES: u8 = 0x15;

/// TODO: Implement reading & writing comments
#[allow(unused)]
pub(crate) const K_COMMENT: u8 = 0x16;
pub(crate) const K_ENCODED_HEADER: u8 = 0x17;
pub(crate) const K_START_POS: u8 = 0x18;
pub(crate) const K_DUMMY: u8 = 0x19;

#[derive(Debug, Default, Clone)]
pub struct Archive {
    /// Offset from beginning of file + SIGNATURE_HEADER_SIZE to packed streams.
    pub(crate) pack_pos: u64,
    pub(crate) pack_sizes: Vec<u64>,
    pub(crate) pack_crcs_defined: BitSet,
    pub(crate) pack_crcs: Vec<u64>,
    pub(crate) sub_streams_info: Option<SubStreamsInfo>,
    pub blocks: Vec<Block>,
    pub files: Vec<ArchiveEntry>,
    pub stream_map: StreamMap,
    pub is_solid: bool,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct SubStreamsInfo {
    pub(crate) unpack_sizes: Vec<u64>,
    pub(crate) has_crc: BitSet,
    pub(crate) crcs: Vec<u64>,
}

#[derive(Debug, Default, Clone)]
pub struct ArchiveEntry {
    pub name: String,
    pub has_stream: bool,
    pub is_directory: bool,
    pub is_anti_item: bool,
    pub has_creation_date: bool,
    pub has_last_modified_date: bool,
    pub has_access_date: bool,
    pub creation_date: NtTime,
    pub last_modified_date: NtTime,
    pub access_date: NtTime,
    pub has_windows_attributes: bool,
    pub windows_attributes: u32,
    pub has_crc: bool,
    pub crc: u64,
    pub compressed_crc: u64,
    pub size: u64,
    pub compressed_size: u64,
}

impl ArchiveEntry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_file(entry_name: &str) -> Self {
        Self {
            name: entry_name.to_string(),
            has_stream: true,
            is_directory: false,
            ..Default::default()
        }
    }

    pub fn new_directory(entry_name: &str) -> Self {
        Self {
            name: entry_name.to_string(),
            has_stream: false,
            is_directory: true,
            ..Default::default()
        }
    }

    pub fn from_path(path: impl AsRef<std::path::Path>, entry_name: String) -> Self {
        let path = path.as_ref();
        #[cfg(target_os = "windows")]
        let entry_name = {
            let mut name_bytes = entry_name.into_bytes();
            for b in &mut name_bytes {
                if *b == b'\\' {
                    *b = b'/';
                }
            }
            String::from_utf8(name_bytes).unwrap()
        };
        let mut entry = ArchiveEntry {
            name: entry_name,
            has_stream: path.is_file(),
            is_directory: path.is_dir(),
            ..Default::default()
        };

        if let Ok(meta) = path.metadata() {
            if let Ok(modified) = meta.modified() {
                if let Ok(date) = NtTime::try_from(modified) {
                    entry.last_modified_date = date;
                    entry.has_last_modified_date = entry.last_modified_date.0 > 0;
                }
            }
            if let Ok(date) = meta.created() {
                if let Ok(date) = NtTime::try_from(date) {
                    entry.creation_date = date;
                    entry.has_creation_date = entry.creation_date.0 > 0;
                }
            }
            if let Ok(date) = meta.accessed() {
                if let Ok(date) = NtTime::try_from(date) {
                    entry.access_date = date;
                    entry.has_access_date = entry.access_date.0 > 0;
                }
            }
        }
        entry
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn is_directory(&self) -> bool {
        self.is_directory
    }

    pub fn has_stream(&self) -> bool {
        self.has_stream
    }

    pub fn creation_date(&self) -> NtTime {
        self.creation_date
    }

    pub fn last_modified_date(&self) -> NtTime {
        self.last_modified_date
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn windows_attributes(&self) -> u32 {
        self.windows_attributes
    }

    pub fn access_date(&self) -> NtTime {
        self.access_date
    }

    pub fn is_anti_item(&self) -> bool {
        self.is_anti_item
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "compress")))]
#[cfg(feature = "compress")]
#[derive(Debug, Default)]
pub struct EncoderConfiguration {
    pub method: EncoderMethod,
    pub options: Option<EncoderOptions>,
}

#[cfg(feature = "compress")]
impl From<EncoderMethod> for EncoderConfiguration {
    fn from(value: EncoderMethod) -> Self {
        Self::new(value)
    }
}

#[cfg(feature = "compress")]
impl Clone for EncoderConfiguration {
    fn clone(&self) -> Self {
        Self {
            method: self.method,
            options: self.options.clone(),
        }
    }
}

#[cfg(feature = "compress")]
impl EncoderConfiguration {
    pub fn new(method: EncoderMethod) -> Self {
        Self {
            method,
            options: None,
        }
    }

    pub fn with_options(mut self, options: EncoderOptions) -> Self {
        self.options = Some(options);
        self
    }
}

/// Encoder method that can be chained (filter, compression and encryption).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default, Hash)]
pub struct EncoderMethod(&'static str, &'static [u8]);

impl EncoderMethod {
    pub const ID_COPY: &'static [u8] = &[0x00];
    pub const ID_DELTA: &'static [u8] = &[0x03];

    pub const ID_LZMA: &'static [u8] = &[0x03, 0x01, 0x01];
    pub const ID_BCJ_X86: &'static [u8] = &[0x03, 0x03, 0x01, 0x03];
    pub const ID_BCJ2: &'static [u8] = &[0x03, 0x03, 0x01, 0x1B];
    pub const ID_BCJ_PPC: &'static [u8] = &[0x03, 0x03, 0x02, 0x05];
    pub const ID_BCJ_IA64: &'static [u8] = &[0x03, 0x03, 0x04, 0x01];
    pub const ID_BCJ_ARM: &'static [u8] = &[0x03, 0x03, 0x05, 0x01];
    pub const ID_BCJ_ARM64: &'static [u8] = &[0xA];
    pub const ID_BCJ_ARM_THUMB: &'static [u8] = &[0x03, 0x03, 0x07, 0x01];
    pub const ID_BCJ_SPARC: &'static [u8] = &[0x03, 0x03, 0x08, 0x05];
    pub const ID_PPMD: &'static [u8] = &[0x03, 0x04, 0x01];

    pub const ID_LZMA2: &'static [u8] = &[0x21];
    pub const ID_BZIP2: &'static [u8] = &[0x04, 0x02, 0x02];
    pub const ID_ZSTD: &'static [u8] = &[0x04, 0xF7, 0x11, 0x01];
    pub const ID_BROTLI: &'static [u8] = &[0x04, 0xF7, 0x11, 0x02];
    pub const ID_LZ4: &'static [u8] = &[0x04, 0xF7, 0x11, 0x04];
    pub const ID_LZS: &'static [u8] = &[0x04, 0xF7, 0x11, 0x05];
    pub const ID_LIZARD: &'static [u8] = &[0x04, 0xF7, 0x11, 0x06];
    pub const ID_DEFLATE: &'static [u8] = &[0x04, 0x01, 0x08];
    pub const ID_DEFLATE64: &'static [u8] = &[0x04, 0x01, 0x09];
    pub const ID_AES256SHA256: &'static [u8] = &[0x06, 0xF1, 0x07, 0x01];

    pub const COPY: Self = Self("COPY", Self::ID_COPY);
    pub const LZMA: Self = Self("LZMA", Self::ID_LZMA);
    pub const LZMA2: Self = Self("LZMA2", Self::ID_LZMA2);
    pub const PPMD: Self = Self("PPMD", Self::ID_PPMD);
    pub const BZIP2: Self = Self("BZIP2", Self::ID_BZIP2);
    pub const ZSTD: Self = Self("ZSTD", Self::ID_ZSTD);
    pub const BROTLI: Self = Self("BROTLI", Self::ID_BROTLI);
    pub const LZ4: Self = Self("LZ4", Self::ID_LZ4);
    pub const LZS: Self = Self("LZS", Self::ID_LZS);
    pub const LIZARD: Self = Self("LIZARD", Self::ID_LIZARD);
    pub const DEFLATE: Self = Self("DEFLATE", Self::ID_DEFLATE);
    pub const DEFLATE64: Self = Self("DEFLATE64", Self::ID_DEFLATE64);
    pub const AES256SHA256: Self = Self("AES256SHA256", Self::ID_AES256SHA256);

    pub const BCJ_X86_FILTER: Self = Self("BCJ_X86", Self::ID_BCJ_X86);
    pub const BCJ_PPC_FILTER: Self = Self("BCJ_PPC", Self::ID_BCJ_PPC);
    pub const BCJ_IA64_FILTER: Self = Self("BCJ_IA64", Self::ID_BCJ_IA64);
    pub const BCJ_ARM_FILTER: Self = Self("BCJ_ARM", Self::ID_BCJ_ARM);
    pub const BCJ_ARM64_FILTER: Self = Self("BCJ_ARM64", Self::ID_BCJ_ARM64);
    pub const BCJ_ARM_THUMB_FILTER: Self = Self("BCJ_ARM_THUMB", Self::ID_BCJ_ARM_THUMB);
    pub const BCJ_SPARC_FILTER: Self = Self("BCJ_SPARC", Self::ID_BCJ_SPARC);
    pub const DELTA_FILTER: Self = Self("DELTA", Self::ID_DELTA);
    pub const BCJ2_FILTER: Self = Self("BCJ2", Self::ID_BCJ2);

    const ENCODING_METHODS: &'static [&'static EncoderMethod] = &[
        &Self::COPY,
        &Self::LZMA,
        &Self::LZMA2,
        &Self::PPMD,
        &Self::BZIP2,
        &Self::ZSTD,
        &Self::BROTLI,
        &Self::LZ4,
        &Self::LZS,
        &Self::LIZARD,
        &Self::DEFLATE,
        &Self::DEFLATE64,
        &Self::AES256SHA256,
        &Self::BCJ_X86_FILTER,
        &Self::BCJ_PPC_FILTER,
        &Self::BCJ_IA64_FILTER,
        &Self::BCJ_ARM_FILTER,
        &Self::BCJ_ARM64_FILTER,
        &Self::BCJ_ARM_THUMB_FILTER,
        &Self::BCJ_SPARC_FILTER,
        &Self::DELTA_FILTER,
        &Self::BCJ2_FILTER,
    ];

    #[inline]
    pub const fn name(&self) -> &'static str {
        self.0
    }

    #[inline]
    pub const fn id(&self) -> &'static [u8] {
        self.1
    }

    #[inline]
    pub fn by_id(id: &[u8]) -> Option<Self> {
        Self::ENCODING_METHODS
            .iter()
            .find(|item| item.id() == id)
            .cloned()
            .cloned()
    }
}

#[derive(Debug, Default, Clone)]
pub struct StreamMap {
    pub(crate) block_first_pack_stream_index: Vec<usize>,
    pub(crate) pack_stream_offsets: Vec<u64>,
    pub block_first_file_index: Vec<usize>,
    pub file_block_index: Vec<Option<usize>>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct StartHeader {
    pub(crate) next_header_offset: u64,
    pub(crate) next_header_size: u64,
    pub(crate) next_header_crc: u64,
}
