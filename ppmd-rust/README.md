# PPMd in native Rust

[![Crate](https://img.shields.io/crates/v/ppmd-rust.svg)](https://crates.io/crates/ppmd-rust)
[![Documentation](https://docs.rs/ppmd-rust/badge.svg)](https://docs.rs/ppmd-rust)

PPMd compression / decompression. It's a port of the PPMd C-code from 7-Zip to Rust.

The following variants are provided:

- The PPMd7 (PPMdH) as used by the 7z archive format
- The PPMd8 (PPMdI rev.1) as used by the zip archive format

## Acknowledgement

This port is based on the 7zip version of PPMd by Igor Pavlov, which in turn was based on the PPMd var.H (2001) /
PPMd var.I (2002) code by Dmitry Shkarin.

## License

The code in this crate is in the public domain as the original code by their authors.
