//! Regression tests for archive sizes above 4 GiB, which wrap on 32-bit targets.

#![cfg(feature = "compress")]

use std::io::{self, Cursor, Read, Seek, SeekFrom};

use sevenz_rust2::{
    Archive, ArchiveEntry, ArchiveReader, ArchiveWriter, BlockDecoder, EncoderMethod, Password,
};

const SEVEN_Z_SIGNATURE: [u8; 6] = [0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C];

const K_END: u8 = 0x00;
const K_HEADER: u8 = 0x01;
const K_MAIN_STREAMS_INFO: u8 = 0x04;
const K_FILES_INFO: u8 = 0x05;
const K_PACK_INFO: u8 = 0x06;
const K_UNPACK_INFO: u8 = 0x07;
const K_SUB_STREAMS_INFO: u8 = 0x08;
const K_SIZE: u8 = 0x09;
const K_CRC: u8 = 0x0A;
const K_FOLDER: u8 = 0x0B;
const K_CODERS_UNPACK_SIZE: u8 = 0x0C;
const K_NUM_UNPACK_STREAM: u8 = 0x0D;
const K_ENCODED_HEADER: u8 = 0x17;

const MIB: u64 = 1 << 20;

/// Wraps to 1 MiB when truncated to 32 bits.
const LARGE_SIZE: u64 = (1 << 32) + MIB;

const READ_LEN: usize = 2 << 20;

fn write_number(out: &mut Vec<u8>, value: u64) {
    let mut first: u8 = 0;
    let mut mask: u8 = 0x80;
    let mut low = Vec::new();
    let mut i = 0u32;
    while i < 8 {
        if value < (1u64 << (7 * (i + 1))) {
            first |= (value >> (8 * i)) as u8;
            break;
        }
        first |= mask;
        mask >>= 1;
        low.push((value >> (8 * i)) as u8);
        i += 1;
    }
    out.push(first);
    out.extend_from_slice(&low);
}

fn simple_folder(coder_id: &[u8], properties: &[u8]) -> Vec<u8> {
    let mut f = Vec::new();
    write_number(&mut f, 1); // num_coders
    let mut flags = coder_id.len() as u8;
    if !properties.is_empty() {
        flags |= 0x20;
    }
    f.push(flags);
    f.extend_from_slice(coder_id);
    if !properties.is_empty() {
        write_number(&mut f, properties.len() as u64);
        f.extend_from_slice(properties);
    }
    f
}

fn streams_info(pack_sizes: &[u64], folder: &[u8], unpack_size: u64) -> Vec<u8> {
    let mut h = vec![K_PACK_INFO];
    write_number(&mut h, 0); // pack_pos
    write_number(&mut h, pack_sizes.len() as u64);
    h.push(K_SIZE);
    for size in pack_sizes {
        write_number(&mut h, *size);
    }
    h.push(K_END);

    h.push(K_UNPACK_INFO);
    h.push(K_FOLDER);
    write_number(&mut h, 1); // num_blocks
    h.push(0); // external
    h.extend_from_slice(folder);
    h.push(K_CODERS_UNPACK_SIZE);
    write_number(&mut h, unpack_size);
    h.push(K_END);
    h
}

fn header(
    pack_sizes: &[u64],
    folder: &[u8],
    unpack_size: u64,
    file_sizes: &[u64],
    file_crc: Option<u32>,
) -> Vec<u8> {
    let mut h = vec![K_HEADER, K_MAIN_STREAMS_INFO];
    h.extend_from_slice(&streams_info(pack_sizes, folder, unpack_size));

    h.push(K_SUB_STREAMS_INFO);
    if file_sizes.len() > 1 {
        h.push(K_NUM_UNPACK_STREAM);
        write_number(&mut h, file_sizes.len() as u64);
        h.push(K_SIZE);
        for size in &file_sizes[..file_sizes.len() - 1] {
            write_number(&mut h, *size);
        }
    }
    if let Some(crc) = file_crc {
        h.push(K_CRC);
        h.push(1); // all defined
        h.extend_from_slice(&crc.to_le_bytes());
    }
    h.push(K_END);
    h.push(K_END); // main streams info

    h.push(K_FILES_INFO);
    write_number(&mut h, file_sizes.len() as u64);
    h.push(K_END);
    h.push(K_END); // header
    h
}

fn signature_header(pack_len: u64, next_header: &[u8]) -> Vec<u8> {
    let mut start = [0u8; 20];
    start[0..8].copy_from_slice(&pack_len.to_le_bytes());
    start[8..16].copy_from_slice(&(next_header.len() as u64).to_le_bytes());
    start[16..20].copy_from_slice(&crc32fast::hash(next_header).to_le_bytes());

    let mut out = Vec::new();
    out.extend_from_slice(&SEVEN_Z_SIGNATURE);
    out.extend_from_slice(&[0x00, 0x04]);
    out.extend_from_slice(&crc32fast::hash(&start).to_le_bytes());
    out.extend_from_slice(&start);
    out
}

struct SparseSource {
    prefix: Vec<u8>,
    zeros: u64,
    suffix: Vec<u8>,
    pos: u64,
}

impl SparseSource {
    fn len(&self) -> u64 {
        self.prefix.len() as u64 + self.zeros + self.suffix.len() as u64
    }
}

impl Read for SparseSource {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let prefix_len = self.prefix.len() as u64;
        let suffix_start = prefix_len + self.zeros;
        if self.pos >= self.len() || buf.is_empty() {
            return Ok(0);
        }
        let n = if self.pos < prefix_len {
            let start = self.pos as usize;
            let n = buf.len().min(self.prefix.len() - start);
            buf[..n].copy_from_slice(&self.prefix[start..start + n]);
            n
        } else if self.pos < suffix_start {
            let n = buf
                .len()
                .min(usize::try_from(suffix_start - self.pos).unwrap_or(usize::MAX));
            buf[..n].fill(0);
            n
        } else {
            let start = (self.pos - suffix_start) as usize;
            let n = buf.len().min(self.suffix.len() - start);
            buf[..n].copy_from_slice(&self.suffix[start..start + n]);
            n
        };
        self.pos += n as u64;
        Ok(n)
    }
}

impl Seek for SparseSource {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(p) => Some(p),
            SeekFrom::End(d) => self.len().checked_add_signed(d),
            SeekFrom::Current(d) => self.pos.checked_add_signed(d),
        };
        self.pos =
            new_pos.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid seek"))?;
        Ok(self.pos)
    }
}

fn read_first_entry(make_source: impl Fn() -> SparseSource) -> (Vec<u8>, Vec<u8>) {
    let read_prefix = |reader: &mut dyn Read| -> Result<Vec<u8>, sevenz_rust2::Error> {
        let mut data = vec![0xFF; READ_LEN];
        reader.read_exact(&mut data)?;
        Ok(data)
    };

    let mut via_reader = None;
    let mut reader = ArchiveReader::new(make_source(), Password::empty()).unwrap();
    reader
        .for_each_entries(|_, r| {
            via_reader = Some(read_prefix(r)?);
            Ok(false)
        })
        .expect("ArchiveReader must decode past the wrapped size");

    let mut via_block = None;
    let mut source = make_source();
    let archive = Archive::read(&mut source, &Password::empty()).unwrap();
    BlockDecoder::new(1, 0, &archive, &Password::empty(), &mut source)
        .for_each_entries(&mut |_, r| {
            via_block = Some(read_prefix(r)?);
            Ok(false)
        })
        .expect("BlockDecoder must decode past the wrapped size");

    (via_reader.unwrap(), via_block.unwrap())
}

#[test]
fn copy_block_larger_than_4_gib_decodes_past_wrapped_size() {
    let make_source = || {
        let next_header = header(
            &[LARGE_SIZE],
            &simple_folder(EncoderMethod::ID_COPY, &[]),
            LARGE_SIZE,
            &[LARGE_SIZE],
            Some(0),
        );
        SparseSource {
            prefix: signature_header(LARGE_SIZE, &next_header),
            zeros: LARGE_SIZE,
            suffix: next_header,
            pos: 0,
        }
    };

    let (via_reader, via_block) = read_first_entry(make_source);
    assert!(via_reader.iter().all(|b| *b == 0));
    assert!(via_block.iter().all(|b| *b == 0));
}

#[test]
fn lzma_block_larger_than_4_gib_decodes_past_wrapped_size() {
    let content = vec![0x5Au8; READ_LEN];

    let mut bytes = Vec::new();
    {
        let mut writer = ArchiveWriter::new(Cursor::new(&mut bytes)).unwrap();
        writer.set_content_methods(vec![EncoderMethod::LZMA.into()]);
        writer
            .push_archive_entry(ArchiveEntry::new_file("data"), Some(content.as_slice()))
            .unwrap();
        writer.finish().unwrap();
    }
    let archive = Archive::read(&mut Cursor::new(bytes.as_slice()), &Password::empty()).unwrap();
    let pack_start = 32 + archive.pack_pos() as usize;
    let packed = bytes[pack_start..pack_start + archive.pack_sizes()[0] as usize].to_vec();
    let properties = archive.blocks[0].coders[0].properties().to_vec();

    let make_source = || {
        let next_header = header(
            &[packed.len() as u64],
            &simple_folder(EncoderMethod::ID_LZMA, &properties),
            LARGE_SIZE,
            &[READ_LEN as u64, LARGE_SIZE - READ_LEN as u64],
            None,
        );
        let mut prefix = signature_header(packed.len() as u64, &next_header);
        prefix.extend_from_slice(&packed);
        SparseSource {
            prefix,
            zeros: 0,
            suffix: next_header,
            pos: 0,
        }
    };

    let (via_reader, via_block) = read_first_entry(make_source);
    assert_eq!(via_reader, content);
    assert_eq!(via_block, content);
}

#[test]
fn bcj2_block_larger_than_4_gib_decodes_past_wrapped_size() {
    let mut folder = Vec::new();
    write_number(&mut folder, 1); // num_coders
    folder.push(0x10 | EncoderMethod::ID_BCJ2.len() as u8);
    folder.extend_from_slice(EncoderMethod::ID_BCJ2);
    write_number(&mut folder, 4); // num_in_streams
    write_number(&mut folder, 1); // num_out_streams
    for index in 0..4 {
        write_number(&mut folder, index); // packed stream index
    }
    let range_coder = [0u8; 5];

    let make_source = || {
        let pack_sizes = [LARGE_SIZE, 0, 0, range_coder.len() as u64];
        let next_header = header(
            &pack_sizes,
            &folder,
            LARGE_SIZE,
            &[READ_LEN as u64, LARGE_SIZE - READ_LEN as u64],
            None,
        );
        let mut suffix = range_coder.to_vec();
        suffix.extend_from_slice(&next_header);
        SparseSource {
            prefix: signature_header(pack_sizes.iter().sum(), &next_header),
            zeros: LARGE_SIZE,
            suffix,
            pos: 0,
        }
    };

    let (via_reader, via_block) = read_first_entry(make_source);
    assert!(via_reader.iter().all(|b| *b == 0));
    assert!(via_block.iter().all(|b| *b == 0));
}

#[test]
fn encoded_header_larger_than_4_gib_is_not_truncated() {
    let plain_header = [K_HEADER, K_END];
    let pack_size = plain_header.len() as u64;

    let mut next_header = vec![K_ENCODED_HEADER];
    next_header.extend_from_slice(&streams_info(
        &[pack_size],
        &simple_folder(EncoderMethod::ID_COPY, &[]),
        (1 << 32) + pack_size,
    ));
    next_header.push(K_END);

    let mut bytes = signature_header(pack_size, &next_header);
    bytes.extend_from_slice(&plain_header);
    bytes.extend_from_slice(&next_header);

    assert!(Archive::read(&mut Cursor::new(bytes), &Password::empty()).is_err());
}
