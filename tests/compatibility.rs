use std::io::Cursor;

use sevenz_rust2::{ArchiveEntry, ArchiveReader, ArchiveWriter, Password};

#[test]
fn simple_file() {
    let apache2_data = std::fs::read("tests/resources/apache2.txt").unwrap();

    let mut archive_writer = ArchiveWriter::new(Cursor::new(Vec::new())).unwrap();
    archive_writer
        .push_archive_entry(
            ArchiveEntry::new_file("apache2.txt"),
            Some(apache2_data.as_slice()),
        )
        .unwrap();
    let data = archive_writer.finish().unwrap().into_inner();

    std::fs::write("apache2.7z", data.as_slice()).unwrap();

    let mut archive_reader = ArchiveReader::new(Cursor::new(data), Password::empty()).unwrap();
    let data = archive_reader.read_file("apache2.txt").unwrap();

    assert_eq!(data, apache2_data);
}
