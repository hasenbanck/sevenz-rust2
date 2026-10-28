use std::fs::File;
#[cfg(feature = "util")]
use std::{
    fs::{read, read_to_string},
    path::PathBuf,
};

#[cfg(feature = "util")]
use sevenz_rust2::decompress_file;
use sevenz_rust2::{Archive, ArchiveReader, BlockDecoder, Password};
#[cfg(feature = "util")]
use tempfile::tempdir;

#[cfg(feature = "util")]
#[test]
fn decompress_single_empty_file_unencoded_header() {
    let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    source_file.push("tests/resources/single_empty_file.7z");
    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().to_path_buf();
    let mut file1_path = target.clone();
    file1_path.push("empty.txt");

    decompress_file(source_file, target).unwrap();

    assert_eq!(read_to_string(file1_path).unwrap(), "");
}

#[cfg(feature = "util")]
#[test]
fn decompress_two_empty_files_unencoded_header() {
    let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    source_file.push("tests/resources/two_empty_file.7z");
    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().to_path_buf();
    let mut file1_path = target.clone();
    file1_path.push("file1.txt");
    let mut file2_path = target.clone();
    file2_path.push("file2.txt");

    decompress_file(source_file, target).unwrap();

    assert_eq!(read_to_string(file1_path).unwrap(), "");
    assert_eq!(read_to_string(file2_path).unwrap(), "");
}

#[cfg(feature = "util")]
#[test]
fn decompress_lzma_single_file_unencoded_header() {
    let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    source_file.push("tests/resources/single_file_with_content_lzma.7z");
    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().to_path_buf();
    let mut file1_path = target.clone();
    file1_path.push("file.txt");

    decompress_file(source_file, target).unwrap();

    assert_eq!(read_to_string(file1_path).unwrap(), "this is a file\n");
}

#[cfg(feature = "util")]
#[test]
fn decompress_lzma2_bcj_x86_file() {
    let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    source_file.push("tests/resources/decompress_example_lzma2_bcj_x86.7z");
    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().to_path_buf();
    let mut file1_path = target.clone();
    file1_path.push("decompress.exe");

    decompress_file(source_file, target).unwrap();

    let mut expected_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    expected_file.push("tests/resources/decompress_x86.exe");

    assert_eq!(
        read(file1_path).unwrap(),
        read(expected_file).unwrap(),
        "decompressed files do not match!"
    );
}

#[cfg(feature = "util")]
#[test]
fn decompress_bcj_arm64_file() {
    let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    source_file.push("tests/resources/decompress_example_bcj_arm64.7z");
    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().to_path_buf();
    let mut file1_path = target.clone();
    file1_path.push("decompress_arm64.exe");

    decompress_file(source_file, target).unwrap();

    let mut expected_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    expected_file.push("tests/resources/decompress_arm64.exe");

    assert_eq!(
        read(file1_path).unwrap(),
        read(expected_file).unwrap(),
        "decompressed files do not match!"
    );
}

#[cfg(feature = "util")]
#[test]
fn decompress_lzma_multiple_files_encoded_header() {
    let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    source_file.push("tests/resources/two_files_with_content_lzma.7z");
    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().to_path_buf();
    let mut file1_path = target.clone();
    file1_path.push("file1.txt");
    let mut file2_path = target.clone();
    file2_path.push("file2.txt");

    decompress_file(source_file, target).unwrap();

    assert_eq!(read_to_string(file1_path).unwrap(), "file one content\n");
    assert_eq!(read_to_string(file2_path).unwrap(), "file two content\n");
}

#[cfg(feature = "util")]
#[test]
fn decompress_delta_lzma_single_file_unencoded_header() {
    let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    source_file.push("tests/resources/delta.7z");
    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().to_path_buf();
    let mut file1_path = target.clone();
    file1_path.push("delta.txt");

    decompress_file(source_file, target).unwrap();

    assert_eq!(read_to_string(file1_path).unwrap(), "aaaabbbbcccc");
}

#[cfg(feature = "util")]
#[test]
fn decompress_copy_lzma2_single_file() {
    let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    source_file.push("tests/resources/copy.7z");
    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().to_path_buf();
    let mut file1_path = target.clone();
    file1_path.push("copy.txt");

    decompress_file(source_file, target).unwrap();

    assert_eq!(read_to_string(file1_path).unwrap(), "simple copy encoding");
}

#[cfg(all(feature = "util", feature = "ppmd"))]
#[test]
fn decompress_ppmd_single_file() {
    let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    source_file.push("tests/resources/ppmd.7z");

    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().to_path_buf();
    let mut file1_path = target.clone();
    file1_path.push("apache2.txt");

    decompress_file(source_file, target).unwrap();
    let decompressed_content = read_to_string(file1_path).unwrap();

    let expected = read_to_string("tests/resources/apache2.txt").unwrap();

    assert_eq!(decompressed_content, expected);
}

#[cfg(all(feature = "util", feature = "bzip2"))]
#[test]
fn decompress_bzip2_file() {
    let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    source_file.push("tests/resources/bzip2_file.7z");
    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().to_path_buf();

    let mut hello_path = target.clone();
    hello_path.push("hello.txt");

    let mut foo_path = target.clone();
    foo_path.push("foo.txt");

    decompress_file(source_file, target).unwrap();

    assert_eq!(read_to_string(hello_path).unwrap(), "world\n");
    assert_eq!(read_to_string(foo_path).unwrap(), "bar\n");
}

/// zstdmt (which 7zip ZS uses), does encapsulate brotli data in a special frames,
/// for which we need to have custom logic to decode and encode to.
#[cfg(all(feature = "util", feature = "brotli"))]
#[test]
fn decompress_zstdmt_brotli_file() {
    let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    source_file.push("tests/resources/zstdmt-brotli.7z");

    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().to_path_buf();

    let mut license_path = target.clone();
    license_path.push("LICENSE");

    decompress_file(source_file, target).unwrap();

    assert!(
        read_to_string(license_path)
            .unwrap()
            .contains("Apache License")
    );
}

#[cfg(all(feature = "util", feature = "lz4"))]
#[test]
fn decompress_zstdmt_lz4_file() {
    let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    source_file.push("tests/resources/zstdmt-lz4.7z");

    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().to_path_buf();

    let mut license_path = target.clone();
    license_path.push("LICENSE");

    decompress_file(source_file, target).unwrap();

    assert!(
        read_to_string(license_path)
            .unwrap()
            .contains("Apache License")
    );
}

#[test]
fn test_bcj2() {
    let mut file = File::open("tests/resources/7za433_7zip_lzma2_bcj2.7z").unwrap();
    let archive = Archive::read(&mut file, &Password::empty()).unwrap();
    for i in 0..archive.blocks.len() {
        let password = Password::empty();
        let fd = BlockDecoder::new(1, i, &archive, &password, &mut file);
        println!("entry_count:{}", fd.entry_count());
        fd.for_each_entries(&mut |entry, reader| {
            println!("{}=>{:?}", entry.has_stream, entry.name());
            std::io::copy(reader, &mut std::io::sink())?;
            Ok(true)
        })
        .unwrap();
    }
}

#[test]
fn test_delta_bcj2() {
    // Regression test for a multi-coder folder that chains a Delta filter on top of a
    // BCJ2 coder (`Method = Delta BCJ2`). Such folders used to fail to decode with
    // `Unsupported method` because the decoder required the folder's main (final
    // output) coder to be BCJ2 itself and could not handle a single-input filter
    // layered on top of it. The fixture was produced by 7-Zip and contains three
    // entries; every entry has a stored CRC that is verified during decoding, so a
    // successful read of every entry also asserts byte correctness.
    let mut file = File::open("tests/resources/delta_bcj2.7z").unwrap();
    let archive = Archive::read(&mut file, &Password::empty()).unwrap();

    let mut decoded = Vec::new();
    for i in 0..archive.blocks.len() {
        let password = Password::empty();
        let fd = BlockDecoder::new(1, i, &archive, &password, &mut file);
        fd.for_each_entries(&mut |entry, reader| {
            let mut data = Vec::new();
            std::io::copy(reader, &mut data)?;
            decoded.push((entry.name().to_string(), data.len()));
            Ok(true)
        })
        .unwrap();
    }

    assert_eq!(
        decoded,
        vec![
            ("c/code1.bin".to_string(), 9000),
            ("c/code2.bin".to_string(), 7000),
            ("c/wave.bin".to_string(), 16000),
        ]
    );
}

#[test]
fn test_empty_entry_between_files_of_a_solid_block() {
    // One solid block holds `d.txt`, `b.txt` and `c.txt`. The empty `a.txt` sits between
    // `d.txt` and `b.txt` in the file list. The block spans four entries for three sub-streams.
    let path = "tests/resources/interleaved_empty_entry.7z";
    let expected = vec![
        ("d.txt".to_string(), 1000),
        ("a.txt".to_string(), 0),
        ("b.txt".to_string(), 1000),
        ("c.txt".to_string(), 1000),
    ];

    let mut reader = ArchiveReader::open(path, Password::empty()).unwrap();
    let mut decoded = Vec::new();
    reader
        .for_each_entries(|entry, data| {
            let mut buf = Vec::new();
            data.read_to_end(&mut buf)?;
            decoded.push((entry.name().to_string(), buf.len()));
            Ok(true)
        })
        .unwrap();
    assert_eq!(decoded, expected);
    assert_eq!(reader.read_file("c.txt").unwrap().len(), 1000);

    let mut file = File::open(path).unwrap();
    let archive = Archive::read(&mut file, &Password::empty()).unwrap();
    let password = Password::empty();
    let fd = BlockDecoder::new(1, 0, &archive, &password, &mut file);
    assert_eq!(fd.entry_count(), 4);
    let names: Vec<&str> = fd.entries().iter().map(|entry| entry.name()).collect();
    assert_eq!(names, ["d.txt", "a.txt", "b.txt", "c.txt"]);
}

#[test]
fn test_unread_entry_data_of_a_solid_block() {
    // One solid block holds `test/test1.txt` and `test/test2.txt`. The callback leaves all or
    // part of `test/test1.txt` unread. `test/test2.txt` must still decode to its own data.
    let path = "tests/resources/solid.7z";
    let mut reader = ArchiveReader::open(path, Password::empty()).unwrap();
    let expected = reader.read_file("test/test2.txt").unwrap();

    for read_len in [0, 100] {
        let mut reader = ArchiveReader::open(path, Password::empty()).unwrap();
        let mut second = None;
        reader
            .for_each_entries(|entry, data| {
                if entry.name() == "test/test1.txt" {
                    let mut buf = vec![0; read_len];
                    data.read_exact(&mut buf)?;
                } else if entry.name() == "test/test2.txt" {
                    let mut buf = Vec::new();
                    data.read_to_end(&mut buf)?;
                    second = Some(buf);
                }
                Ok(true)
            })
            .unwrap();
        assert_eq!(second.as_ref(), Some(&expected), "read_len {read_len}");
    }
}

#[test]
fn test_entry_compressed_size() {
    let dir = std::fs::read_dir("tests/resources").unwrap();
    for entry in dir {
        let path = entry.unwrap().path();
        if path.to_string_lossy().ends_with("7z") {
            println!("{path:?}");
            let mut file = File::open(path).unwrap();
            let archive = Archive::read(&mut file, &Password::empty()).unwrap();
            for i in 0..archive.blocks.len() {
                let fi = archive.stream_map.block_first_file_index[i];
                let file = &archive.files[fi];
                println!(
                    "\t:{}\tsize={}, \tcompressed={}",
                    file.name(),
                    file.size,
                    file.compressed_size
                );
                if file.has_stream && file.size > 0 {
                    assert!(file.compressed_size > 0);
                }
            }
        }
    }
}

#[test]
fn test_get_file_by_path() {
    // non_solid.7z and solid.7z are expected to have the same content.
    let mut non_solid_reader =
        ArchiveReader::open("tests/resources/non_solid.7z", Password::empty()).unwrap();
    let mut solid_reader =
        ArchiveReader::open("tests/resources/solid.7z", Password::empty()).unwrap();

    let paths: Vec<String> = non_solid_reader
        .archive()
        .files
        .iter()
        .filter(|file| !file.is_directory)
        .map(|file| file.name.clone())
        .collect();

    for path in paths.iter() {
        let data0 = non_solid_reader.read_file(path.as_str()).unwrap();
        let data1 = solid_reader.read_file(path.as_str()).unwrap();

        assert!(!data0.is_empty());
        assert!(!data1.is_empty());
        assert_eq!(&data0, &data1);
    }
}

#[cfg(all(feature = "compress", feature = "util"))]
#[test]
fn anti_item_deletes_file_on_extract() {
    use std::io::Cursor;

    use sevenz_rust2::{
        ArchiveEntry, ArchiveWriter, decompress_with_extract_fn, default_entry_extract_fn,
    };

    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().join("to_delete.txt");
    std::fs::write(&target, "will be deleted").unwrap();
    assert!(target.exists());

    let mut bytes = Vec::new();
    {
        let mut writer = ArchiveWriter::new(Cursor::new(&mut bytes)).unwrap();
        let mut entry = ArchiveEntry::new_file("to_delete.txt");
        entry.is_anti_item = true;
        writer.push_archive_entry::<&[u8]>(entry, None).unwrap();
        writer.finish().unwrap();
    }

    let dest = temp_dir.path().to_path_buf();
    decompress_with_extract_fn(
        Cursor::new(bytes.as_slice()),
        &dest,
        |entry, reader, dest_path| {
            if entry.is_anti_item() {
                std::fs::remove_file(dest_path).ok();
                return Ok(true);
            }
            default_entry_extract_fn(entry, reader, dest_path)
        },
    )
    .unwrap();

    assert!(!target.exists(), "anti-item should have been deleted");
}

#[cfg(all(feature = "compress", feature = "util"))]
fn build_archive_with_entry_name(name: &str) -> Vec<u8> {
    use std::io::Cursor;

    use sevenz_rust2::{ArchiveEntry, ArchiveWriter};

    let mut bytes = Vec::new();
    {
        let mut writer = ArchiveWriter::new(Cursor::new(&mut bytes)).unwrap();
        let entry = ArchiveEntry::new_file(name);
        let content = b"pwned" as &[u8];
        writer.push_archive_entry(entry, Some(content)).unwrap();
        writer.finish().unwrap();
    }
    bytes
}

#[cfg(all(feature = "compress", feature = "util"))]
#[test]
fn path_traversal_relative_entry_is_rejected() {
    use std::io::Cursor;

    use sevenz_rust2::decompress;

    let temp_dir = tempdir().unwrap();
    let dest = temp_dir.path().join("out");
    // Escapes `dest` up into the tempdir root, where we can detect a stray write
    // without touching anything outside the test sandbox.
    let escaped = temp_dir.path().join("sevenz_pwned_relative");
    assert!(!escaped.exists());

    // `dest/../sevenz_pwned_relative` resolves to `escaped` above.
    let bytes = build_archive_with_entry_name("../sevenz_pwned_relative");
    let result = decompress(Cursor::new(bytes.as_slice()), &dest);

    assert!(result.is_err(), "traversal entry must be rejected");
    assert!(
        !escaped.exists(),
        "file must not be written outside the destination"
    );
}

#[cfg(all(feature = "compress", feature = "util"))]
#[test]
fn path_traversal_absolute_entry_is_rejected() {
    use std::io::Cursor;

    use sevenz_rust2::decompress;

    let temp_dir = tempdir().unwrap();
    let dest = temp_dir.path().join("out");
    // Absolute path inside the tempdir: `dest.join(abs)` would discard `dest`.
    let escaped = temp_dir.path().join("sevenz_pwned_abs");
    let abs_name = escaped.to_string_lossy().to_string();
    assert!(escaped.is_absolute());
    assert!(!escaped.exists());

    let bytes = build_archive_with_entry_name(&abs_name);
    let result = decompress(Cursor::new(bytes.as_slice()), &dest);

    assert!(result.is_err(), "absolute entry must be rejected");
    assert!(
        !escaped.exists(),
        "file must not be written outside the destination"
    );
}

#[cfg(all(feature = "compress", feature = "util"))]
#[test]
fn path_traversal_normal_nested_entry_still_extracts() {
    use std::io::Cursor;

    use sevenz_rust2::decompress;

    let temp_dir = tempdir().unwrap();
    let dest = temp_dir.path().join("out");

    let bytes = build_archive_with_entry_name("a/b/c.txt");
    decompress(Cursor::new(bytes.as_slice()), &dest).unwrap();

    let extracted = dest.join("a").join("b").join("c.txt");
    assert_eq!(std::fs::read(&extracted).unwrap(), b"pwned");
}

#[cfg(all(feature = "compress", feature = "util"))]
#[test]
fn default_entry_extract_fn_rejects_parent_dir_component() {
    use sevenz_rust2::{ArchiveEntry, default_entry_extract_fn};

    // Defense-in-depth: a caller that bypasses `decompress_impl` and hands an
    // already-joined path containing `..` directly to the default extractor must
    // still be rejected.
    let temp_dir = tempdir().unwrap();
    let escaped = temp_dir.path().join("sevenz_pwned_direct");
    let traversed = temp_dir
        .path()
        .join("out")
        .join("..")
        .join("sevenz_pwned_direct");
    assert!(!escaped.exists());

    let entry = ArchiveEntry::new_file("sevenz_pwned_direct");
    let mut data = b"pwned" as &[u8];
    let result = default_entry_extract_fn(&entry, &mut data, &traversed);

    assert!(result.is_err(), "parent-dir path must be rejected");
    assert!(!escaped.exists(), "no file may be written via a `..` path");
}

/// Regression test for <https://github.com/hasenbanck/sevenz-rust2/issues/127>.
#[test]
fn malformed_coder_stream_counts_are_rejected() {
    let mut file = File::open("tests/resources/issue_127_coder_stream_overflow.bin").unwrap();

    let result = Archive::read(&mut file, &Password::empty());

    assert!(
        result.is_err(),
        "malformed archive must be rejected, got: {result:?}"
    );
}

// 7z structure IDs for the hand-crafted archives below.
const K_END: u8 = 0x00;
const K_HEADER: u8 = 0x01;
const K_MAIN_STREAMS_INFO: u8 = 0x04;
const K_FILES_INFO: u8 = 0x05;
const K_PACK_INFO: u8 = 0x06;
const K_UNPACK_INFO: u8 = 0x07;
const K_SUB_STREAMS_INFO: u8 = 0x08;
const K_SIZE: u8 = 0x09;
const K_CRC: u8 = 0x0A;
const K_FOLDER: u8 = 0x0B;
const K_CODERS_UNPACK_SIZE: u8 = 0x0C;
const K_NUM_UNPACK_STREAM: u8 = 0x0D;
const K_EMPTY_STREAM: u8 = 0x0E;
const K_NAME: u8 = 0x11;

/// Coder description of a single COPY coder: id size 1, no properties, id 0x00.
const COPY_CODER: &[u8] = &[0x01, 0x00];

/// A block of a hand-crafted archive with a single coder.
struct RawBlock {
    /// Coder flags, id and properties of the single coder.
    coder: Vec<u8>,
    /// The unpack size the header declares for the block.
    unpack_size: u64,
    /// The block CRC. The header carries block CRCs when every block has one.
    crc: Option<u32>,
    /// The declared size of each sub-stream. The count can be zero.
    sub_streams: Vec<u64>,
}

/// Encodes a value in the 7z NUMBER format.
fn write_number(out: &mut Vec<u8>, value: u64) {
    let mut first: u8 = 0;
    let mut mask: u8 = 0x80;
    let mut low = Vec::new();
    for i in 0..8 {
        if value < (1u64 << (7 * (i + 1))) {
            first |= (value >> (8 * i)) as u8;
            break;
        }
        first |= mask;
        mask >>= 1;
        low.push((value >> (8 * i)) as u8);
    }
    out.push(first);
    out.extend_from_slice(&low);
}

/// Builds an unencoded header. Each block reads one pack stream of the given size.
/// Files listed in `empty_files` have no stream. The other files map to the
/// sub-streams in block order.
fn raw_header(
    pack_sizes: &[u64],
    blocks: &[RawBlock],
    names: &[&str],
    empty_files: &[usize],
) -> Vec<u8> {
    let mut h = vec![K_HEADER, K_MAIN_STREAMS_INFO, K_PACK_INFO, 0x00];
    write_number(&mut h, pack_sizes.len() as u64);
    h.push(K_SIZE);
    for &size in pack_sizes {
        write_number(&mut h, size);
    }
    h.push(K_END);

    h.extend_from_slice(&[K_UNPACK_INFO, K_FOLDER]);
    write_number(&mut h, blocks.len() as u64);
    h.push(0x00);
    for block in blocks {
        h.push(0x01);
        h.extend_from_slice(&block.coder);
    }
    h.push(K_CODERS_UNPACK_SIZE);
    for block in blocks {
        write_number(&mut h, block.unpack_size);
    }
    if blocks.iter().all(|block| block.crc.is_some()) {
        h.extend_from_slice(&[K_CRC, 0x01]);
        for block in blocks {
            h.extend_from_slice(&block.crc.unwrap().to_le_bytes());
        }
    }
    h.push(K_END);

    h.extend_from_slice(&[K_SUB_STREAMS_INFO, K_NUM_UNPACK_STREAM]);
    for block in blocks {
        write_number(&mut h, block.sub_streams.len() as u64);
    }
    if blocks.iter().any(|block| block.sub_streams.len() > 1) {
        h.push(K_SIZE);
        for block in blocks {
            for &size in block.sub_streams.iter().rev().skip(1).rev() {
                write_number(&mut h, size);
            }
        }
    }
    h.extend_from_slice(&[K_END, K_END]);

    h.push(K_FILES_INFO);
    write_number(&mut h, names.len() as u64);
    if !empty_files.is_empty() {
        let mut bits = vec![0u8; names.len().div_ceil(8)];
        for &index in empty_files {
            bits[index / 8] |= 0x80 >> (index % 8);
        }
        h.push(K_EMPTY_STREAM);
        write_number(&mut h, bits.len() as u64);
        h.extend_from_slice(&bits);
    }
    let mut name_bytes = vec![0x00];
    for name in names {
        for unit in name.encode_utf16().chain([0]) {
            name_bytes.extend_from_slice(&unit.to_le_bytes());
        }
    }
    h.push(K_NAME);
    write_number(&mut h, name_bytes.len() as u64);
    h.extend_from_slice(&name_bytes);
    h.extend_from_slice(&[K_END, K_END]);
    h
}

/// Assembles a 7z file from the pack data and the header that follows it.
fn raw_7z(pack: &[u8], header: &[u8]) -> Vec<u8> {
    let mut start = Vec::with_capacity(20);
    start.extend_from_slice(&(pack.len() as u64).to_le_bytes());
    start.extend_from_slice(&(header.len() as u64).to_le_bytes());
    start.extend_from_slice(&crc32fast::hash(header).to_le_bytes());

    let mut out = vec![0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C, 0x00, 0x04];
    out.extend_from_slice(&crc32fast::hash(&start).to_le_bytes());
    out.extend_from_slice(&start);
    out.extend_from_slice(pack);
    out.extend_from_slice(header);
    out
}

/// A solid LZMA2 block declares two 16 byte entries but decodes to 16 MiB. No entry
/// may hand out more bytes than its declared size, neither through `for_each_entries`
/// nor through `read_file`.
#[cfg(feature = "compress")]
#[test]
fn final_solid_entry_output_is_bounded_by_declared_size() {
    use std::io::{Cursor, Write};

    const DECLARED: u64 = 16;

    let mut options = lzma_rust2::Lzma2Options::with_preset(1);
    options.lzma_options.dict_size = 1 << 20;
    let mut writer = lzma_rust2::Lzma2Writer::new(Vec::new(), options);
    writer.write_all(&vec![0; 16 << 20]).unwrap();
    let pack = writer.finish().unwrap();

    // LZMA2 coder: id size 1 with properties, id 0x21, one property byte for a 1 MiB dictionary.
    let block = RawBlock {
        coder: vec![0x21, 0x21, 0x01, 16],
        unpack_size: 2 * DECLARED,
        crc: None,
        sub_streams: vec![DECLARED, DECLARED],
    };
    let header = raw_header(&[pack.len() as u64], &[block], &["a", "b"], &[]);
    let archive = raw_7z(&pack, &header);

    let mut reader = ArchiveReader::new(Cursor::new(archive.clone()), Password::empty()).unwrap();
    let mut delivered = Vec::new();
    let _ = reader.for_each_entries(|entry, data| {
        let size = std::io::copy(data, &mut std::io::sink())?;
        delivered.push((entry.name().to_string(), size));
        Ok(true)
    });
    for (name, size) in &delivered {
        assert!(
            *size <= DECLARED,
            "entry {name} delivered {size} bytes but declares {DECLARED}"
        );
    }

    let mut reader = ArchiveReader::new(Cursor::new(archive), Password::empty()).unwrap();
    if let Ok(data) = reader.read_file("b") {
        assert!(
            data.len() as u64 <= DECLARED,
            "read_file returned {} bytes but the entry declares {DECLARED}",
            data.len()
        );
    }
}

/// A callback that ignores a checksum error from its reader and returns `true` must not
/// make the iteration succeed.
#[test]
fn ignored_checksum_error_fails_iteration() {
    use std::io::Cursor;

    let payload = b"hello world";
    let block = RawBlock {
        coder: COPY_CODER.to_vec(),
        unpack_size: payload.len() as u64,
        crc: Some(!crc32fast::hash(payload)),
        sub_streams: vec![payload.len() as u64],
    };
    let header = raw_header(&[payload.len() as u64], &[block], &["payload"], &[]);
    let archive = raw_7z(payload, &header);

    let mut reader = ArchiveReader::new(Cursor::new(archive), Password::empty()).unwrap();
    let mut saw_error = false;
    let result = reader.for_each_entries(|_entry, data| {
        saw_error |= data.read_to_end(&mut Vec::new()).is_err();
        Ok(true)
    });

    assert!(saw_error, "the callback must see the checksum error");
    assert!(
        result.is_err(),
        "iteration must fail after a checksum error, got: {result:?}"
    );
}

/// `false` from the `for_each_entries` callback ends the current block. Iteration
/// continues with the next block.
#[test]
fn false_from_callback_ends_only_the_current_block() {
    use std::io::Cursor;

    let first = b"first";
    let second = b"second";
    let blocks = [first.as_slice(), second.as_slice()].map(|data| RawBlock {
        coder: COPY_CODER.to_vec(),
        unpack_size: data.len() as u64,
        crc: Some(crc32fast::hash(data)),
        sub_streams: vec![data.len() as u64],
    });
    let header = raw_header(
        &[first.len() as u64, second.len() as u64],
        &blocks,
        &["first", "second"],
        &[],
    );
    let archive = raw_7z(&[first.as_slice(), second.as_slice()].concat(), &header);

    let mut reader = ArchiveReader::new(Cursor::new(archive), Password::empty()).unwrap();
    let mut visited = Vec::new();
    reader
        .for_each_entries(|entry, _data| {
            visited.push(entry.name().to_string());
            Ok(false)
        })
        .unwrap();

    assert_eq!(visited, ["first", "second"]);
}

/// The 7z format allows a block without sub-streams. Such a block holds no entry
/// and must not fail iteration.
#[test]
fn block_without_sub_streams_is_accepted() {
    use std::io::Cursor;

    let payload = b"data";
    let block = RawBlock {
        coder: COPY_CODER.to_vec(),
        unpack_size: payload.len() as u64,
        crc: Some(crc32fast::hash(payload)),
        sub_streams: Vec::new(),
    };
    let header = raw_header(&[payload.len() as u64], &[block], &["empty"], &[0]);
    let archive = raw_7z(payload, &header);

    let mut reader = ArchiveReader::new(Cursor::new(archive), Password::empty()).unwrap();
    let mut visited = Vec::new();
    let result = reader.for_each_entries(|entry, data| {
        std::io::copy(data, &mut std::io::sink())?;
        visited.push(entry.name().to_string());
        Ok(true)
    });

    assert!(
        result.is_ok(),
        "a block without sub-streams must be accepted, got: {result:?}"
    );
    assert_eq!(visited, ["empty"]);
}

/// An AES coder decrypts whole 16 byte blocks. The padding after the declared unpack
/// size of the coder is not part of the entry, so an encrypted COPY archive decodes to
/// exactly its declared size.
#[cfg(all(feature = "compress", feature = "aes256"))]
#[test]
fn encrypted_copy_entry_excludes_aes_padding() {
    use std::io::Cursor;

    use sevenz_rust2::{
        ArchiveEntry, ArchiveWriter, EncoderConfiguration, EncoderMethod,
        encoder_options::AesEncoderOptions,
    };

    let content = b"hello world, this is not a multiple of sixteen";
    let mut bytes = Vec::new();
    {
        let mut writer = ArchiveWriter::new(Cursor::new(&mut bytes)).unwrap();
        writer.set_content_methods(vec![
            AesEncoderOptions::new(Password::new("test")).into(),
            EncoderConfiguration::new(EncoderMethod::COPY),
        ]);
        writer
            .push_archive_entry(ArchiveEntry::new_file("a.txt"), Some(content.as_slice()))
            .unwrap();
        writer.finish().unwrap();
    }

    let mut reader = ArchiveReader::new(Cursor::new(bytes), Password::new("test")).unwrap();
    let mut data = Vec::new();
    let result = reader.for_each_entries(|_entry, reader| {
        std::io::copy(reader, &mut data)?;
        Ok(true)
    });

    assert!(
        result.is_ok(),
        "a valid encrypted COPY archive must decode, got: {result:?}"
    );
    assert_eq!(data, content);
}
