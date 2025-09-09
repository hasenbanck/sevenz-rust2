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
    ByteReader, Password, archive::*, bitset::BitSet, block::*, decoder::add_decoder, error::Error,
};

const MAX_MEM_LIMIT_KB: usize = usize::MAX / 1024;

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
            Self::init_archive(reader, start_header, password, true, 1)
        } else {
            Self::try_to_locale_end_header(reader, reader_len, password, 1)
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

    fn read_header<R: Read + Seek>(header: &mut R, archive: &mut Archive) -> Result<(), Error> {
        let mut nid = header.read_u8()?;
        if nid == K_ARCHIVE_PROPERTIES {
            Self::read_archive_properties(header)?;
            nid = header.read_u8()?;
        }

        if nid == K_ADDITIONAL_STREAMS_INFO {
            return Err(Error::other("Additional streams unsupported"));
        }
        if nid == K_MAIN_STREAMS_INFO {
            Self::read_streams_info(header, archive)?;
            nid = header.read_u8()?;
        }
        if nid == K_FILES_INFO {
            Self::read_files_info(header, archive)?;
            nid = header.read_u8()?;
        }
        if nid != K_END {
            return Err(Error::BadTerminatedHeader(nid));
        }

        Ok(())
    }

    fn read_archive_properties<R: Read + Seek>(header: &mut R) -> Result<(), Error> {
        let mut nid = header.read_u8()?;
        while nid != K_END {
            let property_size = read_variable_usize(header, "propertySize")?;
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
                let start_header = StartHeader {
                    next_header_offset: pos - prev_data_size,
                    next_header_size: reader_len - pos,
                    next_header_crc: 0,
                };
                let result =
                    Self::init_archive(reader, start_header, password, false, thread_count)?;

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
    ) -> Result<Self, Error> {
        if start_header.next_header_size > usize::MAX as u64 {
            return Err(Error::other(format!(
                "Cannot handle next_header_size {}",
                start_header.next_header_size
            )));
        }

        let next_header_size_int = start_header.next_header_size as usize;

        reader.seek(SeekFrom::Start(
            SIGNATURE_HEADER_SIZE + start_header.next_header_offset,
        ))?;

        let mut buf = vec![0; next_header_size_int];
        reader.read_exact(&mut buf)?;
        if verify_crc && crc32fast::hash(&buf) as u64 != start_header.next_header_crc {
            return Err(Error::NextHeaderCrcMismatch);
        }

        let mut archive = Archive::default();
        let mut buf_reader = buf.as_slice();
        let mut nid = buf_reader.read_u8()?;
        let mut header = if nid == K_ENCODED_HEADER {
            let (mut out_reader, buf_size) = Self::read_encoded_header(
                &mut buf_reader,
                reader,
                &mut archive,
                password,
                thread_count,
            )?;
            buf.clear();
            buf.resize(buf_size, 0);
            out_reader
                .read_exact(&mut buf)
                .map_err(|e| Error::bad_password(e, !password.is_empty()))?;
            archive = Archive::default();
            buf_reader = buf.as_slice();
            nid = buf_reader.read_u8()?;
            buf_reader
        } else {
            buf_reader
        };
        let mut header = std::io::Cursor::new(&mut header);
        if nid == K_HEADER {
            Self::read_header(&mut header, &mut archive)?;
        } else {
            return Err(Error::other("Broken or unsupported archive: no Header"));
        }

        archive.is_solid = archive
            .blocks
            .iter()
            .any(|block| block.num_unpack_sub_streams > 1);

        Ok(archive)
    }

    fn read_encoded_header<'r, R: Read, RI: 'r + Read + Seek>(
        header: &mut R,
        reader: &'r mut RI,
        archive: &mut Archive,
        password: &Password,
        thread_count: u32,
    ) -> Result<(Box<dyn Read + 'r>, usize), Error> {
        Self::read_streams_info(header, archive)?;
        let block = archive
            .blocks
            .first()
            .ok_or(Error::other("no blocks, can't read encoded header"))?;
        let first_pack_stream_index = 0;
        let block_offset = SIGNATURE_HEADER_SIZE + archive.pack_pos;
        if archive.pack_sizes.is_empty() {
            return Err(Error::other("no packed streams, can't read encoded header"));
        }

        reader.seek(SeekFrom::Start(block_offset))?;
        let coder_len = block.coders.len();
        let unpack_size = block.get_unpack_size() as usize;
        let pack_size = archive.pack_sizes[first_pack_stream_index] as usize;
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
                    password,
                    MAX_MEM_LIMIT_KB,
                    thread_count,
                )?;
                decoder = Box::new(next);
            }
            decoder
        } else {
            decoder
        };
        if block.has_crc {
            decoder = Box::new(Crc32VerifyingReader::new(decoder, unpack_size, block.crc));
        }

        Ok((decoder, unpack_size))
    }

    fn read_streams_info<R: Read>(header: &mut R, archive: &mut Archive) -> Result<(), Error> {
        let mut nid = header.read_u8()?;
        if nid == K_PACK_INFO {
            Self::read_pack_info(header, archive)?;
            nid = header.read_u8()?;
        }

        if nid == K_UNPACK_INFO {
            Self::read_unpack_info(header, archive)?;
            nid = header.read_u8()?;
        } else {
            archive.blocks.clear();
        }
        if nid == K_SUB_STREAMS_INFO {
            Self::read_sub_streams_info(header, archive)?;
            nid = header.read_u8()?;
        }
        if nid != K_END {
            return Err(Error::BadTerminatedStreamsInfo(nid));
        }

        Ok(())
    }

    fn read_files_info<R: Read + Seek>(header: &mut R, archive: &mut Archive) -> Result<(), Error> {
        let num_files = read_variable_usize(header, "num files")?;
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
                    is_empty_stream = Some(read_bits(header, num_files)?);
                }
                K_EMPTY_FILE => {
                    let n = if let Some(s) = &is_empty_stream {
                        s.len()
                    } else {
                        return Err(Error::other(
                            "Header format error: kEmptyStream must appear before kEmptyFile",
                        ));
                    };
                    is_empty_file = Some(read_bits(header, n)?);
                }
                K_ANTI => {
                    let n = if let Some(s) = is_empty_stream.as_ref() {
                        s.len()
                    } else {
                        return Err(Error::other(
                            "Header format error: kEmptyStream must appear before kEmptyFile",
                        ));
                    };
                    is_anti = Some(read_bits(header, n)?);
                }
                K_NAME => {
                    let external = header.read_u8()?;
                    if external != 0 {
                        return Err(Error::other("Not implemented:external != 0"));
                    }
                    if (size - 1) & 1 != 0 {
                        return Err(Error::other("file names length invalid"));
                    }

                    let size = assert_usize(size, "file names length")?;
                    // let mut names = vec![0u8; size - 1];
                    // header.read_exact(&mut names)?;
                    let names_reader = NamesReader::new(header, size - 1);

                    let mut next_file = 0;
                    for s in names_reader {
                        files[next_file].name = s?;
                        next_file += 1;
                    }

                    if next_file != files.len() {
                        return Err(Error::other("Error parsing file names"));
                    }
                }
                K_C_TIME => {
                    let times_defined = read_all_or_bits(header, num_files)?;
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
                    let times_defined = read_all_or_bits(header, num_files)?;
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
                    let times_defined = read_all_or_bits(header, num_files)?;
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
                    let times_defined = read_all_or_bits(header, num_files)?;
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
                    header.seek(SeekFrom::Current(size as i64))?;
                }
                _ => {
                    header.seek(SeekFrom::Current(size as i64))?;
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
                file.has_crc = sub_stream_info.has_crc.contains(non_empty_file_counter);
                file.crc = sub_stream_info.crcs[non_empty_file_counter];
                file.size = sub_stream_info.unpack_sizes[non_empty_file_counter];
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

        Self::calculate_stream_map(archive)?;
        Ok(())
    }

    fn calculate_stream_map(archive: &mut Archive) -> Result<(), Error> {
        let mut stream_map = StreamMap::default();

        let mut next_block_pack_stream_index = 0;
        let num_blocks = archive.blocks.len();
        stream_map.block_first_pack_stream_index = vec![0; num_blocks];
        for i in 0..num_blocks {
            stream_map.block_first_pack_stream_index[i] = next_block_pack_stream_index;
            next_block_pack_stream_index += archive.blocks[i].packed_streams.len();
        }

        let mut next_pack_stream_offset = 0;
        let num_pack_sizes = archive.pack_sizes.len();
        stream_map.pack_stream_offsets = vec![0; num_pack_sizes];
        for i in 0..num_pack_sizes {
            stream_map.pack_stream_offsets[i] = next_pack_stream_offset;
            next_pack_stream_offset += archive.pack_sizes[i];
        }

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
                let pack_size = archive.pack_sizes[first_pack_stream_index];

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

        archive.stream_map = stream_map;
        Ok(())
    }

    fn read_pack_info<R: Read>(header: &mut R, archive: &mut Archive) -> Result<(), Error> {
        archive.pack_pos = read_variable_u64(header)?;
        let num_pack_streams = read_variable_usize(header, "num pack streams")?;
        let mut nid = header.read_u8()?;
        if nid == K_SIZE {
            archive.pack_sizes = vec![0u64; num_pack_streams];
            for i in 0..archive.pack_sizes.len() {
                archive.pack_sizes[i] = read_variable_u64(header)?;
            }
            nid = header.read_u8()?;
        }

        if nid == K_CRC {
            archive.pack_crcs_defined = read_all_or_bits(header, num_pack_streams)?;
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
    fn read_unpack_info<R: Read>(header: &mut R, archive: &mut Archive) -> Result<(), Error> {
        let nid = header.read_u8()?;
        if nid != K_FOLDER {
            return Err(Error::other(format!("Expected kFolder, got {nid}")));
        }
        let num_blocks = read_variable_usize(header, "num blocks")?;

        archive.blocks.reserve_exact(num_blocks);
        let external = header.read_u8()?;
        if external != 0 {
            return Err(Error::ExternalUnsupported);
        }

        for _ in 0..num_blocks {
            archive.blocks.push(Self::read_block(header)?);
        }

        let nid = header.read_u8()?;
        if nid != K_CODERS_UNPACK_SIZE {
            return Err(Error::other(format!(
                "Expected kCodersUnpackSize, got {nid}"
            )));
        }

        for block in archive.blocks.iter_mut() {
            let tos = block.total_output_streams;
            block.unpack_sizes.reserve_exact(tos);
            for _ in 0..tos {
                block.unpack_sizes.push(read_variable_u64(header)?);
            }
        }

        let mut nid = header.read_u8()?;
        if nid == K_CRC {
            let crcs_defined = read_all_or_bits(header, num_blocks)?;
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

    fn read_sub_streams_info<R: Read>(header: &mut R, archive: &mut Archive) -> Result<(), Error> {
        for block in archive.blocks.iter_mut() {
            block.num_unpack_sub_streams = 1;
        }
        let mut total_unpack_streams = archive.blocks.len();

        let mut nid = header.read_u8()?;
        if nid == K_NUM_UNPACK_STREAM {
            total_unpack_streams = 0;
            for block in archive.blocks.iter_mut() {
                let num_streams = read_variable_usize(header, "numStreams")?;
                block.num_unpack_sub_streams = num_streams;
                total_unpack_streams += num_streams;
            }
            nid = header.read_u8()?;
        }

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
            let mut sum = 0;
            if nid == K_SIZE {
                for _i in 0..block.num_unpack_sub_streams - 1 {
                    let size = read_variable_u64(header)?;
                    sub_streams_info.unpack_sizes[next_unpack_stream] = size;
                    next_unpack_stream += 1;
                    sum += size;
                }
            }
            if sum > block.get_unpack_size() {
                return Err(Error::other(
                    "sum of unpack sizes of block exceeds total unpack size",
                ));
            }
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
            let has_missing_crc = read_all_or_bits(header, num_digests)?;
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

    fn read_block<R: Read>(header: &mut R) -> Result<Block, Error> {
        let mut block = Block::default();

        let num_coders = read_variable_usize(header, "num coders")?;
        let mut coders = Vec::with_capacity(num_coders);
        let mut total_in_streams = 0;
        let mut total_out_streams = 0;
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
            total_in_streams += coder.num_in_streams;
            total_out_streams += coder.num_out_streams;
            if has_attributes {
                let properties_size = read_variable_usize(header, "properties size")?;
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
        let total_in_streams = assert_usize(total_in_streams, "totalInStreams")?;
        let total_out_streams = assert_usize(total_out_streams, "totalOutStreams")?;
        block.total_input_streams = total_in_streams;
        block.total_output_streams = total_out_streams;

        if total_out_streams == 0 {
            return Err(Error::other("Total output streams can't be 0"));
        }
        let num_bind_pairs = total_out_streams - 1;
        let mut bind_pairs = Vec::with_capacity(num_bind_pairs);
        for _ in 0..num_bind_pairs {
            let bp = BindPair {
                in_index: read_variable_u64(header)?,
                out_index: read_variable_u64(header)?,
            };
            bind_pairs.push(bp);
        }
        block.bind_pairs = bind_pairs;

        if total_in_streams < num_bind_pairs {
            return Err(Error::other(
                "Total input streams can't be less than the number of bind pairs",
            ));
        }
        let num_packed_streams = total_in_streams - num_bind_pairs;
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

#[inline]
fn read_variable_usize<R: Read>(reader: &mut R, field: &str) -> Result<usize, Error> {
    let size = read_variable_u64(reader)?;
    assert_usize(size, field)
}

#[inline]
fn assert_usize(size: u64, field: &str) -> Result<usize, Error> {
    if size > usize::MAX as u64 {
        return Err(Error::other(format!("Cannot handle {field} {size}")));
    }
    Ok(size as usize)
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

fn read_all_or_bits<R: Read>(header: &mut R, size: usize) -> io::Result<BitSet> {
    let all = header.read_u8()?;
    if all != 0 {
        let mut bits = BitSet::with_capacity(size);
        for i in 0..size {
            bits.insert(i);
        }
        Ok(bits)
    } else {
        read_bits(header, size)
    }
}

fn read_bits<R: Read>(header: &mut R, size: usize) -> io::Result<BitSet> {
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

struct NamesReader<'a, R: Read> {
    max_bytes: usize,
    read_bytes: usize,
    cache: Vec<u16>,
    reader: &'a mut R,
}

impl<'a, R: Read> NamesReader<'a, R> {
    fn new(reader: &'a mut R, max_bytes: usize) -> Self {
        Self {
            max_bytes,
            reader,
            read_bytes: 0,
            cache: Vec::with_capacity(16),
        }
    }
}

impl<R: Read> Iterator for NamesReader<'_, R> {
    type Item = Result<String, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.max_bytes <= self.read_bytes {
            return None;
        }
        self.cache.clear();
        let mut buf = [0; 2];
        while self.read_bytes < self.max_bytes {
            let r = self.reader.read_exact(&mut buf);
            self.read_bytes += 2;
            if let Err(e) = r {
                return Some(Err(e.into()));
            }
            let u = u16::from_le_bytes(buf);
            if u == 0 {
                break;
            }
            self.cache.push(u);
        }

        Some(String::from_utf16(&self.cache).map_err(|e| Error::other(e.to_string())))
    }
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
    pub fn new(mut source: R, password: Password) -> Result<Self, Error> {
        let archive = Archive::read(&mut source, &password)?;

        let mut reader = Self {
            source,
            archive,
            password,
            thread_count: 1,
            index: HashMap::default(),
        };

        reader.fill_index();

        let thread_count =
            std::thread::available_parallelism().unwrap_or(NonZeroUsize::new(1).unwrap());
        reader.set_thread_count(thread_count.get() as u32);

        Ok(reader)
    }

    /// Creates an [`ArchiveReader`] from an existing [`Archive`] instance.
    ///
    /// This is useful when you already have a parsed archive and want to create a reader
    /// without re-parsing the archive structure.
    ///
    /// # Arguments
    /// * `archive` - An existing parsed archive instance
    /// * `source` - The reader providing access to the archive data
    /// * `password` - Password for encrypted archives
    #[inline]
    pub fn from_archive(archive: Archive, source: R, password: Password) -> Self {
        let mut reader = Self {
            source,
            archive,
            password,
            thread_count: 1,
            index: HashMap::default(),
        };

        reader.fill_index();

        let thread_count =
            std::thread::available_parallelism().unwrap_or(NonZeroUsize::new(1).unwrap());
        reader.set_thread_count(thread_count.get() as u32);

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
        thread_count: u32,
    ) -> Result<(Box<dyn Read + 'r>, usize), Error> {
        let block = &archive.blocks[block_index];
        if block.total_input_streams > block.total_output_streams {
            return Self::build_decode_stack2(source, archive, block_index, password, thread_count);
        }
        let first_pack_stream_index = archive.stream_map.block_first_pack_stream_index[block_index];
        let block_offset = SIGNATURE_HEADER_SIZE
            + archive.pack_pos
            + archive.stream_map.pack_stream_offsets[first_pack_stream_index];

        source.seek(SeekFrom::Start(block_offset))?;
        let pack_size = archive.pack_sizes[first_pack_stream_index] as usize;

        let mut decoder: Box<dyn Read> = Box::new(BoundedReader::new(source, pack_size));
        let block = &archive.blocks[block_index];
        for (index, coder) in block.ordered_coder_iter() {
            if coder.num_in_streams != 1 || coder.num_out_streams != 1 {
                return Err(Error::unsupported(
                    "Multi input/output stream coders are not yet supported",
                ));
            }
            let next = add_decoder(
                decoder,
                block.get_unpack_size_at_index(index) as usize,
                coder,
                password,
                MAX_MEM_LIMIT_KB,
                thread_count,
            )?;
            decoder = Box::new(next);
        }
        if block.has_crc {
            decoder = Box::new(Crc32VerifyingReader::new(
                decoder,
                block.get_unpack_size() as usize,
                block.crc,
            ));
        }

        Ok((decoder, pack_size))
    }

    fn build_decode_stack2<'r>(
        source: &'r mut R,
        archive: &Archive,
        block_index: usize,
        password: &Password,
        thread_count: u32,
    ) -> Result<(Box<dyn Read + 'r>, usize), Error> {
        const MAX_CODER_COUNT: usize = 32;
        let block = &archive.blocks[block_index];
        if block.coders.len() > MAX_CODER_COUNT {
            return Err(Error::unsupported(format!(
                "Too many coders: {}",
                block.coders.len()
            )));
        }

        assert!(block.total_input_streams > block.total_output_streams);
        let shared_source = Rc::new(RefCell::new(source));
        let first_pack_stream_index = archive.stream_map.block_first_pack_stream_index[block_index];
        let start_pos = SIGNATURE_HEADER_SIZE + archive.pack_pos;
        let offsets = &archive.stream_map.pack_stream_offsets[first_pack_stream_index..];

        let mut sources = Vec::with_capacity(block.packed_streams.len());

        for (i, offset) in offsets[..block.packed_streams.len()].iter().enumerate() {
            let pack_pos = start_pos + offset;
            let pack_size = archive.pack_sizes[first_pack_stream_index + i];

            let pack_reader = SharedBoundedReader::new(
                Rc::clone(&shared_source),
                (pack_pos, pack_pos + pack_size),
            );

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
                coder_used[bp.out_index as usize] = true;
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

        let id = block.coders[main_coder_index].encoder_method_id();
        if id != EncoderMethod::ID_BCJ2 {
            return Err(Error::unsupported(format!("Unsupported method: {id:?}")));
        }

        let num_in_streams = block.coders[main_coder_index].num_in_streams as usize;
        let mut inputs: Vec<Box<dyn Read>> = Vec::with_capacity(num_in_streams);
        let start_i = coder_to_stream_map[main_coder_index];
        for i in start_i..num_in_streams + start_i {
            inputs.push(Self::get_in_stream(
                block,
                &sources,
                &coder_to_stream_map,
                password,
                i,
                thread_count,
            )?);
        }
        let mut decoder: Box<dyn Read> = Box::new(Bcj2Reader::new(inputs, block.get_unpack_size()));
        if block.has_crc {
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
        sources: &[SharedBoundedReader<'r, R>],
        coder_to_stream_map: &[usize],
        password: &Password,
        in_stream_index: usize,
        thread_count: u32,
    ) -> Result<Box<dyn Read + 'r>, Error>
    where
        R: 'r,
    {
        if let Some(index) = block
            .packed_streams
            .iter()
            .position(|&i| i == in_stream_index as u64)
        {
            return Ok(Box::new(sources[index].clone()));
        }

        let bp = block
            .find_bind_pair_for_in_stream(in_stream_index as u64)
            .ok_or_else(|| {
                Error::other(format!(
                    "Couldn't find bind pair for stream {in_stream_index}"
                ))
            })?;
        let out_stream_index = bp.out_index as usize;

        let coder = &block.coders[out_stream_index];
        let start_index = coder_to_stream_map[out_stream_index];
        if start_index == usize::MAX {
            return Err(Error::other("in_stream_index out of range"));
        }
        let uncompressed_len = block.unpack_sizes[out_stream_index] as usize;
        if coder.num_in_streams == 1 {
            let input = Self::get_in_stream(
                block,
                sources,
                coder_to_stream_map,
                password,
                start_index,
                thread_count,
            )?;

            let decoder = add_decoder(
                input,
                uncompressed_len,
                coder,
                password,
                MAX_MEM_LIMIT_KB,
                thread_count,
            )?;
            return Ok(Box::new(decoder));
        }
        Err(Error::unsupported(
            "Multi input stream coders are not yet supported",
        ))
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
            let forder_dec = BlockDecoder::new(
                self.thread_count,
                block_index,
                &self.archive,
                &self.password,
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
        let index_entry = *self.index.get(name).ok_or(Error::FileNotFound)?;
        let file = &self.archive.files[index_entry.file_index];

        if !file.has_stream {
            return Ok(Vec::new());
        }

        let block_index = index_entry
            .block_index
            .ok_or_else(|| Error::other("File has no associated block"))?;

        match self.archive.is_solid {
            true => {
                let mut result = None;
                let target_file_ptr = file as *const _;

                BlockDecoder::new(
                    self.thread_count,
                    block_index,
                    &self.archive,
                    &self.password,
                    &mut self.source,
                )
                .for_each_entries(&mut |archive_entry, reader| {
                    let mut data = Vec::with_capacity(archive_entry.size as usize);
                    reader.read_to_end(&mut data)?;

                    if std::ptr::eq(archive_entry, target_file_ptr) {
                        result = Some(data);
                        Ok(false)
                    } else {
                        Ok(true)
                    }
                })?;

                result.ok_or(Error::FileNotFound)
            }
            false => {
                let pack_index = self.archive.stream_map.block_first_pack_stream_index[block_index];
                let pack_offset = self.archive.stream_map.pack_stream_offsets[pack_index];
                let block_offset = SIGNATURE_HEADER_SIZE + self.archive.pack_pos + pack_offset;

                self.source.seek(SeekFrom::Start(block_offset))?;

                let (mut block_reader, _size) = Self::build_decode_stack(
                    &mut self.source,
                    &self.archive,
                    block_index,
                    &self.password,
                    self.thread_count,
                )?;

                let mut data = Vec::with_capacity(file.size as usize);
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
        let index_entry = self.index.get(file_name).ok_or(Error::FileNotFound)?;
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
        Self {
            thread_count,
            block_index,
            archive,
            password,
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
            source,
        } = self;
        let (mut block_reader, _size) = ArchiveReader::build_decode_stack(
            source,
            archive,
            block_index,
            password,
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
