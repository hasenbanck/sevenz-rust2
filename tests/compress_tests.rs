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
            0x00, 0x50, 0x01, 0xE2, 0xFB, 0xF5, 0x16, 0xDF, 0x8A, 0x30, 0xFF, 0x0C, 0x45, 0x27,
            0x9B, 0x64, 0x5D, 0x23, 0xEA, 0x13, 0x96, 0x12, 0x93, 0x15, 0xA5, 0xDC, 0x12, 0xB6,
            0x3B, 0xB5, 0x65, 0x3D, 0xCE, 0x1E, 0x68, 0xA2, 0x14, 0x40, 0x00, 0x00, 0x00, 0x00,
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

/// Writes an archive with one file entry that carries the Unix mode `st_mode`.
#[cfg(all(feature = "compress", feature = "util", unix))]
fn unix_mode_archive(path: &std::path::Path, name: &str, st_mode: u32, content: &[u8]) {
    let mut writer = ArchiveWriter::new(File::create(path).unwrap()).unwrap();
    let mut entry = ArchiveEntry::new_file(name);
    entry.has_windows_attributes = true;
    entry.windows_attributes = 0x8000 | (st_mode << 16);
    writer.push_archive_entry(entry, Some(content)).unwrap();
    writer.finish().unwrap();
}

#[cfg(all(feature = "compress", feature = "util", unix))]
#[test]
fn decompress_keeps_unix_file_mode() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = tempdir().unwrap();
    let dest = temp_dir.path().join("mode.7z");
    unix_mode_archive(&dest, "run.sh", 0o100755, b"#!/bin/sh\n");

    let out = temp_dir.path().join("out");
    decompress_file(&dest, &out).expect("decompress ok");
    let mode = std::fs::metadata(out.join("run.sh"))
        .unwrap()
        .permissions()
        .mode();
    // the umask still applies, so check the owner's execute bit
    assert_ne!(mode & 0o100, 0, "mode {mode:o}");
}

#[cfg(all(feature = "compress", feature = "util", unix))]
#[test]
fn compress_keeps_unix_file_mode() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = tempdir().unwrap();
    let src = temp_dir.path().join("src");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(src.join("run.sh"), b"#!/bin/sh\n").unwrap();
    std::fs::set_permissions(src.join("run.sh"), std::fs::Permissions::from_mode(0o755)).unwrap();

    let dest = temp_dir.path().join("mode.7z");
    compress_to_path(&src, &dest).expect("compress ok");
    let out = temp_dir.path().join("out");
    decompress_file(&dest, &out).expect("decompress ok");

    let mode = std::fs::metadata(out.join("run.sh"))
        .unwrap()
        .permissions()
        .mode();
    // the umask still applies, so check the owner's execute bit
    assert_ne!(mode & 0o100, 0, "mode {mode:o}");
}

#[cfg(all(feature = "compress", feature = "util", unix))]
#[test]
fn compress_stores_unix_file_mode() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = tempdir().unwrap();
    let src = temp_dir.path().join("src");
    std::fs::create_dir_all(src.join("dir")).unwrap();
    std::fs::write(src.join("run.sh"), b"#!/bin/sh\n").unwrap();
    std::fs::write(src.join("readonly.txt"), b"r").unwrap();
    std::fs::set_permissions(src.join("run.sh"), std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::set_permissions(
        src.join("readonly.txt"),
        std::fs::Permissions::from_mode(0o444),
    )
    .unwrap();
    std::fs::set_permissions(src.join("dir"), std::fs::Permissions::from_mode(0o700)).unwrap();

    let dest = temp_dir.path().join("mode.7z");
    compress_to_path(&src, &dest).expect("compress ok");

    let archive = Archive::open(&dest).unwrap();
    let attributes = |name: &str| {
        let entry = archive.files.iter().find(|e| e.name() == name).unwrap();
        assert!(entry.has_windows_attributes, "{name}");
        entry.windows_attributes()
    };
    assert_eq!(attributes("run.sh"), 0x8000 | 0x20 | (0o100755 << 16));
    assert_eq!(
        attributes("readonly.txt"),
        0x8000 | 0x20 | 0x1 | (0o100444 << 16)
    );
    assert_eq!(attributes("dir"), 0x8000 | 0x10 | (0o40700 << 16));
}

#[cfg(all(feature = "compress", feature = "util", unix))]
#[test]
fn decompress_replaces_read_only_file() {
    let temp_dir = tempdir().unwrap();
    let old = temp_dir.path().join("old.7z");
    let new = temp_dir.path().join("new.7z");
    unix_mode_archive(&old, "ro.txt", 0o100444, b"old");
    unix_mode_archive(&new, "ro.txt", 0o100444, b"new");

    let out = temp_dir.path().join("out");
    decompress_file(&old, &out).expect("first extraction");
    decompress_file(&new, &out).expect("second extraction into the same directory");
    assert_eq!(std::fs::read(out.join("ro.txt")).unwrap(), b"new");
}

#[cfg(all(feature = "compress", feature = "util", unix))]
#[test]
fn decompress_applies_unix_file_mode_to_existing_file() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = tempdir().unwrap();
    let dest = temp_dir.path().join("mode.7z");
    unix_mode_archive(&dest, "run.sh", 0o100755, b"#!/bin/sh\n");

    let out = temp_dir.path().join("out");
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(out.join("run.sh"), b"old").unwrap();
    std::fs::set_permissions(out.join("run.sh"), std::fs::Permissions::from_mode(0o644)).unwrap();

    decompress_file(&dest, &out).expect("decompress ok");
    let mode = std::fs::metadata(out.join("run.sh"))
        .unwrap()
        .permissions()
        .mode();
    // The umask still applies. The owner execute bit survives any common umask.
    assert_ne!(mode & 0o100, 0, "mode {mode:o}");
}

#[cfg(all(feature = "compress", feature = "util", unix))]
#[test]
fn compress_path_follows_symlinks() {
    use std::os::unix::fs::symlink;

    let temp_dir = tempdir().unwrap();
    let folder = temp_dir.path().join("proj");
    std::fs::create_dir_all(folder.join("lib")).unwrap();
    std::fs::write(folder.join("lib/libfoo.so.1"), "data").unwrap();
    symlink("libfoo.so.1", folder.join("lib/libfoo.so")).unwrap();
    symlink("lib", folder.join("lib64")).unwrap();
    symlink("missing", folder.join("dangling")).unwrap();

    let dest = temp_dir.path().join("proj.7z");
    compress_to_path(&folder, &dest).expect("compress ok");

    let mut reader = ArchiveReader::new(File::open(&dest).unwrap(), Password::empty()).unwrap();
    let mut names: Vec<String> = reader
        .archive()
        .files
        .iter()
        .map(|f| f.name().to_string())
        .collect();
    names.sort();
    assert_eq!(
        names,
        [
            "lib",
            "lib/libfoo.so",
            "lib/libfoo.so.1",
            "lib64",
            "lib64/libfoo.so",
            "lib64/libfoo.so.1"
        ]
    );
    assert_eq!(reader.read_file("lib/libfoo.so").unwrap(), b"data");
    assert_eq!(reader.read_file("lib64/libfoo.so.1").unwrap(), b"data");
}
#[cfg(all(feature = "compress", feature = "util", unix))]
#[test]
fn compress_rejects_symlink_cycle() {
    use std::os::unix::fs::symlink;

    let temp_dir = tempdir().unwrap();

    // A link to the parent directory.
    let parent_cycle = temp_dir.path().join("parent");
    std::fs::create_dir_all(parent_cycle.join("sub")).unwrap();
    symlink("..", parent_cycle.join("sub/up")).unwrap();

    // Two sibling directories that link to each other.
    let sibling_cycle = temp_dir.path().join("sibling");
    std::fs::create_dir_all(sibling_cycle.join("x")).unwrap();
    std::fs::create_dir_all(sibling_cycle.join("y")).unwrap();
    symlink("../y", sibling_cycle.join("x/to_y")).unwrap();
    symlink("../x", sibling_cycle.join("y/to_x")).unwrap();

    for folder in [parent_cycle, sibling_cycle] {
        let dest = temp_dir.path().join("cycle.7z");
        let result = compress_to_path(&folder, &dest);
        assert!(result.is_err(), "compress_to_path {folder:?}: {result:?}");

        let mut writer = ArchiveWriter::new(Cursor::new(Vec::new())).unwrap();
        let result = writer.push_source_path(&folder, |_| true).map(|_| ());
        assert!(result.is_err(), "push_source_path {folder:?}: {result:?}");
    }
}

#[cfg(all(feature = "compress", feature = "util", unix))]
#[test]
fn compress_reports_unresolvable_symlink() {
    use std::os::unix::fs::symlink;

    let temp_dir = tempdir().unwrap();
    let folder = temp_dir.path().join("proj");
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(folder.join("a.txt"), "data").unwrap();
    // Each link points to the other. Resolving them fails with ELOOP.
    symlink("y", folder.join("x")).unwrap();
    symlink("x", folder.join("y")).unwrap();

    let dest = temp_dir.path().join("proj.7z");
    let result = compress_to_path(&folder, &dest);
    assert!(result.is_err(), "compress_to_path: {result:?}");
}

#[cfg(all(feature = "compress", feature = "util", unix))]
#[test]
fn push_source_path_follows_symlinks() {
    use std::os::unix::fs::symlink;

    let temp_dir = tempdir().unwrap();
    let folder = temp_dir.path().join("proj");
    std::fs::create_dir_all(folder.join("lib")).unwrap();
    std::fs::write(folder.join("lib/libfoo.so.1"), "data").unwrap();
    symlink("libfoo.so.1", folder.join("lib/libfoo.so")).unwrap();
    symlink("lib", folder.join("lib64")).unwrap();
    symlink("missing", folder.join("dangling")).unwrap();

    let dest = temp_dir.path().join("proj.7z");
    let mut writer = ArchiveWriter::create(&dest).unwrap();
    writer.push_source_path(&folder, |_| true).unwrap();
    writer.finish().unwrap();

    let mut reader = ArchiveReader::new(File::open(&dest).unwrap(), Password::empty()).unwrap();
    let mut names: Vec<String> = reader
        .archive()
        .files
        .iter()
        .map(|f| f.name().to_string())
        .collect();
    names.sort();
    assert_eq!(
        names,
        [
            "lib/libfoo.so",
            "lib/libfoo.so.1",
            "lib64/libfoo.so",
            "lib64/libfoo.so.1"
        ]
    );
    assert_eq!(reader.read_file("lib64/libfoo.so").unwrap(), b"data");
}
