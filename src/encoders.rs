#[cfg(feature = "aes256")]
use crate::aes256sha256::Aes256Sha256Encoder;
use crate::{
    archive::{SevenZMethod, SevenZMethodConfiguration},
    lzma::CountingWriter,
    lzma::{LZMA2Options, LZMA2Writer, LZMAWriter},
    method_options::MethodOptions,
    Error,
};
use std::io::Write;

#[cfg(feature = "brotli")]
use crate::{brotli::BrotliEncoder, BrotliOptions};

#[cfg(feature = "bzip2")]
use crate::Bzip2Options;

#[cfg(feature = "deflate")]
use crate::DeflateOptions;

#[cfg(feature = "lz4")]
use crate::LZ4Options;

#[cfg(feature = "zstd")]
use crate::ZStandardOptions;

#[allow(clippy::upper_case_acronyms)]
pub enum Encoder<W: Write> {
    COPY(CountingWriter<W>),
    LZMA(LZMAWriter<W>),
    LZMA2(LZMA2Writer<W>),
    #[cfg(feature = "brotli")]
    BROTLI(BrotliEncoder<CountingWriter<W>>),
    #[cfg(feature = "bzip2")]
    BZIP2(bzip2::write::BzEncoder<CountingWriter<W>>),
    #[cfg(feature = "deflate")]
    DEFLATE(flate2::write::DeflateEncoder<CountingWriter<W>>),
    #[cfg(feature = "lz4")]
    LZ4(lz4::Encoder<CountingWriter<W>>),
    #[cfg(feature = "zstd")]
    ZSTD(zstd::Encoder<'static, CountingWriter<W>>),
    #[cfg(feature = "aes256")]
    AES(Aes256Sha256Encoder<W>),
}

impl<W: Write> Write for Encoder<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Encoder::COPY(w) => w.write(buf),
            Encoder::LZMA(w) => w.write(buf),
            Encoder::LZMA2(w) => w.write(buf),
            #[cfg(feature = "brotli")]
            Encoder::BROTLI(w) => w.write(buf),
            #[cfg(feature = "bzip2")]
            Encoder::BZIP2(w) => w.write(buf),
            #[cfg(feature = "deflate")]
            Encoder::DEFLATE(w) => w.write(buf),
            #[cfg(feature = "lz4")]
            Encoder::LZ4(w) => w.write(buf),
            #[cfg(feature = "zstd")]
            Encoder::ZSTD(w) => w.write(buf),
            #[cfg(feature = "aes256")]
            Encoder::AES(w) => w.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Encoder::COPY(w) => w.flush(),
            Encoder::LZMA(w) => w.flush(),
            Encoder::LZMA2(w) => w.flush(),
            #[cfg(feature = "brotli")]
            Encoder::BROTLI(w) => w.flush(),
            #[cfg(feature = "bzip2")]
            Encoder::BZIP2(w) => w.flush(),
            #[cfg(feature = "deflate")]
            Encoder::DEFLATE(w) => w.flush(),
            #[cfg(feature = "lz4")]
            Encoder::LZ4(w) => w.flush(),
            #[cfg(feature = "zstd")]
            Encoder::ZSTD(w) => w.flush(),
            #[cfg(feature = "aes256")]
            Encoder::AES(w) => w.flush(),
        }
    }
}

pub fn add_encoder<W: Write>(
    input: CountingWriter<W>,
    method_config: &SevenZMethodConfiguration,
) -> Result<Encoder<W>, Error> {
    let method = method_config.method;

    match method.id() {
        SevenZMethod::ID_COPY => Ok(Encoder::COPY(input)),
        SevenZMethod::ID_LZMA => {
            let mut def_opts = LZMA2Options::default();
            let options = get_lzma2_options(method_config.options.as_ref(), &mut def_opts);
            let lz = LZMAWriter::new_no_header(input, options, false).map_err(Error::io)?;
            Ok(Encoder::LZMA(lz))
        }
        SevenZMethod::ID_LZMA2 => {
            let mut def_opts = LZMA2Options::default();
            let options = get_lzma2_options(method_config.options.as_ref(), &mut def_opts);
            let lz = LZMA2Writer::new(input, options);
            Ok(Encoder::LZMA2(lz))
        }
        #[cfg(feature = "brotli")]
        SevenZMethod::ID_BROTLI => {
            let options = match method_config.options {
                Some(MethodOptions::BROTLI(options)) => options,
                _ => BrotliOptions::default(),
            };

            let brotli_encoder = BrotliEncoder::new(
                input,
                options.quality,
                options.window,
                options.skippable_frame_size as usize,
            )?;

            Ok(Encoder::BROTLI(brotli_encoder))
        }
        #[cfg(feature = "bzip2")]
        SevenZMethod::ID_BZIP2 => {
            let options = match method_config.options {
                Some(MethodOptions::BZIP2(options)) => options,
                _ => Bzip2Options::default(),
            };

            let bzip2_encoder =
                bzip2::write::BzEncoder::new(input, bzip2::Compression::new(options.0));
            Ok(Encoder::BZIP2(bzip2_encoder))
        }
        #[cfg(feature = "deflate")]
        SevenZMethod::ID_DEFLATE => {
            let options = match method_config.options {
                Some(MethodOptions::DEFLATE(options)) => options,
                _ => DeflateOptions::default(),
            };

            let deflate_encoder =
                flate2::write::DeflateEncoder::new(input, flate2::Compression::new(options.0));
            Ok(Encoder::DEFLATE(deflate_encoder))
        }
        #[cfg(feature = "lz4")]
        SevenZMethod::ID_LZ4 => {
            let options = match method_config.options.as_ref() {
                Some(MethodOptions::LZ4(options)) => *options,
                _ => LZ4Options::default(),
            };

            let lz4_encoder = lz4::EncoderBuilder::new()
                .level(options.0)
                .build(input)
                .map_err(Error::io)?;
            Ok(Encoder::LZ4(lz4_encoder))
        }
        #[cfg(feature = "zstd")]
        SevenZMethod::ID_ZSTD => {
            let options = match method_config.options.as_ref() {
                Some(MethodOptions::ZSTD(options)) => *options,
                _ => ZStandardOptions::default(),
            };
            let zstd_encoder = zstd::Encoder::new(input, options.0 as i32).map_err(Error::io)?;
            Ok(Encoder::ZSTD(zstd_encoder))
        }
        #[cfg(feature = "aes256")]
        SevenZMethod::ID_AES256SHA256 => {
            let options = match method_config.options.as_ref() {
                Some(MethodOptions::Aes(p)) => p,
                _ => return Err(Error::PasswordRequired),
            };
            Ok(Encoder::AES(Aes256Sha256Encoder::new(input, options)?))
        }
        _ => Err(Error::UnsupportedCompressionMethod(
            method.name().to_string(),
        )),
    }
}

pub(crate) fn get_options_as_properties<'a>(
    method: SevenZMethod,
    options: Option<&MethodOptions>,
    out: &'a mut [u8],
) -> &'a [u8] {
    match method.id() {
        SevenZMethod::ID_LZMA2 => {
            let dict_size = options
                .map(|o| o.get_lzma2_dict_size())
                .unwrap_or(LZMA2Options::DICT_SIZE_DEFAULT);
            let lead = dict_size.leading_zeros();
            let second_bit = (dict_size >> (30u32.wrapping_sub(lead))).wrapping_sub(2);
            let prop = (19u32.wrapping_sub(lead) * 2 + second_bit) as u8;
            out[0] = prop;
            &out[0..1]
        }
        SevenZMethod::ID_LZMA => {
            let mut def_opts = LZMA2Options::default();
            let options = get_lzma2_options(options, &mut def_opts);
            let dict_size = options.dict_size;
            out[0] = options.get_props();
            out[1..5].copy_from_slice(dict_size.to_le_bytes().as_ref());
            &out[0..5]
        }
        #[cfg(feature = "brotli")]
        SevenZMethod::ID_BROTLI => {
            let version_major = brotli::VERSION;
            let version_minor = 0;
            let options = match options {
                Some(MethodOptions::BROTLI(options)) => *options,
                _ => BrotliOptions::default(),
            };

            out[0] = version_major;
            out[1] = version_minor;
            out[2] = options.quality as u8;
            &out[0..3]
        }
        #[cfg(feature = "lz4")]
        SevenZMethod::ID_LZ4 => {
            let version = lz4::version();
            let version_major = version / (10000);
            let version_minor = (version / 100) % 100;
            let options = match options {
                Some(MethodOptions::LZ4(options)) => *options,
                _ => LZ4Options::default(),
            };

            out[0] = version_major as u8;
            out[1] = version_minor as u8;
            out[2] = options.0 as u8;
            &out[0..3]
        }
        #[cfg(feature = "zstd")]
        SevenZMethod::ID_ZSTD => {
            let version_major = zstd::zstd_safe::VERSION_MAJOR;
            let version_minor = zstd::zstd_safe::VERSION_MINOR;
            let options = match options {
                Some(MethodOptions::ZSTD(options)) => *options,
                _ => ZStandardOptions::default(),
            };

            out[0] = version_major as u8;
            out[1] = version_minor as u8;
            out[2] = options.0 as u8;
            &out[0..3]
        }
        #[cfg(feature = "aes256")]
        SevenZMethod::ID_AES256SHA256 => {
            let options = match options.as_ref() {
                Some(MethodOptions::Aes(p)) => p,
                _ => return &[],
            };
            options.write_properties(out);
            &out[..34]
        }
        _ => &[],
    }
}

#[inline]
pub(crate) fn get_lzma2_options<'a>(
    options: Option<&'a MethodOptions>,
    def_opt: &'a mut LZMA2Options,
) -> &'a LZMA2Options {
    let options = match options.as_ref() {
        Some(MethodOptions::LZMA2(opts)) => opts,
        Some(MethodOptions::Num(n)) => {
            def_opt.dict_size = *n;
            def_opt
        }
        _ => {
            def_opt.dict_size = LZMA2Options::DICT_SIZE_DEFAULT;
            def_opt
        }
    };
    options
}
