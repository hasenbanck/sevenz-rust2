use std::io::{self, Write};
use std::ptr;

use lzma_sdk_sys::{
    Allocator, Byte, CLzmaEncProps, LzmaEnc_Create, LzmaEnc_Destroy, LzmaEnc_SetProps, LzmaEncode,
    LZMA_PROPS_SIZE, SizeT,
};

use super::error::check_res;
use super::options::LzmaOptions;

/// Safe streaming LZMA encoder wrapping lzma-sdk-sys.
///
/// Buffers all input data, then encodes on `finish()`. The LZMA-SDK encode API
/// is fundamentally a one-shot operation, so streaming is simulated via buffering.
pub struct LzmaWriter<W: Write> {
    inner: Option<W>,
    props: CLzmaEncProps,
    alloc: Allocator,
    buffer: Vec<u8>,
    write_header: bool,
}

impl<W: Write> LzmaWriter<W> {
    /// Creates a new LZMA encoder that does NOT write the LZMA header.
    /// This is the mode used by 7z containers (properties are stored separately).
    pub fn new_no_header(
        inner: W,
        options: &LzmaOptions,
        _write_end_mark: bool,
    ) -> io::Result<Self> {
        let alloc = Allocator::default();
        let mut props = options.props;

        // Validate props by creating a temporary encoder
        let enc = unsafe { LzmaEnc_Create(alloc.as_ref()) };
        if enc.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::OutOfMemory,
                "LZMA: failed to create encoder",
            ));
        }
        let res = unsafe { LzmaEnc_SetProps(enc, &props) };
        unsafe { LzmaEnc_Destroy(enc, alloc.as_ref(), alloc.as_ref()) };
        check_res(res)?;

        // Normalize props so dictSize etc. have proper values
        unsafe { lzma_sdk_sys::LzmaEncProps_Normalize(&mut props) };

        Ok(Self {
            inner: Some(inner),
            props,
            alloc,
            buffer: Vec::new(),
            write_header: false,
        })
    }

    /// Finishes encoding and returns the inner writer.
    pub fn finish(mut self) -> io::Result<W> {
        self.do_encode()?;
        Ok(self.inner.take().unwrap())
    }

    fn do_encode(&mut self) -> io::Result<()> {
        let writer = self.inner.as_mut().unwrap();
        let alloc = &self.alloc;

        let src = &self.buffer;
        let src_len = src.len();

        // Worst case: output can be slightly larger than input for incompressible data.
        // LZMA SDK documentation says output can be up to src_len + src_len/8 + 64 bytes.
        let max_dest_len = src_len + src_len / 8 + 1024;
        let mut dest = vec![0u8; max_dest_len];
        let mut dest_len: SizeT = max_dest_len;

        let mut props_encoded = [0u8; LZMA_PROPS_SIZE as usize];
        let mut props_size: SizeT = LZMA_PROPS_SIZE as SizeT;

        let res = unsafe {
            LzmaEncode(
                dest.as_mut_ptr() as *mut Byte,
                &mut dest_len,
                src.as_ptr() as *const Byte,
                src_len,
                &self.props,
                props_encoded.as_mut_ptr() as *mut Byte,
                &mut props_size,
                1, // writeEndMark - needed for 7z LZMA1 streams
                ptr::null_mut(),
                alloc.as_ref(),
                alloc.as_ref(),
            )
        };
        check_res(res)?;

        if self.write_header {
            writer.write_all(&props_encoded[..props_size])?;
            writer.write_all(&(src_len as u64).to_le_bytes())?;
        }

        writer.write_all(&dest[..dest_len])?;
        self.buffer.clear();
        Ok(())
    }
}

impl<W: Write> Write for LzmaWriter<W> {
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

impl<W: Write> Drop for LzmaWriter<W> {
    fn drop(&mut self) {
        // If finish() was not called, the data is lost silently.
        // This matches the behavior of other encoders in the codebase.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lzma::LzmaReader;
    use std::io::{Cursor, Read};

    #[test]
    fn test_lzma_roundtrip() {
        let original = b"Hello, World! This is a test of LZMA compression. Repeating data helps.";
        let opts = LzmaOptions::with_preset(6);

        let mut encoded = Vec::new();
        {
            let mut writer = LzmaWriter::new_no_header(
                Cursor::new(&mut encoded),
                &opts,
                false,
            )
            .unwrap();
            writer.write_all(original).unwrap();
            writer.finish().unwrap();
        }

        let mut decoded = Vec::new();
        {
            let mut reader = LzmaReader::new_with_props(
                Cursor::new(&encoded),
                original.len() as u64,
                opts.get_props(),
                opts.dict_size(),
                None,
            )
            .unwrap();
            reader.read_to_end(&mut decoded).unwrap();
        }

        assert_eq!(decoded, original);
    }
}
