use std::{
    cell::RefCell,
    collections::HashMap,
    fs::File,
    io,
    io::{Read, Seek, SeekFrom},
    num::NonZeroUsize,
    rc::Rc,
};

use crc32fast::Hasher;
use lzma_rust2::filter::bcj2::Bcj2Reader;

use crate::{
    ByteReader, Password,
    archive::*,
    bitset::BitSet,
    block::*,
    decoder::{BCJ2_BUFFER_KIB, DecoderMemoryBudget, WRAPPER_MEMORY_KIB, add_decoder},
    error::Error,
};

const MAX_MEM_LIMIT_KB: usize = usize::MAX / 1024;
const MAX_CODER_COUNT: usize = 32;

/// Resource limits for reading untrusted archives.
///
/// An exceeded limit is reported as [`Error::ResourceLimit`] before the allocation is made.
/// The default is [`ArchiveReaderLimits::permissive`].
#[derive(Debug, Clone, Copy)]
pub struct ArchiveReaderLimits {
    /// Maximum size of the raw header in bytes.
    pub max_next_header_bytes: usize,
    /// Maximum size of the decoded header in bytes.
    pub max_decoded_header_bytes: usize,
    /// Maximum LZMA and LZMA2 dictionary size in bytes.
    pub max_dictionary_bytes: u32,
    /// Maximum estimated decoder memory of one block in KiB. When set, decoding is
    /// single-threaded and codecs without an estimate (Brotli, BZip2, Deflate, LZ4, Zstd) fail.
    pub max_decoder_memory_kib: usize,
    /// Maximum estimated memory of the parsed metadata (entries, names, stream info) in bytes.
    pub max_metadata_bytes: usize,
    /// Maximum number of entries.
    pub max_files: usize,
    /// Maximum entry size in bytes that [`ArchiveReader::read_file`] reads into memory.
    pub max_read_file_bytes: usize,
}

impl ArchiveReaderLimits {
    /// Suggested limits for untrusted archives.
    #[must_use]
    pub const fn untrusted_default() -> Self {
        Self {
            max_next_header_bytes: 32 << 20,
            max_decoded_header_bytes: 32 << 20,
            max_dictionary_bytes: 256 << 20,
            max_decoder_memory_kib: 512 << 10,
            max_metadata_bytes: 64 << 20,
            max_files: 10_000,
            max_read_file_bytes: 256 << 20,
        }
    }

    /// No limits, as used by [`ArchiveReader::new`] and [`Archive::read`].
    #[must_use]
    pub const fn permissive() -> Self {
        Self {
            max_next_header_bytes: usize::MAX,
            max_decoded_header_bytes: usize::MAX,
            max_dictionary_bytes: u32::MAX,
            max_decoder_memory_kib: MAX_MEM_LIMIT_KB,
            max_metadata_bytes: usize::MAX,
            max_files: usize::MAX,
            max_read_file_bytes: usize::MAX,
        }
    }

    pub(crate) const fn decoder_memory_is_bounded(&self) -> bool {
        self.max_decoder_memory_kib != MAX_MEM_LIMIT_KB
    }
}

impl Default for ArchiveReaderLimits {
    fn default() -> Self {
        Self::permissive()
    }
}

struct MetadataBudget {
    used: usize,
    limit: usize,
}

impl MetadataBudget {
    const fn new(limit: usize) -> Self {
        Self { used: 0, limit }
    }

    const fn with_used(limit: usize, used: usize) -> Self {
        Self { used, limit }
    }

    fn charge(
        &mut self,
        count: usize,
        bytes_per_item: usize,
        resource: &'static str,
    ) -> Result<(), Error> {
        // Assume one allocator header per non-empty allocation.
        let allocation_overhead = if count == 0 {
            0
        } else {
            2 * std::mem::size_of::<usize>()
        };
        let required = count
            .checked_mul(bytes_per_item)
            .and_then(|bytes| bytes.checked_add(allocation_overhead))
            .and_then(|bytes| self.used.checked_add(bytes))
            .ok_or(Error::ResourceLimit {
                resource,
                limit: self.limit,
                required: usize::MAX,
            })?;
        if required > self.limit {
            return Err(Error::ResourceLimit {
                resource,
                limit: self.limit,
                required,
            });
        }
        self.used = required;
        Ok(())
    }
}

struct EncodedHeaderContext<'a> {
    password: &'a Password,
    limit: usize,
    thread_count: u32,
    limits: ArchiveReaderLimits,
    metadata: &'a mut MetadataBudget,
}

struct DecoderGraph<'a, 'r, R: Read + Seek> {
    sources: &'a [SharedBoundedReader<'r, R>],
    coder_to_stream_map: &'a [usize],
    password: &'a Password,
    limits: &'a ArchiveReaderLimits,
    metadata: &'a mut MetadataBudget,
    decoder_memory: &'a mut DecoderMemoryBudget,
    thread_count: u32,
}

fn reject_bounded_multithreading(
    limits: &ArchiveReaderLimits,
    thread_count: u32,
) -> Result<(), Error> {
    if limits.decoder_memory_is_bounded() && thread_count > 1 {
        return Err(Error::ResourceLimit {
            resource: "decoder_threads",
            limit: 1,
            required: thread_count as usize,
        });
    }
    Ok(())
}

fn reject_bounded_coder_count(
    limits: &ArchiveReaderLimits,
    coder_count: usize,
) -> Result<(), Error> {
    if limits.decoder_memory_is_bounded() && coder_count > MAX_CODER_COUNT {
        return Err(Error::ResourceLimit {
            resource: "coder_count",
            limit: MAX_CODER_COUNT,
            required: coder_count,
        });
    }
    Ok(())
}

/// Upper bound for eagerly pre-allocating an output buffer from an archive-declared
/// (untrusted) uncompressed size. The buffer still grows to the real size as data is
/// read; this only stops a tiny archive from forcing a huge up-front allocation.
const MAX_PREALLOC_BYTES: usize = 4 << 20;
const DECODED_HEADER_INITIAL_BYTES: usize = 64 << 10;

/// Reads up to `declared_len` bytes, never reserving more than that.
fn read_decoded_header<R: Read>(reader: &mut R, declared_len: usize) -> io::Result<Vec<u8>> {
    let mut decoded = Vec::new();
    let mut filled = 0;
    while filled < declared_len {
        if filled == decoded.len() {
            let next_len =
                declared_len.min(DECODED_HEADER_INITIAL_BYTES.max(decoded.len().saturating_mul(2)));
            let reserve = next_len - decoded.len();
            // `reserve` may over-allocate past `declared_len`.
            decoded
                .try_reserve_exact(reserve)
                .map_err(|_| io::Error::other("decoded header allocation failed"))?;
            decoded.resize(next_len, 0);
        }
        match reader.read(&mut decoded[filled..]) {
            Ok(0) => break,
            Ok(read) => filled += read,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error),
        }
    }
    decoded.truncate(filled);
    Ok(decoded)
}

pub struct BoundedReader<R: Read> {
    inner: R,
    remain: usize,
}

impl<R: Read> BoundedReader<R> {
    pub fn new(inner: R, max_size: usize) -> Self {
        Self {
            inner,
            remain: max_size,
        }
    }
}

impl<R: Read> Read for BoundedReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.remain == 0 {
            return Ok(0);
        }
        let bound = buf.len().min(self.remain);
        let size = self.inner.read(&mut buf[..bound])?;
        self.remain -= size;
        Ok(size)
    }
}

/// A special reader that shares it's inner reader with other instances and
/// needs to re-seek every read operation.
#[derive(Debug)]
pub(crate) struct SharedBoundedReader<'a, R> {
    inner: Rc<RefCell<&'a mut R>>,
    cur: u64,
    bounds: (u64, u64),
}

impl<'a, R> Clone for SharedBoundedReader<'a, R> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
            cur: self.cur,
            bounds: self.bounds,
        }
    }
}

impl<'a, R: Read + Seek> Seek for SharedBoundedReader<'a, R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(pos) => self.bounds.0 as i64 + pos as i64,
            SeekFrom::End(pos) => self.bounds.1 as i64 + pos,
            SeekFrom::Current(pos) => self.cur as i64 + pos,
        };
        if new_pos < 0 {
            return Err(io::Error::other("SeekBeforeStart"));
        }
        self.cur = new_pos as u64;
        self.inner.borrow_mut().seek(SeekFrom::Start(self.cur))
    }
}

impl<'a, R: Read + Seek> Read for SharedBoundedReader<'a, R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.cur >= self.bounds.1 {
            return Ok(0);
        }

        let mut inner = self.inner.borrow_mut();

        inner.seek(SeekFrom::Start(self.cur))?;

        let bound = buf.len().min((self.bounds.1 - self.cur) as usize);
        let size = inner.read(&mut buf[..bound])?;
        self.cur += size as u64;
        Ok(size)
    }
}

impl<'a, R: Read + Seek> SharedBoundedReader<'a, R> {
    fn new(inner: Rc<RefCell<&'a mut R>>, bounds: (u64, u64)) -> Self {
        Self {
            inner,
            cur: bounds.0,
            bounds,
        }
    }
}

struct Crc32VerifyingReader<R> {
    inner: R,
    crc_digest: Hasher,
    expected_value: u64,
    remaining: i64,
}

impl<R: Read> Crc32VerifyingReader<R> {
    fn new(inner: R, remaining: usize, expected_value: u64) -> Self {
        Self {
            inner,
            crc_digest: Hasher::new(),
            expected_value,
            remaining: remaining as i64,
        }
    }
}

impl<R: Read> Read for Crc32VerifyingReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.remaining <= 0 {
            return Ok(0);
        }
        let size = self.inner.read(buf)?;
        if size > 0 {
            self.remaining -= size as i64;
            self.crc_digest.update(&buf[..size]);
        }
        if self.remaining <= 0 {
            let d = std::mem::replace(&mut self.crc_digest, Hasher::new()).finalize();
            if d as u64 != self.expected_value {
                return Err(std::io::Error::other(Error::ChecksumVerificationFailed));
            }
        }
        Ok(size)
    }
}

impl Archive {
    /// Open 7z file under specified `path`.
    #[inline]
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Archive, Error> {
        Self::open_with_password(path, &Password::empty())
    }

    /// Open an encrypted 7z file under specified `path` with `password`.
    ///
    /// # Parameters
    /// - `reader`   - the path to the 7z file
    /// - `password` - archive password encoded in utf16 little endian
    #[inline]
    pub fn open_with_password(
        path: impl AsRef<std::path::Path>,
        password: &Password,
    ) -> Result<Archive, Error> {
        let mut file = File::open(path)?;
        Self::read(&mut file, password)
    }

    /// Read 7z file archive info use the specified `reader`.
    ///
    /// # Parameters
    /// - `reader`   - the reader of the 7z filr archive
    /// - `password` - archive password encoded in utf16 little endian
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::{
    ///     fs::File,
    ///     io::{Read, Seek},
    /// };
    ///
    /// use sevenz_rust2::*;
    ///
    /// let mut reader = File::open("example.7z").unwrap();
    ///
    /// let password = Password::from("the password");
    /// let archive = Archive::read(&mut reader, &password).unwrap();
    ///
    /// for entry in &archive.files {
    ///     println!("{}", entry.name());
    /// }
    /// ```
    pub fn read<R: Read + Seek>(reader: &mut R, password: &Password) -> Result<Archive, Error> {
        Self::read_with_limits(reader, password, ArchiveReaderLimits::permissive())
    }

    /// Like [`Archive::read`], but enforces `limits`.
    pub fn read_with_limits<R: Read + Seek>(
        reader: &mut R,
        password: &Password,
        limits: ArchiveReaderLimits,
    ) -> Result<Archive, Error> {
        let reader_len = reader.seek(SeekFrom::End(0))?;
        reader.seek(SeekFrom::Start(0))?;

        let mut signature = [0; 6];
        reader.read_exact(&mut signature)?;
        if signature != SEVEN_Z_SIGNATURE {
            return Err(Error::BadSignature(signature));
        }
        let mut versions = [0; 2];
        reader.read_exact(&mut versions)?;
        let version_major = versions[0];
        let version_minor = versions[1];
        if version_major != 0 {
            return Err(Error::UnsupportedVersion {
                major: version_major,
                minor: version_minor,
            });
        }

        let start_header_crc = reader.read_u32()?;

        let header_valid = if start_header_crc == 0 {
            let current_position = reader.stream_position()?;
            let mut buf = [0; 20];
            reader.read_exact(&mut buf)?;
            reader.seek(SeekFrom::Start(current_position))?;
            buf.iter().any(|a| *a != 0)
        } else {
            true
        };
        if header_valid {
            let start_header = Self::read_start_header(reader, start_header_crc)?;
            Self::init_archive(reader, start_header, password, true, 1, limits)
        } else {
            Self::try_to_locale_end_header(reader, reader_len, password, 1, limits)
        }
    }

    fn read_start_header<R: Read>(
        reader: &mut R,
        start_header_crc: u32,
    ) -> Result<StartHeader, Error> {
        let mut buf = [0; 20];
        reader.read_exact(&mut buf)?;
        let crc32 = crc32fast::hash(&buf);
        if crc32 != start_header_crc {
            return Err(Error::ChecksumVerificationFailed);
        }
        let mut buf_read = buf.as_slice();
        let offset = buf_read.read_u64()?;

        let size = buf_read.read_u64()?;
        let crc = buf_read.read_u32()?;
        Ok(StartHeader {
            next_header_offset: offset,
            next_header_size: size,
            next_header_crc: crc as u64,
        })
    }

    fn read_header<R: Read + Seek>(
        header: &mut R,
        archive: &mut Archive,
        limit: usize,
        limits: ArchiveReaderLimits,
        metadata: &mut MetadataBudget,
    ) -> Result<(), Error> {
        let mut nid = header.read_u8()?;
        if nid == K_ARCHIVE_PROPERTIES {
            Self::read_archive_properties(header, limit)?;
            nid = header.read_u8()?;
        }

        if nid == K_ADDITIONAL_STREAMS_INFO {
            return Err(Error::other("Additional streams unsupported"));
        }
        if nid == K_MAIN_STREAMS_INFO {
            Self::read_streams_info(header, archive, limit, metadata)?;
            nid = header.read_u8()?;
        }
        if nid == K_FILES_INFO {
            Self::read_files_info(header, archive, limit, limits, metadata)?;
            nid = header.read_u8()?;
        }
        if nid != K_END {
            return Err(Error::BadTerminatedHeader(nid));
        }

        Ok(())
    }

    fn read_archive_properties<R: Read + Seek>(header: &mut R, limit: usize) -> Result<(), Error> {
        let mut nid = header.read_u8()?;
        while nid != K_END {
            // Bound the skip length against the buffer: an unbounded value cast to `i64`
            // could go negative and seek backwards, re-reading the same bytes forever.
            let property_size = bounded_count(read_variable_u64(header)?, limit, "propertySize")?;
            header.seek(SeekFrom::Current(property_size as i64))?;
            nid = header.read_u8()?;
        }
        Ok(())
    }

    fn try_to_locale_end_header<R: Read + Seek>(
        reader: &mut R,
        reader_len: u64,
        password: &Password,
        thread_count: u32,
        limits: ArchiveReaderLimits,
    ) -> Result<Self, Error> {
        let search_limit = 1024 * 1024;
        let prev_data_size = reader.stream_position()? + 20;
        let size = reader_len;
        let min_pos = if reader.stream_position()? + search_limit > size {
            reader.stream_position()?
        } else {
            size - search_limit
        };
        let mut pos = reader_len - 1;
        while pos > min_pos {
            pos -= 1;

            reader.seek(SeekFrom::Start(pos))?;
            let nid = reader.read_u8()?;
            if nid == K_ENCODED_HEADER || nid == K_HEADER {
                // `pos` scans down and can fall below `prev_data_size`; skip such candidates
                // instead of underflowing the subtraction.
                let Some(next_header_offset) = pos.checked_sub(prev_data_size) else {
                    continue;
                };
                let start_header = StartHeader {
                    next_header_offset,
                    next_header_size: reader_len - pos,
                    next_header_crc: 0,
                };
                let result = Self::init_archive(
                    reader,
                    start_header,
                    password,
                    false,
                    thread_count,
                    limits,
                )?;

                if !result.files.is_empty() {
                    return Ok(result);
                }
            }
        }
        Err(Error::other(
            "Start header corrupt and unable to guess end header",
        ))
    }

    fn init_archive<R: Read + Seek>(
        reader: &mut R,
        start_header: StartHeader,
        password: &Password,
        verify_crc: bool,
        thread_count: u32,
        limits: ArchiveReaderLimits,
    ) -> Result<Self, Error> {
        // Bound the declared next-header size against the actual file length before allocating.
        let reader_len = reader.seek(SeekFrom::End(0))?;
        if start_header.next_header_size > usize::MAX as u64
            || start_header.next_header_size > reader_len
        {
            return Err(Error::other(format!(
                "Cannot handle next_header_size {}",
                start_header.next_header_size
            )));
        }

        let next_header_size_int = start_header.next_header_size as usize;
        if next_header_size_int > limits.max_next_header_bytes {
            return Err(Error::ResourceLimit {
                resource: "next_header_bytes",
                limit: limits.max_next_header_bytes,
                required: next_header_size_int,
            });
        }

        // Bound the header position too: `next_header_offset` is an unbounded `u64`, so the
        // addition can overflow (a panic under overflow checks) and any value past the file
        // end is invalid anyway.
        let header_pos = SIGNATURE_HEADER_SIZE
            .checked_add(start_header.next_header_offset)
            .filter(|pos| *pos <= reader_len)
            .ok_or_else(|| Error::other("next header offset out of range"))?;
        reader.seek(SeekFrom::Start(header_pos))?;

        let mut buf = vec![0; next_header_size_int];
        reader.read_exact(&mut buf)?;
        if verify_crc && crc32fast::hash(&buf) as u64 != start_header.next_header_crc {
            return Err(Error::NextHeaderCrcMismatch);
        }

        let mut archive = Archive::default();
        let mut metadata = MetadataBudget::new(limits.max_metadata_bytes);
        let mut buf_reader = buf.as_slice();
        let mut nid = buf_reader.read_u8()?;
        let mut header = if nid == K_ENCODED_HEADER {
            let mut context = EncodedHeaderContext {
                password,
                limit: next_header_size_int,
                thread_count,
                limits,
                metadata: &mut metadata,
            };
            let (mut out_reader, buf_size) =
                Self::read_encoded_header(&mut buf_reader, reader, &mut archive, &mut context)?;
            let decoded = read_decoded_header(&mut out_reader, buf_size)
                .map_err(|e| Error::bad_password(e, !password.is_empty()))?;
            buf = decoded;
            if buf.len() != buf_size {
                return Err(Error::bad_password(
                    io::Error::from(io::ErrorKind::UnexpectedEof),
                    !password.is_empty(),
                ));
            }
            archive = Archive::default();
            buf_reader = buf.as_slice();
            nid = buf_reader.read_u8()?;
            buf_reader
        } else {
            buf_reader
        };
        // Upper bound for any header-declared count/size: it can never exceed the number
        // of bytes in the header buffer, since every counted element consumes at least
        // one header byte. This kills the "tiny file declares a huge count" OOM vector
        // without rejecting any legitimate archive.
        let header_len_bound = header.len();
        let mut header = std::io::Cursor::new(&mut header);
        if nid == K_HEADER {
            Self::read_header(
                &mut header,
                &mut archive,
                header_len_bound,
                limits,
                &mut metadata,
            )?;
        } else {
            return Err(Error::other("Broken or unsupported archive: no Header"));
        }

        archive.is_solid = archive
            .blocks
            .iter()
            .any(|block| block.num_unpack_sub_streams > 1);
        archive.metadata_bytes = metadata.used;
        archive.limits = limits;

        Ok(archive)
    }

    fn read_encoded_header<'r, R: Read, RI: 'r + Read + Seek>(
        header: &mut R,
        reader: &'r mut RI,
        archive: &mut Archive,
        context: &mut EncodedHeaderContext<'_>,
    ) -> Result<(Box<dyn Read + 'r>, usize), Error> {
        reject_bounded_multithreading(&context.limits, context.thread_count)?;
        Self::read_streams_info(header, archive, context.limit, context.metadata)?;
        let block = archive
            .blocks
            .first()
            .ok_or(Error::other("no blocks, can't read encoded header"))?;
        reject_bounded_coder_count(&context.limits, block.coders.len())?;
        let unpack_size =
            usize::try_from(block.get_unpack_size()).map_err(|_| Error::ResourceLimit {
                resource: "decoded_header_bytes",
                limit: context.limits.max_decoded_header_bytes,
                required: usize::MAX,
            })?;
        // Check before the codec allocates its dictionary.
        if unpack_size > context.limits.max_decoded_header_bytes {
            return Err(Error::ResourceLimit {
                resource: "decoded_header_bytes",
                limit: context.limits.max_decoded_header_bytes,
                required: unpack_size,
            });
        }
        let first_pack_stream_index = 0;
        let block_offset = SIGNATURE_HEADER_SIZE
            .checked_add(archive.pack_pos)
            .ok_or_else(|| Error::other("pack position out of range"))?;
        if archive.pack_sizes.is_empty() {
            return Err(Error::other("no packed streams, can't read encoded header"));
        }

        reader.seek(SeekFrom::Start(block_offset))?;
        let coder_len = block.coders.len();
        let pack_size = archive.pack_sizes[first_pack_stream_index] as usize;
        let mut decoder_memory = DecoderMemoryBudget::new(context.limits.max_decoder_memory_kib);
        decoder_memory.charge(WRAPPER_MEMORY_KIB)?;
        let input_reader = BoundedReader::new(reader, pack_size);
        let mut decoder: Box<dyn Read> = Box::new(input_reader);
        let mut decoder = if coder_len > 0 {
            for (index, coder) in block.ordered_coder_iter() {
                if coder.num_in_streams != 1 || coder.num_out_streams != 1 {
                    return Err(Error::other(
                        "Multi input/output stream coders are not yet supported",
                    ));
                }
                let next = add_decoder(
                    decoder,
                    block.get_unpack_size_at_index(index) as usize,
                    coder,
                    context.password,
                    &context.limits,
                    &mut decoder_memory,
                    context.thread_count,
                )?;
                decoder = Box::new(next);
            }
            decoder
        } else {
            decoder
        };
        if block.has_crc {
            decoder_memory.charge(WRAPPER_MEMORY_KIB)?;
            decoder = Box::new(Crc32VerifyingReader::new(decoder, unpack_size, block.crc));
        }

        Ok((decoder, unpack_size))
    }

    fn read_streams_info<R: Read>(
        header: &mut R,
        archive: &mut Archive,
        limit: usize,
        metadata: &mut MetadataBudget,
    ) -> Result<(), Error> {
        let mut nid = header.read_u8()?;
        if nid == K_PACK_INFO {
            Self::read_pack_info(header, archive, limit, metadata)?;
            nid = header.read_u8()?;
        }

        if nid == K_UNPACK_INFO {
            Self::read_unpack_info(header, archive, limit, metadata)?;
            nid = header.read_u8()?;
        } else {
            archive.blocks.clear();
        }
        if nid == K_SUB_STREAMS_INFO {
            Self::read_sub_streams_info(header, archive, limit, metadata)?;
            nid = header.read_u8()?;
        }
        if nid != K_END {
            return Err(Error::BadTerminatedStreamsInfo(nid));
        }

        Ok(())
    }

    fn read_files_info<R: Read + Seek>(
        header: &mut R,
        archive: &mut Archive,
        limit: usize,
        limits: ArchiveReaderLimits,
        metadata: &mut MetadataBudget,
    ) -> Result<(), Error> {
        let num_files = bounded_count(read_variable_u64(header)?, limit, "num files")?;
        if num_files > limits.max_files {
            return Err(Error::ResourceLimit {
                resource: "file_count",
                limit: limits.max_files,
                required: num_files,
            });
        }
        metadata.charge(
            num_files,
            std::mem::size_of::<ArchiveEntry>(),
            "file_metadata",
        )?;
        let mut files: Vec<ArchiveEntry> = vec![Default::default(); num_files];

        let mut is_empty_stream: Option<BitSet> = None;
        let mut is_empty_file: Option<BitSet> = None;
        let mut is_anti: Option<BitSet> = None;
        loop {
            let prop_type = header.read_u8()?;
            if prop_type == 0 {
                break;
            }
            let size = read_variable_u64(header)?;
            match prop_type {
                K_EMPTY_STREAM => {
                    is_empty_stream = Some(read_bits(header, num_files, metadata)?);
                }
                K_EMPTY_FILE => {
                    let n = if let Some(s) = &is_empty_stream {
                        s.len()
                    } else {
                        return Err(Error::other(
                            "Header format error: kEmptyStream must appear before kEmptyFile",
                        ));
                    };
                    is_empty_file = Some(read_bits(header, n, metadata)?);
                }
                K_ANTI => {
                    let n = if let Some(s) = is_empty_stream.as_ref() {
                        s.len()
                    } else {
                        return Err(Error::other(
                            "Header format error: kEmptyStream must appear before kEmptyFile",
                        ));
                    };
                    is_anti = Some(read_bits(header, n, metadata)?);
                }
                K_NAME => {
                    let external = header.read_u8()?;
                    if external != 0 {
                        return Err(Error::other("Not implemented:external != 0"));
                    }
                    // A zero `size` would underflow `size - 1`; reject it explicitly.
                    if size == 0 || (size - 1) & 1 != 0 {
                        return Err(Error::other("file names length invalid"));
                    }

                    let size = bounded_count(size, limit, "file names length")?;
                    read_names(header, size - 1, &mut files, metadata)?;
                }
                K_C_TIME => {
                    let times_defined = read_all_or_bits(header, num_files, metadata)?;
                    let external = header.read_u8()?;
                    if external != 0 {
                        return Err(Error::other(format!(
                            "kCTime Unimplemented:external={external}"
                        )));
                    }
                    for (i, file) in files.iter_mut().enumerate() {
                        file.has_creation_date = times_defined.contains(i);
                        if file.has_creation_date {
                            file.creation_date = header.read_u64()?.into();
                        }
                    }
                }
                K_A_TIME => {
                    let times_defined = read_all_or_bits(header, num_files, metadata)?;
                    let external = header.read_u8()?;
                    if external != 0 {
                        return Err(Error::other(format!(
                            "kATime Unimplemented:external={external}"
                        )));
                    }
                    for (i, file) in files.iter_mut().enumerate() {
                        file.has_access_date = times_defined.contains(i);
                        if file.has_access_date {
                            file.access_date = header.read_u64()?.into();
                        }
                    }
                }
                K_M_TIME => {
                    let times_defined = read_all_or_bits(header, num_files, metadata)?;
                    let external = header.read_u8()?;
                    if external != 0 {
                        return Err(Error::other(format!(
                            "kMTime Unimplemented:external={external}"
                        )));
                    }
                    for (i, file) in files.iter_mut().enumerate() {
                        file.has_last_modified_date = times_defined.contains(i);
                        if file.has_last_modified_date {
                            file.last_modified_date = header.read_u64()?.into();
                        }
                    }
                }
                K_WIN_ATTRIBUTES => {
                    let times_defined = read_all_or_bits(header, num_files, metadata)?;
                    let external = header.read_u8()?;
                    if external != 0 {
                        return Err(Error::other(format!(
                            "kWinAttributes Unimplemented:external={external}"
                        )));
                    }
                    for (i, file) in files.iter_mut().enumerate() {
                        file.has_windows_attributes = times_defined.contains(i);
                        if file.has_windows_attributes {
                            file.windows_attributes = header.read_u32()?;
                        }
                    }
                }
                K_START_POS => return Err(Error::other("kStartPos is unsupported, please report")),
                K_DUMMY => {
                    // Bound the skip against the buffer: an unbounded value cast to `i64`
                    // could go negative and seek backwards, re-reading the same bytes forever.
                    let skip = bounded_count(size, limit, "files-info property size")?;
                    header.seek(SeekFrom::Current(skip as i64))?;
                }
                _ => {
                    let skip = bounded_count(size, limit, "files-info property size")?;
                    header.seek(SeekFrom::Current(skip as i64))?;
                }
            };
        }

        let mut non_empty_file_counter = 0;
        let mut empty_file_counter = 0;
        for (i, file) in files.iter_mut().enumerate() {
            file.has_stream = is_empty_stream
                .as_ref()
                .map(|s| !s.contains(i))
                .unwrap_or(true);
            if file.has_stream {
                let sub_stream_info = if let Some(s) = archive.sub_streams_info.as_ref() {
                    s
                } else {
                    return Err(Error::other(
                        "Archive contains file with streams but no subStreamsInfo",
                    ));
                };
                file.is_directory = false;
                file.is_anti_item = false;
                // The count of streamed files and `total_unpack_streams` are independent
                // header quantities. Reject a mismatch instead of indexing out of bounds.
                let (Some(&crc), Some(&size)) = (
                    sub_stream_info.crcs.get(non_empty_file_counter),
                    sub_stream_info.unpack_sizes.get(non_empty_file_counter),
                ) else {
                    return Err(Error::other(
                        "Archive declares more streamed files than sub-streams",
                    ));
                };
                file.has_crc = sub_stream_info.has_crc.contains(non_empty_file_counter);
                file.crc = crc;
                file.size = size;
                non_empty_file_counter += 1;
            } else {
                file.is_directory = if let Some(s) = &is_empty_file {
                    !s.contains(empty_file_counter)
                } else {
                    true
                };
                file.is_anti_item = is_anti
                    .as_ref()
                    .map(|s| s.contains(empty_file_counter))
                    .unwrap_or(false);
                file.has_crc = false;
                file.size = 0;
                empty_file_counter += 1;
            }
        }
        archive.files = files;

        Self::calculate_stream_map(archive, metadata)?;
        Ok(())
    }

    fn calculate_stream_map(
        archive: &mut Archive,
        metadata: &mut MetadataBudget,
    ) -> Result<(), Error> {
        let mut stream_map = StreamMap::default();

        let mut next_block_pack_stream_index = 0;
        let num_blocks = archive.blocks.len();
        metadata.charge(num_blocks, std::mem::size_of::<usize>(), "stream_map")?;
        stream_map.block_first_pack_stream_index = vec![0; num_blocks];
        for i in 0..num_blocks {
            stream_map.block_first_pack_stream_index[i] = next_block_pack_stream_index;
            // A block's pack-stream span `[first .. first + packed_streams.len())` is later
            // used to slice `pack_stream_offsets`/`pack_sizes` in `build_decode_stack{,2}`.
            // Reject a block whose span runs past the available pack streams instead of
            // panicking there.
            next_block_pack_stream_index = next_block_pack_stream_index
                .checked_add(archive.blocks[i].packed_streams.len())
                .ok_or_else(|| Error::other("pack stream index overflow"))?;
            if next_block_pack_stream_index > archive.pack_sizes.len() {
                return Err(Error::other(
                    "block references pack streams beyond the available pack sizes",
                ));
            }
        }

        // `SubStreamsInfo` is indexed by the running sub-stream count over all
        // blocks. Record each block's first index here so `build_decode_stack`
        // does not re-sum the preceding blocks every time it opens one, which
        // made extracting an archive of many non-solid blocks quadratic.
        let mut next_sub_stream_index: usize = 0;
        metadata.charge(num_blocks, std::mem::size_of::<usize>(), "stream_map")?;
        stream_map.block_first_sub_stream_index = vec![0; num_blocks];
        for i in 0..num_blocks {
            stream_map.block_first_sub_stream_index[i] = next_sub_stream_index;
            next_sub_stream_index = next_sub_stream_index
                .checked_add(archive.blocks[i].num_unpack_sub_streams)
                .ok_or_else(|| Error::other("sub-stream index overflow"))?;
        }

        let mut next_pack_stream_offset: u64 = 0;
        let num_pack_sizes = archive.pack_sizes.len();
        metadata.charge(num_pack_sizes, std::mem::size_of::<u64>(), "stream_map")?;
        stream_map.pack_stream_offsets = vec![0; num_pack_sizes];
        for i in 0..num_pack_sizes {
            stream_map.pack_stream_offsets[i] = next_pack_stream_offset;
            next_pack_stream_offset = next_pack_stream_offset
                .checked_add(archive.pack_sizes[i])
                .ok_or_else(|| Error::other("pack stream offset overflow"))?;
        }

        metadata.charge(num_blocks, std::mem::size_of::<usize>(), "stream_map")?;
        metadata.charge(
            archive.files.len(),
            std::mem::size_of::<Option<usize>>(),
            "stream_map",
        )?;
        stream_map.block_first_file_index = vec![0; num_blocks];
        stream_map.file_block_index = vec![None; archive.files.len()];
        let mut next_block_index = 0;
        let mut next_block_unpack_stream_index = 0;
        for i in 0..archive.files.len() {
            if !archive.files[i].has_stream && next_block_unpack_stream_index == 0 {
                stream_map.file_block_index[i] = None;
                continue;
            }
            if next_block_unpack_stream_index == 0 {
                while next_block_index < archive.blocks.len() {
                    stream_map.block_first_file_index[next_block_index] = i;
                    if archive.blocks[next_block_index].num_unpack_sub_streams > 0 {
                        break;
                    }
                    next_block_index += 1;
                }
                if next_block_index >= archive.blocks.len() {
                    return Err(Error::other("Too few blocks in archive"));
                }
            }
            stream_map.file_block_index[i] = Some(next_block_index);
            if !archive.files[i].has_stream {
                continue;
            }

            //set `compressed_size` of first file in block
            if stream_map.block_first_file_index[next_block_index] == i {
                let first_pack_stream_index =
                    stream_map.block_first_pack_stream_index[next_block_index];
                let pack_size =
                    *archive
                        .pack_sizes
                        .get(first_pack_stream_index)
                        .ok_or_else(|| {
                            Error::other("block references a pack stream index beyond pack_sizes")
                        })?;

                archive.files[i].compressed_size = pack_size;
            }

            next_block_unpack_stream_index += 1;
            if next_block_unpack_stream_index
                >= archive.blocks[next_block_index].num_unpack_sub_streams
            {
                next_block_index += 1;
                next_block_unpack_stream_index = 0;
            }
        }

        // Each block's file span `[first_file .. first_file + num_unpack_sub_streams)` is
        // later iterated directly against `archive.files` (in `BlockDecoder`). The
        // sub-stream count is an independent header quantity, so reject a block that
        // declares more sub-streams than there are files instead of indexing out of bounds.
        for (b, block) in archive.blocks.iter().enumerate() {
            let start = stream_map.block_first_file_index[b];
            let end = start
                .checked_add(block.num_unpack_sub_streams)
                .ok_or_else(|| Error::other("block file span overflow"))?;
            if end > archive.files.len() {
                return Err(Error::other(
                    "block declares more sub-streams than the archive has files",
                ));
            }
        }

        archive.stream_map = stream_map;
        Ok(())
    }

    fn read_pack_info<R: Read>(
        header: &mut R,
        archive: &mut Archive,
        limit: usize,
        metadata: &mut MetadataBudget,
    ) -> Result<(), Error> {
        archive.pack_pos = read_variable_u64(header)?;
        let num_pack_streams =
            bounded_count(read_variable_u64(header)?, limit, "num pack streams")?;
        let mut nid = header.read_u8()?;
        if nid == K_SIZE {
            metadata.charge(num_pack_streams, std::mem::size_of::<u64>(), "pack_sizes")?;
            archive.pack_sizes = vec![0u64; num_pack_streams];
            for i in 0..archive.pack_sizes.len() {
                archive.pack_sizes[i] = read_variable_u64(header)?;
            }
            nid = header.read_u8()?;
        }

        if nid == K_CRC {
            archive.pack_crcs_defined = read_all_or_bits(header, num_pack_streams, metadata)?;
            metadata.charge(num_pack_streams, std::mem::size_of::<u64>(), "pack_crcs")?;
            archive.pack_crcs = vec![0; num_pack_streams];
            for i in 0..num_pack_streams {
                if archive.pack_crcs_defined.contains(i) {
                    archive.pack_crcs[i] = header.read_u32()? as u64;
                }
            }
            nid = header.read_u8()?;
        }

        if nid != K_END {
            return Err(Error::BadTerminatedPackInfo(nid));
        }

        Ok(())
    }
    fn read_unpack_info<R: Read>(
        header: &mut R,
        archive: &mut Archive,
        limit: usize,
        metadata: &mut MetadataBudget,
    ) -> Result<(), Error> {
        let nid = header.read_u8()?;
        if nid != K_FOLDER {
            return Err(Error::other(format!("Expected kFolder, got {nid}")));
        }
        let num_blocks = bounded_count(read_variable_u64(header)?, limit, "num blocks")?;

        metadata.charge(num_blocks, std::mem::size_of::<Block>(), "blocks")?;
        archive.blocks.reserve_exact(num_blocks);
        let external = header.read_u8()?;
        if external != 0 {
            return Err(Error::ExternalUnsupported);
        }

        for _ in 0..num_blocks {
            archive
                .blocks
                .push(Self::read_block(header, limit, metadata)?);
        }

        let nid = header.read_u8()?;
        if nid != K_CODERS_UNPACK_SIZE {
            return Err(Error::other(format!(
                "Expected kCodersUnpackSize, got {nid}"
            )));
        }

        for block in archive.blocks.iter_mut() {
            // `total_output_streams` is bounded in `read_block`, but clamp the eager
            // reservation to `limit` as well so it can never over-allocate.
            let tos = block.total_output_streams;
            metadata.charge(tos, std::mem::size_of::<u64>(), "block_unpack_sizes")?;
            block.unpack_sizes.reserve_exact(tos.min(limit));
            for _ in 0..tos {
                block.unpack_sizes.push(read_variable_u64(header)?);
            }
        }

        let mut nid = header.read_u8()?;
        if nid == K_CRC {
            let crcs_defined = read_all_or_bits(header, num_blocks, metadata)?;
            for i in 0..num_blocks {
                if crcs_defined.contains(i) {
                    archive.blocks[i].has_crc = true;
                    archive.blocks[i].crc = header.read_u32()? as u64;
                } else {
                    archive.blocks[i].has_crc = false;
                }
            }
            nid = header.read_u8()?;
        }
        if nid != K_END {
            return Err(Error::BadTerminatedUnpackInfo);
        }

        Ok(())
    }

    fn read_sub_streams_info<R: Read>(
        header: &mut R,
        archive: &mut Archive,
        limit: usize,
        metadata: &mut MetadataBudget,
    ) -> Result<(), Error> {
        for block in archive.blocks.iter_mut() {
            block.num_unpack_sub_streams = 1;
        }
        let mut total_unpack_streams = archive.blocks.len();

        let mut nid = header.read_u8()?;
        if nid == K_NUM_UNPACK_STREAM {
            total_unpack_streams = 0;
            for block in archive.blocks.iter_mut() {
                let num_streams = bounded_count(read_variable_u64(header)?, limit, "numStreams")?;
                block.num_unpack_sub_streams = num_streams;
                // Each sub-stream still consumes header bytes downstream, so the running
                // total stays bounded by `limit`; reject anything larger up front.
                total_unpack_streams += num_streams;
                if total_unpack_streams > limit {
                    return Err(Error::other("total unpack streams exceeds available input"));
                }
            }
            nid = header.read_u8()?;
        }

        metadata.charge(
            total_unpack_streams,
            std::mem::size_of::<u64>(),
            "substream_metadata",
        )?;
        metadata.charge(
            total_unpack_streams,
            std::mem::size_of::<u64>(),
            "substream_metadata",
        )?;
        metadata.charge(
            total_unpack_streams,
            std::mem::size_of::<usize>(),
            "substream_metadata",
        )?;
        let mut sub_streams_info = SubStreamsInfo::default();
        sub_streams_info
            .unpack_sizes
            .resize(total_unpack_streams, Default::default());
        sub_streams_info
            .has_crc
            .reserve_len_exact(total_unpack_streams);
        sub_streams_info.crcs = vec![0; total_unpack_streams];

        let mut next_unpack_stream = 0;
        for block in archive.blocks.iter() {
            if block.num_unpack_sub_streams == 0 {
                continue;
            }
            let mut sum: u64 = 0;
            if nid == K_SIZE {
                for _i in 0..block.num_unpack_sub_streams - 1 {
                    let size = read_variable_u64(header)?;
                    sub_streams_info.unpack_sizes[next_unpack_stream] = size;
                    next_unpack_stream += 1;
                    sum = sum
                        .checked_add(size)
                        .ok_or_else(|| Error::other("sub-stream size sum overflow"))?;
                }
            }
            if sum > block.get_unpack_size() {
                return Err(Error::other(
                    "sum of unpack sizes of block exceeds total unpack size",
                ));
            }
            // Calculate the last size from the total minus the sum of N-1 sizes.
            sub_streams_info.unpack_sizes[next_unpack_stream] = block.get_unpack_size() - sum;
            next_unpack_stream += 1;
        }
        if nid == K_SIZE {
            nid = header.read_u8()?;
        }

        let mut num_digests = 0;
        for block in archive.blocks.iter() {
            if block.num_unpack_sub_streams != 1 || !block.has_crc {
                num_digests += block.num_unpack_sub_streams;
            }
        }

        if nid == K_CRC {
            let has_missing_crc = read_all_or_bits(header, num_digests, metadata)?;
            metadata.charge(num_digests, std::mem::size_of::<u64>(), "substream_crcs")?;
            let mut missing_crcs = vec![0; num_digests];
            for (i, missing_crc) in missing_crcs.iter_mut().enumerate() {
                if has_missing_crc.contains(i) {
                    *missing_crc = header.read_u32()? as u64;
                }
            }
            let mut next_crc = 0;
            let mut next_missing_crc = 0;
            for block in archive.blocks.iter() {
                if block.num_unpack_sub_streams == 1 && block.has_crc {
                    sub_streams_info.has_crc.insert(next_crc);
                    sub_streams_info.crcs[next_crc] = block.crc;
                    next_crc += 1;
                } else {
                    for _i in 0..block.num_unpack_sub_streams {
                        if has_missing_crc.contains(next_missing_crc) {
                            sub_streams_info.has_crc.insert(next_crc);
                        } else {
                            sub_streams_info.has_crc.remove(next_crc);
                        }
                        sub_streams_info.crcs[next_crc] = missing_crcs[next_missing_crc];
                        next_crc += 1;
                        next_missing_crc += 1;
                    }
                }
            }

            nid = header.read_u8()?;
        }

        if nid != K_END {
            return Err(Error::BadTerminatedSubStreamsInfo);
        }

        archive.sub_streams_info = Some(sub_streams_info);
        Ok(())
    }

    fn read_block<R: Read>(
        header: &mut R,
        limit: usize,
        metadata: &mut MetadataBudget,
    ) -> Result<Block, Error> {
        let mut block = Block::default();

        let num_coders = bounded_count(read_variable_u64(header)?, limit, "num coders")?;
        metadata.charge(num_coders, std::mem::size_of::<Coder>(), "coders")?;
        let mut coders = Vec::with_capacity(num_coders);
        let mut total_in_streams: u64 = 0;
        let mut total_out_streams: u64 = 0;
        for _i in 0..num_coders {
            let mut coder = Coder::default();
            let bits = header.read_u8()?;
            let id_size = bits & 0xF;
            let is_simple = (bits & 0x10) == 0;
            let has_attributes = (bits & 0x20) != 0;
            let more_alternative_methods = (bits & 0x80) != 0;

            coder.id_size = id_size as usize;

            header.read_exact(coder.decompression_method_id_mut())?;
            if is_simple {
                coder.num_in_streams = 1;
                coder.num_out_streams = 1;
            } else {
                coder.num_in_streams = read_variable_u64(header)?;
                coder.num_out_streams = read_variable_u64(header)?;
            }
            // Each stream is referenced by a bind-pair/packed-stream entry that consumes
            // header bytes, so the totals cannot legitimately exceed `limit`. The counts are
            // unbounded attacker-controlled varints, so the sums are checked: an overflowing
            // addition must be rejected rather than wrap past the bound below.
            total_in_streams = total_in_streams
                .checked_add(coder.num_in_streams)
                .ok_or_else(|| Error::other("coder stream counts exceed available input"))?;
            total_out_streams = total_out_streams
                .checked_add(coder.num_out_streams)
                .ok_or_else(|| Error::other("coder stream counts exceed available input"))?;
            if total_in_streams > limit as u64 || total_out_streams > limit as u64 {
                return Err(Error::other("coder stream counts exceed available input"));
            }
            if has_attributes {
                let properties_size =
                    bounded_count(read_variable_u64(header)?, limit, "properties size")?;
                metadata.charge(properties_size, 1, "coder_properties")?;
                let mut props = vec![0u8; properties_size];
                header.read_exact(&mut props)?;
                coder.properties = props;
            }
            coders.push(coder);
            // would need to keep looping as above:
            if more_alternative_methods {
                return Err(Error::other(
                    "Alternative methods are unsupported, please report. The reference implementation doesn't support them either.",
                ));
            }
        }
        block.coders = coders;
        let total_in_streams = total_in_streams as usize;
        let total_out_streams = total_out_streams as usize;
        block.total_input_streams = total_in_streams;
        block.total_output_streams = total_out_streams;

        if total_out_streams == 0 {
            return Err(Error::other("Total output streams can't be 0"));
        }
        let num_bind_pairs = total_out_streams - 1;
        metadata.charge(
            num_bind_pairs,
            std::mem::size_of::<BindPair>(),
            "bind_pairs",
        )?;
        let mut bind_pairs = Vec::with_capacity(num_bind_pairs);
        for _ in 0..num_bind_pairs {
            let bp = BindPair {
                in_index: read_variable_u64(header)?,
                out_index: read_variable_u64(header)?,
            };
            // Bind-pair indices are later used to index fixed-size coder arrays and the
            // coder graph. Validate them at parse time so decoding cannot panic on an
            // out-of-range index.
            if bp.in_index >= total_in_streams as u64 || bp.out_index >= total_out_streams as u64 {
                return Err(Error::other("bind pair references an out-of-range stream"));
            }
            bind_pairs.push(bp);
        }
        block.bind_pairs = bind_pairs;

        if total_in_streams < num_bind_pairs {
            return Err(Error::other(
                "Total input streams can't be less than the number of bind pairs",
            ));
        }
        let num_packed_streams = total_in_streams - num_bind_pairs;
        metadata.charge(
            num_packed_streams,
            std::mem::size_of::<u64>(),
            "packed_streams",
        )?;
        let mut packed_streams = vec![0; num_packed_streams];
        if num_packed_streams == 1 {
            let mut index = u64::MAX;
            for i in 0..total_in_streams {
                if block.find_bind_pair_for_in_stream(i as u64).is_none() {
                    index = i as u64;
                    break;
                }
            }
            if index == u64::MAX {
                return Err(Error::other("Couldn't find stream's bind pair index"));
            }
            packed_streams[0] = index;
        } else {
            for packed_stream in packed_streams.iter_mut() {
                *packed_stream = read_variable_u64(header)?;
            }
        }
        block.packed_streams = packed_streams;

        Ok(block)
    }
}

/// Validates an attacker-controlled count/size decoded from the header against an upper
/// bound derived from the input (the number of bytes in the header buffer).
///
/// Every counted element (a file, pack stream, coder, sub-stream, name byte, …) consumes
/// at least one header byte downstream, so a legitimate count can never exceed the buffer
/// length. Rejecting anything larger stops a tiny archive from declaring a huge count and
/// forcing an out-of-memory allocation, without rejecting any valid archive.
#[inline]
fn bounded_count(value: u64, limit: usize, field: &str) -> Result<usize, Error> {
    if value > limit as u64 {
        return Err(Error::other(format!(
            "{field} ({value}) exceeds the available input ({limit} bytes)"
        )));
    }
    Ok(value as usize)
}

fn read_variable_u64<R: Read>(reader: &mut R) -> io::Result<u64> {
    let first = reader.read_u8()? as u64;
    let mut mask = 0x80_u64;
    let mut value = 0;
    for i in 0..8 {
        if (first & mask) == 0 {
            return Ok(value | ((first & (mask - 1)) << (8 * i)));
        }
        let b = reader.read_u8()? as u64;
        value |= b << (8 * i);
        mask >>= 1;
    }
    Ok(value)
}

fn read_all_or_bits<R: Read>(
    header: &mut R,
    size: usize,
    metadata: &mut MetadataBudget,
) -> Result<BitSet, Error> {
    let all = header.read_u8()?;
    if all != 0 {
        metadata.charge(size, std::mem::size_of::<usize>(), "bitset")?;
        let mut bits = BitSet::with_capacity(size);
        for i in 0..size {
            bits.insert(i);
        }
        Ok(bits)
    } else {
        read_bits(header, size, metadata)
    }
}

fn read_bits<R: Read>(
    header: &mut R,
    size: usize,
    metadata: &mut MetadataBudget,
) -> Result<BitSet, Error> {
    metadata.charge(size, std::mem::size_of::<usize>(), "bitset")?;
    let mut bits = BitSet::with_capacity(size);
    let mut mask = 0u32;
    let mut cache = 0u32;
    for i in 0..size {
        if mask == 0 {
            mask = 0x80;
            cache = header.read_u8()? as u32;
        }
        if (cache & mask) != 0 {
            bits.insert(i);
        }
        mask >>= 1;
    }
    Ok(bits)
}

fn read_names<R: Read>(
    header: &mut R,
    byte_len: usize,
    files: &mut [ArchiveEntry],
    metadata: &mut MetadataBudget,
) -> Result<(), Error> {
    let unit_count = byte_len / 2;
    metadata.charge(unit_count, std::mem::size_of::<u16>(), "file_names_utf16")?;
    let mut units = vec![0_u16; unit_count];
    for unit in &mut units {
        let mut encoded = [0_u8; 2];
        header.read_exact(&mut encoded)?;
        *unit = u16::from_le_bytes(encoded);
    }

    let mut start = 0;
    for file in files {
        if start >= units.len() {
            return Err(Error::other("Error parsing file names"));
        }
        let relative_end = units[start..]
            .iter()
            .position(|unit| *unit == 0)
            .unwrap_or(units.len() - start);
        let end = start + relative_end;
        let name_units = &units[start..end];
        let mut utf8_len = 0_usize;
        for scalar in char::decode_utf16(name_units.iter().copied()) {
            let scalar = scalar.map_err(|error| Error::other(error.to_string()))?;
            utf8_len = utf8_len
                .checked_add(scalar.len_utf8())
                .ok_or(Error::ResourceLimit {
                    resource: "file_name_strings",
                    limit: metadata.limit,
                    required: usize::MAX,
                })?;
        }
        // `String::from_utf16` may grow geometrically, so charge twice the next power of two.
        let basis = name_units.len().max(utf8_len);
        let capacity_bound = if basis == 0 {
            0
        } else {
            basis
                .checked_next_power_of_two()
                .and_then(|capacity| capacity.checked_mul(2))
                .ok_or(Error::ResourceLimit {
                    resource: "file_name_strings",
                    limit: metadata.limit,
                    required: usize::MAX,
                })?
        };
        metadata.charge(1, capacity_bound, "file_name_strings")?;
        file.name =
            String::from_utf16(name_units).map_err(|error| Error::other(error.to_string()))?;
        start = end + 1;
    }
    if start < units.len() {
        return Err(Error::other("Error parsing file names"));
    }
    Ok(())
}

#[derive(Copy, Clone)]
struct IndexEntry {
    block_index: Option<usize>,
    file_index: usize,
}

/// Reads a 7z archive file.
pub struct ArchiveReader<R: Read + Seek> {
    source: R,
    archive: Archive,
    password: Password,
    thread_count: u32,
    limits: ArchiveReaderLimits,
    index: HashMap<String, IndexEntry>,
}

#[cfg(not(target_arch = "wasm32"))]
impl ArchiveReader<File> {
    /// Opens a 7z archive file at the given `path` and creates a [`ArchiveReader`] to read it.
    #[inline]
    pub fn open(path: impl AsRef<std::path::Path>, password: Password) -> Result<Self, Error> {
        let file = File::open(path.as_ref())
            .map_err(|e| Error::file_open(e, path.as_ref().to_string_lossy().to_string()))?;
        Self::new(file, password)
    }
}

impl<R: Read + Seek> ArchiveReader<R> {
    /// Creates a [`ArchiveReader`] to read a 7z archive file from the given `source` reader.
    #[inline]
    pub fn new(source: R, password: Password) -> Result<Self, Error> {
        Self::new_with_limits(source, password, ArchiveReaderLimits::permissive())
    }

    /// Like [`ArchiveReader::new`], but enforces `limits`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::fs::File;
    ///
    /// use sevenz_rust2::*;
    ///
    /// let file = File::open("untrusted.7z").unwrap();
    /// let limits = ArchiveReaderLimits::untrusted_default();
    /// let mut reader = ArchiveReader::new_with_limits(file, Password::empty(), limits).unwrap();
    /// let data = reader.read_file("data.txt").unwrap();
    /// ```
    #[inline]
    pub fn new_with_limits(
        mut source: R,
        password: Password,
        limits: ArchiveReaderLimits,
    ) -> Result<Self, Error> {
        let archive = Archive::read_with_limits(&mut source, &password, limits)?;
        Ok(Self::from_archive(archive, source, password))
    }

    /// Creates an [`ArchiveReader`] from an existing [`Archive`] instance.
    ///
    /// This is useful when you already have a parsed archive and want to create a reader
    /// without re-parsing the archive structure. The limits the archive was read with apply.
    ///
    /// # Arguments
    /// * `archive` - An existing parsed archive instance
    /// * `source` - The reader providing access to the archive data
    /// * `password` - Password for encrypted archives
    #[inline]
    pub fn from_archive(archive: Archive, source: R, password: Password) -> Self {
        let limits = archive.limits;
        let mut reader = Self {
            source,
            archive,
            password,
            thread_count: 1,
            limits,
            index: HashMap::default(),
        };

        if limits.max_metadata_bytes == usize::MAX {
            reader.fill_index();
        }

        // Multi-threaded decoding is not covered by the decoder memory limit.
        if !limits.decoder_memory_is_bounded() {
            let thread_count =
                std::thread::available_parallelism().unwrap_or(NonZeroUsize::new(1).unwrap());
            reader.set_thread_count(thread_count.get() as u32);
        }

        reader
    }

    /// Sets the thread count to use when multi-threading is supported by the de-compression
    /// (currently only LZMA2 if encoded with MT support).
    ///
    /// Defaults to `std::thread::available_parallelism()` if not set manually.
    pub fn set_thread_count(&mut self, thread_count: u32) {
        self.thread_count = thread_count.clamp(1, 256);
    }

    fn fill_index(&mut self) {
        for (file_index, file) in self.archive.files.iter().enumerate() {
            let block_index = self.archive.stream_map.file_block_index[file_index];

            self.index.insert(
                file.name.clone(),
                IndexEntry {
                    block_index,
                    file_index,
                },
            );
        }
    }

    fn index_entry(&self, name: &str) -> Option<IndexEntry> {
        self.index.get(name).copied().or_else(|| {
            // No index under a metadata limit; the last duplicate wins, as in the index.
            self.archive
                .files
                .iter()
                .enumerate()
                .rev()
                .find(|(_, file)| file.name == name)
                .map(|(file_index, _)| IndexEntry {
                    block_index: self.archive.stream_map.file_block_index[file_index],
                    file_index,
                })
        })
    }

    /// Returns a reference to the underlying [`Archive`] structure.
    ///
    /// This provides access to the archive metadata including files, blocks,
    /// and compression information.
    #[inline]
    pub fn archive(&self) -> &Archive {
        &self.archive
    }

    fn build_decode_stack<'r>(
        source: &'r mut R,
        archive: &Archive,
        block_index: usize,
        password: &Password,
        limits: &ArchiveReaderLimits,
        thread_count: u32,
    ) -> Result<(Box<dyn Read + 'r>, usize), Error> {
        reject_bounded_multithreading(limits, thread_count)?;
        let block = &archive.blocks[block_index];
        reject_bounded_coder_count(limits, block.coders.len())?;
        if block.total_input_streams > block.total_output_streams {
            return Self::build_decode_stack2(
                source,
                archive,
                block_index,
                password,
                limits,
                thread_count,
            );
        }
        let first_pack_stream_index = archive.stream_map.block_first_pack_stream_index[block_index];
        let block_offset = SIGNATURE_HEADER_SIZE
            .checked_add(archive.pack_pos)
            .and_then(|v| {
                v.checked_add(archive.stream_map.pack_stream_offsets[first_pack_stream_index])
            })
            .ok_or_else(|| Error::other("block offset out of range"))?;

        let (mut has_crc, mut crc) = (block.has_crc, block.crc);

        // Single stream blocks might have it's CRC stored in the single substream information.
        if !has_crc
            && block.num_unpack_sub_streams == 1
            && let Some(sub_streams_info) = archive.sub_streams_info.as_ref()
        {
            let substream_index = archive.stream_map.block_first_sub_stream_index[block_index];

            // Only when there is a single stream, we can use it's CRC to verify the compressed block data.
            // Multiple streams would contain the CRC of the compressed data for each file in the block.
            if sub_streams_info.has_crc.contains(substream_index) {
                has_crc = true;
                crc = sub_streams_info.crcs[substream_index];
            }
        }

        source.seek(SeekFrom::Start(block_offset))?;
        let pack_size = archive.pack_sizes[first_pack_stream_index] as usize;

        let mut decoder_memory = DecoderMemoryBudget::new(limits.max_decoder_memory_kib);
        decoder_memory.charge(WRAPPER_MEMORY_KIB)?;
        let mut decoder: Box<dyn Read> = Box::new(BoundedReader::new(source, pack_size));
        let block = &archive.blocks[block_index];
        for (index, coder) in block.ordered_coder_iter() {
            if coder.num_in_streams != 1 || coder.num_out_streams != 1 {
                return Err(Error::unsupported(
                    "Multi input/output stream coders are not supported",
                ));
            }
            let next = add_decoder(
                decoder,
                block.get_unpack_size_at_index(index) as usize,
                coder,
                password,
                limits,
                &mut decoder_memory,
                thread_count,
            )?;
            decoder = Box::new(next);
        }
        if has_crc {
            decoder_memory.charge(WRAPPER_MEMORY_KIB)?;
            decoder = Box::new(Crc32VerifyingReader::new(
                decoder,
                block.get_unpack_size() as usize,
                crc,
            ));
        }

        Ok((decoder, pack_size))
    }

    fn build_decode_stack2<'r>(
        source: &'r mut R,
        archive: &Archive,
        block_index: usize,
        password: &Password,
        limits: &ArchiveReaderLimits,
        thread_count: u32,
    ) -> Result<(Box<dyn Read + 'r>, usize), Error> {
        let block = &archive.blocks[block_index];
        reject_bounded_coder_count(limits, block.coders.len())?;
        if block.coders.len() > MAX_CODER_COUNT {
            return Err(Error::unsupported(format!(
                "Too many coders: {}",
                block.coders.len()
            )));
        }

        assert!(block.total_input_streams > block.total_output_streams);
        let shared_source = Rc::new(RefCell::new(source));
        let first_pack_stream_index = archive.stream_map.block_first_pack_stream_index[block_index];
        let start_pos = SIGNATURE_HEADER_SIZE
            .checked_add(archive.pack_pos)
            .ok_or_else(|| Error::other("pack position out of range"))?;
        let offsets = &archive.stream_map.pack_stream_offsets[first_pack_stream_index..];

        let mut decoder_metadata =
            MetadataBudget::with_used(limits.max_metadata_bytes, archive.metadata_bytes);
        decoder_metadata.charge(
            block.packed_streams.len(),
            std::mem::size_of::<SharedBoundedReader<R>>(),
            "decoder_sources",
        )?;
        let mut sources = Vec::with_capacity(block.packed_streams.len());

        for (i, offset) in offsets[..block.packed_streams.len()].iter().enumerate() {
            let pack_pos = start_pos
                .checked_add(*offset)
                .ok_or_else(|| Error::other("pack stream offset out of range"))?;
            let pack_size = archive.pack_sizes[first_pack_stream_index + i];
            let pack_end = pack_pos
                .checked_add(pack_size)
                .ok_or_else(|| Error::other("pack stream size out of range"))?;

            let pack_reader =
                SharedBoundedReader::new(Rc::clone(&shared_source), (pack_pos, pack_end));

            sources.push(pack_reader);
        }

        let mut coder_to_stream_map = [usize::MAX; MAX_CODER_COUNT];

        let mut si = 0;
        for (i, coder) in block.coders.iter().enumerate() {
            coder_to_stream_map[i] = si;
            si += coder.num_in_streams as usize;
        }

        let main_coder_index = {
            let mut coder_used = [false; MAX_CODER_COUNT];
            for bp in block.bind_pairs.iter() {
                // `out_index` is validated `< total_output_streams` at parse time, but the
                // `coder_used` array indexes coders (max `MAX_CODER_COUNT`); reject an
                // index that would fall outside it instead of panicking.
                let out_index = bp.out_index as usize;
                if out_index >= block.coders.len() {
                    return Err(Error::other("bind pair out index exceeds coder count"));
                }
                coder_used[out_index] = true;
            }
            let mut mci = 0;
            for (i, used) in coder_used[..block.coders.len()].iter().enumerate() {
                if !used {
                    mci = i;
                    break;
                }
            }
            mci
        };

        // Build the decoder for the folder's final output by resolving the main coder's
        // output stream. `get_in_stream2` recursively wires up the whole coder graph,
        // so this also handles single-input filters (e.g. Delta) layered on top of a
        // BCJ2 coder's output, not just a bare BCJ2 main coder.
        let mut decoder_memory = DecoderMemoryBudget::new(limits.max_decoder_memory_kib);
        decoder_memory.charge(WRAPPER_MEMORY_KIB)?;
        let mut decoder = {
            let mut graph = DecoderGraph {
                sources: &sources,
                coder_to_stream_map: &coder_to_stream_map,
                password,
                limits,
                metadata: &mut decoder_metadata,
                decoder_memory: &mut decoder_memory,
                thread_count,
            };
            Self::get_in_stream2(block, &mut graph, main_coder_index, 0)?
        };
        if block.has_crc {
            decoder_memory.charge(WRAPPER_MEMORY_KIB)?;
            decoder = Box::new(Crc32VerifyingReader::new(
                decoder,
                block.get_unpack_size() as usize,
                block.crc,
            ));
        }
        Ok((
            decoder,
            archive.pack_sizes[first_pack_stream_index] as usize,
        ))
    }

    fn get_in_stream<'r>(
        block: &Block,
        graph: &mut DecoderGraph<'_, 'r, R>,
        in_stream_index: usize,
        depth: usize,
    ) -> Result<Box<dyn Read + 'r>, Error>
    where
        R: 'r,
    {
        let index = block
            .packed_streams
            .iter()
            .position(|&i| i == in_stream_index as u64);
        if let Some(index) = index {
            return Ok(Box::new(graph.sources[index].clone()));
        }

        let bp = block
            .find_bind_pair_for_in_stream(in_stream_index as u64)
            .ok_or_else(|| {
                Error::other(format!(
                    "Couldn't find bind pair for stream {in_stream_index}"
                ))
            })?;
        let index = bp.out_index as usize;

        Self::get_in_stream2(block, graph, index, depth)
    }

    fn get_in_stream2<'r>(
        block: &Block,
        graph: &mut DecoderGraph<'_, 'r, R>,
        in_stream_index: usize,
        depth: usize,
    ) -> Result<Box<dyn Read + 'r>, Error>
    where
        R: 'r,
    {
        // Each coder is visited at most once in an acyclic graph, so a traversal deeper
        // than the coder count means the bind pairs form a cycle. Bail out instead of
        // recursing until the stack overflows (an uncatchable abort).
        if depth > block.coders.len() {
            return Err(Error::other("cyclic coder bind-pair graph"));
        }
        let (Some(coder), Some(&start_index)) = (
            block.coders.get(in_stream_index),
            graph.coder_to_stream_map.get(in_stream_index),
        ) else {
            return Err(Error::other("in_stream_index out of range"));
        };
        if start_index == usize::MAX {
            return Err(Error::other("in_stream_index out of range"));
        }
        let uncompressed_len = *block
            .unpack_sizes
            .get(in_stream_index)
            .ok_or_else(|| Error::other("in_stream_index out of range"))?
            as usize;
        if coder.num_in_streams == 1 {
            let input = Self::get_in_stream(block, graph, start_index, depth + 1)?;

            let decoder = add_decoder(
                input,
                uncompressed_len,
                coder,
                graph.password,
                graph.limits,
                graph.decoder_memory,
                graph.thread_count,
            )?;
            return Ok(Box::new(decoder));
        }

        // BCJ2 is the only multi-input coder we support. It takes four input streams
        // (main, call, jump and range-coder) and produces a single output stream.
        if coder.encoder_method_id() == EncoderMethod::ID_BCJ2 {
            let num_in_streams = coder.num_in_streams as usize;
            // The BCJ2 decoder indexes exactly four input streams; reject a malformed count
            // up front instead of handing a short/long input list to the upstream decoder.
            if num_in_streams != 4 {
                return Err(Error::other(
                    "BCJ2 coder must declare exactly four input streams",
                ));
            }
            graph
                .decoder_memory
                .charge(WRAPPER_MEMORY_KIB + BCJ2_BUFFER_KIB)?;
            graph.metadata.charge(
                num_in_streams,
                std::mem::size_of::<Box<dyn Read>>(),
                "bcj2_inputs",
            )?;
            let mut inputs: Vec<Box<dyn Read>> = Vec::with_capacity(num_in_streams);
            for i in start_index..start_index + num_in_streams {
                inputs.push(Self::get_in_stream(block, graph, i, depth + 1)?);
            }
            return Ok(Box::new(Bcj2Reader::new(inputs, uncompressed_len as u64)));
        }

        Err(Error::unsupported(format!(
            "Unsupported multi-input coder: {:?}",
            coder.encoder_method_id()
        )))
    }

    /// Takes a closure to decode each files in the archive.
    ///
    /// Attention about solid archive:
    /// When decoding a solid archive, the data to be decompressed depends on the data in front of it,
    /// you cannot simply skip the previous data and only decompress the data in the back.
    pub fn for_each_entries<F: FnMut(&ArchiveEntry, &mut dyn Read) -> Result<bool, Error>>(
        &mut self,
        mut each: F,
    ) -> Result<(), Error> {
        let block_count = self.archive.blocks.len();
        for block_index in 0..block_count {
            let forder_dec = BlockDecoder::new_with_limits(
                self.thread_count,
                block_index,
                &self.archive,
                &self.password,
                self.limits,
                &mut self.source,
            );
            forder_dec.for_each_entries(&mut each)?;
        }
        // decode empty files
        for file_index in 0..self.archive.files.len() {
            let block_index = self.archive.stream_map.file_block_index[file_index];
            if block_index.is_none() {
                let file = &self.archive.files[file_index];
                let empty_reader: &mut dyn Read = &mut ([0u8; 0].as_slice());
                if !each(file, empty_reader)? {
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    /// Returns the data of a file with the given path inside the archive.
    ///
    /// # Notice
    /// This function is very inefficient when used with solid archives, since
    /// it needs to decode all data before the actual file.
    pub fn read_file(&mut self, name: &str) -> Result<Vec<u8>, Error> {
        let index_entry = self.index_entry(name).ok_or(Error::FileNotFound)?;
        let file = &self.archive.files[index_entry.file_index];

        if !file.has_stream {
            return Ok(Vec::new());
        }

        if file.size > self.limits.max_read_file_bytes as u64 {
            return Err(Error::ResourceLimit {
                resource: "read_file_bytes",
                limit: self.limits.max_read_file_bytes,
                required: usize::try_from(file.size).unwrap_or(usize::MAX),
            });
        }

        let block_index = index_entry
            .block_index
            .ok_or_else(|| Error::other("File has no associated block"))?;

        match self.archive.is_solid {
            true => {
                let mut result = None;
                let target_file_ptr = file as *const _;

                BlockDecoder::new_with_limits(
                    self.thread_count,
                    block_index,
                    &self.archive,
                    &self.password,
                    self.limits,
                    &mut self.source,
                )
                .for_each_entries(&mut |archive_entry, reader| {
                    if !std::ptr::eq(archive_entry, target_file_ptr) {
                        std::io::copy(reader, &mut std::io::sink())?;
                        return Ok(true);
                    }

                    let mut data =
                        Vec::with_capacity((archive_entry.size as usize).min(MAX_PREALLOC_BYTES));
                    reader.read_to_end(&mut data)?;
                    result = Some(data);
                    Ok(false)
                })?;

                result.ok_or(Error::FileNotFound)
            }
            false => {
                let pack_index = self.archive.stream_map.block_first_pack_stream_index[block_index];
                let pack_offset = self.archive.stream_map.pack_stream_offsets[pack_index];
                let block_offset = SIGNATURE_HEADER_SIZE
                    .checked_add(self.archive.pack_pos)
                    .and_then(|v| v.checked_add(pack_offset))
                    .ok_or_else(|| Error::other("block offset out of range"))?;

                self.source.seek(SeekFrom::Start(block_offset))?;

                let (mut block_reader, _size) = Self::build_decode_stack(
                    &mut self.source,
                    &self.archive,
                    block_index,
                    &self.password,
                    &self.limits,
                    self.thread_count,
                )?;

                let mut data = Vec::with_capacity((file.size as usize).min(MAX_PREALLOC_BYTES));
                let mut decoder: Box<dyn Read> =
                    Box::new(BoundedReader::new(&mut block_reader, file.size as usize));

                if file.has_crc {
                    decoder = Box::new(Crc32VerifyingReader::new(
                        decoder,
                        file.size as usize,
                        file.crc,
                    ));
                }

                decoder.read_to_end(&mut data)?;

                Ok(data)
            }
        }
    }

    /// Get the compression method(s) used for a specific file in the archive.
    pub fn file_compression_methods(
        &self,
        file_name: &str,
        methods: &mut Vec<EncoderMethod>,
    ) -> Result<(), Error> {
        let index_entry = self.index_entry(file_name).ok_or(Error::FileNotFound)?;
        let file = &self.archive.files[index_entry.file_index];

        if !file.has_stream {
            return Ok(());
        }

        let block_index = index_entry
            .block_index
            .ok_or_else(|| Error::other("File has no associated block"))?;

        let block = self
            .archive
            .blocks
            .get(block_index)
            .ok_or_else(|| Error::other("Block not found"))?;

        block
            .coders
            .iter()
            .filter_map(|coder| EncoderMethod::by_id(coder.encoder_method_id()))
            .for_each(|method| {
                methods.push(method);
            });

        Ok(())
    }
}

/// Decoder for a specific block within a 7z archive.
///
/// Provides access to entries within a single compression block and allows
/// decoding files from that block.
pub struct BlockDecoder<'a, R: Read + Seek> {
    thread_count: u32,
    block_index: usize,
    archive: &'a Archive,
    password: &'a Password,
    limits: ArchiveReaderLimits,
    source: &'a mut R,
}

impl<'a, R: Read + Seek> BlockDecoder<'a, R> {
    /// Creates a new [`BlockDecoder`] for decoding a specific block in the archive.
    ///
    /// # Arguments
    /// * `thread_count` - Number of threads to use for multi-threaded decompression (if supported
    ///   by the codec)
    /// * `block_index` - Index of the block to decode within the archive
    /// * `archive` - Reference to the archive containing the block
    /// * `password` - Password for encrypted blocks
    /// * `source` - Mutable reference to the reader providing archive data
    pub fn new(
        thread_count: u32,
        block_index: usize,
        archive: &'a Archive,
        password: &'a Password,
        source: &'a mut R,
    ) -> Self {
        Self::new_with_limits(
            thread_count,
            block_index,
            archive,
            password,
            archive.limits,
            source,
        )
    }

    /// Like [`BlockDecoder::new`], but enforces `limits`.
    pub fn new_with_limits(
        thread_count: u32,
        block_index: usize,
        archive: &'a Archive,
        password: &'a Password,
        limits: ArchiveReaderLimits,
        source: &'a mut R,
    ) -> Self {
        Self {
            thread_count,
            block_index,
            archive,
            password,
            limits,
            source,
        }
    }

    /// Sets the thread count to use when multi-threading is supported by the de-compression
    /// (currently only LZMA2 if encoded with MT support).
    pub fn set_thread_count(&mut self, thread_count: u32) {
        self.thread_count = thread_count.clamp(1, 256);
    }

    /// Returns a slice of archive entries contained in this block.
    ///
    /// The entries are returned in the order they appear in the block.
    pub fn entries(&self) -> &[ArchiveEntry] {
        let start = self.archive.stream_map.block_first_file_index[self.block_index];
        let file_count = self.archive.blocks[self.block_index].num_unpack_sub_streams;
        &self.archive.files[start..(file_count + start)]
    }

    /// Returns the number of entries contained in this block.
    pub fn entry_count(&self) -> usize {
        self.archive.blocks[self.block_index].num_unpack_sub_streams
    }

    /// Takes a closure to decode each files in this block.
    ///
    /// When decoding files in a block, the data to be decompressed depends on the data in front of
    /// it, you cannot simply skip the previous data and only decompress the data in the back.
    ///
    /// Non-solid archives use one block per file and allow more effective decoding of single files.
    pub fn for_each_entries<F: FnMut(&ArchiveEntry, &mut dyn Read) -> Result<bool, Error>>(
        self,
        each: &mut F,
    ) -> Result<bool, Error> {
        let Self {
            thread_count,
            block_index,
            archive,
            password,
            limits,
            source,
        } = self;
        let (mut block_reader, _size) = ArchiveReader::build_decode_stack(
            source,
            archive,
            block_index,
            password,
            &limits,
            thread_count,
        )?;
        let start = archive.stream_map.block_first_file_index[block_index];
        let file_count = archive.blocks[block_index].num_unpack_sub_streams;

        for file_index in start..(file_count + start) {
            let file = &archive.files[file_index];
            if file.has_stream && file.size > 0 {
                let mut decoder: Box<dyn Read> =
                    Box::new(BoundedReader::new(&mut block_reader, file.size as usize));
                if file.has_crc {
                    decoder = Box::new(Crc32VerifyingReader::new(
                        decoder,
                        file.size as usize,
                        file.crc,
                    ));
                }
                if !each(file, &mut decoder)
                    .map_err(|e| e.maybe_bad_password(!self.password.is_empty()))?
                {
                    return Ok(false);
                }
            } else {
                let empty_reader: &mut dyn Read = &mut ([0u8; 0].as_slice());
                if !each(file, empty_reader)? {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct InterruptedOneByteReader {
        remaining: usize,
        interrupted: bool,
    }

    impl Read for InterruptedOneByteReader {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if self.interrupted {
                self.interrupted = false;
                return Err(io::Error::new(io::ErrorKind::Interrupted, "retry"));
            }
            if self.remaining == 0 {
                return Ok(0);
            }
            buf[0] = 0xA5;
            self.remaining -= 1;
            self.interrupted = true;
            Ok(1)
        }
    }

    #[test]
    fn decoded_header_short_reads_never_grow_capacity_past_the_declared_limit() {
        let declared_len = DECODED_HEADER_INITIAL_BYTES + 1;
        let mut source = InterruptedOneByteReader {
            remaining: declared_len,
            interrupted: false,
        };

        let decoded = read_decoded_header(&mut source, declared_len).expect("decode header");

        assert_eq!(decoded.len(), declared_len);
        assert!(
            decoded.capacity() <= declared_len,
            "capacity {} exceeded declared limit {declared_len}",
            decoded.capacity()
        );
        assert!(decoded.iter().all(|byte| *byte == 0xA5));
    }
}
