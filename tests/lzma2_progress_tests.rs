#![cfg(all(feature = "compress", target_has_atomic = "64"))]

use std::{
    io::Cursor,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use sevenz_rust2::{encoder_options::*, *};

/// The smallest chunk the level 0 dictionary allows, so the multi-threaded encoder splits the
/// input into several chunks.
const CHUNK_SIZE: u64 = 1 << 18;

/// Entries of mixed sizes, including an empty one, with compressible contents.
fn sample_entries() -> Vec<(String, Vec<u8>)> {
    [300_000, 0, 1, 700_000, 65_537, 250_000]
        .into_iter()
        .enumerate()
        .map(|(index, len)| {
            let mut state = 0x9E37_79B9_7F4A_7C15u64 ^ index as u64;
            let content = (0..len)
                .map(|_| {
                    state ^= state << 13;
                    state ^= state >> 7;
                    state ^= state << 17;
                    b'a' + (state % 16) as u8
                })
                .collect();
            (format!("file{index}.bin"), content)
        })
        .collect()
}

fn total_size(entries: &[(String, Vec<u8>)]) -> u64 {
    entries
        .iter()
        .map(|(_, content)| content.len() as u64)
        .sum()
}

/// Writes all entries but the last into one solid block and the last into a block of its own.
fn compress(entries: &[(String, Vec<u8>)], methods: Vec<EncoderConfiguration>) -> Vec<u8> {
    let (last, solid) = entries.split_last().unwrap();
    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new())).unwrap();
    writer.set_content_methods(methods);
    writer
        .push_archive_entries(
            solid
                .iter()
                .map(|(name, _)| ArchiveEntry::new_file(name))
                .collect(),
            solid
                .iter()
                .map(|(_, content)| Cursor::new(content.as_slice()).into())
                .collect(),
        )
        .unwrap();
    writer
        .push_archive_entry(ArchiveEntry::new_file(&last.0), Some(last.1.as_slice()))
        .unwrap();
    writer.finish().unwrap().into_inner()
}

/// Every entry of an archive as `(name, contents)`, in order.
fn read_entries(archive: Vec<u8>, password: Password) -> Vec<(String, Vec<u8>)> {
    let mut reader = ArchiveReader::new(Cursor::new(archive), password).unwrap();
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

/// Single-threaded and multi-threaded LZMA2 options.
fn lzma2_options() -> [Lzma2Options; 3] {
    [
        Lzma2Options::from_level(0),
        Lzma2Options::from_level_mt(0, 2, CHUNK_SIZE),
        Lzma2Options::from_level_mt(0, 4, CHUNK_SIZE),
    ]
}

#[test]
fn lzma2_progress_counts_uncompressed_bytes() {
    let entries = sample_entries();
    for mut options in lzma2_options() {
        let progress = Arc::new(AtomicU64::new(0));
        options.set_progress(Arc::clone(&progress));

        let archive = compress(&entries, vec![options.into()]);

        assert_eq!(progress.load(Ordering::Relaxed), total_size(&entries));
        assert_eq!(read_entries(archive, Password::empty()), entries);
    }
}

#[test]
fn lzma2_progress_counts_filtered_bytes() {
    let entries = sample_entries();
    let filters: [EncoderConfiguration; 2] = [
        EncoderMethod::BCJ_X86_FILTER.into(),
        DeltaOptions::from_distance(4).into(),
    ];
    for filter in filters {
        for mut options in lzma2_options() {
            let progress = Arc::new(AtomicU64::new(0));
            options.set_progress(Arc::clone(&progress));

            let archive = compress(&entries, vec![options.into(), filter.clone()]);

            assert_eq!(progress.load(Ordering::Relaxed), total_size(&entries));
            assert_eq!(read_entries(archive, Password::empty()), entries);
        }
    }
}

#[test]
fn lzma2_progress_does_not_change_archive() {
    let entries = sample_entries();
    for options in lzma2_options() {
        let mut with_progress = options.clone();
        with_progress.set_progress(Arc::new(AtomicU64::new(0)));

        assert_eq!(
            compress(&entries, vec![options.into()]),
            compress(&entries, vec![with_progress.into()])
        );
    }
}

#[cfg(feature = "aes256")]
#[test]
fn lzma2_progress_counts_uncompressed_bytes_encrypted() {
    let entries = sample_entries();
    for mut options in lzma2_options() {
        let progress = Arc::new(AtomicU64::new(0));
        options.set_progress(Arc::clone(&progress));

        let archive = compress(
            &entries,
            vec![
                AesEncoderOptions::new(Password::new("rust")).into(),
                options.into(),
            ],
        );

        assert_eq!(progress.load(Ordering::Relaxed), total_size(&entries));
        assert_eq!(read_entries(archive, Password::new("rust")), entries);
    }
}
