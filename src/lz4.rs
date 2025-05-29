use crate::Error;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};

/// Magic bytes of a skippable frame as used in L4 by zstdmt.
const SKIPPABLE_FRAME_MAGIC: u32 = 0x184D2A50;

/// Custom decoder to support the custom format first implemented by zstdmt, which allows to have
/// optional skippable frames.
pub(crate) struct Lz4Decoder<R: Read> {
    inner: Option<lz4_flex::frame::FrameDecoder<InnerReader<R>>>,
}

impl<R: Read> Lz4Decoder<R> {
    pub(crate) fn new(mut input: R) -> Result<Self, Error> {
        let mut header = [0u8; 12];
        let header_read = match Read::read(&mut input, &mut header) {
            Ok(n) if n >= 4 => n,
            Ok(_) => return Err(Error::other("Input too short")),
            Err(e) => return Err(Error::io(e)),
        };

        let magic_value = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);

        let inner_reader = if magic_value == SKIPPABLE_FRAME_MAGIC && header_read >= 12 {
            let skippable_size = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
            if skippable_size != 4 {
                return Err(Error::other("Invalid lz4 skippable frame size"));
            }

            let compressed_size =
                u32::from_le_bytes([header[8], header[9], header[10], header[11]]);

            InnerReader::new_skippable(input, compressed_size)
        } else {
            InnerReader::new_standard(input, header[..header_read].to_vec())
        };

        let decoder = lz4_flex::frame::FrameDecoder::new(inner_reader);

        Ok(Lz4Decoder {
            inner: Some(decoder),
        })
    }
}

impl<R: Read> Read for Lz4Decoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if let Some(inner) = &mut self.inner {
            match inner.read(buf) {
                Ok(0) => {
                    let inner_reader = inner.get_mut();

                    if inner_reader.read_next_frame_header()? {
                        let reader = std::mem::replace(inner_reader, InnerReader::empty());
                        let mut decompressor = lz4_flex::frame::FrameDecoder::new(reader);
                        let result = decompressor.read(buf);
                        self.inner = Some(decompressor);
                        result
                    } else {
                        self.inner = None;
                        Ok(0)
                    }
                }
                result => result,
            }
        } else {
            Ok(0)
        }
    }
}

enum InnerReader<R: Read> {
    Empty,
    Standard {
        reader: R,
        header_buffer: Cursor<Vec<u8>>,
        header_finished: bool,
    },
    Skippable {
        reader: R,
        remaining_in_frame: u32,
        frame_finished: bool,
    },
}

impl<R: Read> InnerReader<R> {
    fn empty() -> Self {
        InnerReader::Empty
    }

    fn new_standard(reader: R, header: Vec<u8>) -> Self {
        InnerReader::Standard {
            reader,
            header_buffer: Cursor::new(header),
            header_finished: false,
        }
    }

    fn new_skippable(reader: R, remaining_in_frame: u32) -> Self {
        InnerReader::Skippable {
            reader,
            remaining_in_frame,
            frame_finished: false,
        }
    }

    fn read_next_frame_header(&mut self) -> std::io::Result<bool> {
        match self {
            InnerReader::Empty => Ok(false),
            InnerReader::Standard { .. } => Ok(false),
            InnerReader::Skippable {
                reader,
                remaining_in_frame,
                frame_finished,
            } => {
                if !*frame_finished {
                    return Ok(false);
                }

                match reader.read_u32::<LittleEndian>() {
                    Ok(magic) => {
                        if magic != SKIPPABLE_FRAME_MAGIC {
                            return Ok(false);
                        }

                        let skippable_size = reader.read_u32::<LittleEndian>()?;
                        if skippable_size != 4 {
                            return Ok(false);
                        }

                        let compressed_size = reader.read_u32::<LittleEndian>()?;

                        *remaining_in_frame = compressed_size;
                        *frame_finished = false;

                        Ok(true)
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Ok(false),
                    Err(e) => Err(e),
                }
            }
        }
    }
}

impl<R: Read> Read for InnerReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            InnerReader::Empty => Ok(0),
            InnerReader::Standard {
                reader,
                header_buffer,
                header_finished,
            } => {
                if !*header_finished {
                    let bytes_read = header_buffer.read(buf)?;
                    if bytes_read > 0 {
                        return Ok(bytes_read);
                    }
                    *header_finished = true;
                }
                reader.read(buf)
            }
            InnerReader::Skippable {
                reader,
                remaining_in_frame,
                frame_finished,
            } => {
                if *frame_finished || *remaining_in_frame == 0 {
                    return Ok(0);
                }

                let bytes_to_read = std::cmp::min(*remaining_in_frame as usize, buf.len());
                let bytes_read = reader.read(&mut buf[..bytes_to_read])?;

                if bytes_read == 0 {
                    *frame_finished = true;
                    return Ok(0);
                }

                *remaining_in_frame -= bytes_read as u32;
                if *remaining_in_frame == 0 {
                    *frame_finished = true;
                }

                Ok(bytes_read)
            }
        }
    }
}
