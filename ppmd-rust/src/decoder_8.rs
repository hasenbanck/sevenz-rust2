use std::io::Read;

use crate::{
    Error, PPMD8_MAX_ORDER, PPMD8_MIN_ORDER, RestoreMethod, SYM_END,
    byte_reader::ByteReader,
    internal::ppmd8::{Ppmd8, alloc, construct, decode_symbol, free, init, init_range_dec},
    memory::Memory,
};

/// A decoder to decode PPMd8 (PPMdI) compressed data.
pub struct Ppmd8Decoder<R: Read> {
    ppmd: Ppmd8,
    _reader: ByteReader<R>,
    memory: Memory,
    finished: bool,
}

impl<R: Read> Ppmd8Decoder<R> {
    /// Creates a new [`Ppmd8Decoder`].
    pub fn new(
        reader: R,
        order: u32,
        mem_size: u32,
        restore_method: RestoreMethod,
    ) -> crate::Result<Self> {
        if !(PPMD8_MIN_ORDER..=PPMD8_MAX_ORDER).contains(&order) {
            return Err(Error::InvalidParameter);
        }

        let mut ppmd = unsafe { std::mem::zeroed::<Ppmd8>() };
        unsafe { construct(&mut ppmd) };

        let mut memory = Memory::new(mem_size);

        let success = unsafe { alloc(&mut ppmd, mem_size, memory.allocation()) };

        if success == 0 {
            return Err(Error::InternalError("Failed to allocate memory"));
        }

        let mut reader = ByteReader::new(reader);
        ppmd.stream.input = reader.byte_in_ptr();

        let success = unsafe { init_range_dec(&mut ppmd) };

        if success == 0 {
            return Err(Error::InternalError("Failed to initialize range decoder"));
        }

        unsafe { init(&mut ppmd, order, restore_method as _) };

        Ok(Self {
            ppmd,
            _reader: reader,
            memory,
            finished: false,
        })
    }
}

impl<R: Read> Drop for Ppmd8Decoder<R> {
    fn drop(&mut self) {
        unsafe { free(&mut self.ppmd, self.memory.allocation()) }
    }
}

impl<R: Read> Read for Ppmd8Decoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.finished {
            return Ok(0);
        }

        if buf.is_empty() {
            return Ok(0);
        }

        let mut sym = 0;
        let mut decoded = 0;

        unsafe {
            for byte in buf.iter_mut() {
                sym = decode_symbol(&mut self.ppmd);

                if sym < 0 {
                    break;
                }

                *byte = sym as u8;
                decoded += 1;
            }
        }

        let code = self.ppmd.code;

        if sym >= 0 && (!self.finished || decoded != buf.len() || code == 0) {
            return Ok(decoded);
        }

        self.finished = true;

        if sym != SYM_END || code != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Error during PPMd decoding",
            ));
        }

        Ok(decoded)
    }
}

#[cfg(test)]
mod test {
    use super::{Ppmd8Decoder, RestoreMethod};

    const ORDER: u32 = 8;
    const MEM_SIZE: u32 = 262144;
    const RESTORE_METHOD: RestoreMethod = RestoreMethod::CutOff;

    #[test]
    fn ppmd8zdecoder_init_drop() {
        let reader: &[u8] = &[];
        let decoder = Ppmd8Decoder::new(reader, ORDER, MEM_SIZE, RESTORE_METHOD).unwrap();
        assert!(!decoder.ppmd.base.is_null());
    }
}
