use crate::byte_writer::ByteWriter;
use crate::internal::ppmd8::{
    CPpmd8, Ppmd8_Alloc, Ppmd8_Construct, Ppmd8_EncodeSymbol, Ppmd8_Flush_RangeEnc, Ppmd8_Init,
};
use crate::memory::Memory;
use crate::{Error, PPMD8_MAX_ORDER, PPMD8_MIN_ORDER, RestoreMethod};
use std::io::Write;

/// A encoder to encode PPMd8 (PPMdI) compressed data.
pub struct Ppmd8Encoder<W: Write> {
    ppmd: CPpmd8,
    writer: ByteWriter<W>,
    _memory: Memory,
}

impl<W: Write> Ppmd8Encoder<W> {
    /// Creates a new [`Ppmd8Encoder`].
    pub fn new(
        writer: W,
        order: u32,
        mem_size: u32,
        restore_method: RestoreMethod,
    ) -> crate::Result<Self> {
        if !(PPMD8_MIN_ORDER..=PPMD8_MAX_ORDER).contains(&order) {
            return Err(Error::InvalidParameter);
        }

        let mut ppmd = unsafe { std::mem::zeroed::<CPpmd8>() };
        unsafe { Ppmd8_Construct(&mut ppmd) };

        let mut memory = Memory::new(mem_size);

        let success = unsafe { Ppmd8_Alloc(&mut ppmd, mem_size, memory.allocation()) };

        if success == 0 {
            return Err(Error::InternalError("Failed to allocate memory"));
        }

        let mut writer = ByteWriter::new(writer);
        ppmd.Stream.Out = writer.byte_out_ptr();

        // #define Ppmd8_Init_RangeEnc(p) { (p)->Low = 0; (p)->Range = 0xFFFFFFFF; }
        ppmd.Low = 0;
        ppmd.Range = 0xFFFFFFFF;

        unsafe { Ppmd8_Init(&mut ppmd, order, restore_method as _) };

        Ok(Self {
            ppmd,
            writer,
            _memory: memory,
        })
    }

    fn inner_flush(&mut self) {
        unsafe { Ppmd8_Flush_RangeEnc(&mut self.ppmd) };
        self.writer.flush();
    }
}

impl<W: Write> Write for Ppmd8Encoder<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        buf.iter()
            .for_each(|byte| unsafe { Ppmd8_EncodeSymbol(&mut self.ppmd as *mut _, *byte as _) });

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner_flush();
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::Ppmd8Encoder;
    use crate::{Ppmd8Decoder, RestoreMethod};
    use std::io::{Read, Write};

    const ORDER: u32 = 8;
    const MEM_SIZE: u32 = 262144;
    const RESTORE_METHOD: RestoreMethod = RestoreMethod::Restart;

    #[test]
    fn ppmd8encoder_init_drop() {
        let writer = Vec::new();
        let encoder = Ppmd8Encoder::new(writer, ORDER, MEM_SIZE, RESTORE_METHOD).unwrap();
        assert!(!encoder.ppmd.Base.is_null());
    }

    #[test]
    fn ppmd8encoder_encode_decode() {
        let test_data = "Lorem ipsum dolor sit amet. ";

        let mut writer = Vec::new();
        {
            let mut encoder =
                Ppmd8Encoder::new(&mut writer, ORDER, MEM_SIZE, RESTORE_METHOD).unwrap();
            encoder.write_all(test_data.as_bytes()).unwrap();
            encoder.flush().unwrap();
        }

        let mut decoder =
            Ppmd8Decoder::new(writer.as_slice(), ORDER, MEM_SIZE, RESTORE_METHOD).unwrap();

        let mut decoded = vec![0; test_data.len()];
        decoder.read_exact(&mut decoded).unwrap();

        assert_eq!(decoded.as_slice(), test_data.as_bytes());

        let decoded_data = String::from_utf8(decoded).unwrap();

        assert_eq!(decoded_data, test_data);
    }
}
