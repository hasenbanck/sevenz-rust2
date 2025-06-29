use std::{
    fs::{File, read, read_to_string},
    path::PathBuf,
};

use sevenz_rust2::{Archive, ArchiveReader, BlockDecoder, Password, decompress_file};
use tempfile::tempdir;

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

#[cfg(feature = "ppmd")]
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

#[cfg(feature = "bzip2")]
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
#[cfg(feature = "brotli")]
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

#[cfg(feature = "lz4")]
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
    let archive = Archive::read(&mut file, &[]).unwrap();
    for i in 0..archive.folders.len() {
        let fd = BlockDecoder::new(i, &archive, &[], &mut file);
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
fn test_entry_compressed_size() {
    let dir = std::fs::read_dir("tests/resources").unwrap();
    for entry in dir {
        let path = entry.unwrap().path();
        if path.to_string_lossy().ends_with("7z") {
            println!("{path:?}");
            let mut file = File::open(path).unwrap();
            let archive = Archive::read(&mut file, &[]).unwrap();
            for i in 0..archive.folders.len() {
                let fi = archive.stream_map.folder_first_file_index[i];
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
