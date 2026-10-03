#![cfg(feature = "compress")]

use std::{
    fs::File,
    io::{Cursor, Seek, SeekFrom},
    path::Path,
    process::Command,
};

use sevenz_rust2::*;

/// Distinct, compressible contents, so a mix-up between entries shows.
fn content(name: &str) -> Vec<u8> {
    name.repeat(4096).into_bytes()
}

/// An archive holding one solid block of three entries, then one block per entry for two more,
/// written with `methods`.
fn source_archive(methods: Vec<EncoderConfiguration>) -> Cursor<Vec<u8>> {
    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new())).unwrap();
    writer.set_content_methods(methods);
    let solid = ["a.bin", "b.bin", "c.bin"];
    writer
        .push_archive_entries(
            solid
                .iter()
                .map(|name| ArchiveEntry::new_file(name))
                .collect(),
            solid
                .iter()
                .map(|name| Cursor::new(content(name)).into())
                .collect(),
        )
        .unwrap();
    for name in ["d.bin", "e.bin"] {
        writer
            .push_archive_entry(
                ArchiveEntry::new_file(name),
                Some(Cursor::new(content(name))),
            )
            .unwrap();
    }
    let mut cursor = writer.finish().unwrap();
    cursor.seek(SeekFrom::Start(0)).unwrap();
    cursor
}

/// Every entry of an archive as `(name, contents)`, in order.
fn entries(archive: Cursor<Vec<u8>>) -> Vec<(String, Vec<u8>)> {
    let mut reader = ArchiveReader::new(archive, Password::empty()).unwrap();
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

/// Copy every block of `source` raw into a new archive, renaming entries with `rename`.
fn rewrite(
    source: &mut Cursor<Vec<u8>>,
    rename: impl Fn(&ArchiveEntry) -> String,
) -> Cursor<Vec<u8>> {
    let archive = Archive::read(source, &Password::empty()).unwrap();
    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new())).unwrap();
    for block in 0..archive.blocks.len() {
        writer
            .push_raw_block(&archive, block, source, &rename)
            .unwrap();
    }
    let mut cursor = writer.finish().unwrap();
    cursor.seek(SeekFrom::Start(0)).unwrap();
    cursor
}

#[test]
fn raw_blocks_keep_their_data_under_new_names() {
    let mut source = source_archive(vec![EncoderMethod::LZMA2.into()]);
    let copy = rewrite(&mut source, |entry| format!("renamed/{}", entry.name));

    let expected: Vec<(String, Vec<u8>)> = ["a.bin", "b.bin", "c.bin", "d.bin", "e.bin"]
        .iter()
        .map(|name| (format!("renamed/{name}"), content(name)))
        .collect();
    assert_eq!(entries(copy.clone()), expected);

    // Nothing was re-encoded: the same packed streams, block for block.
    let before = Archive::read(&mut source, &Password::empty()).unwrap();
    let after = Archive::read(&mut copy.clone(), &Password::empty()).unwrap();
    assert_eq!(before.pack_sizes(), after.pack_sizes());
    assert_eq!(before.blocks.len(), after.blocks.len());
    assert!(after.is_solid);
}

#[test]
fn a_filter_chain_survives_the_copy() {
    // Two coders joined by a bind pair.
    let methods = vec![
        EncoderMethod::BCJ_X86_FILTER.into(),
        EncoderMethod::LZMA2.into(),
    ];
    let mut source = source_archive(methods);
    let copy = rewrite(&mut source, |entry| entry.name.clone());

    let after = Archive::read(&mut copy.clone(), &Password::empty()).unwrap();
    assert_eq!(after.blocks[0].coders.len(), 2);
    assert_eq!(entries(copy), entries(source));
}

#[test]
fn raw_and_encoded_blocks_mix() {
    let mut source = source_archive(vec![EncoderMethod::LZMA2.into()]);
    let archive = Archive::read(&mut source, &Password::empty()).unwrap();
    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new())).unwrap();
    writer
        .push_archive_entry(
            ArchiveEntry::new_file("new.bin"),
            Some(Cursor::new(content("new"))),
        )
        .unwrap();
    writer
        .push_raw_block(&archive, 0, &mut source, |entry| entry.name.clone())
        .unwrap();
    writer
        .push_archive_entry::<&[u8]>(ArchiveEntry::new_directory("folder"), None)
        .unwrap();
    let mut copy = writer.finish().unwrap();
    copy.seek(SeekFrom::Start(0)).unwrap();

    let names: Vec<String> = entries(copy).into_iter().map(|(name, _)| name).collect();
    assert_eq!(names, ["new.bin", "a.bin", "b.bin", "c.bin", "folder"]);
}

#[test]
fn a_missing_block_is_an_error() {
    let mut source = source_archive(vec![EncoderMethod::LZMA2.into()]);
    let archive = Archive::read(&mut source, &Password::empty()).unwrap();
    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new())).unwrap();
    assert!(
        writer
            .push_raw_block(&archive, archive.blocks.len(), &mut source, |entry| entry
                .name
                .clone())
            .is_err()
    );
}

/// 7-Zip reads what the copy wrote. Skipped when no 7-Zip is installed.
#[test]
fn seven_zip_tests_the_copy_ok() {
    let Some(seven_zip) = ["7zz", "7z"].into_iter().find(|name| {
        Command::new(name)
            .arg("i")
            .output()
            .is_ok_and(|output| output.status.success())
    }) else {
        eprintln!("no 7-Zip installed, skipping");
        return;
    };
    let methods = vec![
        EncoderMethod::BCJ_X86_FILTER.into(),
        EncoderMethod::LZMA2.into(),
    ];
    let mut source = source_archive(methods);
    let copy = rewrite(&mut source, |entry| format!("renamed/{}", entry.name));

    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("copy.7z");
    std::io::copy(&mut copy.clone(), &mut File::create(&path).unwrap()).unwrap();
    let output = Command::new(seven_zip)
        .arg("t")
        .arg(Path::new(&path))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("Files: 5"));
}

// One COPY block, defined file CRCs, and no packed CRC.
// The ordinary fixture contains file "a" with content "A".
// The solid variant contains ["a", "empty", "b"] with contents ["A", "", "B"].
fn copy_fixture(pack_pos: u64, include_files: bool, interleaved_empty: bool) -> Vec<u8> {
    let data = if interleaved_empty {
        b"AB".as_slice()
    } else {
        b"A".as_slice()
    };
    let mut header = vec![1, 4, 6]; // Header, MainStreamsInfo, PackInfo.
    // A 7z NUMBER prefixed with 0xFF stores all eight following bytes verbatim.
    header.push(0xff);
    header.extend(pack_pos.to_le_bytes());
    header.extend([1, 9, data.len() as u8, 0]); // One packed stream, size, End.
    header.extend([7, 11, 1, 0]); // UnpackInfo, Folder, one inline block.
    header.extend([1, 1, 0]); // One simple COPY coder.
    header.extend([12, data.len() as u8, 0]); // CodersUnpackSize, End.
    header.push(8); // SubStreamsInfo.
    if interleaved_empty {
        header.extend([13, 2, 9, 1]); // Two sub-streams. First size is one byte.
    }
    header.extend([10, 1]); // CRC, all file CRCs defined.
    header.extend(crc32fast::hash(b"A").to_le_bytes());
    if interleaved_empty {
        header.extend(crc32fast::hash(b"B").to_le_bytes());
    }
    header.extend([0, 0]); // End SubStreamsInfo, End MainStreamsInfo.

    if include_files {
        header.extend([5, if interleaved_empty { 3 } else { 1 }]); // FilesInfo, count.
        if interleaved_empty {
            header.extend([14, 1, 0x40]); // EmptyStream: only the middle entry.
            header.extend([15, 1, 0x80]); // EmptyFile: that entry is a file.
        }
        let mut names = vec![0]; // Names are inline.
        let file_names: &[&str] = if interleaved_empty {
            &["a", "empty", "b"]
        } else {
            &["a"]
        };
        for name in file_names {
            for code in name.encode_utf16().chain(std::iter::once(0)) {
                names.extend(code.to_le_bytes());
            }
        }
        header.extend([17, names.len() as u8]); // Name property and byte count (< 128).
        header.extend(names);
        header.push(0); // End FilesInfo.
    }
    header.push(0); // End Header.

    let mut start = Vec::new();
    start.extend((data.len() as u64).to_le_bytes()); // NextHeaderOffset.
    start.extend((header.len() as u64).to_le_bytes()); // NextHeaderSize.
    start.extend(crc32fast::hash(&header).to_le_bytes());
    let mut bytes = vec![0x37, 0x7a, 0xbc, 0xaf, 0x27, 0x1c, 0, 4];
    bytes.extend(crc32fast::hash(&start).to_le_bytes());
    bytes.extend(start);
    bytes.extend(data);
    bytes.extend(header);
    bytes
}

fn read_entries(bytes: Vec<u8>) -> Vec<(String, Vec<u8>)> {
    let mut reader = ArchiveReader::new(Cursor::new(bytes), Password::empty()).unwrap();
    let mut entries = Vec::new();
    reader
        .for_each_entries(|entry, data| {
            let mut bytes = Vec::new();
            data.read_to_end(&mut bytes)?;
            entries.push((entry.name.clone(), bytes));
            Ok(true)
        })
        .unwrap();
    entries.sort();
    entries
}

#[test]
fn raw_copy_without_files_info_returns_error() {
    let mut source = Cursor::new(copy_fixture(0, false, false));
    let archive = match Archive::read(&mut source, &Password::empty()) {
        Ok(archive) => archive,
        Err(_) => return, // Rejecting the malformed archive during parsing is fine.
    };
    assert_eq!(archive.blocks.len(), 1);

    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new())).unwrap();
    assert!(
        writer
            .push_raw_block(&archive, 0, &mut source, |entry| entry.name.clone())
            .is_err()
    );
}

#[test]
fn raw_copy_without_packed_crc_can_mix_with_encoded_entries() {
    for raw_first in [true, false] {
        let mut source = Cursor::new(copy_fixture(0, true, false));
        let archive = Archive::read(&mut source, &Password::empty()).unwrap();
        let mut writer = ArchiveWriter::new(Cursor::new(Vec::new())).unwrap();

        for copy_raw in [raw_first, !raw_first] {
            if copy_raw {
                writer
                    .push_raw_block(&archive, 0, &mut source, |entry| entry.name.clone())
                    .unwrap();
            } else {
                writer
                    .push_archive_entry(ArchiveEntry::new_file("new"), Some(b"new data".as_slice()))
                    .unwrap();
            }
        }

        let bytes = writer.finish().unwrap().into_inner();
        assert_eq!(
            read_entries(bytes),
            vec![
                ("a".into(), b"A".to_vec()),
                ("new".into(), b"new data".to_vec())
            ]
        );
    }
}

#[test]
fn raw_copy_rejects_overflowing_pack_position() {
    let mut source = Cursor::new(copy_fixture(u64::MAX - 31, true, false));
    let archive = match Archive::read(&mut source, &Password::empty()) {
        Ok(archive) => archive,
        Err(_) => return, // Rejecting the invalid offset during parsing is fine.
    };
    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new())).unwrap();

    assert!(
        writer
            .push_raw_block(&archive, 0, &mut source, |entry| entry.name.clone())
            .is_err(),
        "the packed-stream offset overflowed, but the copy succeeded"
    );
}

#[test]
fn raw_copy_handles_an_empty_file_inside_a_solid_block() {
    let mut source = Cursor::new(copy_fixture(0, true, true));
    let archive = Archive::read(&mut source, &Password::empty()).unwrap();
    assert_eq!(archive.files.len(), 3);
    assert!(!archive.files[1].has_stream);
    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new())).unwrap();

    writer
        .push_raw_block(&archive, 0, &mut source, |entry| entry.name.clone())
        .unwrap();
    writer
        .push_archive_entry::<&[u8]>(archive.files[1].clone(), None)
        .unwrap();

    let bytes = writer.finish().unwrap().into_inner();
    assert_eq!(
        read_entries(bytes),
        vec![
            ("a".into(), b"A".to_vec()),
            ("b".into(), b"B".to_vec()),
            ("empty".into(), Vec::new()),
        ]
    );
}
