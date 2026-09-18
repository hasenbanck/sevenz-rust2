#[cfg(all(feature = "compress", feature = "util"))]
use std::{
    fs::File,
    hash::{Hash, Hasher},
    io::{Cursor, Read},
};

#[cfg(all(feature = "compress", feature = "util"))]
use sevenz_rust2::encoder_options::*;
#[cfg(all(feature = "compress", feature = "util"))]
use sevenz_rust2::*;
#[cfg(all(feature = "compress", feature = "util"))]
use tempfile::*;

#[cfg(all(feature = "compress", feature = "util"))]
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

#[cfg(all(feature = "compress", feature = "util"))]
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

#[cfg(all(feature = "compress", feature = "util"))]
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

#[cfg(all(feature = "compress", feature = "util"))]
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

#[cfg(all(feature = "compress", feature = "util"))]
#[test]
fn compress_folder_with_multi_file() {
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

#[cfg(all(feature = "compress", feature = "util"))]
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

#[cfg(all(feature = "compress", feature = "util", feature = "aes256"))]
#[test]
fn compress_one_file_with_random_content_encrypted() {
    use rand::prelude::*;
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

#[cfg(all(feature = "compress", feature = "util"))]
fn test_compression_method(methods: &[EncoderConfiguration]) {
    let mut content = Vec::new();
    File::open("tests/resources/decompress_x86.exe")
        .unwrap()
        .read_to_end(&mut content)
        .unwrap();

    let mut bytes = Vec::new();

    {
        let mut writer = ArchiveWriter::new(Cursor::new(&mut bytes)).unwrap();
        let file = ArchiveEntry::new_file("data/decompress_x86.exe");
        let directory = ArchiveEntry::new_directory("data");

        writer.set_content_methods(methods.to_vec());
        writer
            .push_archive_entry(file, Some(content.as_slice()))
            .unwrap();
        writer.push_archive_entry::<&[u8]>(directory, None).unwrap();
        writer.finish().unwrap();
    }

    let mut reader = ArchiveReader::new(Cursor::new(bytes.as_slice()), Password::empty()).unwrap();

    assert_eq!(reader.archive().files.len(), 2);

    reader
        .archive()
        .files
        .iter()
        .filter(|file| !file.is_directory)
        .for_each(|file| {
            let mut file_methods = Vec::<EncoderMethod>::new();
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

#[cfg(all(feature = "compress", feature = "util"))]
#[test]
fn compress_with_copy_algorithm() {
    test_compression_method(&[EncoderMethod::COPY.into()]);
}

#[cfg(all(feature = "compress", feature = "util"))]
#[test]
fn compress_with_delta_lzma_algorithm() {
    for i in 1..=4 {
        test_compression_method(&[
            EncoderMethod::LZMA.into(),
            DeltaOptions::from_distance(i).into(),
        ]);
    }
}

#[cfg(all(feature = "compress", feature = "util"))]
#[test]
fn compress_with_delta_lzma2_algorithm() {
    for i in 1..=4 {
        test_compression_method(&[
            EncoderMethod::LZMA2.into(),
            DeltaOptions::from_distance(i).into(),
        ]);
    }
}

#[cfg(all(feature = "compress", feature = "util"))]
#[test]
fn compress_with_bcj_x86_lzma2_algorithm() {
    test_compression_method(&[
        EncoderMethod::LZMA2.into(),
        EncoderMethod::BCJ_X86_FILTER.into(),
    ]);
}

#[cfg(all(feature = "compress", feature = "util"))]
#[test]
fn compress_with_bcj_arm_lzma2_algorithm() {
    test_compression_method(&[
        EncoderMethod::LZMA2.into(),
        EncoderMethod::BCJ_ARM_FILTER.into(),
    ]);
}

#[cfg(all(feature = "compress", feature = "util"))]
#[test]
fn compress_with_bcj_arm64_lzma2_algorithm() {
    test_compression_method(&[
        EncoderMethod::LZMA2.into(),
        EncoderMethod::BCJ_ARM64_FILTER.into(),
    ]);
}

#[cfg(all(feature = "compress", feature = "util"))]
#[test]
fn compress_with_bcj_arm_thumb_lzma2_algorithm() {
    test_compression_method(&[
        EncoderMethod::LZMA2.into(),
        EncoderMethod::BCJ_ARM_THUMB_FILTER.into(),
    ]);
}

#[cfg(all(feature = "compress", feature = "util"))]
#[test]
fn compress_with_bcj_ia64_lzma2_algorithm() {
    test_compression_method(&[
        EncoderMethod::LZMA2.into(),
        EncoderMethod::BCJ_IA64_FILTER.into(),
    ]);
}

#[cfg(all(feature = "compress", feature = "util"))]
#[test]
fn compress_with_bcj_sparc_lzma2_algorithm() {
    test_compression_method(&[
        EncoderMethod::LZMA2.into(),
        EncoderMethod::BCJ_SPARC_FILTER.into(),
    ]);
}

#[cfg(all(feature = "compress", feature = "util"))]
#[test]
fn compress_with_bcj_ppc_lzma2_algorithm() {
    test_compression_method(&[
        EncoderMethod::LZMA2.into(),
        EncoderMethod::BCJ_PPC_FILTER.into(),
    ]);
}

#[cfg(all(feature = "compress", feature = "util"))]
#[test]
fn compress_with_bcj_riscv_lzma2_algorithm() {
    test_compression_method(&[
        EncoderMethod::LZMA2.into(),
        EncoderMethod::BCJ_RISCV_FILTER.into(),
    ]);
}

#[cfg(all(feature = "compress", feature = "util"))]
#[test]
fn compress_with_lzma_algorithm() {
    test_compression_method(&[EncoderMethod::LZMA.into()]);
}

#[cfg(all(feature = "compress", feature = "util"))]
#[test]
fn compress_with_lzma2_algorithm() {
    test_compression_method(&[EncoderMethod::LZMA2.into()]);
}

#[cfg(all(feature = "compress", feature = "util", feature = "ppmd"))]
#[test]
fn compress_with_ppmd_algorithm() {
    test_compression_method(&[EncoderMethod::PPMD.into()]);
}

#[cfg(all(feature = "compress", feature = "ppmd"))]
mod ppmd_regressions {
    use std::io::Cursor;

    use sevenz_rust2::{
        Archive, ArchiveEntry, ArchiveWriter, Password, encoder_options::PpmdOptions,
    };

    #[test]
    fn packed_stream_matches_7zip() {
        let input = b"PPMd regression: one stream, one finish.\n".repeat(3);
        // Independently generated with 7-Zip 26.03:
        // 7z a reference.7z sample.txt -m0=PPMd:o6:mem24 -mhc=off
        // sample.txt contains `input`; these are its packed bytes starting at offset 32.
        let expected = [
            0x00, 0x50, 0x01, 0xe2, 0xfb, 0xf5, 0x16, 0xdf, 0x8a, 0x30, 0xff, 0x0c, 0x45, 0x27,
            0x9b, 0x64, 0x5d, 0x23, 0xea, 0x13, 0x96, 0x12, 0x93, 0x15, 0xa5, 0xdc, 0x12, 0xb6,
            0x3b, 0xb5, 0x65, 0x3d, 0xce, 0x1e, 0x68, 0xa2, 0x14, 0x40, 0x00, 0x00, 0x00, 0x00,
        ];

        let mut writer = ArchiveWriter::new(Cursor::new(Vec::new())).unwrap();
        writer.set_content_methods(vec![PpmdOptions::from_order_memory_size(6, 1 << 24).into()]);
        writer
            .push_archive_entry(ArchiveEntry::new_file("sample.txt"), Some(input.as_slice()))
            .unwrap();
        let mut output = writer.finish().unwrap();
        output.set_position(0);
        let archive = Archive::read(&mut output, &Password::empty()).unwrap();
        assert_eq!(archive.pack_sizes(), &[expected.len() as u64]);
        let start = 32 + archive.pack_pos() as usize;
        assert_eq!(&output.get_ref()[start..start + expected.len()], &expected);
    }

    #[cfg(feature = "aes256")]
    #[test]
    fn encrypted_final_block_preserves_output_error() {
        use std::io::{self, Seek, SeekFrom, Write};

        use sevenz_rust2::{Error, encoder_options::AesEncoderOptions};

        #[derive(Default)]
        struct RejectPayload {
            storage: Cursor<Vec<u8>>,
            rejected_lengths: Vec<usize>,
        }

        impl Write for RejectPayload {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                if self.storage.position() >= 32 && !bytes.is_empty() {
                    self.rejected_lengths.push(bytes.len());
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "payload rejected",
                    ));
                }
                self.storage.write(bytes)
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        impl Seek for RejectPayload {
            fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
                self.storage.seek(position)
            }
        }

        let mut output = RejectPayload::default();
        {
            let mut writer = ArchiveWriter::new(&mut output).unwrap();
            writer.set_content_methods(vec![
                AesEncoderOptions::new(Password::new("ppmd-regression")).into(),
                PpmdOptions::from_order_memory_size(6, 1 << 24).into(),
            ]);
            // A single symbol produces less than one AES block, so the output
            // failure can only occur when the encryption buffer is finalized.
            let error = writer
                .push_archive_entry(ArchiveEntry::new_file("symbol.txt"), Some(&b"x"[..]))
                .expect_err("the final encrypted block must reach the output");
            match error {
                Error::Io(source, _) => {
                    assert_eq!(source.kind(), io::ErrorKind::PermissionDenied);
                    assert_eq!(source.to_string(), "payload rejected");
                }
                other => panic!("expected the output error, got {other:?}"),
            }
        }
        assert_eq!(output.rejected_lengths, [16]);
    }
}

#[cfg(all(feature = "compress", feature = "util", feature = "brotli"))]
#[test]
fn compress_with_brotli_standard_algorithm() {
    test_compression_method(&[BrotliOptions::default().with_skippable_frame_size(0).into()]);
}

#[cfg(all(feature = "compress", feature = "util", feature = "brotli"))]
#[test]
fn compress_with_brotli_skippable_algorithm() {
    test_compression_method(&[BrotliOptions::default()
        .with_skippable_frame_size(64 * 1024)
        .into()]);
}

#[cfg(all(feature = "compress", feature = "util", feature = "bzip2"))]
#[test]
fn compress_with_bzip2_algorithm() {
    test_compression_method(&[EncoderMethod::BZIP2.into()]);
}

#[cfg(all(feature = "compress", feature = "util", feature = "deflate"))]
#[test]
fn compress_with_deflate_algorithm() {
    test_compression_method(&[EncoderMethod::DEFLATE.into()]);
}

#[cfg(all(feature = "compress", feature = "util", feature = "lz4"))]
#[test]
fn compress_with_lz4_algorithm() {
    test_compression_method(&[Lz4Options::default().with_skippable_frame_size(0).into()]);
}

#[cfg(all(feature = "compress", feature = "util", feature = "lz4"))]
#[test]
fn compress_with_lz4_skippable_algorithm() {
    test_compression_method(&[Lz4Options::default()
        .with_skippable_frame_size(128 * 1024)
        .into()]);
}

#[cfg(all(feature = "compress", feature = "util", feature = "lz4"))]
#[test]
fn compress_with_zstd_algorithm() {
    test_compression_method(&[EncoderMethod::ZSTD.into()]);
}

#[cfg(all(feature = "compress", feature = "util"))]
#[test]
fn anti_item_roundtrip() {
    let mut bytes = Vec::new();
    {
        let mut writer = ArchiveWriter::new(Cursor::new(&mut bytes)).unwrap();
        let mut entry = ArchiveEntry::new_file("deleted.txt");
        entry.is_anti_item = true;
        writer.push_archive_entry::<&[u8]>(entry, None).unwrap();
        writer.finish().unwrap();
    }

    let reader = ArchiveReader::new(Cursor::new(bytes.as_slice()), Password::empty()).unwrap();
    assert_eq!(reader.archive().files.len(), 1);
    let entry = &reader.archive().files[0];
    assert_eq!(entry.name(), "deleted.txt");
    assert!(entry.is_anti_item(), "entry should be an anti-item");
}

#[cfg(all(feature = "compress", feature = "aes256"))]
#[test]
fn encrypted_file_header_requires_password_to_read() {
    use std::io::Cursor;

    use sevenz_rust2::{
        Archive, ArchiveEntry, ArchiveWriter, Password,
        encoder_options::{AesEncoderOptions, Lzma2Options},
    };

    let content = std::fs::read("tests/resources/apache2.txt").unwrap();

    let mut bytes = Vec::new();

    {
        let mut writer = ArchiveWriter::new(Cursor::new(&mut bytes)).unwrap();
        writer.set_content_methods(vec![
            AesEncoderOptions::new(Password::new("test")).into(),
            Lzma2Options::default().into(),
        ]);
        let entry = ArchiveEntry::new_file("apache2.txt");
        writer
            .push_archive_entry(entry, Some(content.as_slice()))
            .unwrap();
        writer.finish().unwrap();
    }

    let result = Archive::read(&mut Cursor::new(bytes.as_slice()), &Password::empty());
    assert!(
        result.is_err(),
        "Reading an encrypted archive header without a password should not be possible"
    );
}

#[cfg(all(feature = "compress", feature = "util"))]
#[test]
fn compress_path_does_not_emit_root_dir_entry() {
    let temp_dir = tempdir().unwrap();
    let folder = temp_dir.path().join("folder");
    std::fs::create_dir(&folder).unwrap();
    std::fs::write(folder.join("file1.txt"), "hello").unwrap();
    std::fs::create_dir(folder.join("sub")).unwrap();
    std::fs::write(folder.join("sub").join("file2.txt"), "world").unwrap();

    let dest = temp_dir.path().join("folder.7z");
    compress_to_path(&folder, &dest).expect("compress ok");

    let reader = ArchiveReader::new(File::open(&dest).unwrap(), Password::empty()).unwrap();
    let names: Vec<String> = reader
        .archive()
        .files
        .iter()
        .map(|f| f.name().replace('\\', "/"))
        .collect();

    assert!(
        !names.iter().any(|n| n.is_empty() || n == "folder"),
        "root directory should not appear as an entry, got: {names:?}"
    );
    assert!(names.iter().any(|n| n == "file1.txt"));
    assert!(names.iter().any(|n| n == "sub"));
    assert!(names.iter().any(|n| n == "sub/file2.txt"));
}
