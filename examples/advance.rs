use std::{collections::HashMap, env::temp_dir, time::Instant};

use rand::Rng;
use sevenz_rust2::{
    encoder_options::{AesEncoderOptions, Lzma2Options},
    *,
};

fn main() {
    let temp_dir = temp_dir();
    let src = temp_dir.join("compress/advance");
    if src.exists() {
        let _ = std::fs::remove_dir_all(&src);
    }
    let _ = std::fs::create_dir_all(&src);
    let file_count = 100;
    let mut contents = HashMap::with_capacity(file_count);
    let mut unpack_size = 0;
    // generate random content files
    {
        for i in 0..file_count {
            let c = gen_random_contents(rand::rng().random_range(1024..10240));
            unpack_size += c.len();
            contents.insert(format!("file{i}.txt"), c);
        }
        for (filename, content) in contents.iter() {
            let _ = std::fs::write(src.join(filename), content);
        }
    }
    let dest = temp_dir.join("compress/compress.7z");

    let time = Instant::now();

    // start to compress
    let mut archive_writer = ArchiveWriter::create(&dest).expect("create writer ok");
    archive_writer.set_encrypt_header(true);

    #[cfg(feature = "aes256")]
    {
        archive_writer.set_content_methods(vec![
            AesEncoderOptions::new(Password::new("sevenz-rust")).into(),
            // We configure LZMA2 to use multiple threads to encode the data.
            Lzma2Options::from_level_mt(9, 4, 1 << 18).into(),
        ]);
    }

    archive_writer
        .push_source_path(&src, |_| true)
        .expect("pack ok");
    println!("finish");
    archive_writer.finish().expect("compress ok");
    println!("compress took {:?}/{:?}", time.elapsed(), dest);
    if src.exists() {
        let _ = std::fs::remove_dir_all(&src);
    }
    assert!(dest.exists());
    let dest_file = std::fs::File::open(&dest).unwrap();
    let metadata = dest_file.metadata().unwrap();
    println!("src  file len:{unpack_size:?}");
    println!("dest file len:{:?}", metadata.len());
    println!("ratio:{:?}", metadata.len() as f64 / unpack_size as f64);

    // start to decompress
    let mut archive_reader =
        ArchiveReader::open(&dest, "sevenz-rust".into()).expect("create reader ok");
    assert_eq!(contents.len(), archive_reader.archive().files.len());
    assert_eq!(1, archive_reader.archive().blocks.len());

    let mut block_iter = archive_reader.block_iter();
    while let Some(block_decoder) = block_iter.next_block_decoder() {
        let mut entries_iter = block_decoder.entries_iter().expect("create entries iter");
        while let Some(Ok(entry)) = entries_iter.next_entry() {
            if entry.has_stream() {
                let content = std::io::read_to_string(&mut entries_iter).expect("read content");
                assert_eq!(content, contents[entry.name()]);
            }
        }
    }

    let _ = std::fs::remove_file(dest);
}

fn gen_random_contents(len: usize) -> String {
    let mut s = String::with_capacity(len);
    let mut rng = rand::rng();
    for _ in 0..len {
        let ch = rng.random_range('A'..='Z');
        s.push(ch);
    }
    s
}
