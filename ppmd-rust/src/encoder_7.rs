use crate::Error;
use crate::byte_writer::ByteWriter;
use crate::memory::Memory;
use ppmd_sys::{
    CPpmd7, PPMD7_MAX_MEM_SIZE, PPMD7_MAX_ORDER, PPMD7_MIN_MEM_SIZE, PPMD7_MIN_ORDER, Ppmd7_Alloc,
    Ppmd7_Construct, Ppmd7_Init, Ppmd7z_EncodeSymbols, Ppmd7z_Flush_RangeEnc, Ppmd7z_Init_RangeEnc,
};
use std::io::Write;

/// An encoder to encode data using PPMd7 (PPMdH) with the 7z range coder.
pub struct Ppmd7Encoder<W: Write> {
    ppmd: CPpmd7,
    writer: ByteWriter<W>,
    _memory: Memory,
}

impl<W: Write> Ppmd7Encoder<W> {
    /// Creates a new [`Ppmd7Encoder`].
    pub fn new(writer: W, order: u32, mem_size: u32) -> crate::Result<Self> {
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
            return Err(Error::InternalError("Failed to initialize range decoder"));
        }

        let mut writer = ByteWriter::new(writer);
        let range_encoder = unsafe { &mut ppmd.rc.enc };
        range_encoder.Stream = writer.byte_out_ptr();

        unsafe { Ppmd7z_Init_RangeEnc(&mut ppmd) };
        unsafe { Ppmd7_Init(&mut ppmd, order) };

        Ok(Self {
            ppmd,
            writer,
            _memory: memory,
        })
    }

    fn inner_flush(&mut self) {
        unsafe { Ppmd7z_Flush_RangeEnc(&mut self.ppmd) };
        self.writer.flush();
    }
}

impl<W: Write> Write for Ppmd7Encoder<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        let pointer_range = buf.as_ptr_range();
        unsafe { Ppmd7z_EncodeSymbols(&mut self.ppmd, pointer_range.start, pointer_range.end) };

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner_flush();
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::Ppmd7Encoder;
    use crate::Ppmd7Decoder;
    use std::io::{Read, Write};

    const ORDER: u32 = 8;
    const MEM_SIZE: u32 = 262144;

    #[test]
    fn ppmd7encoder_init_drop() {
        let writer = Vec::new();
        let encoder = Ppmd7Encoder::new(writer, ORDER, MEM_SIZE).unwrap();
        assert!(!encoder.ppmd.Base.is_null());
    }

    #[test]
    fn ppmd7encoder_encode_decode() {
        let test_data = "Lorem ipsum dolor sit amet. ";

        let mut writer = Vec::new();
        {
            let mut encoder = Ppmd7Encoder::new(&mut writer, ORDER, MEM_SIZE).unwrap();
            encoder.write_all(test_data.as_bytes()).unwrap();
            encoder.flush().unwrap();
        }

        let mut decoder = Ppmd7Decoder::new(writer.as_slice(), ORDER, MEM_SIZE).unwrap();

        let mut decoded = vec![0; test_data.len()];
        decoder.read_exact(&mut decoded).unwrap();

        assert_eq!(decoded.as_slice(), test_data.as_bytes());

        let decoded_data = String::from_utf8(decoded).unwrap();

        assert_eq!(decoded_data, test_data);
    }
}
