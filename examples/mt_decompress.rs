use std::{path::PathBuf, sync::Arc};

use sevenz_rust2::{Archive, BlockDecoder, Password};

fn main() {
    let time = std::time::Instant::now();
    let mut file = std::fs::File::open("examples/data/sample.7z").unwrap();
    let password = Password::empty();
    let archive = Archive::read(&mut file, &password).unwrap();
    let block_count = archive.blocks.len();
    if block_count <= 1 {
        println!("block count less than 1, use single thread");
        //TODO use single thread
    }
    let archive = Arc::new(archive);
    let password = Arc::new(password);

    let mut threads = Vec::new();
    for block_index in 0..block_count {
        let archive = archive.clone();
        let password = password.clone();
        //TODO: use thread pool
        let handle = std::thread::spawn(move || {
            let mut source = std::fs::File::open("examples/data/sample.7z").unwrap();
            let forder_dec = BlockDecoder::new(block_index, &archive, &password, &mut source);
            let dest = PathBuf::from("examples/data/sample_mt/");
            forder_dec
                .for_each_entries(&mut |entry, reader| {
                    let dest = dest.join(entry.name());
                    sevenz_rust2::default_entry_extract_fn(entry, reader, &dest)?;
                    Ok(true)
                })
                .expect("ok");
        });
        threads.push(handle);
    }

    threads
        .into_iter()
        .for_each(|handle| handle.join().unwrap());
    println!("multi-thread decompress use time:{:?}", time.elapsed());
}
