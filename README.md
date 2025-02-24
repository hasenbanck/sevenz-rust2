[![Crate](https://img.shields.io/crates/v/sevenz-rust2.svg)](https://crates.io/crates/sevenz-rust2)
[![Documentation](https://docs.rs/sevenz-rust2/badge.svg)](https://docs.rs/sevenz-rust2)

This project is a 7z compressor/decompressor written in pure Rust.

And it's very much inspired by the [apache commons-compress](https://commons.apache.org/proper/commons-compress/)
project.

The LZMA/LZMA2 decoder and all filters code was ported from [tukaani xz for java](https://tukaani.org/xz/java.html)

This is a fork of the original, unmaintained sevenz-rust crate to continue the development and maintenance.

## Decompression

Supported codecs:

- [x] BZIP2 (requires feature 'bzip2')
- [x] COPY
- [x] LZMA
- [x] LZMA2
- [x] ZSTD  (requires feature 'zstd')

Supported filters:

- [x] BCJ X86
- [x] BCJ PPC
- [x] BCJ IA64
- [x] BCJ ARM
- [x] BCJ ARM_THUMB
- [x] BCJ SPARC
- [x] DELTA
- [x] BJC2

### Usage

```toml
[dependencies]
sevenz-rust = { version = "0.7" }
```

Decompress source file "data/sample.7z" to destination path "data/sample":

```rust
sevenz_rust2::decompress_file("data/sample.7z", "data/sample").expect("complete");
```

#### Decompress an encrypted 7z file

Add the 'aes256' feature:

```toml
[dependencies]
sevenz-rust2 = { version = "0.7", features = ["aes256"] }
```

```rust
sevenz_rust2::decompress_file_with_password("path/to/encrypted.7z", "path/to/output", "password".into()).expect("complete");
```

#### Multi-thread decompress

Please check [examples/mt_decompress](https://github.com/hasenbanck/sevenz-rust2/blob/main/examples/mt_decompress.rs)

## Compression

Currently, this crate only supports the COPY, LZMA2 and optionally ZStandard compression algorithm.

```toml
[dependencies]
sevenz-rust2 = { version = "0.7", features = ["compress"] }
```

Use the helper function to create a 7z file with source path:

```rust
sevenz_rust2::compress_to_path("examples/data/sample", "examples/data/sample.7z").expect("compress ok");
```

### With AES encryption

```toml
[dependencies]
sevenz-rust2 = { version = "0.7", features = ["compress", "aes256"] }
```

Use the helper function to create a 7z file with source path and password:

```rust
sevenz_rust2::compress_to_path_encrypted("examples/data/sample", "examples/data/sample.7z", "password".into()).expect("compress ok");
```

### Advance

```toml
[dependencies]
sevenz-rust2 = { version = "0.7", features = ["compress", "aes256"] }
```

#### Solid compression

```rust
use sevenz_rust2::*;

let mut sz = SevenZWriter::create("dest.7z").expect("create writer ok");
sz.push_source_path("path/to/compress", | _ | true).expect("pack ok");
sz.finish().expect("compress ok");
```

#### Compression methods

With encryption and lzma2 options:

```rust
use sevenz_rust2::*;

let mut sz = SevenZWriter::create("dest.7z").expect("create writer ok");
sz.set_content_methods(vec![
    sevenz_rust::AesEncoderOptions::new("sevenz-rust".into()).into(),
    lzma::LZMA2Options::with_preset(9).into(),
]);
sz.push_source_path("path/to/compress", | _ | true).expect("pack ok");
sz.finish().expect("compress ok");
```

## Licence

Licensed under the [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0).
