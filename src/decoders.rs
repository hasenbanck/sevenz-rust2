use std::io::Read;

#[cfg(feature = "aes256")]
use crate::aes256sha256::Aes256Sha256Decoder;
#[cfg(feature = "brotli")]
use crate::brotli::BrotliDecoder;
#[cfg(feature = "lz4")]
use crate::lz4::Lz4Decoder;
use crate::{
    archive::SevenZMethod,
    bcj::SimpleReader,
    delta::DeltaReader,
    error::Error,
    folder::Coder,
    lzma::{LZMA2Reader, LZMAReader, lzma2_get_memory_usage},
};
use byteorder::{LittleEndian, ReadBytesExt};
#[cfg(feature = "bzip2")]
use bzip2::read::BzDecoder;
#[cfg(feature = "deflate")]
use flate2::bufread::DeflateDecoder;
#[cfg(feature = "ppmd")]
use ppmd_rust::{
    PPMD7_MAX_MEM_SIZE, PPMD7_MAX_ORDER, PPMD7_MIN_MEM_SIZE, PPMD7_MIN_ORDER, Ppmd7Decoder,
};

#[allow(clippy::upper_case_acronyms)]
pub enum Decoder<R: Read> {
    COPY(R),
    LZMA(LZMAReader<R>),
    LZMA2(LZMA2Reader<R>),
    #[cfg(feature = "ppmd")]
    PPMD(Box<Ppmd7Decoder<R>>),
    BCJ(SimpleReader<R>),
    Delta(DeltaReader<R>),
    #[cfg(feature = "brotli")]
    Brotli(BrotliDecoder<R>),
    #[cfg(feature = "bzip2")]
    BZip2(BzDecoder<R>),
    #[cfg(feature = "deflate")]
    Deflate(DeflateDecoder<std::io::BufReader<R>>),
    #[cfg(feature = "lz4")]
    LZ4(Lz4Decoder<R>),
    #[cfg(feature = "zstd")]
    ZSTD(zstd::Decoder<'static, std::io::BufReader<R>>),
    #[cfg(feature = "aes256")]
    AES256SHA256(Aes256Sha256Decoder<R>),
}

impl<R: Read> Read for Decoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Decoder::COPY(r) => r.read(buf),
            Decoder::LZMA(r) => r.read(buf),
            Decoder::LZMA2(r) => r.read(buf),
            #[cfg(feature = "ppmd")]
            Decoder::PPMD(r) => r.read(buf),
            Decoder::BCJ(r) => r.read(buf),
            Decoder::Delta(r) => r.read(buf),
            #[cfg(feature = "brotli")]
            Decoder::Brotli(r) => r.read(buf),
            #[cfg(feature = "bzip2")]
            Decoder::BZip2(r) => r.read(buf),
            #[cfg(feature = "deflate")]
            Decoder::Deflate(r) => r.read(buf),
            #[cfg(feature = "lz4")]
            Decoder::LZ4(r) => r.read(buf),
            #[cfg(feature = "zstd")]
            Decoder::ZSTD(r) => r.read(buf),
            #[cfg(feature = "aes256")]
            Decoder::AES256SHA256(r) => r.read(buf),
        }
    }
}

pub fn add_decoder<I: Read>(
    input: I,
    uncompressed_len: usize,
    coder: &Coder,
    #[allow(unused)] password: &[u8],
    max_mem_limit_kb: usize,
) -> Result<Decoder<I>, Error> {
    let method = SevenZMethod::by_id(coder.decompression_method_id());
    let method = if let Some(m) = method {
        m
    } else {
        return Err(Error::UnsupportedCompressionMethod(format!(
            "{:?}",
            coder.decompression_method_id()
        )));
    };
    match method.id() {
        SevenZMethod::ID_COPY => Ok(Decoder::COPY(input)),
        SevenZMethod::ID_LZMA => {
            let dict_size = get_lzma_dic_size(coder)?;
            if coder.properties.is_empty() {
                return Err(Error::Other("LZMA properties too short".into()));
            }
            let props = coder.properties[0];
            let lz =
                LZMAReader::new_with_props(input, uncompressed_len as _, props, dict_size, None)
                    .map_err(|e| Error::bad_password(e, !password.is_empty()))?;
            Ok(Decoder::LZMA(lz))
        }
        SevenZMethod::ID_LZMA2 => {
            let dic_size = get_lzma2_dic_size(coder)?;
            let mem_size = lzma2_get_memory_usage(dic_size) as usize;
            if mem_size > max_mem_limit_kb {
                return Err(Error::MaxMemLimited {
                    max_kb: max_mem_limit_kb,
                    actaul_kb: mem_size,
                });
            }
            let lz = LZMA2Reader::new(input, dic_size, None);
            Ok(Decoder::LZMA2(lz))
        }
        #[cfg(feature = "ppmd")]
        SevenZMethod::ID_PPMD => {
            let (order, memory_size) = get_ppmd_order_memory_size(coder, max_mem_limit_kb)?;
            let ppmd = Ppmd7Decoder::new(input, order, memory_size)
                .map_err(|err| Error::other(err.to_string()))?;
            Ok(Decoder::PPMD(Box::new(ppmd)))
        }
        #[cfg(feature = "brotli")]
        SevenZMethod::ID_BROTLI => {
            let de = BrotliDecoder::new(input, 4096)?;
            Ok(Decoder::Brotli(de))
        }
        #[cfg(feature = "bzip2")]
        SevenZMethod::ID_BZIP2 => {
            let de = BzDecoder::new(input);
            Ok(Decoder::BZip2(de))
        }
        #[cfg(feature = "deflate")]
        SevenZMethod::ID_DEFLATE => {
            let buf_read = std::io::BufReader::new(input);
            let de = DeflateDecoder::new(buf_read);
            Ok(Decoder::Deflate(de))
        }
        #[cfg(feature = "lz4")]
        SevenZMethod::ID_LZ4 => {
            let de = Lz4Decoder::new(input)?;
            Ok(Decoder::LZ4(de))
        }
        #[cfg(feature = "zstd")]
        SevenZMethod::ID_ZSTD => {
            let zs = zstd::Decoder::new(input).map_err(Error::io)?;
            Ok(Decoder::ZSTD(zs))
        }
        SevenZMethod::ID_BCJ_X86 => {
            let de = SimpleReader::new_x86(input);
            Ok(Decoder::BCJ(de))
        }
        SevenZMethod::ID_BCJ_ARM => {
            let de = SimpleReader::new_arm(input);
            Ok(Decoder::BCJ(de))
        }
        SevenZMethod::ID_BCJ_ARM64 => {
            let de = SimpleReader::new_arm64(input);
            Ok(Decoder::BCJ(de))
        }
        SevenZMethod::ID_BCJ_ARM_THUMB => {
            let de = SimpleReader::new_arm_thumb(input);
            Ok(Decoder::BCJ(de))
        }
        SevenZMethod::ID_BCJ_PPC => {
            let de = SimpleReader::new_ppc(input);
            Ok(Decoder::BCJ(de))
        }
        SevenZMethod::ID_BCJ_SPARC => {
            let de = SimpleReader::new_sparc(input);
            Ok(Decoder::BCJ(de))
        }
        SevenZMethod::ID_DELTA => {
            let d = if coder.properties.is_empty() {
                1
            } else {
                coder.properties[0].wrapping_add(1)
            };
            let de = DeltaReader::new(input, d as usize);
            Ok(Decoder::Delta(de))
        }
        #[cfg(feature = "aes256")]
        SevenZMethod::ID_AES256SHA256 => {
            if password.is_empty() {
                return Err(Error::PasswordRequired);
            }
            let de = Aes256Sha256Decoder::new(input, &coder.properties, password)?;
            Ok(Decoder::AES256SHA256(de))
        }
        _ => Err(Error::UnsupportedCompressionMethod(
            method.name().to_string(),
        )),
    }
}

#[cfg(feature = "ppmd")]
fn get_ppmd_order_memory_size(coder: &Coder, max_mem_limit_kb: usize) -> Result<(u32, u32), Error> {
    if coder.properties.len() < 5 {
        return Err(Error::other("PPMD properties too short"));
    }
    let order = coder.properties[0] as u32;
    let memory_size = u32::from_le_bytes([
        coder.properties[1],
        coder.properties[2],
        coder.properties[3],
        coder.properties[4],
    ]);

    if order < PPMD7_MIN_ORDER {
        return Err(Error::other("PPMD order smaller than PPMD7_MIN_ORDER"));
    }

    if order > PPMD7_MAX_ORDER {
        return Err(Error::other("PPMD order larger than PPMD7_MAX_ORDER"));
    }

    if memory_size < PPMD7_MIN_MEM_SIZE {
        return Err(Error::other(
            "PPMD memory size smaller than PPMD7_MIN_MEM_SIZE",
        ));
    }

    if memory_size > PPMD7_MAX_MEM_SIZE {
        return Err(Error::other(
            "PPMD memory size larger than PPMD7_MAX_MEM_SIZE",
        ));
    }

    if memory_size as usize > max_mem_limit_kb {
        return Err(Error::MaxMemLimited {
            max_kb: max_mem_limit_kb,
            actaul_kb: memory_size as usize,
        });
    }

    Ok((order, memory_size))
}

fn get_lzma2_dic_size(coder: &Coder) -> Result<u32, Error> {
    if coder.properties.is_empty() {
        return Err(Error::other("LZMA2 properties too short"));
    }
    let dict_size_bits = 0xff & coder.properties[0] as u32;
    if (dict_size_bits & (!0x3f)) != 0 {
        return Err(Error::other("Unsupported LZMA2 property bits"));
    }
    if dict_size_bits > 40 {
        return Err(Error::other("Dictionary larger than 4GiB maximum size"));
    }
    if dict_size_bits == 40 {
        return Ok(0xFFFFFFFF);
    }
    let size = (2 | (dict_size_bits & 0x1)) << (dict_size_bits / 2 + 11);
    Ok(size)
}

fn get_lzma_dic_size(coder: &Coder) -> Result<u32, Error> {
    let mut props = &coder.properties[1..5];
    props.read_u32::<LittleEndian>().map_err(Error::io)
}
