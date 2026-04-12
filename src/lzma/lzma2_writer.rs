use std::ffi::c_void;
use std::io::{self, Write};

use lzma_sdk_sys::{
    Allocator, Byte, ISeqOutStream_, ISeqOutStreamPtr, Lzma2Enc_Create, Lzma2Enc_Destroy,
    Lzma2Enc_Encode2, Lzma2Enc_SetProps, SZ_OK,
};

use super::error::check_res;
use super::options::Lzma2Options;

/// Adapter that bridges `ISeqOutStream_` (C vtable) to a Rust `Write` impl.
///
/// Placed in a `#[repr(C)]` struct so that a pointer to `vtable` is also
/// a valid `ISeqOutStreamPtr`.
#[repr(C)]
struct OutStreamAdapter<W: Write> {
    vtable: ISeqOutStream_,
    writer: *mut W,
    error: Option<io::Error>,
}

unsafe extern "C" fn out_stream_write<W: Write>(
    p: ISeqOutStreamPtr,
    buf: *const c_void,
    size: usize,
) -> usize {
    unsafe {
        let adapter = &mut *(p as *mut OutStreamAdapter<W>);
        let data = std::slice::from_raw_parts(buf as *const u8, size);
        match (*adapter.writer).write_all(data) {
            Ok(()) => size,
            Err(e) => {
                adapter.error = Some(e);
                0
            }
        }
    }
}

/// Safe streaming LZMA2 encoder wrapping lzma-sdk-sys.
pub struct Lzma2Writer<W: Write> {
    inner: Option<W>,
    options: Lzma2Options,
    alloc: Allocator,
    buffer: Vec<u8>,
}

impl<W: Write> Lzma2Writer<W> {
    pub fn new(inner: W, options: Lzma2Options) -> Self {
        Self {
            inner: Some(inner),
            options,
            alloc: Allocator::default(),
            buffer: Vec::new(),
        }
    }

    pub fn finish(mut self) -> io::Result<W> {
        self.do_encode()?;
        Ok(self.inner.take().unwrap())
    }

    fn do_encode(&mut self) -> io::Result<()> {
        let writer = self.inner.as_mut().unwrap();
        let alloc = &self.alloc;

        let enc = unsafe { Lzma2Enc_Create(alloc.as_ref(), alloc.as_ref()) };
        if enc.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::OutOfMemory,
                "LZMA2: failed to create encoder",
            ));
        }

        let mut props = self.options.props;
        props.numTotalThreads = 1;
        props.numBlockThreads_Max = 1;

        let res = unsafe { Lzma2Enc_SetProps(enc, &mut props) };
        if res != SZ_OK as i32 {
            unsafe { Lzma2Enc_Destroy(enc) };
            return Err(super::error::res_to_io_error(res));
        }

        let mut adapter = OutStreamAdapter::<W> {
            vtable: ISeqOutStream_ {
                Write: Some(out_stream_write::<W>),
            },
            writer: writer as *mut W,
            error: None,
        };

        let out_stream_ptr = &mut adapter.vtable as *mut ISeqOutStream_ as ISeqOutStreamPtr;

        let res = unsafe {
            Lzma2Enc_Encode2(
                enc,
                out_stream_ptr,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                self.buffer.as_ptr() as *const Byte,
                self.buffer.len(),
                std::ptr::null_mut(),
            )
        };

        unsafe { Lzma2Enc_Destroy(enc) };

        if let Some(e) = adapter.error.take() {
            return Err(e);
        }
        check_res(res)?;

        self.buffer.clear();
        Ok(())
    }
}

impl<W: Write> Write for Lzma2Writer<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if let Some(w) = self.inner.as_mut() {
            w.flush()
        } else {
            Ok(())
        }
    }
}

/// Multithreaded LZMA2 encoder wrapping lzma-sdk-sys.
///
/// Uses `CLzma2EncProps.numTotalThreads` for native MT support in LZMA-SDK.
pub struct Lzma2WriterMt<W: Write> {
    inner: Option<W>,
    options: Lzma2Options,
    threads: u32,
    alloc: Allocator,
    buffer: Vec<u8>,
}

impl<W: Write> Lzma2WriterMt<W> {
    pub fn new(inner: W, options: Lzma2Options, threads: u32) -> io::Result<Self> {
        Ok(Self {
            inner: Some(inner),
            options,
            threads,
            alloc: Allocator::default(),
            buffer: Vec::new(),
        })
    }

    pub fn finish(mut self) -> io::Result<W> {
        self.do_encode()?;
        Ok(self.inner.take().unwrap())
    }

    fn do_encode(&mut self) -> io::Result<()> {
        let writer = self.inner.as_mut().unwrap();
        let alloc = &self.alloc;

        let enc = unsafe { Lzma2Enc_Create(alloc.as_ref(), alloc.as_ref()) };
        if enc.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::OutOfMemory,
                "LZMA2: failed to create MT encoder",
            ));
        }

        let mut props = self.options.props;
        props.numTotalThreads = self.threads as i32;

        let res = unsafe { Lzma2Enc_SetProps(enc, &mut props) };
        if res != SZ_OK as i32 {
            unsafe { Lzma2Enc_Destroy(enc) };
            return Err(super::error::res_to_io_error(res));
        }

        let mut adapter = OutStreamAdapter::<W> {
            vtable: ISeqOutStream_ {
                Write: Some(out_stream_write::<W>),
            },
            writer: writer as *mut W,
            error: None,
        };

        let out_stream_ptr = &mut adapter.vtable as *mut ISeqOutStream_ as ISeqOutStreamPtr;

        let res = unsafe {
            Lzma2Enc_Encode2(
                enc,
                out_stream_ptr,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                self.buffer.as_ptr() as *const Byte,
                self.buffer.len(),
                std::ptr::null_mut(),
            )
        };

        unsafe { Lzma2Enc_Destroy(enc) };

        if let Some(e) = adapter.error.take() {
            return Err(e);
        }
        check_res(res)?;

        self.buffer.clear();
        Ok(())
    }
}

impl<W: Write> Write for Lzma2WriterMt<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if let Some(w) = self.inner.as_mut() {
            w.flush()
        } else {
            Ok(())
        }
    }
}
