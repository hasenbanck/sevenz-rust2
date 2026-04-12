mod error;
mod lzma2_reader;
#[cfg(feature = "compress")]
mod lzma2_writer;
mod lzma_reader;
#[cfg(feature = "compress")]
mod lzma_writer;
#[allow(dead_code)]
pub(crate) mod options;

pub(crate) use lzma2_reader::{Lzma2Reader, Lzma2ReaderMt};
#[cfg(feature = "compress")]
pub(crate) use lzma2_writer::{Lzma2Writer, Lzma2WriterMt};
pub(crate) use lzma_reader::LzmaReader;
#[cfg(feature = "compress")]
pub(crate) use lzma_writer::LzmaWriter;
pub(crate) use options::{DICT_SIZE_MAX, DICT_SIZE_MIN};

/// Estimate LZMA2 decoder memory usage in KiB for a given dictionary size.
///
/// Formula from LZMA-SDK: memory = dict_size + dict_size / 2 + 36 KiB overhead.
pub(crate) fn lzma2_get_memory_usage(dict_size: u32) -> u64 {
    let dict = dict_size as u64;
    (dict + dict / 2 + 36 * 1024) / 1024
}
