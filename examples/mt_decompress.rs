use std::{path::PathBuf, sync::Arc};

use sevenz_rust2::{Archive, BlockDecoder, Password};

// 0. The simplest way to use multi threading is to use simply the ArchiveReader.
//    If the compression of the archive blocks supports multi threading, which is supported
//    by this crate, then the ArchiveReader will use multiple threads to decode the blocks.
//    We currently only support multi threading for decoding & encoding LZMA2.
//    Brotli, LZ4 and ZSTD could we supported in the future, if there is ever demand to do so.
//
//    See `ArchiveReader::set_thread_count()` for more information.`
fn main() {
    let time = std::time::Instant::now();
    let mut file = std::fs::File::open("examples/data/sample.7z").unwrap();
    let password = Password::empty();
    let archive = Archive::read(&mut file, &password).unwrap();
    let block_count = archive.blocks.len();
    if block_count <= 1 {
        println!("block count less than 1, use single thread");
    }
    let archive = Arc::new(archive);
    let password = Arc::new(password);

    let mut threads = Vec::new();

    // 1. We multi-thread by decompressing each block itself in parallel.
    for block_index in 0..block_count {
        let archive = archive.clone();
        let password = password.clone();

        let handle = std::thread::spawn(move || {
            let mut source = std::fs::File::open("examples/data/sample.7z").unwrap();

            // 2. For decoders that supports it, we can set the thread_count on the block decoder
            //    so that it uses multiple threads to decode the block. Currently only LZMA2 is
            //    supporting this. In this example we try to use 4 threads.
            let block_decoder = BlockDecoder::new(4, block_index, &archive, &password, &mut source);

            let dest = PathBuf::from("examples/data/sample_mt/");
            let mut entries_iter = block_decoder.entries_iter().expect("create entries iter");
            while let Some(Ok(entry)) = entries_iter.next_entry() {
                let dest = dest.join(entry.name());
                sevenz_rust2::default_entry_extract_fn(&entry, &mut entries_iter, &dest)
                    .expect("extract ok");
            }
        });
        threads.push(handle);
    }

    threads
        .into_iter()
        .for_each(|handle| handle.join().unwrap());

    println!(
        "multi-thread decompress took {:?} ms",
        time.elapsed().as_millis()
    );
}
