use crate::Error;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{self, Cursor, Read};

const BROTLI_ZSTDMT_MAGIC: u32 = 0x184D2A50;
/// "BR" in little-endian
const BROTLI_MAGIC: u16 = 0x5242;

pub struct BrotliDecoder<R: Read> {
    inner: brotli::Decompressor<InnerReader<R>>,
}

impl<R: Read> BrotliDecoder<R> {
    pub fn new(mut input: R, buffer_size: usize) -> Result<Self, Error> {
        let mut header = [0u8; 16];
        let header_read = match Read::read(&mut input, &mut header) {
            Ok(n) if n >= 4 => n,
            Ok(_) => return Err(Error::other("Input too short")),
            Err(e) => return Err(Error::io(e)),
        };

        let zstdmt_magic = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);

        if zstdmt_magic == BROTLI_ZSTDMT_MAGIC && header_read >= 16 {
            // This is a zstdmt Brotli stream
            let skippable_size = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
            if skippable_size != 8 {
                return Err(Error::other("Invalid zstdmt brotli skippable size"));
            }

            let compressed_size =
                u32::from_le_bytes([header[8], header[9], header[10], header[11]]);

            let brotli_magic = u16::from_le_bytes([header[12], header[13]]);
            if brotli_magic != BROTLI_MAGIC {
                return Err(Error::other("Invalid zstdmt brotli magic"));
            }

            let inner_reader = InnerReader::new_zstdmt(input, compressed_size);
            let decompressor = brotli::Decompressor::new(inner_reader, buffer_size);

            Ok(BrotliDecoder {
                inner: decompressor,
            })
        } else {
            // This is a standard Brotli stream
            let inner_reader = InnerReader::new_standard(input, header[..header_read].to_vec());
            let decompressor = brotli::Decompressor::new(inner_reader, buffer_size);

            Ok(BrotliDecoder {
                inner: decompressor,
            })
        }
    }
}

impl<R: Read> Read for BrotliDecoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

enum InnerReader<R: Read> {
    Standard {
        reader: R,
        header_buffer: Cursor<Vec<u8>>,
        header_first: bool,
    },
    Zstdmt {
        reader: R,
        remaining_in_frame: u32,
        frame_finished: bool,
    },
}

impl<R: Read> InnerReader<R> {
    fn new_standard(reader: R, header: Vec<u8>) -> Self {
        InnerReader::Standard {
            reader,
            header_buffer: Cursor::new(header),
            header_first: true,
        }
    }

    fn new_zstdmt(reader: R, remaining_in_frame: u32) -> Self {
        InnerReader::Zstdmt {
            reader,
            remaining_in_frame,
            frame_finished: false,
        }
    }

    fn read_next_frame_header(
        reader: &mut R,
        remaining_in_frame: &mut u32,
        frame_finished: &mut bool,
    ) -> io::Result<bool> {
        match reader.read_u32::<LittleEndian>() {
            Ok(magic) => {
                if magic != BROTLI_ZSTDMT_MAGIC {
                    return Ok(false);
                }

                let skippable_size = reader.read_u32::<LittleEndian>()?;
                if skippable_size != 8 {
                    return Ok(false);
                }

                let compressed_size = reader.read_u32::<LittleEndian>()?;

                let brotli_magic = reader.read_u16::<LittleEndian>()?;
                if brotli_magic != BROTLI_MAGIC {
                    return Ok(false);
                }

                let _uncompressed_hint = reader.read_u16::<LittleEndian>()?;

                *remaining_in_frame = compressed_size;
                *frame_finished = false;

                Ok(true)
            }
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => Ok(false),
            Err(e) => Err(e),
        }
    }
}

impl<R: Read> Read for InnerReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            InnerReader::Standard {
                reader,
                header_buffer,
                header_first,
            } => {
                if *header_first {
                    let bytes_from_header = header_buffer.read(buf)?;
                    if bytes_from_header == 0 {
                        *header_first = false;
                        reader.read(buf)
                    } else {
                        Ok(bytes_from_header)
                    }
                } else {
                    reader.read(buf)
                }
            }
            InnerReader::Zstdmt {
                reader,
                remaining_in_frame,
                frame_finished,
            } => {
                if *frame_finished {
                    match Self::read_next_frame_header(reader, remaining_in_frame, frame_finished) {
                        Ok(true) => {}
                        Ok(false) => return Ok(0),
                        Err(e) => return Err(e),
                    }
                }

                if *remaining_in_frame > 0 {
                    let to_read = std::cmp::min(buf.len() as u32, *remaining_in_frame) as usize;

                    let bytes_read = reader.read(&mut buf[..to_read])?;
                    *remaining_in_frame -= bytes_read as u32;

                    if *remaining_in_frame == 0 {
                        *frame_finished = true;
                    }

                    Ok(bytes_read)
                } else {
                    *frame_finished = true;
                    Ok(0)
                }
            }
        }
    }
}
