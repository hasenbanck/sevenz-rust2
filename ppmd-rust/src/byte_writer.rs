use std::io::Write;

use crate::internal::ppmd8::{IByteOut, IByteOutPtr};

const BUFFER_SIZE: usize = 4096;

pub(crate) struct ByteWriter<W: Write> {
    inner: Box<ByteWriterInner<W>>,
}

#[repr(C)]
struct ByteWriterInner<W> {
    byte_out: IByteOut,
    writer: W,
    buffer: Vec<u8>,
}

impl<W: Write> ByteWriter<W> {
    pub(crate) fn new(writer: W) -> Self {
        let writer = ByteWriterInner {
            byte_out: IByteOut {
                write: Self::write_byte,
            },
            writer,
            buffer: Vec::with_capacity(BUFFER_SIZE),
        };

        Self {
            inner: Box::new(writer),
        }
    }

    pub(crate) fn byte_out_ptr(&mut self) -> IByteOutPtr {
        &mut self.inner.byte_out as *const _
    }

    #[inline(always)]
    fn get_inner_writer<'a>(p: IByteOutPtr) -> &'a mut ByteWriterInner<W> {
        // Safety: This is safe because we make sure that `byte_out` is the first field
        // of the `ByteWriterInner` and also `ByteWriterInner` is boxed and can't break out of it.
        unsafe { &mut *(p as *mut ByteWriterInner<W>) }
    }

    fn write_byte(p: IByteOutPtr, byte: u8) {
        let writer = Self::get_inner_writer(p);

        writer.buffer.push(byte);

        if writer.buffer.len() >= BUFFER_SIZE {
            let _ = writer.writer.write_all(writer.buffer.as_slice());
            writer.buffer.clear();
        }
    }

    pub(crate) fn flush(&mut self) {
        if !self.inner.buffer.is_empty() {
            let _ = self.inner.writer.write_all(self.inner.buffer.as_slice());
            self.inner.buffer.clear();
        }
    }
}
