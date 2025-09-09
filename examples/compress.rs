use std::{env, fs::File, time::Instant};

use sevenz_rust2::{ArchiveEntry, ArchiveReader, ArchiveWriter, NtTime, Password, SourceReader};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} [--solid] <file1> [file2] ...", args[0]);
        eprintln!("  --solid: Create a solid archive (all files compressed together)");
        std::process::exit(1);
    }

    let mut solid = false;
    let mut file_paths = Vec::new();

    for arg in &args[1..] {
        if arg == "--solid" {
            solid = true;
        } else {
            file_paths.push(arg.clone());
        }
    }

    if file_paths.is_empty() {
        eprintln!("Error: No files specified");
        std::process::exit(1);
    }

    let output_path = "output.7z";
    println!(
        "Creating {} archive: {output_path}",
        if solid { "solid" } else { "non-solid" }
    );

    let now = Instant::now();

    let mut writer = ArchiveWriter::create(output_path).expect("Failed to create archive");

    if solid {
        let mut entries = Vec::new();
        let mut readers = Vec::new();

        for file_path in &file_paths {
            let file = File::open(file_path)
                .unwrap_or_else(|error| panic!("Failed to open file '{file_path}': {error}"));

            let modification_time = file.metadata().unwrap().modified().unwrap();

            let mut entry = ArchiveEntry::new_file(file_path);
            entry.has_last_modified_date = true;
            entry.last_modified_date = NtTime::try_from(modification_time).unwrap();
            entries.push(entry);
            readers.push(SourceReader::new(file));

            println!("Added file: {file_path}");
        }

        writer
            .push_archive_entries(entries, readers)
            .expect("Failed to add files to solid archive");
    } else {
        for file_path in &file_paths {
            let file = File::open(file_path)
                .unwrap_or_else(|error| panic!("Failed to open file '{file_path}': {error}"));

            let entry = ArchiveEntry::new_file(file_path);
            let reader = SourceReader::new(file);

            writer
                .push_archive_entry(entry, Some(reader))
                .expect("Failed to add file to archive");

            println!("Added file: {file_path}");
        }
    }

    writer.finish().expect("Failed to finalize archive");

    let _archive_reader = ArchiveReader::new(File::open(output_path).unwrap(), Password::empty())
        .expect("Failed to open output file");

    println!("Compress done: {:?}", now.elapsed());
}
