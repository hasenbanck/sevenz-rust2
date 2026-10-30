#![cfg(feature = "compress")]

use std::io::{Cursor, ErrorKind};

use sevenz_rust2::{
    Archive, ArchiveEntry, ArchiveWriter, EncoderConfiguration, Error, Password,
    encoder_options::{Lzma2Options, LzmaOptions},
};

fn assert_dictionary_rejected(config: EncoderConfiguration) {
    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new())).unwrap();
    writer.set_content_methods(vec![config]);

    let result = writer.push_archive_entry(ArchiveEntry::new_file("data"), Some(&b"x"[..]));
    assert!(matches!(
        result,
        Err(Error::Io(error, _)) if error.kind() == ErrorKind::InvalidInput
    ));
}

#[test]
fn lzma_rejects_unsupported_dictionary() {
    for level in [0, 6] {
        for dict_size in [1 << 30, 1 << 31, u32::MAX] {
            let mut options = LzmaOptions::from_level(level);
            options.set_dictionary_size(dict_size);
            assert_dictionary_rejected(options.into());
        }
        #[cfg(target_pointer_width = "32")]
        {
            let mut options = LzmaOptions::from_level(level);
            options.set_dictionary_size(268435455);
            assert_dictionary_rejected(options.into());
        }
    }
}

#[test]
fn lzma2_rejects_unsupported_dictionary() {
    for level in [0, 6] {
        for threads in [0, 1, 2] {
            for dict_size in [1 << 30, 1 << 31, u32::MAX] {
                let mut options = Lzma2Options::from_level_mt(level, threads, 65536);
                options.set_dictionary_size(dict_size);
                assert_dictionary_rejected(options.into());
            }
            #[cfg(target_pointer_width = "32")]
            {
                let mut options = Lzma2Options::from_level_mt(level, threads, 65536);
                options.set_dictionary_size(268435455);
                assert_dictionary_rejected(options.into());
            }
        }
    }
}

/// Writes `first`, then 20 directories, and reads the archive back. The 21 entries have no
/// stream. The bit vectors with one bit for every entry without a stream span 3 bytes.
fn write_with_directories_after(first: ArchiveEntry) -> Archive {
    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new())).unwrap();
    writer.push_archive_entry::<&[u8]>(first, None).unwrap();
    for i in 0..20 {
        let directory = ArchiveEntry::new_directory(&format!("dir{i:02}"));
        writer.push_archive_entry::<&[u8]>(directory, None).unwrap();
    }
    let mut output = writer.finish().unwrap();
    output.set_position(0);
    Archive::read(&mut output, &Password::empty()).unwrap()
}

fn assert_directories_after(archive: &Archive, first: &str, first_is_directory: bool) {
    assert_eq!(archive.files.len(), 21);
    assert_eq!(archive.files[0].name(), first);
    assert_eq!(archive.files[0].is_directory(), first_is_directory);
    for (i, entry) in archive.files[1..].iter().enumerate() {
        assert_eq!(entry.name(), format!("dir{i:02}"));
        assert!(entry.is_directory());
    }
}

#[test]
fn empty_file_followed_by_directories_round_trips() {
    let archive = write_with_directories_after(ArchiveEntry::new_file("empty.txt"));

    assert_directories_after(&archive, "empty.txt", false);
}

/// All entries are directories. The writer writes no `kEmptyFile` vector and only the `kAnti`
/// vector is under test.
#[test]
fn anti_item_followed_by_directories_round_trips() {
    let mut anti = ArchiveEntry::new_directory("removed");
    anti.is_anti_item = true;
    let archive = write_with_directories_after(anti);

    assert_directories_after(&archive, "removed", true);
    assert!(archive.files[0].is_anti_item());
    assert!(archive.files[1..].iter().all(|entry| !entry.is_anti_item()));
}

/// A file time of 2^63 or more exceeds the range of `SystemTime` on Windows. Extracting an
/// entry with such times must succeed.
#[cfg(all(windows, feature = "util"))]
#[test]
fn extracting_an_entry_with_file_times_beyond_system_time_succeeds() {
    use sevenz_rust2::{NtTime, decompress};

    let time = NtTime::new(u64::MAX);
    let mut entry = ArchiveEntry::new_file("file.txt");
    entry.has_creation_date = true;
    entry.creation_date = time;
    entry.has_last_modified_date = true;
    entry.last_modified_date = time;
    entry.has_access_date = true;
    entry.access_date = time;
    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new())).unwrap();
    writer
        .push_archive_entry(entry, Some(&b"data"[..]))
        .unwrap();
    let mut archive = writer.finish().unwrap();
    archive.set_position(0);

    let read = Archive::read(&mut archive, &Password::empty()).unwrap();
    assert_eq!(read.files[0].last_modified_date(), time);
    archive.set_position(0);

    let dest = tempfile::tempdir().unwrap();
    decompress(archive, dest.path()).unwrap();
    assert_eq!(
        std::fs::read(dest.path().join("file.txt")).unwrap(),
        b"data"
    );
}
