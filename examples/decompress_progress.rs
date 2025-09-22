use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

fn main() {
    let mut sz =
        sevenz_rust2::ArchiveReader::open("examples/data/sample.7z", "pass".into()).unwrap();
    let total_size: u64 = sz
        .archive()
        .files
        .iter()
        .filter(|e| e.has_stream())
        .map(|e| e.size())
        .sum();
    let mut uncompressed_size = 0;
    let dest = PathBuf::from("examples/data/sample");

    // Process empty entries first (directories and empty files).
    // This ensures directories exist with proper attributes before files are created inside them.
    let mut empty_iter = sz.empty_entries_iter();
    while let Some(entry) = empty_iter.next_entry() {
        let path = dest.join(entry.name());
        if entry.is_directory() {
            std::fs::create_dir_all(path).unwrap();
        } else {
            // Empty file
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            File::create(path).unwrap();
        }
    }

    // Then process entries with content.
    let mut file_iter = sz.block_iter();
    while let Some(decoder) = file_iter.next_block_decoder() {
        let mut entries_iter = decoder.entries_iter().unwrap();
        while let Some(Ok(entry)) = entries_iter.next_entry() {
            let mut buf = [0u8; 1024];
            let path = dest.join(entry.name());
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();

            if entry.has_stream() && entry.size() > 0 {
                let mut file = File::create(path).unwrap();
                loop {
                    let read_size = entries_iter.read(&mut buf).unwrap();
                    if read_size == 0 {
                        break;
                    }
                    file.write_all(&buf[..read_size]).unwrap();
                    uncompressed_size += read_size;
                    println!(
                        "progress:{:.2}%",
                        (uncompressed_size as f64 / total_size as f64) * 100f64
                    );
                }
            }
        }
    }
}
