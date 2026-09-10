#![cfg(feature = "compress")]

use std::io::{Cursor, ErrorKind};

use sevenz_rust2::{
    ArchiveEntry, ArchiveWriter, EncoderConfiguration, Error,
    encoder_options::{Lzma2Options, LzmaOptions},
};

fn assert_dictionary_rejected(config: EncoderConfiguration) {
    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new())).unwrap();
    writer.set_content_methods(vec![config]);

    let result = writer.push_archive_entry(ArchiveEntry::new_file("data"), Some(&b"x"[..]));
    assert!(matches!(
        result,
        Err(Error::Io(error, _)) if error.kind() == ErrorKind::InvalidInput
    ));
}

#[test]
fn lzma_rejects_unsupported_dictionary() {
    for level in [0, 6] {
        for dict_size in [1 << 30, 1 << 31, u32::MAX] {
            let mut options = LzmaOptions::from_level(level);
            options.set_dictionary_size(dict_size);
            assert_dictionary_rejected(options.into());
        }
        #[cfg(target_pointer_width = "32")]
        {
            let mut options = LzmaOptions::from_level(level);
            options.set_dictionary_size(268435455);
            assert_dictionary_rejected(options.into());
        }
    }
}

#[test]
fn lzma2_rejects_unsupported_dictionary() {
    for level in [0, 6] {
        for threads in [0, 1, 2] {
            for dict_size in [1 << 30, 1 << 31, u32::MAX] {
                let mut options = Lzma2Options::from_level_mt(level, threads, 65536);
                options.set_dictionary_size(dict_size);
                assert_dictionary_rejected(options.into());
            }
            #[cfg(target_pointer_width = "32")]
            {
                let mut options = Lzma2Options::from_level_mt(level, threads, 65536);
                options.set_dictionary_size(268435455);
                assert_dictionary_rejected(options.into());
            }
        }
    }
}
