use std::num::NonZeroU64;

use lzma_sdk_sys::{CLzma2EncProps, CLzmaEncProps, Lzma2EncProps_Init, LzmaEncProps_Init};

pub const DICT_SIZE_MIN: u32 = 1 << 12; // 4 KiB
pub const DICT_SIZE_MAX: u32 = 0xFFFF_FFF0; // ~4 GiB

/// LZMA encoder properties, wrapping CLzmaEncProps from lzma-sdk-sys.
#[derive(Debug, Clone)]
pub struct LzmaOptions {
    pub(crate) props: CLzmaEncProps,
}

impl LzmaOptions {
    pub fn with_preset(level: u32) -> Self {
        let mut props = CLzmaEncProps::default();
        unsafe { LzmaEncProps_Init(&mut props) };
        props.level = level.min(9) as i32;
        Self { props }
    }

    /// Returns the LZMA properties byte: `(pb * 5 + lp) * 9 + lc`.
    /// This normalizes props first so all fields have actual values.
    pub fn get_props(&self) -> u8 {
        let mut p = self.props;
        unsafe { lzma_sdk_sys::LzmaEncProps_Normalize(&mut p) };
        let lc = p.lc.max(0) as u8;
        let lp = p.lp.max(0) as u8;
        let pb = p.pb.max(0) as u8;
        (pb * 5 + lp) * 9 + lc
    }

    pub fn dict_size(&self) -> u32 {
        let mut p = self.props;
        unsafe { lzma_sdk_sys::LzmaEncProps_Normalize(&mut p) };
        p.dictSize
    }

    pub fn set_dict_size(&mut self, dict_size: u32) {
        self.props.dictSize = dict_size.clamp(DICT_SIZE_MIN, DICT_SIZE_MAX);
    }
}

/// LZMA2 encoder properties, wrapping CLzma2EncProps from lzma-sdk-sys.
#[derive(Debug, Clone)]
pub struct Lzma2Options {
    pub(crate) props: CLzma2EncProps,
}

impl Lzma2Options {
    pub fn with_preset(level: u32) -> Self {
        let mut props = CLzma2EncProps::default();
        unsafe { Lzma2EncProps_Init(&mut props) };
        props.lzmaProps.level = level.min(9) as i32;
        Self { props }
    }

    pub fn lzma_options(&self) -> LzmaOptions {
        LzmaOptions {
            props: self.props.lzmaProps,
        }
    }

    pub fn dict_size(&self) -> u32 {
        let mut p = self.props.lzmaProps;
        unsafe { lzma_sdk_sys::LzmaEncProps_Normalize(&mut p) };
        p.dictSize
    }

    pub fn set_dict_size(&mut self, dict_size: u32) {
        self.props.lzmaProps.dictSize = dict_size.clamp(DICT_SIZE_MIN, DICT_SIZE_MAX);
    }

    pub fn set_chunk_size(&mut self, chunk_size: Option<NonZeroU64>) {
        match chunk_size {
            Some(size) => self.props.blockSize = size.get(),
            None => self.props.blockSize = 0,
        }
    }

    pub fn set_num_threads(&mut self, threads: i32) {
        self.props.numTotalThreads = threads;
    }
}
