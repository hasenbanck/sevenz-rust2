use sevenz_rust2::*;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{Cursor, Read};
use tempfile::*;

#[cfg(feature = "compress")]
#[test]
fn compress_empty_file() {
    let temp_dir = tempdir().unwrap();
    let source = temp_dir.path().join("empty.txt");
    File::create(&source).unwrap();
    let dest = temp_dir.path().join("empty.7z");
    compress_to_path(source, &dest).expect("compress ok");

    let decompress_dest = temp_dir.path().join("decompress");
    decompress_file(dest, &decompress_dest).expect("decompress ok");
    assert!(decompress_dest.exists());
    let decompress_file = decompress_dest.join("empty.txt");
    assert!(decompress_file.exists());

    assert_eq!(std::fs::read_to_string(&decompress_file).unwrap(), "");
}

#[cfg(feature = "compress")]
#[test]
fn compress_one_file_with_content() {
    let temp_dir = tempdir().unwrap();
    let source = temp_dir.path().join("file1.txt");
    std::fs::write(&source, "file1 with content").unwrap();
    let dest = temp_dir.path().join("file1.7z");
    compress_to_path(source, &dest).expect("compress ok");

    let decompress_dest = temp_dir.path().join("decompress");
    decompress_file(dest, &decompress_dest).expect("decompress ok");
    assert!(decompress_dest.exists());
    let decompress_file = decompress_dest.join("file1.txt");
    assert!(decompress_file.exists());

    assert_eq!(
        std::fs::read_to_string(&decompress_file).unwrap(),
        "file1 with content"
    );
}

#[cfg(feature = "compress")]
#[test]
fn compress_empty_folder() {
    let temp_dir = tempdir().unwrap();
    let folder = temp_dir.path().join("folder");
    std::fs::create_dir(&folder).unwrap();
    let dest = temp_dir.path().join("folder.7z");
    compress_to_path(&folder, &dest).expect("compress ok");

    let decompress_dest = temp_dir.path().join("decompress");
    decompress_file(dest, &decompress_dest).expect("decompress ok");
    assert!(decompress_dest.exists());
    assert!(decompress_dest.read_dir().unwrap().next().is_none());
}

#[cfg(feature = "compress")]
#[test]
fn compress_folder_with_one_file() {
    let temp_dir = tempdir().unwrap();
    let folder = temp_dir.path().join("folder");
    std::fs::create_dir(&folder).unwrap();
    std::fs::write(folder.join("file1.txt"), "file1 with content").unwrap();
    let dest = temp_dir.path().join("folder.7z");
    compress_to_path(&folder, &dest).expect("compress ok");

    let decompress_dest = temp_dir.path().join("decompress");
    decompress_file(dest, &decompress_dest).expect("decompress ok");
    assert!(decompress_dest.exists());
    let decompress_file = decompress_dest.join("file1.txt");
    assert!(decompress_file.exists());

    assert_eq!(
        std::fs::read_to_string(&decompress_file).unwrap(),
        "file1 with content"
    );
}

#[cfg(feature = "compress")]
#[test]
fn compress_folder_with_multi_file() {
    let temp_dir = tempdir().unwrap();
    let folder = temp_dir.path().join("folder");
    std::fs::create_dir(&folder).unwrap();
    let mut files = Vec::with_capacity(100);
    let mut contents = Vec::with_capacity(100);
    for i in 1..=100 {
        let name = format!("file{}.txt", i);
        let content = format!("file{} with content", i);
        std::fs::write(folder.join(&name), &content).unwrap();
        files.push(name);
        contents.push(content);
    }
    let dest = temp_dir.path().join("folder.7z");
    compress_to_path(&folder, &dest).expect("compress ok");

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

#[cfg(feature = "compress")]
#[test]
fn compress_folder_with_nested_folder() {
    let temp_dir = tempdir().unwrap();
    let folder = temp_dir.path().join("folder");
    let inner = folder.join("a/b/c");
    std::fs::create_dir_all(&inner).unwrap();
    std::fs::write(inner.join("file1.txt"), "file1 with content").unwrap();
    let dest = temp_dir.path().join("folder.7z");
    compress_to_path(&folder, &dest).expect("compress ok");

    let decompress_dest = temp_dir.path().join("decompress");
    decompress_file(dest, &decompress_dest).expect("decompress ok");
    assert!(decompress_dest.exists());
    let decompress_file = decompress_dest.join("a/b/c/file1.txt");
    assert!(decompress_file.exists());

    assert_eq!(
        std::fs::read_to_string(&decompress_file).unwrap(),
        "file1 with content"
    );
}

#[cfg(all(feature = "compress", feature = "aes256"))]
#[test]
fn compress_one_file_with_random_content_encrypted() {
    use rand::Rng;
    for _ in 0..10 {
        let temp_dir = tempdir().unwrap();
        let source = temp_dir.path().join("file1.txt");
        let mut rng = rand::rng();
        let mut content = String::with_capacity(rng.random_range(1..10240));

        for _ in 0..content.capacity() {
            let c = rng.random_range(' '..'~');
            content.push(c);
        }
        std::fs::write(&source, &content).unwrap();
        let dest = temp_dir.path().join("file1.7z");

        compress_to_path_encrypted(source, &dest, "rust".into()).expect("compress ok");

        let decompress_dest = temp_dir.path().join("decompress");
        decompress_file_with_password(dest, &decompress_dest, "rust".into())
            .expect("decompress ok");
        assert!(decompress_dest.exists());
        let decompress_file = decompress_dest.join("file1.txt");
        assert!(decompress_file.exists());

        assert_eq!(std::fs::read_to_string(&decompress_file).unwrap(), content);
    }
}

fn test_compression_method(methods: &[SevenZMethodConfiguration]) {
    let mut content = Vec::new();
    File::open("tests/resources/decompress_x86.exe")
        .unwrap()
        .read_to_end(&mut content)
        .unwrap();

    let mut bytes = Vec::new();

    {
        let mut writer = SevenZWriter::new(Cursor::new(&mut bytes)).unwrap();

        let file = SevenZArchiveEntry::new_file("data/decompress_x86.exe");
        let folder = SevenZArchiveEntry::new_folder("data");

        writer.set_content_methods(methods.to_vec());
        writer
            .push_archive_entry(file, Some(content.as_slice()))
            .unwrap();
        writer.push_archive_entry::<&[u8]>(folder, None).unwrap();
        writer.finish().unwrap();
    }

    //std::fs::write("test.7z", bytes.as_slice()).unwrap();

    let mut reader = SevenZReader::new(Cursor::new(bytes.as_slice()), Password::empty()).unwrap();

    assert_eq!(reader.archive().files.len(), 2);

    reader
        .archive()
        .files
        .iter()
        .filter(|file| !file.is_directory)
        .for_each(|file| {
            let mut file_methods = Vec::<SevenZMethod>::new();
            reader
                .file_compression_methods(file.name(), &mut file_methods)
                .expect("can't read compression method");

            for (file_method, method) in file_methods.iter().zip(methods) {
                assert_eq!(file_method.name(), method.method.name());
            }
        });

    assert!(
        reader
            .archive()
            .files
            .iter()
            .any(|file| file.name() == "data")
    );
    assert!(
        reader
            .archive()
            .files
            .iter()
            .any(|file| file.name() == "data/decompress_x86.exe")
    );

    let data = reader.read_file("data/decompress_x86.exe").unwrap();

    fn hash(data: &[u8]) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }

    assert_eq!(hash(&content), hash(&data));
}

#[cfg(feature = "compress")]
#[test]
fn compress_with_copy_algorithm() {
    test_compression_method(&[SevenZMethod::COPY.into()]);
}

#[cfg(feature = "compress")]
#[test]
fn compress_with_delta_lzma_algorithm() {
    for i in 1..=4 {
        test_compression_method(&[
            SevenZMethod::LZMA.into(),
            DeltaOptions::from_distance(i).into(),
        ]);
    }
}

#[cfg(feature = "compress")]
#[test]
fn compress_with_delta_lzma2_algorithm() {
    for i in 1..=4 {
        test_compression_method(&[
            SevenZMethod::LZMA2.into(),
            DeltaOptions::from_distance(i).into(),
        ]);
    }
}

#[cfg(feature = "compress")]
#[test]
fn compress_with_lzma_algorithm() {
    test_compression_method(&[SevenZMethod::LZMA.into()]);
}

#[cfg(feature = "compress")]
#[test]
fn compress_with_lzma2_algorithm() {
    test_compression_method(&[SevenZMethod::LZMA2.into()]);
}

#[cfg(feature = "brotli")]
#[test]
fn compress_with_brotli_standard_algorithm() {
    test_compression_method(&[BrotliOptions::default().with_skippable_frame_size(0).into()]);
}

#[cfg(feature = "brotli")]
#[test]
fn compress_with_brotli_skippable_algorithm() {
    test_compression_method(&[BrotliOptions::default()
        .with_skippable_frame_size(64 * 1024)
        .into()]);
}

#[cfg(feature = "bzip2")]
#[test]
fn compress_with_bzip2_algorithm() {
    test_compression_method(&[SevenZMethod::BZIP2.into()]);
}

#[cfg(feature = "deflate")]
#[test]
fn compress_with_deflate_algorithm() {
    test_compression_method(&[SevenZMethod::DEFLATE.into()]);
}

#[cfg(feature = "lz4")]
#[test]
fn compress_with_lz4_algorithm() {
    test_compression_method(&[SevenZMethod::LZ4.into()]);
}

#[cfg(feature = "zstd")]
#[test]
fn compress_with_zstd_algorithm() {
    test_compression_method(&[SevenZMethod::ZSTD.into()]);
}
