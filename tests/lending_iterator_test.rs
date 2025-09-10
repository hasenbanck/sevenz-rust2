use std::io::Read;

use sevenz_rust2::{ArchiveReader, Password};

const TEST_FILE: &str = "tests/resources/solid_test_iterator.7z";
const TEST_WITH_DIRS: &str = "tests/resources/test_with_dirs.7z";

#[test]
fn test_file_block_iter_basic() {
    let mut reader = ArchiveReader::open(TEST_FILE, Password::empty()).unwrap();
    let mut iter = reader.block_iter();

    let mut block_count = 0;
    let mut total_entries = 0;

    while let Some(decoder) = iter.next_block_decoder() {
        block_count += 1;
        let mut entries_iter = decoder.entries_iter().unwrap();

        while let Some(Ok(entry)) = entries_iter.next_entry() {
            total_entries += 1;
            let entry_has_stream = entry.has_stream;
            let entry_has_crc = entry.has_crc;

            if entry_has_stream && entry.size > 0 {
                assert!(entry_has_crc);
                assert_eq!(entry.size, 11556);

                let mut data = Vec::new();
                let bytes_read = entries_iter.read_to_end(&mut data).unwrap();
                assert_eq!(bytes_read, entry.size as usize);
                assert_eq!(data.len(), entry.size as usize);

                let content_str = String::from_utf8_lossy(&data);
                assert!(content_str.contains("Apache License"));
            }
        }
    }

    assert_eq!(block_count, 1);
    assert_eq!(total_entries, 4);
}

#[test]
fn test_file_block_iter_partial_read() {
    let mut reader = ArchiveReader::open(TEST_FILE, Password::empty()).unwrap();
    let mut iter = reader.block_iter();

    let mut entry_count = 0;
    let mut partial_read_count = 0;

    while let Some(decoder) = iter.next_block_decoder() {
        let mut entries_iter = decoder.entries_iter().unwrap();

        while let Some(Ok(entry)) = entries_iter.next_entry() {
            entry_count += 1;
            let entry_has_stream = entry.has_stream;
            let entry_size = entry.size;

            if entry_has_stream && entry_size > 0 {
                let bytes_to_read = std::cmp::min(100, entry_size as usize);
                let mut buffer = vec![0u8; bytes_to_read];
                entries_iter.read_exact(&mut buffer).unwrap();

                assert_eq!(buffer.len(), bytes_to_read);
                partial_read_count += 1;
            }
        }
    }

    assert_eq!(entry_count, 4);
    assert_eq!(partial_read_count, 4);
}

#[test]
fn test_file_block_iter_skip_entries() {
    let mut reader = ArchiveReader::open(TEST_FILE, Password::empty()).unwrap();
    let mut iter = reader.block_iter();

    let mut entry_count = 0;
    let mut read_count = 0;

    while let Some(decoder) = iter.next_block_decoder() {
        let mut entries_iter = decoder.entries_iter().unwrap();

        while let Some(Ok(entry)) = entries_iter.next_entry() {
            entry_count += 1;
            let entry_has_stream = entry.has_stream;

            if entry_count % 2 == 1 && entry_has_stream {
                let mut data = Vec::new();
                entries_iter.read_to_end(&mut data).unwrap();
                read_count += 1;
            }
        }
    }

    assert_eq!(entry_count, 4);
    assert_eq!(read_count, 2);
}

#[test]
fn test_file_block_iter_all_entry_names() {
    let mut reader = ArchiveReader::open(TEST_FILE, Password::empty()).unwrap();

    let expected_files: Vec<String> = reader
        .archive()
        .files
        .iter()
        .filter(|file| !file.is_directory)
        .map(|file| file.name.clone())
        .collect();

    let mut iter = reader.block_iter();
    let mut found_entries = Vec::new();

    while let Some(decoder) = iter.next_block_decoder() {
        let mut entries_iter = decoder.entries_iter().unwrap();

        while let Some(Ok(entry)) = entries_iter.next_entry() {
            found_entries.push(entry.name.clone());

            if entry.has_stream {
                let mut data = Vec::new();
                entries_iter.read_to_end(&mut data).unwrap();
            }
        }
    }

    assert_eq!(found_entries.len(), 4);

    for expected_file in expected_files {
        assert!(found_entries.contains(&expected_file));
    }
}

#[test]
fn test_invalid_block_index() {
    let mut reader = ArchiveReader::open(TEST_FILE, Password::empty()).unwrap();

    assert!(reader.block_decoder(999).is_none());
    assert!(reader.block_decoder(1).is_none());

    assert!(reader.block_entries_iter(999).is_err());
    assert!(reader.block_entries_iter(1).is_err());
}

#[test]
fn test_block_entries_iter_basic() {
    let mut reader = ArchiveReader::open(TEST_FILE, Password::empty()).unwrap();
    let decoder = reader.block_decoder(0).unwrap();
    let mut iter = decoder.entries_iter().unwrap();

    let mut entry_count = 0;
    while let Some(Ok(entry)) = iter.next_entry() {
        entry_count += 1;
        let entry_size = entry.size;
        let entry_has_stream = entry.has_stream;
        let entry_has_crc = entry.has_crc;

        assert!(entry_has_stream);
        assert_eq!(entry_size, 11556);
        assert!(entry_has_crc);

        let mut data = Vec::new();
        let bytes_read = iter.read_to_end(&mut data).unwrap();
        assert_eq!(bytes_read, entry_size as usize);
        assert_eq!(data.len(), entry_size as usize);

        let content_str = String::from_utf8_lossy(&data);
        assert!(content_str.contains("Apache License"));
    }

    assert_eq!(entry_count, 4);
}

#[test]
fn test_block_entries_iter_partial_read() {
    let mut reader = ArchiveReader::open(TEST_FILE, Password::empty()).unwrap();
    let decoder = reader.block_decoder(0).unwrap();
    let mut iter = decoder.entries_iter().unwrap();

    let mut entry_count = 0;
    while let Some(Ok(entry)) = iter.next_entry() {
        entry_count += 1;
        let entry_has_stream = entry.has_stream;
        let entry_size = entry.size;

        if entry_has_stream && entry_size > 0 {
            let bytes_to_read = std::cmp::min(100, entry_size as usize);
            let mut buffer = vec![0u8; bytes_to_read];
            iter.read_exact(&mut buffer).unwrap();

            assert_eq!(buffer.len(), bytes_to_read);
        }
    }

    assert_eq!(entry_count, 4);
}

#[test]
fn test_block_entries_iter_skip_entries() {
    let mut reader = ArchiveReader::open(TEST_FILE, Password::empty()).unwrap();
    let decoder = reader.block_decoder(0).unwrap();
    let mut iter = decoder.entries_iter().unwrap();

    let mut entry_count = 0;
    let mut read_count = 0;

    while let Some(Ok(entry)) = iter.next_entry() {
        entry_count += 1;
        let entry_has_stream = entry.has_stream;

        if entry_count % 2 == 1 && entry_has_stream {
            let mut data = Vec::new();
            iter.read_to_end(&mut data).unwrap();
            read_count += 1;
        }
    }

    assert_eq!(entry_count, 4);
    assert_eq!(read_count, 2);
}

#[test]
fn test_block_decoder_for_file() {
    let mut reader = ArchiveReader::open(TEST_FILE, Password::empty()).unwrap();

    let files: Vec<String> = reader
        .archive()
        .files
        .iter()
        .filter(|file| !file.is_directory)
        .map(|file| file.name.clone())
        .collect();

    let target_file = if files.len() >= 3 {
        &files[2]
    } else {
        &files[0]
    };

    let decoder = reader.block_decoder_for_file(target_file).unwrap();
    let mut iter = decoder.entries_iter().unwrap();

    let mut found_target = false;
    let mut files_processed = 0;

    while let Some(Ok(entry)) = iter.next_entry() {
        files_processed += 1;

        if entry.name == *target_file {
            found_target = true;

            let mut data = Vec::new();
            iter.read_to_end(&mut data).unwrap();

            assert_eq!(data.len(), 11556);
            assert_eq!(data.len(), entry.size as usize);

            let content_str = String::from_utf8_lossy(&data);
            assert!(content_str.contains("Apache License"));

            break;
        }
    }

    assert!(found_target);
    assert_eq!(files_processed, 3);
}

#[test]
fn test_block_entries_iter() {
    let mut reader = ArchiveReader::open(TEST_FILE, Password::empty()).unwrap();

    let mut iter = reader.block_entries_iter(0).unwrap();

    let mut entry_count = 0;
    let mut total_bytes_read = 0;

    while let Some(Ok(entry)) = iter.next_entry() {
        entry_count += 1;

        let mut data = Vec::new();
        let bytes_read = iter.read_to_end(&mut data).unwrap();
        total_bytes_read += bytes_read;

        assert_eq!(bytes_read, entry.size as usize);
        assert_eq!(data.len(), entry.size as usize);

        let content_str = String::from_utf8_lossy(&data);
        assert!(content_str.contains("Apache License"));
    }

    assert_eq!(entry_count, 4);
    assert_eq!(total_bytes_read, 4 * 11556);
}

#[test]
fn test_empty_entries_iter_basic() {
    let reader = ArchiveReader::open(TEST_WITH_DIRS, Password::empty()).unwrap();
    let mut iter = reader.empty_entries_iter();

    let mut empty_entries = Vec::new();
    while let Some(entry) = iter.next_entry() {
        empty_entries.push((entry.name.clone(), entry.is_directory(), entry.size));
    }

    assert_eq!(empty_entries.len(), 6);

    let dir_names: Vec<_> = empty_entries
        .iter()
        .filter(|(_, is_dir, _)| *is_dir)
        .map(|(name, _, _)| name.as_str())
        .collect();
    assert_eq!(dir_names.len(), 4);
    assert!(dir_names.contains(&"test_structure"));
    assert!(dir_names.contains(&"test_structure/subdir1"));
    assert!(dir_names.contains(&"test_structure/subdir1/subsubdir"));
    assert!(dir_names.contains(&"test_structure/subdir2"));

    let empty_file_names: Vec<_> = empty_entries
        .iter()
        .filter(|(_, is_dir, size)| !*is_dir && *size == 0)
        .map(|(name, _, _)| name.as_str())
        .collect();
    assert_eq!(empty_file_names.len(), 2);
    assert!(empty_file_names.contains(&"test_structure/empty_file.txt"));
    assert!(empty_file_names.contains(&"test_structure/subdir1/another_empty.txt"));
}

#[test]
fn test_empty_entries_iter_completeness() {
    let reader = ArchiveReader::open(TEST_WITH_DIRS, Password::empty()).unwrap();

    let mut empty_iter = reader.empty_entries_iter();
    let mut empty_entries = Vec::new();
    while let Some(entry) = empty_iter.next_entry() {
        empty_entries.push(entry.name.clone());
    }

    let mut reader = ArchiveReader::open(TEST_WITH_DIRS, Password::empty()).unwrap();
    let mut block_iter = reader.block_iter();
    let mut file_entries = Vec::new();
    while let Some(decoder) = block_iter.next_block_decoder() {
        let mut entries_iter = decoder.entries_iter().unwrap();
        while let Some(Ok(entry)) = entries_iter.next_entry() {
            file_entries.push(entry.name.clone());
        }
    }

    let reader = ArchiveReader::open(TEST_WITH_DIRS, Password::empty()).unwrap();
    let all_files: Vec<_> = reader
        .archive()
        .files
        .iter()
        .map(|f| f.name.clone())
        .collect();

    let mut all_found = empty_entries.clone();
    all_found.extend(file_entries.clone());
    all_found.sort();
    let mut expected = all_files;
    expected.sort();

    assert_eq!(all_found, expected);

    for empty_name in &empty_entries {
        assert!(!file_entries.contains(empty_name));
    }
}

#[test]
fn test_empty_entries_iter_directory_properties() {
    let reader = ArchiveReader::open(TEST_WITH_DIRS, Password::empty()).unwrap();
    let mut iter = reader.empty_entries_iter();

    let mut directories_found = 0;
    let mut empty_files_found = 0;

    while let Some(entry) = iter.next_entry() {
        if entry.is_directory() {
            directories_found += 1;
            assert_eq!(entry.size, 0);
            assert!(!entry.has_stream());
        } else {
            empty_files_found += 1;
            assert_eq!(entry.size, 0);
        }
    }

    assert_eq!(directories_found, 4);
    assert_eq!(empty_files_found, 2);
}

#[test]
fn test_empty_entries_iter_vs_for_each_entries() {
    let reader = ArchiveReader::open(TEST_WITH_DIRS, Password::empty()).unwrap();

    let mut empty_iter = reader.empty_entries_iter();
    let mut iter_empty_entries = Vec::new();
    while let Some(entry) = empty_iter.next_entry() {
        iter_empty_entries.push((entry.name.clone(), entry.is_directory(), entry.size));
    }

    let mut reader = ArchiveReader::open(TEST_WITH_DIRS, Password::empty()).unwrap();
    let mut closure_empty_entries = Vec::new();
    #[allow(deprecated)]
    reader
        .for_each_entries(|entry, reader| {
            if (!entry.has_stream || entry.size == 0) && reader.read(&mut [0u8; 1]).unwrap() == 0 {
                closure_empty_entries.push((entry.name.clone(), entry.is_directory(), entry.size));
            }
            Ok(true)
        })
        .unwrap();

    iter_empty_entries.sort();
    closure_empty_entries.sort();
    assert_eq!(iter_empty_entries, closure_empty_entries);
}

#[test]
fn test_empty_entries_iter_with_existing_test_files() {
    let reader =
        ArchiveReader::open("tests/resources/two_empty_file.7z", Password::empty()).unwrap();
    let mut iter = reader.empty_entries_iter();

    let mut empty_count = 0;
    while let Some(entry) = iter.next_entry() {
        empty_count += 1;
        assert_eq!(entry.size, 0);
        assert!(!entry.is_directory());
    }

    assert_eq!(empty_count, 2);
}
