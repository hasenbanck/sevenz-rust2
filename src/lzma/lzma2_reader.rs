use std::io::{self, Read};

use lzma_sdk_sys::{
    Allocator, Byte, CLzma2Dec, ELzmaFinishMode, ELzmaStatus, Lzma2Dec_Allocate,
    Lzma2Dec_DecodeToBuf, Lzma2Dec_Init, SizeT, SZ_OK,
};

use super::error::check_res;

const INPUT_BUF_SIZE: usize = 1 << 16; // 64 KiB

/// Safe streaming LZMA2 decoder wrapping lzma-sdk-sys.
pub struct Lzma2Reader<R: Read> {
    inner: R,
    state: Box<CLzma2Dec>,
    alloc: Allocator,
    input_buf: Vec<u8>,
    input_pos: usize,
    input_len: usize,
    finished: bool,
}

impl<R: Read> Lzma2Reader<R> {
    /// Creates a new LZMA2 decoder.
    ///
    /// `dict_size` is the dictionary size. The `prop` byte encodes it according to
    /// the LZMA2 spec. Pass `_memlimit` for API compatibility (currently unused).
    pub fn new(inner: R, dict_size: u32, _memlimit: Option<usize>) -> Self {
        let alloc = Allocator::default();
        let mut state = Box::new(CLzma2Dec::default());

        let prop = dict_size_to_lzma2_prop(dict_size);

        let res = unsafe { Lzma2Dec_Allocate(&mut *state, prop, alloc.as_ref()) };
        if res != SZ_OK as i32 {
            panic!("Lzma2Dec_Allocate failed with code {res}");
        }

        unsafe { Lzma2Dec_Init(&mut *state) };

        Self {
            inner,
            state,
            alloc,
            input_buf: vec![0u8; INPUT_BUF_SIZE],
            input_pos: 0,
            input_len: 0,
            finished: false,
        }
    }
}

/// Convert a dictionary size to the LZMA2 property byte.
fn dict_size_to_lzma2_prop(dict_size: u32) -> u8 {
    if dict_size == 0xFFFFFFFF {
        return 40;
    }
    let mut d: u32 = 2;
    let mut prop: u8 = 0;
    while prop < 40 {
        if d >= dict_size {
            break;
        }
        d += d >> 1;
        prop += 1;
        if d >= dict_size {
            break;
        }
        d <<= 1;
        prop += 1;
    }
    prop
}

impl<R: Read> Read for Lzma2Reader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.finished || buf.is_empty() {
            return Ok(0);
        }

        loop {
            //TODO место для continue флага
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

            let res = unsafe {
                Lzma2Dec_DecodeToBuf(
                    &mut *self.state,
                    buf.as_mut_ptr() as *mut Byte,
                    &mut dest_len,
                    src.as_ptr() as *const Byte,
                    &mut src_len,
                    ELzmaFinishMode::LZMA_FINISH_ANY,
                    &mut status,
                )
            };
            check_res(res)?;

            self.input_pos += src_len;

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

impl<R: Read> Drop for Lzma2Reader<R> {
    fn drop(&mut self) {
        // Lzma2Dec_Free is a C macro: LzmaDec_Free(&p->decoder, alloc)
        unsafe {
            lzma_sdk_sys::LzmaDec_Free(&mut self.state.decoder, self.alloc.as_ref());
        }
    }
}

/// Multithreaded LZMA2 decoder.
///
/// LZMA-SDK does not natively support MT decoding, so this is a thin wrapper
/// around the single-threaded decoder. The ASM-optimized single-threaded decode
/// from LZMA-SDK is typically faster than pure-Rust MT decode.
pub struct Lzma2ReaderMt<R: Read> {
    inner: Lzma2Reader<R>,
}

impl<R: Read> Lzma2ReaderMt<R> {
    pub fn new(inner: R, dict_size: u32, memlimit: Option<usize>, _threads: u32) -> Self {
        Self {
            inner: Lzma2Reader::new(inner, dict_size, memlimit),
        }
    }
}

impl<R: Read> Read for Lzma2ReaderMt<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}
