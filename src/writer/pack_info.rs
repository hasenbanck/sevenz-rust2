use super::*;

#[derive(Debug, Default, Clone)]
pub(crate) struct PackInfo {
    /// Each packed stream's CRC, `None` where it is not defined: `0` is a valid CRC.
    pub(crate) crcs: Vec<Option<u32>>,
    pub(crate) sizes: Vec<u64>,
    pub(crate) pos: u64,
}

impl PackInfo {
    pub(crate) fn write_to<H: Write>(&mut self, header: &mut H) -> std::io::Result<()> {
        header.write_u8(K_PACK_INFO)?;
        write_u64(header, self.pos)?;
        write_u64(header, self.len() as u64)?;
        header.write_u8(K_SIZE)?;
        for size in &self.sizes {
            write_u64(header, *size)?;
        }
        header.write_u8(K_CRC)?;
        let all_crc_defined = self.crcs.iter().all(Option::is_some);
        if all_crc_defined {
            header.write_u8(1)?; // all defined
        } else {
            header.write_u8(0)?; // not all defined
            let mut crc_define_bits = BitSet::with_capacity(self.crcs.len());
            for (i, crc) in self.crcs.iter().enumerate() {
                if crc.is_some() {
                    crc_define_bits.insert(i);
                }
            }
            let mut temp = Vec::with_capacity(self.len());
            write_bit_set(&mut temp, &crc_define_bits)?;
            header.write_all(&temp)?;
        }
        // the defined CRCs follow, in order, either way
        for crc in self.crcs.iter().flatten() {
            header.write_u32(*crc)?;
        }

        header.write_u8(K_END)?;
        Ok(())
    }
}

impl PackInfo {
    #[inline]
    pub(crate) fn add_stream(&mut self, size: u64, crc: Option<u32>) {
        self.sizes.push(size);
        self.crcs.push(crc);
    }

    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.sizes.len()
    }
}
