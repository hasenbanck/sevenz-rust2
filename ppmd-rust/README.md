[![Crate](https://img.shields.io/crates/v/ppmd-rust.svg)](https://crates.io/crates/ppmd-rust)
[![Documentation](https://docs.rs/ppmd-rust/badge.svg)](https://docs.rs/ppmd-rust)

PPMd compression / decompression. It's a port of the PPMd C-code from 7-Zip to Rust.
Only the PPMd7 (PPMdH) with the 7z range coder is ported (as used by the 7z archive format).

## Acknowledgement

This port is based on the 7zip version of PPMd by Igor Pavlov, which in turn was based on the PPMd var.H (2001)
code by Dmitry Shkarin.

## License

The code in this crate is in the public domain as the original code by their authors.
