use std::{io::Write, sync::Arc};

use super::*;
use crate::EncoderConfiguration;
#[derive(Debug, Clone, Default)]
pub(crate) struct UnpackInfo {
    pub(crate) blocks: Vec<BlockInfo>,
}

impl UnpackInfo {
    pub(crate) fn add(
        &mut self,
        methods: Arc<Vec<EncoderConfiguration>>,
        sizes: Vec<u64>,
        crc: u32,
    ) {
        self.blocks.push(BlockInfo {
            methods,
            sizes,
            crc,
            num_sub_unpack_streams: 1,
            ..Default::default()
        })
    }

    pub(crate) fn add_multiple(
        &mut self,
        methods: Arc<Vec<EncoderConfiguration>>,
        sizes: Vec<u64>,
        crc: u32,
        num_sub_unpack_streams: u64,
        sub_stream_sizes: Vec<u64>,
        sub_stream_crcs: Vec<u32>,
    ) {
        self.blocks.push(BlockInfo {
            methods,
            sizes,
            crc,
            num_sub_unpack_streams,
            sub_stream_crcs,
            sub_stream_sizes,
        })
    }

    pub(crate) fn write_to<H: Write>(&mut self, header: &mut H) -> std::io::Result<()> {
        header.write_u8(K_UNPACK_INFO)?;
        header.write_u8(K_FOLDER)?;
        write_u64(header, self.blocks.len() as u64)?;
        header.write_u8(0)?;
        let mut cache = Vec::with_capacity(32);
        for block in self.blocks.iter() {
            block.write_to(header, &mut cache)?;
        }
        header.write_u8(K_CODERS_UNPACK_SIZE)?;
        for block in self.blocks.iter() {
            for size in block.sizes.iter().copied() {
                write_u64(header, size)?;
            }
        }
        header.write_u8(K_CRC)?;
        header.write_u8(1)?; //all defined
        for block in self.blocks.iter() {
            header.write_u32::<LittleEndian>(block.crc)?;
        }
        header.write_u8(K_END)?;
        Ok(())
    }

    pub(crate) fn write_substreams<H: Write>(&self, header: &mut H) -> std::io::Result<()> {
        header.write_u8(K_SUB_STREAMS_INFO)?;

        header.write_u8(K_NUM_UNPACK_STREAM)?;
        for f in &self.blocks {
            write_u64(header, f.num_sub_unpack_streams)?;
        }
        header.write_u8(K_SIZE)?;
        for f in &self.blocks {
            if f.sub_stream_sizes.len() <= 1 {
                continue;
            }
            for i in 0..f.sub_stream_sizes.len() - 1 {
                let size = f.sub_stream_sizes[i];
                write_u64(header, size)?;
            }
        }
        header.write_u8(K_CRC)?;
        header.write_u8(1)?; // all crc defined
        for f in &self.blocks {
            if f.sub_stream_crcs.len() <= 1 && f.crc != 0 {
                continue;
            }
            for i in 0..f.sub_stream_crcs.len() {
                let crc = f.sub_stream_crcs[i];
                header.write_u32::<LittleEndian>(crc)?;
            }
        }
        header.write_u8(K_END)?;

        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct BlockInfo {
    pub(crate) methods: Arc<Vec<EncoderConfiguration>>,
    pub(crate) sizes: Vec<u64>,
    pub(crate) crc: u32,
    pub(crate) num_sub_unpack_streams: u64,
    pub(crate) sub_stream_sizes: Vec<u64>,
    pub(crate) sub_stream_crcs: Vec<u32>,
}

impl BlockInfo {
    pub(crate) fn write_to<W: Write>(
        &self,
        header: &mut W,
        cache: &mut Vec<u8>,
    ) -> std::io::Result<()> {
        cache.clear();
        let mut num_coders = 0;
        for mc in self.methods.iter() {
            num_coders += 1;
            self.write_single_codec(mc, cache)?;
        }
        write_u64(header, num_coders as u64)?;
        header.write_all(cache)?;
        for i in 0..num_coders - 1 {
            write_u64(header, i as u64 + 1)?;
            write_u64(header, i as u64)?;
        }
        Ok(())
    }

    fn write_single_codec<H: Write>(
        &self,
        mc: &EncoderConfiguration,
        out: &mut H,
    ) -> std::io::Result<()> {
        let id = mc.method.id();
        let mut temp = [0u8; 256];
        let props = encoder::get_options_as_properties(mc.method, mc.options.as_ref(), &mut temp);
        let mut codec_flags = id.len() as u8;
        if !props.is_empty() {
            codec_flags |= 0x20;
        }
        out.write_u8(codec_flags)?;
        out.write_all(id)?;
        if !props.is_empty() {
            out.write_u8(props.len() as u8)?;
            out.write_all(props)?;
        }
        Ok(())
    }
}
