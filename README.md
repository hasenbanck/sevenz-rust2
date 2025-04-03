[![Crate](https://img.shields.io/crates/v/sevenz-rust2.svg)](https://crates.io/crates/sevenz-rust2)
[![Documentation](https://docs.rs/sevenz-rust2/badge.svg)](https://docs.rs/sevenz-rust2)

This project is a 7z compressor/decompressor written in pure Rust.

This is a fork of the original, unmaintained sevenz-rust crate to continue the development and maintenance.

## Supported Codecs & filters

| Codec       | Decompression | Compression |
|-------------|---------------|-------------|
| COPY        | ✓             | ✓           |
| LZMA        | ✓             | ✓           |
| LZMA2       | ✓             | ✓           |
| BROTLI (*)  | ✓             | ✓           |
| BZIP2 (*)   | ✓             | ✓           |
| DEFLATE (*) | ✓             | ✓           |
| PPMD (*)    | ✓             | ✓           |
| LZ4 (*)     | ✓             | ✓           |
| ZSTD (*)    | ✓             | ✓           |

(*) Require optional cargo feature.

| Filter        | Decompression | Compression |
|---------------|---------------|-------------|
| BCJ X86       | ✓             |             |
| BCJ PPC       | ✓             |             |
| BCJ IA64      | ✓             |             |
| BCJ ARM       | ✓             |             |
| BCJ ARM_THUMB | ✓             |             |
| BCJ SPARC     | ✓             |             |
| DELTA         | ✓             | ✓           |
| BCJ2          | ✓             |             |

### Usage

```toml
[dependencies]
sevenz-rust = { version = "0.17" }
```

Decompress source file "data/sample.7z" to destination path "data/sample":

```rust
sevenz_rust2::decompress_file("data/sample.7z", "data/sample").expect("complete");
```

#### Decompress an encrypted 7z file

Add the 'aes256' feature:

```toml
[dependencies]
sevenz-rust2 = { version = "0.17", features = ["aes256"] }
```

```rust
sevenz_rust2::decompress_file_with_password("path/to/encrypted.7z", "path/to/output", "password".into()).expect("complete");
```

## Compression

Add the 'compress' feature:

```toml
[dependencies]
sevenz-rust2 = { version = "0.17", features = ["compress"] }
```

Use the helper function to create a 7z file with source path:

```rust
sevenz_rust2::compress_to_path("examples/data/sample", "examples/data/sample.7z").expect("compress ok");
```

### Compress with AES encryption

Add the 'compress' and 'aes256' feature:

```toml
[dependencies]
sevenz-rust2 = { version = "0.17", features = ["compress", "aes256"] }
```

Use the helper function to create a 7z file with source path and password:

```rust
sevenz_rust2::compress_to_path_encrypted("examples/data/sample", "examples/data/sample.7z", "password".into()).expect("compress ok");
```

### Advanced Usage

#### Solid compression

Solid archives can in theory provide better compression rates, but decompressing a file needs all previous data to also
be decompressed.

```rust
use sevenz_rust2::*;

let mut sz = SevenZWriter::create("dest.7z").expect("create writer ok");
sz.push_source_path("path/to/compress", | _ | true).expect("pack ok");
sz.finish().expect("compress ok");
```

#### Configure the compression methods

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

### WASM support

Not all optional features are supported. To build default feature and compression:

```bash
cargo build --target wasm32-unknown-unknown --features=compress
```

Encryption is supported, but need a special feature and RUSTFLAGS option:

```bash
RUSTFLAGS='--cfg getrandom_backend="wasm_js"' cargo build --target wasm32-unknown-unknown --features=aes256_wasm
```

## Licence

Licensed under the [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0).
