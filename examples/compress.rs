use std::{env, fs::File, time::Instant};

use sevenz_rust2::{ArchiveEntry, ArchiveReader, ArchiveWriter, NtTime, Password, SourceReader};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!(
            "Usage: {} [--solid] [-o output.7z] <file1> [file2] ...",
            args[0]
        );
        eprintln!("  --solid: Create a solid archive (all files compressed together)");
        eprintln!("  -o <filename>: Specify output filename (default: output.7z)");
        std::process::exit(1);
    }

    let mut solid = false;
    let mut output_path = String::from("output.7z");
    let mut file_paths = Vec::new();
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "--solid" => {
                solid = true;
                i += 1;
            }
            "-o" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: -o option requires an output filename");
                    std::process::exit(1);
                }
                output_path = args[i + 1].clone();
                i += 2;
            }
            _ => {
                file_paths.push(args[i].clone());
                i += 1;
            }
        }
    }

    if file_paths.is_empty() {
        eprintln!("Error: No files specified");
        std::process::exit(1);
    }

    println!(
        "Creating {} archive: {output_path}",
        if solid { "solid" } else { "non-solid" }
    );

    let now = Instant::now();

    let mut writer = ArchiveWriter::create(&output_path)
        .unwrap_or_else(|error| panic!("Failed to create archive '{output_path}': {error}"));

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

    let _archive_reader = ArchiveReader::new(File::open(&output_path).unwrap(), Password::empty())
        .unwrap_or_else(|error| panic!("Failed to open output file '{output_path}': {error}"));

    println!("Archive created: {output_path}");
    println!("Compress done: {:?}", now.elapsed());
}
