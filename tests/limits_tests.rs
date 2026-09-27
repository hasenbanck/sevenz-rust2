use std::{fs::File, path::PathBuf};

#[cfg(feature = "compress")]
use std::io::Cursor;

use sevenz_rust2::{Archive, ArchiveReader, ArchiveReaderLimits, BlockDecoder, Error, Password};

#[cfg(feature = "ppmd")]
use sevenz_rust2::encoder_options::PpmdOptions;
#[cfg(feature = "compress")]
use sevenz_rust2::{ArchiveEntry, ArchiveWriter, EncoderConfiguration, EncoderMethod};

fn fixture(name: &str) -> File {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/resources");
    path.push(name);
    File::open(path).unwrap()
}

fn required_limit<T>(result: Result<T, Error>, resource: &'static str) -> usize {
    match result {
        Err(Error::ResourceLimit {
            resource: actual,
            required,
            ..
        }) if actual == resource => required,
        Err(error) => panic!("expected {resource} resource limit, got {error:?}"),
        Ok(_) => panic!("expected {resource} resource limit"),
    }
}

fn expect_limit<T>(result: Result<T, Error>, resource: &'static str) {
    let _ = required_limit(result, resource);
}

fn read_fixture(name: &str, limits: ArchiveReaderLimits) -> Result<Archive, Error> {
    let mut source = fixture(name);
    Archive::read_with_limits(&mut source, &Password::empty(), limits)
}

#[test]
fn raw_next_header_accepts_exact_limit_and_rejects_one_less_before_allocation() {
    let mut limits = ArchiveReaderLimits::permissive();
    limits.max_next_header_bytes = 0;
    let required = required_limit(read_fixture("copy.7z", limits), "next_header_bytes");
    assert!(required > 0);

    limits.max_next_header_bytes = required;
    read_fixture("copy.7z", limits).unwrap();

    limits.max_next_header_bytes = required - 1;
    expect_limit(read_fixture("copy.7z", limits), "next_header_bytes");
}

#[test]
fn encoded_header_declared_size_is_rejected_before_decoder_setup() {
    let mut limits = ArchiveReaderLimits::permissive();
    limits.max_decoded_header_bytes = 0;
    limits.max_decoder_memory_kib = 0;
    let required = required_limit(read_fixture("solid.7z", limits), "decoded_header_bytes");
    assert!(required > 0);

    limits.max_decoded_header_bytes = required;
    limits.max_decoder_memory_kib = usize::MAX;
    read_fixture("solid.7z", limits).unwrap();

    limits.max_decoded_header_bytes = required - 1;
    expect_limit(read_fixture("solid.7z", limits), "decoded_header_bytes");
}

#[test]
fn lzma_dictionary_accepts_exact_limit_and_rejects_one_less() {
    let mut limits = ArchiveReaderLimits::permissive();
    limits.max_dictionary_bytes = 0;
    let source = fixture("single_file_with_content_lzma.7z");
    let mut reader = ArchiveReader::new_with_limits(source, Password::empty(), limits).unwrap();
    let required = required_limit(reader.read_file("file.txt"), "lzma_dictionary_bytes");
    assert!(required > 0);

    limits.max_dictionary_bytes = required.try_into().unwrap();
    let source = fixture("single_file_with_content_lzma.7z");
    let mut reader = ArchiveReader::new_with_limits(source, Password::empty(), limits).unwrap();
    reader.read_file("file.txt").unwrap();

    limits.max_dictionary_bytes = (required - 1).try_into().unwrap();
    let source = fixture("single_file_with_content_lzma.7z");
    let mut reader = ArchiveReader::new_with_limits(source, Password::empty(), limits).unwrap();
    expect_limit(reader.read_file("file.txt"), "lzma_dictionary_bytes");
}

#[test]
fn lzma_zero_memory_limit_rejects_rounded_minimum_dictionary_estimate() {
    let mut limits = ArchiveReaderLimits::permissive();
    limits.max_decoder_memory_kib = 0;
    let source = fixture("single_file_with_content_lzma.7z");
    let mut reader = ArchiveReader::new_with_limits(source, Password::empty(), limits).unwrap();
    assert_eq!(
        required_limit(reader.read_file("file.txt"), "decoder_memory_kib"),
        1,
        "the initial reader wrapper must be charged before allocation"
    );

    limits.max_decoder_memory_kib = 2;
    let source = fixture("single_file_with_content_lzma.7z");
    let mut reader = ArchiveReader::new_with_limits(source, Password::empty(), limits).unwrap();
    let required = required_limit(reader.read_file("file.txt"), "decoder_memory_kib");
    assert!(
        required >= 2 + 4 + 64,
        "LZMA must account for its rounded minimum dictionary and its input buffer"
    );

    limits.max_decoder_memory_kib = required + 1;
    let source = fixture("single_file_with_content_lzma.7z");
    let mut reader = ArchiveReader::new_with_limits(source, Password::empty(), limits).unwrap();
    reader.read_file("file.txt").unwrap();
}

#[test]
fn real_lzma2_entry_obeys_decoder_memory_cap() {
    let mut limits = ArchiveReaderLimits::permissive();
    limits.max_decoder_memory_kib = 0;
    let source = fixture("decompress_example_lzma2_bcj_x86.7z");
    let mut reader = ArchiveReader::new_with_limits(source, Password::empty(), limits).unwrap();
    expect_limit(reader.read_file("decompress.exe"), "decoder_memory_kib");
}

#[test]
fn from_archive_and_block_decoder_inherit_the_read_limits() {
    let mut limits = ArchiveReaderLimits::permissive();
    limits.max_decoder_memory_kib = 0;
    let mut source = fixture("decompress_example_lzma2_bcj_x86.7z");
    let archive = Archive::read_with_limits(&mut source, &Password::empty(), limits).unwrap();
    let password = Password::empty();
    let result = BlockDecoder::new(1, 0, &archive, &password, &mut source).for_each_entries(
        &mut |_, reader| {
            std::io::copy(reader, &mut std::io::sink())?;
            Ok(true)
        },
    );
    expect_limit(result, "decoder_memory_kib");

    let mut reader = ArchiveReader::from_archive(archive, source, password);
    expect_limit(reader.read_file("decompress.exe"), "decoder_memory_kib");
}

fn decode_bcj2_block(name: &str, limits: ArchiveReaderLimits) -> Result<bool, Error> {
    let mut source = fixture(name);
    let archive = Archive::read(&mut source, &Password::empty()).unwrap();
    let block = archive
        .blocks
        .iter()
        .position(|block| {
            block
                .coders
                .iter()
                .any(|coder| coder.encoder_method_id() == sevenz_rust2::EncoderMethod::ID_BCJ2)
        })
        .expect("the fixture has a BCJ2 block");
    let password = Password::empty();
    BlockDecoder::new_with_limits(1, block, &archive, &password, limits, &mut source)
        .for_each_entries(&mut |_, reader| {
            std::io::copy(reader, &mut std::io::sink())?;
            Ok(true)
        })
}

#[test]
fn bcj2_graph_charges_its_stream_buffers() {
    let name = "7za433_7zip_lzma2_bcj2.7z";
    let mut limits = ArchiveReaderLimits::permissive();
    let mut minimum = 0usize;
    for _ in 0..64 {
        limits.max_decoder_memory_kib = minimum;
        match decode_bcj2_block(name, limits) {
            Ok(_) => break,
            Err(Error::ResourceLimit {
                resource: "decoder_memory_kib",
                required,
                ..
            }) if required > minimum => minimum = required,
            Err(error) => panic!("could not derive decoder memory threshold: {error:?}"),
        }
    }

    assert!(minimum > 1 + 1024, "BCJ2 buffers must be charged");
    limits.max_decoder_memory_kib = minimum;
    decode_bcj2_block(name, limits).unwrap();

    limits.max_decoder_memory_kib = minimum - 1;
    expect_limit(decode_bcj2_block(name, limits), "decoder_memory_kib");
}

#[test]
fn file_count_accepts_exact_limit_and_rejects_one_less() {
    let mut limits = ArchiveReaderLimits::permissive();
    limits.max_files = 1;
    read_fixture("single_empty_file.7z", limits).unwrap();

    limits.max_files = 0;
    expect_limit(read_fixture("single_empty_file.7z", limits), "file_count");
}

#[cfg(feature = "compress")]
fn generated_archive(methods: Vec<EncoderConfiguration>, entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new())).unwrap();
    writer.set_encrypt_header(false);
    writer.set_content_methods(methods);
    for (name, content) in entries {
        writer
            .push_archive_entry(ArchiveEntry::new_file(name), Some(*content))
            .unwrap();
    }
    writer.finish().unwrap().into_inner()
}

#[cfg(feature = "compress")]
fn read_generated(
    bytes: Vec<u8>,
    limits: ArchiveReaderLimits,
) -> Result<ArchiveReader<Cursor<Vec<u8>>>, Error> {
    ArchiveReader::new_with_limits(Cursor::new(bytes), Password::empty(), limits)
}

#[cfg(feature = "compress")]
#[test]
fn read_file_accepts_exact_entry_size_and_rejects_one_less_before_decoding() {
    let payload = b"small payload";
    let bytes = generated_archive(vec![EncoderMethod::LZMA.into()], &[("data", payload)]);
    let mut limits = ArchiveReaderLimits::permissive();
    limits.max_read_file_bytes = payload.len();
    let mut reader = read_generated(bytes.clone(), limits).unwrap();
    assert_eq!(reader.read_file("data").unwrap(), payload);

    limits.max_read_file_bytes = payload.len() - 1;
    limits.max_decoder_memory_kib = 0;
    let mut reader = read_generated(bytes, limits).unwrap();
    let required = required_limit(reader.read_file("data"), "read_file_bytes");
    assert_eq!(required, payload.len());
}

#[cfg(feature = "compress")]
fn minimum_decoder_memory_for(bytes: &[u8], name: &str) -> usize {
    let mut limit = 0usize;
    for _ in 0..256 {
        let mut limits = ArchiveReaderLimits::permissive();
        limits.max_decoder_memory_kib = limit;
        let mut reader = read_generated(bytes.to_vec(), limits).unwrap();
        match reader.read_file(name) {
            Ok(_) => return limit,
            Err(Error::ResourceLimit {
                resource: "decoder_memory_kib",
                required,
                ..
            }) if required > limit => limit = required,
            Err(Error::ResourceLimit {
                resource: "decoder_memory_kib",
                ..
            }) => panic!("decoder threshold did not advance"),
            Err(error) => panic!("could not derive decoder memory threshold: {error:?}"),
        }
    }
    panic!("too many decoder allocations while deriving threshold")
}

#[cfg(feature = "compress")]
#[test]
fn decoder_graph_charges_cumulative_lzma_estimates() {
    let entry = [("data", b"small payload".as_slice())];
    let single = generated_archive(vec![EncoderMethod::LZMA.into()], &entry);
    let chained = generated_archive(
        vec![EncoderMethod::LZMA.into(), EncoderMethod::LZMA.into()],
        &entry,
    );

    let one_decoder = minimum_decoder_memory_for(&single, "data");

    let mut limits = ArchiveReaderLimits::permissive();
    limits.max_decoder_memory_kib = one_decoder;
    let mut reader = read_generated(single, limits).unwrap();
    reader.read_file("data").unwrap();

    let mut reader = read_generated(chained, limits).unwrap();
    let cumulative = required_limit(reader.read_file("data"), "decoder_memory_kib");
    assert!(cumulative > one_decoder);
}

#[cfg(feature = "compress")]
#[test]
fn finite_policy_bounds_simple_copy_coder_stack() {
    let entry = [("data", b"small payload".as_slice())];
    let small = generated_archive(
        vec![EncoderMethod::COPY.into(), EncoderMethod::COPY.into()],
        &entry,
    );
    // The writer nests one wrapper per coder, so 33 coders need a large stack.
    let oversized = std::thread::Builder::new()
        .stack_size(16 << 20)
        .spawn(|| {
            generated_archive(
                (0..33).map(|_| EncoderMethod::COPY.into()).collect(),
                &[("data", b"small payload")],
            )
        })
        .unwrap()
        .join()
        .unwrap();
    let mut limits = ArchiveReaderLimits::permissive();
    limits.max_decoder_memory_kib = 16 << 10;

    let mut reader = read_generated(small, limits).unwrap();
    reader.read_file("data").unwrap();

    let mut reader = read_generated(oversized, limits).unwrap();
    expect_limit(reader.read_file("data"), "coder_count");
}

#[cfg(feature = "compress")]
fn minimum_metadata_for(bytes: &[u8]) -> usize {
    let mut low = 0usize;
    let mut high = 1 << 20;
    while low < high {
        let middle = low + (high - low) / 2;
        let mut limits = ArchiveReaderLimits::permissive();
        limits.max_metadata_bytes = middle;
        if read_generated(bytes.to_vec(), limits).is_ok() {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    low
}

#[cfg(feature = "compress")]
fn required_metadata_for_resource(bytes: &[u8], target: &'static str) -> usize {
    let mut limit = 0usize;
    for _ in 0..256 {
        let mut limits = ArchiveReaderLimits::permissive();
        limits.max_metadata_bytes = limit;
        match read_generated(bytes.to_vec(), limits) {
            Err(Error::ResourceLimit {
                resource, required, ..
            }) if resource == target => return required,
            Err(Error::ResourceLimit { required, .. }) if required > limit => limit = required,
            Err(error) => panic!("could not reach {target} metadata charge: {error:?}"),
            Ok(_) => panic!("archive succeeded before reaching {target} metadata charge"),
        }
    }
    panic!("too many metadata allocations before {target}")
}

#[cfg(feature = "compress")]
#[test]
fn unicode_names_are_charged_before_growth_and_bounded_lookup_allocates_nothing() {
    let short = generated_archive(
        vec![EncoderMethod::COPY.into()],
        &[("a", b"1"), ("b", b"2")],
    );
    let unicode_name = "🙂🙂🙂🙂🙂🙂🙂🙂.txt";
    let unicode = generated_archive(
        vec![EncoderMethod::COPY.into()],
        &[(unicode_name, b"1"), ("b", b"2")],
    );
    let short_minimum = minimum_metadata_for(&short);
    let unicode_minimum = minimum_metadata_for(&unicode);
    assert!(unicode_minimum > short_minimum);
    let name_required = required_metadata_for_resource(&unicode, "file_name_strings");
    assert!(name_required > 0);

    let mut limits = ArchiveReaderLimits::permissive();
    limits.max_metadata_bytes = name_required - 1;
    expect_limit(read_generated(unicode.clone(), limits), "file_name_strings");

    limits.max_metadata_bytes = unicode_minimum;
    let mut reader = read_generated(unicode, limits).unwrap();
    assert_eq!(reader.read_file(unicode_name).unwrap(), b"1");
}

#[cfg(feature = "compress")]
#[test]
fn finite_decoder_memory_policy_refuses_multithreaded_setup() {
    let bytes = generated_archive(
        vec![EncoderMethod::LZMA2.into()],
        &[("data", b"small payload")],
    );
    let mut limits = ArchiveReaderLimits::permissive();
    limits.max_decoder_memory_kib = usize::MAX / 2;
    let mut reader = read_generated(bytes, limits).unwrap();
    reader.set_thread_count(2);
    expect_limit(reader.read_file("data"), "decoder_threads");
}

#[cfg(all(feature = "compress", feature = "bzip2"))]
#[test]
fn finite_policy_refuses_codec_without_defensible_memory_estimate() {
    let bytes = generated_archive(
        vec![EncoderMethod::BZIP2.into()],
        &[("data", b"small payload")],
    );
    let mut limits = ArchiveReaderLimits::permissive();
    limits.max_decoder_memory_kib = usize::MAX / 2;
    let mut reader = read_generated(bytes.clone(), limits).unwrap();
    expect_limit(reader.read_file("data"), "decoder_memory_unbounded_method");

    let mut reader = read_generated(bytes, ArchiveReaderLimits::permissive()).unwrap();
    reader.read_file("data").unwrap();
}

#[cfg(all(feature = "compress", feature = "ppmd"))]
#[test]
fn ppmd_memory_bytes_are_converted_to_kib_before_comparison() {
    let method: EncoderConfiguration = PpmdOptions::from_order_memory_size(6, 1 << 20).into();
    let bytes = generated_archive(vec![method], &[("data", b"small payload")]);
    let mut limits = ArchiveReaderLimits::permissive();
    limits.max_decoder_memory_kib = 2;
    let mut reader = read_generated(bytes.clone(), limits).unwrap();
    let required = required_limit(reader.read_file("data"), "decoder_memory_kib");
    assert_eq!(required, 1046);

    limits.max_decoder_memory_kib = required + 1;
    let mut reader = read_generated(bytes, limits).unwrap();
    reader.read_file("data").unwrap();
}
