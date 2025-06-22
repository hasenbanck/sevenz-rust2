use std::io::Read;

use crate::internal::ppmd8::{IByteIn, IByteInPtr};

pub(crate) struct ByteReader<R: Read> {
    inner: Box<ByteReaderInner<R>>,
}

#[repr(C)]
struct ByteReaderInner<R> {
    byte_in: IByteIn,
    buffer: Vec<u8>,
    reader: R,
    pos: usize,
    end: usize,
    eof: bool,
}

impl<R: Read> ByteReader<R> {
    pub(crate) fn new(reader: R) -> Self {
        let reader = ByteReaderInner {
            byte_in: IByteIn {
                read: Some(Self::read_byte),
            },
            buffer: vec![0; 4096],
            reader,
            pos: 0,
            end: 0,
            eof: false,
        };

        Self {
            inner: Box::new(reader),
        }
    }

    pub(crate) fn byte_in_ptr(&mut self) -> IByteInPtr {
        &mut self.inner.byte_in as *const _
    }

    #[inline(always)]
    fn get_inner_reader<'a>(p: IByteInPtr) -> &'a mut ByteReaderInner<R> {
        // Safety: This is safe because we make sure that `byte_in` is the first field
        // of the `ByteReaderInner` and also `ByteReaderInner` is boxed and can't break out of it.
        unsafe { &mut *(p as *mut ByteReaderInner<R>) }
    }

    unsafe extern "C" fn read_byte(p: IByteInPtr) -> u8 {
        let reader = Self::get_inner_reader(p);

        if reader.eof {
            return 0;
        }

        if reader.pos >= reader.end {
            match reader.reader.read(&mut reader.buffer) {
                Ok(n) => {
                    if n == 0 {
                        reader.eof = true;
                        return 0;
                    }
                    reader.end = n;
                    reader.pos = 0;
                }
                Err(_) => {
                    reader.eof = true;
                    return 0;
                }
            }
        }

        let byte = reader.buffer[reader.pos];
        reader.pos += 1;

        byte
    }
}
