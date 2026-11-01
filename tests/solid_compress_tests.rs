#[cfg(feature = "compress")]
use sevenz_rust2::*;
#[cfg(all(feature = "compress", feature = "util"))]
use tempfile::*;

#[cfg(all(feature = "compress", feature = "util"))]
#[test]
fn compress_multi_files_solid() {
    let temp_dir = tempdir().unwrap();
    let folder = temp_dir.path().join("folder");
    std::fs::create_dir(&folder).unwrap();
    let mut files = Vec::with_capacity(100);
    let mut contents = Vec::with_capacity(100);
    for i in 1..=10000 {
        let name = format!("file{i}.txt");
        let content = format!("file{i} with content");
        std::fs::write(folder.join(&name), &content).unwrap();
        files.push(name);
        contents.push(content);
    }
    let dest = temp_dir.path().join("folder.7z");

    let mut sz = ArchiveWriter::create(&dest).unwrap();
    sz.push_source_path(&folder, |_| true).unwrap();
    sz.finish().expect("compress ok");

    let decompress_dest = temp_dir.path().join("decompress");
    decompress_file(dest, &decompress_dest).expect("decompress ok");
    assert!(decompress_dest.exists());
    for i in 0..files.len() {
        let name = &files[i];
        let content = &contents[i];
        let decompress_file = decompress_dest.join(name);
        assert!(decompress_file.exists());
        assert_eq!(&std::fs::read_to_string(&decompress_file).unwrap(), content);
    }
}

#[cfg(all(feature = "compress", feature = "util"))]
#[test]
fn compress_multi_files_mix_solid_and_non_solid() {
    use std::fs::File;

    let temp_dir = tempdir().unwrap();
    let folder = temp_dir.path().join("folder");
    std::fs::create_dir(&folder).unwrap();
    let mut files = Vec::with_capacity(100);
    let mut contents = Vec::with_capacity(100);
    for i in 1..=100 {
        let name = format!("file{i}.txt");
        let content = format!("file{i} with content");
        std::fs::write(folder.join(&name), &content).unwrap();
        files.push(name);
        contents.push(content);
    }
    let dest = temp_dir.path().join("folder.7z");

    let mut sz = ArchiveWriter::create(&dest).unwrap();

    // solid compression
    sz.push_source_path(&folder, |_| true).unwrap();

    // non solid compression
    for i in 101..=200 {
        let name = format!("file{i}.txt");
        let content = format!("file{i} with content");
        std::fs::write(folder.join(&name), &content).unwrap();
        files.push(name.clone());
        contents.push(content);

        let src = folder.join(&name);
        sz.push_archive_entry(
            ArchiveEntry::from_path(&src, name),
            Some(File::open(src).unwrap()),
        )
        .expect("ok");
    }

    sz.finish().expect("compress ok");

    let decompress_dest = temp_dir.path().join("decompress");
    decompress_file(dest, &decompress_dest).expect("decompress ok");
    assert!(decompress_dest.exists());
    for i in 0..files.len() {
        let name = &files[i];
        let content = &contents[i];
        let decompress_file = decompress_dest.join(name);
        assert!(decompress_file.exists());
        assert_eq!(&std::fs::read_to_string(&decompress_file).unwrap(), content);
    }
}

#[cfg(all(feature = "compress", feature = "util"))]
#[test]
fn prepare_block_round_trips_through_push_prepared_block() {
    use std::{io::Cursor, sync::Arc};

    let temp_dir = tempdir().unwrap();
    let dest = temp_dir.path().join("prepared.7z");

    let contents: Vec<String> = (1..=200).map(|i| format!("file{i} with content")).collect();
    let entries: Vec<ArchiveEntry> = (1..=200)
        .map(|i| ArchiveEntry::new_file(&format!("file{i}.txt")))
        .collect();
    let readers: Vec<SourceReader<Cursor<Vec<u8>>>> = contents
        .iter()
        .map(|c| SourceReader::new(Cursor::new(c.clone().into_bytes())))
        .collect();

    let methods = Arc::new(vec![EncoderConfiguration::new(EncoderMethod::LZMA2)]);
    let block = prepare_block(methods, entries, readers).expect("prepare ok");
    assert_eq!(block.len(), 200);
    assert!(!block.is_empty());
    assert!(block.compressed_len() > 0);

    let mut sz = ArchiveWriter::create(&dest).unwrap();
    sz.push_prepared_block(block).expect("push ok");
    sz.finish().expect("finish ok");

    let out = temp_dir.path().join("out");
    decompress_file(&dest, &out).expect("decompress ok");
    for (i, content) in contents.iter().enumerate() {
        let f = out.join(format!("file{}.txt", i + 1));
        assert!(f.exists(), "missing {}", f.display());
        assert_eq!(&std::fs::read_to_string(&f).unwrap(), content);
    }
}

#[cfg(feature = "compress")]
type Sources = Vec<SourceReader<std::io::Cursor<Vec<u8>>>>;

/// The entries of issue 157: a directory and an empty file between files with data.
#[cfg(feature = "compress")]
fn mixed_entries() -> (Vec<ArchiveEntry>, Sources) {
    use std::io::Cursor;

    let entries = vec![
        ArchiveEntry::new_file("a.txt"),
        ArchiveEntry::new_directory("dir"),
        ArchiveEntry::new_file("b.txt"),
        ArchiveEntry::new_file("empty.txt"),
    ];
    let readers = [&b"AAAAA"[..], b"", b"BBBBB", b""]
        .iter()
        .map(|data| SourceReader::new(Cursor::new(data.to_vec())))
        .collect();
    (entries, readers)
}

/// Every entry of an archive as `(name, contents)`, in order.
#[cfg(feature = "compress")]
fn read_entries(archive: Vec<u8>) -> Vec<(String, Vec<u8>)> {
    let mut reader = ArchiveReader::new(std::io::Cursor::new(archive), Password::empty()).unwrap();
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

#[cfg(feature = "compress")]
fn expected_mixed_entries() -> Vec<(String, Vec<u8>)> {
    vec![
        ("a.txt".to_string(), b"AAAAA".to_vec()),
        ("dir".to_string(), Vec::new()),
        ("b.txt".to_string(), b"BBBBB".to_vec()),
        ("empty.txt".to_string(), Vec::new()),
    ]
}

#[cfg(feature = "compress")]
#[test]
fn push_archive_entries_with_a_directory_round_trips() {
    let (entries, readers) = mixed_entries();
    let mut writer = ArchiveWriter::new(std::io::Cursor::new(Vec::new())).unwrap();
    writer.push_archive_entries(entries, readers).unwrap();
    let archive = writer.finish().unwrap().into_inner();

    assert_eq!(read_entries(archive), expected_mixed_entries());
}

#[cfg(feature = "compress")]
#[test]
fn prepare_block_with_a_directory_round_trips() {
    use std::sync::Arc;

    let (entries, readers) = mixed_entries();
    let methods = Arc::new(vec![EncoderConfiguration::new(EncoderMethod::LZMA2)]);
    let block = prepare_block(methods, entries, readers).unwrap();
    let mut writer = ArchiveWriter::new(std::io::Cursor::new(Vec::new())).unwrap();
    writer.push_prepared_block(block).unwrap();
    let archive = writer.finish().unwrap().into_inner();

    assert_eq!(read_entries(archive), expected_mixed_entries());
}

#[cfg(feature = "compress")]
#[test]
fn push_archive_entries_rejects_data_for_a_directory() {
    use std::io::Cursor;

    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new())).unwrap();
    let result = writer.push_archive_entries(
        vec![
            ArchiveEntry::new_file("a.txt"),
            ArchiveEntry::new_directory("dir"),
        ],
        vec![
            SourceReader::new(Cursor::new(b"AAAAA".to_vec())),
            SourceReader::new(Cursor::new(b"DIR".to_vec())),
        ],
    );

    assert!(result.is_err());
}

#[cfg(feature = "compress")]
#[test]
fn prepare_block_refuses_an_empty_method_list() {
    use std::{io::Cursor, sync::Arc};

    // A block with no coders writes an archive that no reader can open, and `finish()` reports
    // success while doing it, so this has to fail at the point the caller can still act on it.
    let err = prepare_block(
        Arc::new(Vec::new()),
        vec![ArchiveEntry::new_file("a.txt")],
        vec![SourceReader::new(Cursor::new(b"hello".to_vec()))],
    )
    .expect_err("an empty method list must be refused");
    assert!(
        format!("{err}").contains("must not be empty"),
        "unexpected error: {err}"
    );
}

#[cfg(feature = "compress")]
#[test]
fn prepare_block_refuses_mismatched_entries_and_readers() {
    use std::{io::Cursor, sync::Arc};

    let err = prepare_block(
        Arc::new(vec![EncoderConfiguration::new(EncoderMethod::LZMA2)]),
        vec![
            ArchiveEntry::new_file("a.txt"),
            ArchiveEntry::new_file("b.txt"),
        ],
        vec![SourceReader::new(Cursor::new(b"only one".to_vec()))],
    )
    .expect_err("a length mismatch must be refused");
    assert!(
        format!("{err}").contains("against"),
        "unexpected error: {err}"
    );
}
