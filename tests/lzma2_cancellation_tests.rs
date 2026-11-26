#![cfg(feature = "compress")]

use std::{
    io::{Cursor, Read},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use sevenz_rust2::{encoder_options::*, *};

/// The smallest chunk the level 0 dictionary allows, so the multi-threaded encoder splits the
/// input into several chunks.
const CHUNK_SIZE: u64 = 1 << 18;

fn content() -> Vec<u8> {
    (0..1_000_000u32)
        .flat_map(|i| (i % 251).to_le_bytes())
        .collect()
}

/// Sets `flag` once the first bytes are read.
struct CancelAfterFirstRead<R> {
    inner: R,
    flag: Arc<AtomicBool>,
}

impl<R: Read> Read for CancelAfterFirstRead<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let read = self.inner.read(buf)?;
        self.flag.store(true, Ordering::Relaxed);
        Ok(read)
    }
}

fn compress(content: &[u8], options: Lzma2Options) -> Vec<u8> {
    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new())).unwrap();
    writer.set_content_methods(vec![options.into()]);
    writer
        .push_archive_entry(ArchiveEntry::new_file("data.bin"), Some(content))
        .unwrap();
    writer.finish().unwrap().into_inner()
}

fn read_content(archive: Vec<u8>) -> Vec<u8> {
    let mut reader = ArchiveReader::new(Cursor::new(archive), Password::empty()).unwrap();
    reader.read_file("data.bin").unwrap()
}

#[test]
fn lzma2_cancellation_stops_multi_threaded_encoding() {
    let content = content();
    let flag = Arc::new(AtomicBool::new(false));
    let mut options = Lzma2Options::from_level_mt(0, 2, CHUNK_SIZE);
    options.set_cancellation(Arc::clone(&flag));

    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new())).unwrap();
    writer.set_content_methods(vec![options.into()]);
    let reader = CancelAfterFirstRead {
        inner: content.as_slice(),
        flag,
    };
    let result = writer.push_archive_entry(ArchiveEntry::new_file("data.bin"), Some(reader));

    match result {
        Err(Error::Io(error, _)) => assert!(
            error
                .get_ref()
                .is_some_and(|error| error.is::<lzma_rust2::EncoderCancelled>()),
            "unexpected error: {error}"
        ),
        Err(error) => panic!("unexpected error: {error}"),
        Ok(_) => panic!("encoding should have been cancelled"),
    }
}

#[test]
fn lzma2_cancellation_unset_does_not_change_archive() {
    let content = content();
    let options = Lzma2Options::from_level_mt(0, 2, CHUNK_SIZE);
    let mut with_cancellation = options.clone();
    with_cancellation.set_cancellation(Arc::new(AtomicBool::new(false)));

    let archive = compress(&content, with_cancellation);

    assert_eq!(archive, compress(&content, options));
    assert_eq!(read_content(archive), content);
}

#[test]
fn lzma2_cancellation_is_ignored_single_threaded() {
    let content = content();
    let mut options = Lzma2Options::from_level(0);
    options.set_cancellation(Arc::new(AtomicBool::new(true)));

    let archive = compress(&content, options);

    assert_eq!(read_content(archive), content);
}
