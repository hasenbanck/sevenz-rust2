use std::path::PathBuf;

use sevenz_rust2::{Archive, BlockDecoder, Password};

fn main() {
    let mut file = std::fs::File::open("examples/data/sample.7z").unwrap();
    let password = Password::empty();
    let archive = Archive::read(&mut file, &password).unwrap();
    let block_count = archive.blocks.len();
    let my_file_name = "7zFormat.txt";

    for block_index in 0..block_count {
        let block_decoder = BlockDecoder::new(1, block_index, &archive, &password, &mut file);

        if !block_decoder
            .entries()
            .iter()
            .any(|entry| entry.name() == my_file_name)
        {
            // skip the block if it does not contain the file we want
            continue;
        }
        let dest = PathBuf::from("examples/data/sample_mt/");

        let mut entries_iter = block_decoder.entries_iter().expect("create entries iter");
        while let Some(Ok(entry)) = entries_iter.next_entry() {
            if entry.name() == my_file_name {
                // only extract the file we want
                let dest = dest.join(entry.name());
                sevenz_rust2::default_entry_extract_fn(&entry, &mut entries_iter, &dest)
                    .expect("extract ok");
            }
        }
    }
}
