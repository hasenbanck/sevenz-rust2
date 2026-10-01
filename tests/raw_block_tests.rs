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
            solid.iter().map(|name| ArchiveEntry::new_file(name)).collect(),
            solid
                .iter()
                .map(|name| Cursor::new(content(name)).into())
                .collect(),
        )
        .unwrap();
    for name in ["d.bin", "e.bin"] {
        writer
            .push_archive_entry(ArchiveEntry::new_file(name), Some(Cursor::new(content(name))))
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
    let methods = vec![EncoderMethod::BCJ_X86_FILTER.into(), EncoderMethod::LZMA2.into()];
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
        .push_archive_entry(ArchiveEntry::new_file("new.bin"), Some(Cursor::new(content("new"))))
        .unwrap();
    writer
        .push_raw_block(&archive, 0, &mut source, |entry| entry.name.clone())
        .unwrap();
    writer.push_archive_entry::<&[u8]>(ArchiveEntry::new_directory("folder"), None).unwrap();
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
            .push_raw_block(&archive, archive.blocks.len(), &mut source, |entry| entry.name.clone())
            .is_err()
    );
}

/// 7-Zip reads what the copy wrote. Skipped when no 7-Zip is installed.
#[test]
fn seven_zip_tests_the_copy_ok() {
    let Some(seven_zip) = ["7zz", "7z"].into_iter().find(|name| {
        Command::new(name).arg("i").output().is_ok_and(|output| output.status.success())
    }) else {
        eprintln!("no 7-Zip installed, skipping");
        return;
    };
    let methods = vec![EncoderMethod::BCJ_X86_FILTER.into(), EncoderMethod::LZMA2.into()];
    let mut source = source_archive(methods);
    let copy = rewrite(&mut source, |entry| format!("renamed/{}", entry.name));

    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("copy.7z");
    std::io::copy(&mut copy.clone(), &mut File::create(&path).unwrap()).unwrap();
    let output = Command::new(seven_zip).arg("t").arg(Path::new(&path)).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stdout));
    assert!(String::from_utf8_lossy(&output.stdout).contains("Files: 5"));
}
