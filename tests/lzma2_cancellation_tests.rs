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

/// Requests cancellation once `limit` bytes are read.
struct CancelAfter<R> {
    inner: R,
    limit: usize,
    flag: Arc<AtomicBool>,
}

impl<R: Read> Read for CancelAfter<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let read = self.inner.read(buf)?;
        self.limit = self.limit.saturating_sub(read);
        if self.limit == 0 {
            self.flag.store(true, Ordering::Relaxed);
        }
        Ok(read)
    }
}

fn assert_cancelled<T>(result: Result<T, Error>) {
    match result {
        Err(Error::Io(error, _)) => assert!(
            error
                .get_ref()
                .is_some_and(|error| error.is::<EncoderCancelled>()),
            "unexpected error: {error}"
        ),
        Err(error) => panic!("unexpected error: {error}"),
        Ok(_) => panic!("encoding should have been cancelled"),
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

fn read_entries(archive: Vec<u8>) -> Vec<(String, Vec<u8>)> {
    let mut reader = ArchiveReader::new(Cursor::new(archive), Password::empty()).unwrap();
    let mut entries = Vec::new();
    reader
        .for_each_entries(|entry, data| {
            let mut buffer = Vec::new();
            data.read_to_end(&mut buffer)?;
            entries.push((entry.name.clone(), buffer));
            Ok(true)
        })
        .unwrap();
    entries
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

    assert_cancelled(result);
}

#[test]
fn lzma2_cancellation_stops_single_threaded_encoding() {
    let content = content();
    for mut options in [
        Lzma2Options::from_level(0),
        Lzma2Options::from_level_mt(0, 1, CHUNK_SIZE),
    ] {
        let flag = Arc::new(AtomicBool::new(false));
        options.set_cancellation(Arc::clone(&flag));

        let mut writer = ArchiveWriter::new(Cursor::new(Vec::new())).unwrap();
        writer.set_content_methods(vec![options.into()]);
        let reader = CancelAfterFirstRead {
            inner: content.as_slice(),
            flag,
        };

        assert_cancelled(
            writer.push_archive_entry(ArchiveEntry::new_file("data.bin"), Some(reader)),
        );
    }
}

#[test]
fn lzma2_cancellation_keeps_the_writer_usable() {
    let content = content();
    for mut options in [
        Lzma2Options::from_level(0),
        Lzma2Options::from_level_mt(0, 2, CHUNK_SIZE),
    ] {
        // The second round pushes the cancelled entry as a solid block.
        for solid in [false, true] {
            // The second round finishes the archive right after the cancellation.
            for push_last in [true, false] {
                let flag = Arc::new(AtomicBool::new(false));
                options.set_cancellation(Arc::clone(&flag));

                let mut writer = ArchiveWriter::new(Cursor::new(Vec::new())).unwrap();
                writer.set_content_methods(vec![options.clone().into()]);
                writer
                    .push_archive_entry(ArchiveEntry::new_file("first"), Some(content.as_slice()))
                    .unwrap();

                // The encoder has written a part of the entry when the flag is set.
                let reader = CancelAfter {
                    inner: content.as_slice(),
                    limit: content.len() / 2,
                    flag: Arc::clone(&flag),
                };
                let entry = ArchiveEntry::new_file("cancelled");
                if solid {
                    assert_cancelled(writer.push_archive_entries(vec![entry], vec![reader.into()]));
                } else {
                    assert_cancelled(writer.push_archive_entry(entry, Some(reader)));
                }

                let mut expected = vec![("first".to_string(), content.clone())];
                if push_last {
                    flag.store(false, Ordering::Relaxed);
                    writer
                        .push_archive_entry(ArchiveEntry::new_file("last"), Some(&b"last"[..]))
                        .unwrap();
                    expected.push(("last".to_string(), b"last".to_vec()));
                }

                let archive = writer.finish().unwrap().into_inner();
                assert_eq!(read_entries(archive), expected);
            }
        }
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
