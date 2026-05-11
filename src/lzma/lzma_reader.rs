use std::io::{self, Read};

use lzma_sdk_sys::{
    Allocator, Byte, CLzmaDec, ELzmaFinishMode, ELzmaStatus, LzmaDec_Allocate,
    LzmaDec_DecodeToBuf, LzmaDec_Free, LzmaDec_Init, SizeT,
};

use super::error::check_res;

const INPUT_BUF_SIZE: usize = 1 << 16; // 64 KiB

/// Safe streaming LZMA decoder wrapping lzma-sdk-sys.
///
/// Implements `Read` by incrementally decoding data from the inner reader.
pub struct LzmaReader<R: Read> {
    inner: R,
    state: CLzmaDec,
    alloc: Allocator,
    input_buf: Vec<u8>,
    input_pos: usize,
    input_len: usize,
    finished: bool,
    uncompressed_size: u64,
    decoded_count: u64,
}

impl<R: Read> LzmaReader<R> {
    /// Creates a new LZMA decoder.
    ///
    /// `props` is the LZMA properties byte, `dict_size` is the dictionary size.
    /// `uncompressed_size` is the expected output size, or `u64::MAX` if unknown.
    pub fn new_with_props(
        inner: R,
        uncompressed_size: u64,
        props_byte: u8,
        dict_size: u32,
        _memlimit: Option<usize>,
    ) -> io::Result<Self> {
        let alloc = Allocator::default();
        let mut state = CLzmaDec::default();

        // Encode props in the 5-byte LZMA header format:
        // byte 0 = props_byte, bytes 1-4 = dict_size (LE)
        let mut props_data = [0u8; 5];
        props_data[0] = props_byte;
        props_data[1..5].copy_from_slice(&dict_size.to_le_bytes());

        let res = unsafe {
            LzmaDec_Allocate(
                &mut state,
                props_data.as_ptr() as *const Byte,
                5,
                alloc.as_ref(),
            )
        };
        check_res(res)?;

        unsafe { LzmaDec_Init(&mut state) };

        Ok(Self {
            inner,
            state,
            alloc,
            input_buf: vec![0u8; INPUT_BUF_SIZE],
            input_pos: 0,
            input_len: 0,
            finished: false,
            uncompressed_size,
            decoded_count: 0,
        })
    }
}

impl<R: Read> Read for LzmaReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.finished || buf.is_empty() {
            return Ok(0);
        }

        if self.uncompressed_size != u64::MAX && self.decoded_count >= self.uncompressed_size {
            self.finished = true;
            return Ok(0);
        }

        loop {
            //TODO Место для continue флага
            if self.input_pos >= self.input_len {
                let n = self.inner.read(&mut self.input_buf)?;
                if n == 0 {
                    self.finished = true;
                    return Ok(0);
                }
                self.input_pos = 0;
                self.input_len = n;
            }

            let src = &self.input_buf[self.input_pos..self.input_len];
            let mut src_len: SizeT = src.len();
            let mut dest_len: SizeT = buf.len();
            let mut status = ELzmaStatus::LZMA_STATUS_NOT_SPECIFIED;

            let finish_mode = if self.uncompressed_size != u64::MAX {
                let remaining = self.uncompressed_size - self.decoded_count;
                if (dest_len as u64) >= remaining {
                    dest_len = remaining as SizeT;
                    ELzmaFinishMode::LZMA_FINISH_END
                } else {
                    ELzmaFinishMode::LZMA_FINISH_ANY
                }
            } else {
                ELzmaFinishMode::LZMA_FINISH_ANY
            };

            let res = unsafe {
                LzmaDec_DecodeToBuf(
                    &mut self.state,
                    buf.as_mut_ptr() as *mut Byte,
                    &mut dest_len,
                    src.as_ptr() as *const Byte,
                    &mut src_len,
                    finish_mode,
                    &mut status,
                )
            };
            check_res(res)?;

            self.input_pos += src_len;
            self.decoded_count += dest_len as u64;

            if dest_len > 0 {
                return Ok(dest_len);
            }

            match status {
                ELzmaStatus::LZMA_STATUS_FINISHED_WITH_MARK
                | ELzmaStatus::LZMA_STATUS_MAYBE_FINISHED_WITHOUT_MARK => {
                    self.finished = true;
                    return Ok(0);
                }
                ELzmaStatus::LZMA_STATUS_NEEDS_MORE_INPUT => {
                    self.input_pos = self.input_len;
                    continue;
                }
                _ => {
                    if src_len == 0 && dest_len == 0 {
                        self.finished = true;
                        return Ok(0);
                    }
                }
            }
        }
    }
}

impl<R: Read> Drop for LzmaReader<R> {
    fn drop(&mut self) {
        unsafe {
            LzmaDec_Free(&mut self.state, self.alloc.as_ref());
        }
    }
}
