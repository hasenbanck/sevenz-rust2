# LZMA / LZMA2 in native Rust

[![Crate](https://img.shields.io/crates/v/lzma-rust2.svg)](https://crates.io/crates/lzma-rust2)
[![Documentation](https://docs.rs/lzma-rust2/badge.svg)](https://docs.rs/lzma-rust2)

LZMA/LZMA2 codec ported from [tukaani xz for java](https://tukaani.org/xz/java.html).

This is a fork of the original, unmaintained lzma-rust crate to continue the development and maintenance.

## Usage

### lzma

```rust
use std::io::{Read, Write};
use lzma_rust2::*;

let s = b"Hello, world!";
let mut out = Vec::new();
let mut options = LZMA2Options::with_preset(6);
options.dict_size = LZMA2Options::DICT_SIZE_DEFAULT;

let mut w = LZMAWriter::new_use_header(CountingWriter::new( & mut out), & options, None).unwrap();
w.write_all(s).unwrap();
w.write( & []).unwrap();
let mut r = LZMAReader::new_mem_limit( & out[..], u32::MAX, None).unwrap();
let mut s2 = vec![0; s.len()];
r.read_exact( & mut s2).unwrap();
println!("{:?}", &out[..]);
assert_eq!(s, &s2[..]);
```

### lzma2

```rust
use std::io::{Read, Write};
use lzma_rust2::*;

let s = b"Hello, world!";
let mut out = Vec::new();
let mut options = LZMA2Options::with_preset(6);
options.dict_size = LZMA2Options::DICT_SIZE_DEFAULT;
{
let mut w = LZMA2Writer::new(CountingWriter::new( & mut out), & options);
w.write_all(s).unwrap();
w.write( & []).unwrap();
}
let mut r = LZMA2Reader::new( & out[..], options.dict_size, None);
let mut s2 = vec![0; s.len()];
r.read_exact( & mut s2).unwrap();
println!("{:?}", &out[..]);
assert_eq!(s, &s2[..]);
````
