use crate::Error;
use crate::byte_reader::ByteReader;
use crate::memory::Memory;
use ppmd_sys::{
    CPpmd7, PPMD7_MAX_MEM_SIZE, PPMD7_MAX_ORDER, PPMD7_MIN_MEM_SIZE, PPMD7_MIN_ORDER,
    PPMD7_SYM_END, Ppmd7_Alloc, Ppmd7_Construct, Ppmd7_Free, Ppmd7_Init, Ppmd7z_DecodeSymbol,
    Ppmd7z_RangeDec_Init,
};
use std::io::Read;

/// A decoder to decode PPMd7 (PPMdH) with the 7z range coder.
pub struct Ppmd7Decoder<R: Read> {
    ppmd: CPpmd7,
    _reader: ByteReader<R>,
    memory: Memory,
    finished: bool,
}

impl<R: Read> Ppmd7Decoder<R> {
    /// Creates a new [`Ppmd7Decoder`].
    pub fn new(reader: R, order: u32, mem_size: u32) -> crate::Result<Self> {
        if !(PPMD7_MIN_ORDER..=PPMD7_MAX_ORDER).contains(&order)
            || !(PPMD7_MIN_MEM_SIZE..=PPMD7_MAX_MEM_SIZE).contains(&mem_size)
        {
            return Err(Error::InvalidParameter);
        }

        let mut ppmd = unsafe { std::mem::zeroed::<CPpmd7>() };
        unsafe { Ppmd7_Construct(&mut ppmd) };

        let mut memory = Memory::new(mem_size);

        let success = unsafe { Ppmd7_Alloc(&mut ppmd, mem_size, memory.allocation()) };

        if success == 0 {
            return Err(Error::InternalError("Failed to allocate memory"));
        }

        let mut reader = ByteReader::new(reader);
        let range_decoder = unsafe { &mut ppmd.rc.dec };
        range_decoder.Stream = reader.byte_in_ptr();

        let success = unsafe { Ppmd7z_RangeDec_Init(&mut ppmd.rc.dec) };

        if success == 0 {
            return Err(Error::InternalError("Failed to initialize range decoder"));
        }

        unsafe { Ppmd7_Init(&mut ppmd, order) };

        Ok(Self {
            ppmd,
            _reader: reader,
            memory,
            finished: false,
        })
    }
}

impl<R: Read> Drop for Ppmd7Decoder<R> {
    fn drop(&mut self) {
        unsafe { Ppmd7_Free(&mut self.ppmd, self.memory.allocation()) }
    }
}

impl<R: Read> Read for Ppmd7Decoder<R> {
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
                sym = Ppmd7z_DecodeSymbol(&mut self.ppmd);

                if sym < 0 {
                    break;
                }

                *byte = sym as u8;
                decoded += 1;
            }
        }

        let code = unsafe { self.ppmd.rc.dec.Code };

        if sym >= 0 && (!self.finished || decoded != buf.len() || code == 0) {
            return Ok(decoded);
        }

        self.finished = true;

        if sym != PPMD7_SYM_END || code != 0 {
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
    use super::Ppmd7Decoder;

    const ORDER: u32 = 8;
    const MEM_SIZE: u32 = 262144;

    #[test]
    fn ppmd7decoder_init_drop() {
        let reader: &[u8] = &[];
        let decoder = Ppmd7Decoder::new(reader, ORDER, MEM_SIZE).unwrap();
        assert!(!decoder.ppmd.Base.is_null());
    }
}
