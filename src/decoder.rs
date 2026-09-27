use std::{io, io::Read};

#[cfg(feature = "bzip2")]
use bzip2::read::BzDecoder;
#[cfg(feature = "deflate")]
use flate2::bufread::DeflateDecoder;
use lzma_rust2::{
    Lzma2Reader, Lzma2ReaderMt, LzmaReader,
    filter::{bcj::BcjReader, delta::DeltaReader},
    lzma_get_memory_usage_by_props, lzma2_get_memory_usage,
};
#[cfg(feature = "ppmd")]
use ppmd_rust::{
    PPMD7_MAX_MEM_SIZE, PPMD7_MAX_ORDER, PPMD7_MIN_MEM_SIZE, PPMD7_MIN_ORDER, Ppmd7Decoder,
};

#[cfg(feature = "brotli")]
use crate::codec::brotli::BrotliDecoder;
#[cfg(feature = "lz4")]
use crate::codec::lz4::Lz4Decoder;
#[cfg(feature = "aes256")]
use crate::encryption::Aes256Sha256Decoder;
use crate::{
    ByteReader, Password, archive::EncoderMethod, block::Coder, error::Error,
    reader::ArchiveReaderLimits,
};

pub enum Decoder<R: Read> {
    Copy(R),
    Lzma(Box<LzmaReader<R>>),
    Lzma2(Box<Lzma2Reader<R>>),
    Lzma2Mt(Box<Lzma2ReaderMt<R>>),
    #[cfg(feature = "ppmd")]
    Ppmd(Box<Ppmd7Decoder<R>>),
    Bcj(BcjReader<R>),
    Delta(DeltaReader<R>),
    #[cfg(feature = "brotli")]
    Brotli(Box<BrotliDecoder<R>>),
    #[cfg(feature = "bzip2")]
    Bzip2(BzDecoder<R>),
    #[cfg(feature = "deflate")]
    Deflate(DeflateDecoder<std::io::BufReader<R>>),
    #[cfg(feature = "lz4")]
    Lz4(Lz4Decoder<R>),
    #[cfg(feature = "zstd")]
    Zstd(zstd::Decoder<'static, std::io::BufReader<R>>),
    #[cfg(feature = "aes256")]
    Aes256Sha256(Box<Aes256Sha256Decoder<R>>),
}

/// Charge in KiB for a reader or wrapper that only holds a few fields.
pub(crate) const WRAPPER_MEMORY_KIB: usize = 1;

/// Input buffer of `LzmaReader`, not covered by `lzma_get_memory_usage_by_props`.
const LZMA_INPUT_BUFFER_KIB: usize = 64;

/// Fixed buffer of `BcjReader`.
const BCJ_BUFFER_KIB: usize = 4;

/// Fixed PPMd7 tables next to the arena (about 19 KiB).
#[cfg(feature = "ppmd")]
const PPMD7_STATE_KIB: usize = 20;

/// `Bcj2Reader` has one 256 KiB buffer per input stream.
pub(crate) const BCJ2_BUFFER_KIB: usize = 4 * 256;

/// Running estimate of the decoder memory of one decoder graph.
pub(crate) struct DecoderMemoryBudget {
    used_kib: usize,
    limit_kib: usize,
}

impl DecoderMemoryBudget {
    pub(crate) const fn new(limit_kib: usize) -> Self {
        Self {
            used_kib: 0,
            limit_kib,
        }
    }

    pub(crate) fn charge(&mut self, required_kib: usize) -> Result<(), Error> {
        let total = self
            .used_kib
            .checked_add(required_kib)
            .ok_or(Error::ResourceLimit {
                resource: "decoder_memory_kib",
                limit: self.limit_kib,
                required: usize::MAX,
            })?;
        if total > self.limit_kib {
            return Err(Error::ResourceLimit {
                resource: "decoder_memory_kib",
                limit: self.limit_kib,
                required: total,
            });
        }
        self.used_kib = total;
        Ok(())
    }
}

impl<R: Read> Read for Decoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Decoder::Copy(r) => r.read(buf),
            Decoder::Lzma(r) => r.read(buf),
            Decoder::Lzma2(r) => r.read(buf),
            Decoder::Lzma2Mt(r) => r.read(buf),
            #[cfg(feature = "ppmd")]
            Decoder::Ppmd(r) => r.read(buf),
            Decoder::Bcj(r) => r.read(buf),
            Decoder::Delta(r) => r.read(buf),
            #[cfg(feature = "brotli")]
            Decoder::Brotli(r) => r.read(buf),
            #[cfg(feature = "bzip2")]
            Decoder::Bzip2(r) => r.read(buf),
            #[cfg(feature = "deflate")]
            Decoder::Deflate(r) => r.read(buf),
            #[cfg(feature = "lz4")]
            Decoder::Lz4(r) => r.read(buf),
            #[cfg(feature = "zstd")]
            Decoder::Zstd(r) => r.read(buf),
            #[cfg(feature = "aes256")]
            Decoder::Aes256Sha256(r) => r.read(buf),
        }
    }
}

pub fn add_decoder<I: Read>(
    input: I,
    uncompressed_len: usize,
    coder: &Coder,
    #[allow(unused)] password: &Password,
    limits: &ArchiveReaderLimits,
    memory: &mut DecoderMemoryBudget,
    threads: u32,
) -> Result<Decoder<I>, Error> {
    let method = EncoderMethod::by_id(coder.encoder_method_id());
    let method = if let Some(m) = method {
        m
    } else {
        return Err(Error::UnsupportedCompressionMethod(format!(
            "{:?}",
            coder.encoder_method_id()
        )));
    };
    match method.id() {
        EncoderMethod::ID_COPY => {
            memory.charge(WRAPPER_MEMORY_KIB)?;
            Ok(Decoder::Copy(input))
        }
        EncoderMethod::ID_LZMA => {
            memory.charge(WRAPPER_MEMORY_KIB)?;
            // Validate the length before touching the properties: `get_lzma_dic_size`
            // slices `[1..5]`, which would panic on an attacker-supplied short field.
            if coder.properties.len() < 5 {
                return Err(Error::Other("LZMA properties too short".into()));
            }
            let dict_size = get_lzma_dic_size(coder)?;
            if dict_size > limits.max_dictionary_bytes {
                return Err(Error::ResourceLimit {
                    resource: "lzma_dictionary_bytes",
                    limit: limits.max_dictionary_bytes as usize,
                    required: dict_size as usize,
                });
            }
            let props = coder.properties[0];
            // Invalid properties are reported by `LzmaReader` itself.
            if let Ok(mem_size) = lzma_get_memory_usage_by_props(dict_size, props) {
                memory.charge(mem_size as usize + LZMA_INPUT_BUFFER_KIB)?;
            }
            let lz =
                LzmaReader::new_with_props(input, uncompressed_len as _, props, dict_size, None)
                    .map_err(|e| Error::bad_password(e, !password.is_empty()))?;
            Ok(Decoder::Lzma(Box::new(lz)))
        }
        EncoderMethod::ID_LZMA2 => {
            memory.charge(WRAPPER_MEMORY_KIB)?;
            if limits.decoder_memory_is_bounded() && threads > 1 {
                return Err(Error::ResourceLimit {
                    resource: "decoder_threads",
                    limit: 1,
                    required: threads as usize,
                });
            }
            let dic_size = get_lzma2_dic_size(coder)?;
            if dic_size > limits.max_dictionary_bytes {
                return Err(Error::ResourceLimit {
                    resource: "lzma2_dictionary_bytes",
                    limit: limits.max_dictionary_bytes as usize,
                    required: dic_size as usize,
                });
            }
            let mem_size = lzma2_get_memory_usage(dic_size) as usize;
            memory.charge(mem_size)?;

            let lz = if threads < 2 {
                Decoder::Lzma2(Box::new(Lzma2Reader::new(input, dic_size, None)))
            } else {
                Decoder::Lzma2Mt(Box::new(Lzma2ReaderMt::new(input, dic_size, None, threads)))
            };

            Ok(lz)
        }
        #[cfg(feature = "ppmd")]
        EncoderMethod::ID_PPMD => {
            memory.charge(WRAPPER_MEMORY_KIB)?;
            let (order, memory_size) = get_ppmd_order_memory_size(coder)?;
            memory.charge((memory_size as usize).div_ceil(1024) + PPMD7_STATE_KIB)?;
            let ppmd = Ppmd7Decoder::new(input, order, memory_size)
                .map_err(|err| Error::other(err.to_string()))?;
            Ok(Decoder::Ppmd(Box::new(ppmd)))
        }
        #[cfg(feature = "brotli")]
        EncoderMethod::ID_BROTLI => {
            refuse_unbounded_method(limits)?;
            let de = BrotliDecoder::new(input, 4096)?;
            Ok(Decoder::Brotli(Box::new(de)))
        }
        #[cfg(feature = "bzip2")]
        EncoderMethod::ID_BZIP2 => {
            refuse_unbounded_method(limits)?;
            let de = BzDecoder::new(input);
            Ok(Decoder::Bzip2(de))
        }
        #[cfg(feature = "deflate")]
        EncoderMethod::ID_DEFLATE => {
            refuse_unbounded_method(limits)?;
            let buf_read = std::io::BufReader::new(input);
            let de = DeflateDecoder::new(buf_read);
            Ok(Decoder::Deflate(de))
        }
        #[cfg(feature = "lz4")]
        EncoderMethod::ID_LZ4 => {
            refuse_unbounded_method(limits)?;
            let de = Lz4Decoder::new(input)?;
            Ok(Decoder::Lz4(de))
        }
        #[cfg(feature = "zstd")]
        EncoderMethod::ID_ZSTD => {
            refuse_unbounded_method(limits)?;
            let zs = zstd::Decoder::new(input)?;
            Ok(Decoder::Zstd(zs))
        }
        EncoderMethod::ID_BCJ_X86 => {
            memory.charge(WRAPPER_MEMORY_KIB + BCJ_BUFFER_KIB)?;
            let de = BcjReader::new_x86(input, 0);
            Ok(Decoder::Bcj(de))
        }
        EncoderMethod::ID_BCJ_ARM => {
            memory.charge(WRAPPER_MEMORY_KIB + BCJ_BUFFER_KIB)?;
            let de = BcjReader::new_arm(input, 0);
            Ok(Decoder::Bcj(de))
        }
        EncoderMethod::ID_BCJ_ARM64 => {
            memory.charge(WRAPPER_MEMORY_KIB + BCJ_BUFFER_KIB)?;
            let de = BcjReader::new_arm64(input, 0);
            Ok(Decoder::Bcj(de))
        }
        EncoderMethod::ID_BCJ_ARM_THUMB => {
            memory.charge(WRAPPER_MEMORY_KIB + BCJ_BUFFER_KIB)?;
            let de = BcjReader::new_arm_thumb(input, 0);
            Ok(Decoder::Bcj(de))
        }
        EncoderMethod::ID_BCJ_PPC => {
            memory.charge(WRAPPER_MEMORY_KIB + BCJ_BUFFER_KIB)?;
            let de = BcjReader::new_ppc(input, 0);
            Ok(Decoder::Bcj(de))
        }
        EncoderMethod::ID_BCJ_IA64 => {
            memory.charge(WRAPPER_MEMORY_KIB + BCJ_BUFFER_KIB)?;
            let de = BcjReader::new_ia64(input, 0);
            Ok(Decoder::Bcj(de))
        }
        EncoderMethod::ID_BCJ_SPARC => {
            memory.charge(WRAPPER_MEMORY_KIB + BCJ_BUFFER_KIB)?;
            let de = BcjReader::new_sparc(input, 0);
            Ok(Decoder::Bcj(de))
        }
        EncoderMethod::ID_BCJ_RISCV => {
            memory.charge(WRAPPER_MEMORY_KIB + BCJ_BUFFER_KIB)?;
            let de = BcjReader::new_riscv(input, 0);
            Ok(Decoder::Bcj(de))
        }
        EncoderMethod::ID_DELTA => {
            memory.charge(WRAPPER_MEMORY_KIB)?;
            // The distance is `properties[0] + 1` in the range 1..=256. Widen to `usize`
            // before the `+1` so a property byte of `0xFF` yields 256, not 0 (a `u8`
            // `wrapping_add` would wrap to a zero distance and mis-decode / divide by zero).
            let d = coder.properties.first().map_or(1, |b| *b as usize + 1);
            let de = DeltaReader::new(input, d);
            Ok(Decoder::Delta(de))
        }
        #[cfg(feature = "aes256")]
        EncoderMethod::ID_AES256_SHA256 => {
            memory.charge(WRAPPER_MEMORY_KIB)?;
            if password.is_empty() {
                return Err(Error::PasswordRequired);
            }
            let de = Aes256Sha256Decoder::new(input, &coder.properties, password)?;
            Ok(Decoder::Aes256Sha256(Box::new(de)))
        }
        _ => Err(Error::UnsupportedCompressionMethod(
            method.name().to_string(),
        )),
    }
}

#[cfg(any(
    feature = "brotli",
    feature = "bzip2",
    feature = "deflate",
    feature = "lz4",
    feature = "zstd"
))]
fn refuse_unbounded_method(limits: &ArchiveReaderLimits) -> Result<(), Error> {
    if limits.decoder_memory_is_bounded() {
        Err(Error::ResourceLimit {
            resource: "decoder_memory_unbounded_method",
            limit: limits.max_decoder_memory_kib,
            required: usize::MAX,
        })
    } else {
        Ok(())
    }
}

#[cfg(feature = "ppmd")]
fn get_ppmd_order_memory_size(coder: &Coder) -> Result<(u32, u32), Error> {
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

    Ok((order, memory_size))
}

fn get_lzma2_dic_size(coder: &Coder) -> Result<u32, Error> {
    if coder.properties.is_empty() {
        return Err(Error::other("LZMA2 properties too short"));
    }
    let dict_size_bits = 0xFF & coder.properties[0] as u32;
    if (dict_size_bits & (!0x3F)) != 0 {
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

fn get_lzma_dic_size(coder: &Coder) -> io::Result<u32> {
    let mut props = &coder.properties[1..5];
    props.read_u32()
}
