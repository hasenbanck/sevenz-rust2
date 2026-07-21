//! Tests for [`ArchiveWriter::push_packed_entry`] — re-emit already-compressed
//! non-solid pack streams without re-encoding (archive update / pack-copy).

#[cfg(feature = "compress")]
use std::io::{Cursor, Read, Seek, SeekFrom};

#[cfg(feature = "compress")]
use sevenz_rust2::{
    ArchiveEntry, ArchiveReader, ArchiveWriter, EncoderMethod, Password, SIGNATURE_HEADER_SIZE,
};

#[cfg(feature = "compress")]
fn write_three_file_archive() -> Vec<u8> {
    let mut out = Cursor::new(Vec::new());
    let mut writer = ArchiveWriter::new(&mut out).expect("writer");
    for (name, data) in [
        ("a.txt", b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa payload-a".as_slice()),
        ("b.txt", b"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb payload-b".as_slice()),
        ("c.txt", b"cccccccccccccccccccccccccccccccc payload-c".as_slice()),
    ] {
        let entry = ArchiveEntry::new_file(name);
        writer
            .push_archive_entry(entry, Some(Cursor::new(data.to_vec())))
            .expect("push entry");
    }
    writer.finish().expect("finish");
    out.into_inner()
}

/// Delete the middle member by pack-copying the other two streams only.
#[cfg(feature = "compress")]
#[test]
fn push_packed_entry_copy_two_of_three_after_delete() {
    let source_bytes = write_three_file_archive();

    let reader = ArchiveReader::new(Cursor::new(source_bytes.clone()), Password::empty())
        .expect("open source");
    let archive = reader.archive().clone();
    drop(reader);

    assert!(!archive.is_solid, "test fixture must be non-solid");
    assert_eq!(archive.files.iter().filter(|f| f.has_stream).count(), 3);

    let pack_base = SIGNATURE_HEADER_SIZE + archive.pack_pos();
    let pack_sizes = archive.pack_sizes();
    let stream_map = &archive.stream_map;
    let pack_offsets = stream_map.pack_stream_offsets();
    let block_first = stream_map.block_first_pack_stream_index();

    let mut dest = Cursor::new(Vec::new());
    {
        let mut writer = ArchiveWriter::new(&mut dest).expect("dest writer");

        for (file_index, file) in archive.files.iter().enumerate() {
            if !file.has_stream || file.is_directory {
                continue;
            }
            // Drop middle file "b.txt".
            if file.name() == "b.txt" {
                continue;
            }

            let block_index = stream_map.file_block_index[file_index].expect("block index");
            let block = &archive.blocks[block_index];
            assert_eq!(block.num_unpack_sub_streams(), 1);
            assert_eq!(block.packed_streams_count(), 1);
            assert_eq!(block.coders.len(), 1);

            let pack_stream_index = block_first[block_index];
            let pack_size = pack_sizes[pack_stream_index];
            let abs = pack_base + pack_offsets[pack_stream_index];

            let mut pack_cursor = Cursor::new(source_bytes.clone());
            pack_cursor.seek(SeekFrom::Start(abs)).expect("seek pack");
            let mut pack_slice = pack_cursor.take(pack_size);

            let coder = &block.coders[0];
            let method =
                EncoderMethod::by_id(coder.encoder_method_id()).expect("known encoder method");
            let props = coder.properties().to_vec();
            let unpack_sizes = block.unpack_sizes().to_vec();

            let entry = file.clone();
            // Keep original name/size/crc; only rewrite pack region.
            writer
                .push_packed_entry(
                    entry,
                    &mut pack_slice,
                    &[(method, props.as_slice())],
                    unpack_sizes,
                )
                .expect("push packed");
        }

        writer.finish().expect("finish dest");
    }

    let out_bytes = dest.into_inner();
    let mut out_reader =
        ArchiveReader::new(Cursor::new(out_bytes), Password::empty()).expect("open result");
    assert!(!out_reader.archive().is_solid);

    let names: Vec<_> = out_reader
        .archive()
        .files
        .iter()
        .filter(|f| f.has_stream)
        .map(|f| f.name().to_string())
        .collect();
    assert_eq!(names, vec!["a.txt".to_string(), "c.txt".to_string()]);

    let mut extracted = std::collections::HashMap::new();
    out_reader
        .for_each_entries(|entry, reader| {
            if entry.has_stream && !entry.is_directory {
                let mut buf = Vec::new();
                reader.read_to_end(&mut buf)?;
                extracted.insert(entry.name().to_string(), buf);
            }
            Ok(true)
        })
        .expect("extract");

    assert_eq!(
        extracted.get("a.txt").map(|b| b.as_slice()),
        Some(b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa payload-a".as_slice())
    );
    assert_eq!(
        extracted.get("c.txt").map(|b| b.as_slice()),
        Some(b"cccccccccccccccccccccccccccccccc payload-c".as_slice())
    );
    assert!(!extracted.contains_key("b.txt"));
}

/// Rename via pack-copy: same pack bytes, different entry name.
#[cfg(feature = "compress")]
#[test]
fn push_packed_entry_rename_keeps_payload() {
    let source_bytes = write_three_file_archive();
    let reader = ArchiveReader::new(Cursor::new(source_bytes.clone()), Password::empty())
        .expect("open source");
    let archive = reader.archive().clone();
    drop(reader);

    let pack_base = SIGNATURE_HEADER_SIZE + archive.pack_pos();
    let pack_sizes = archive.pack_sizes();
    let stream_map = &archive.stream_map;
    let pack_offsets = stream_map.pack_stream_offsets();
    let block_first = stream_map.block_first_pack_stream_index();

    let mut dest = Cursor::new(Vec::new());
    {
        let mut writer = ArchiveWriter::new(&mut dest).expect("dest writer");
        for (file_index, file) in archive.files.iter().enumerate() {
            if !file.has_stream || file.is_directory {
                continue;
            }
            let block_index = stream_map.file_block_index[file_index].expect("block");
            let block = &archive.blocks[block_index];
            let pack_stream_index = block_first[block_index];
            let pack_size = pack_sizes[pack_stream_index];
            let abs = pack_base + pack_offsets[pack_stream_index];

            let mut pack_cursor = Cursor::new(source_bytes.clone());
            pack_cursor.seek(SeekFrom::Start(abs)).unwrap();
            let mut pack_slice = pack_cursor.take(pack_size);

            let coder = &block.coders[0];
            let method = EncoderMethod::by_id(coder.encoder_method_id()).unwrap();
            let props = coder.properties().to_vec();

            let mut entry = file.clone();
            if entry.name() == "a.txt" {
                entry.name = "a-renamed.txt".into();
            }
            writer
                .push_packed_entry(
                    entry,
                    &mut pack_slice,
                    &[(method, props.as_slice())],
                    block.unpack_sizes().to_vec(),
                )
                .unwrap();
        }
        writer.finish().unwrap();
    }

    let mut out_reader =
        ArchiveReader::new(Cursor::new(dest.into_inner()), Password::empty()).unwrap();
    let mut found = None;
    out_reader
        .for_each_entries(|entry, reader| {
            if entry.name() == "a-renamed.txt" {
                let mut buf = Vec::new();
                reader.read_to_end(&mut buf)?;
                found = Some(buf);
            }
            Ok(true)
        })
        .unwrap();
    assert_eq!(
        found.as_deref(),
        Some(b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa payload-a".as_slice())
    );
}
