# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

### Unreleased

### Added

- Optional support for compressing with ZStandard

### Changed

- Replaced all unsafe code from svenz-rust2 crate

### 0.7.0 - 2025-02-24

This release should be mostly compatible with the old 0.6.1. The breaking changes are:

- Previously deprecated functionality were removed
- Spelling issues were fixed in some API names

### Added

- `SevenZReader::readFile()` added
- `SevenZArchiveEntry::new_file()` and `SevenZArchiveEntry::new_folder()` factory functions added
- Compression method COPY is now supported

### Changed

- Updated dependency bit-set v0.8
- Updated dependency bzip2 v0.5
- Updated dependency nt-time v0.10
- Use crc32fast instead of crc

### Fixed

- Replaces insecure usage of rand with getrandom
- Renamed `get_memery_usage()` into `get_memory_usage()`
- Renamed `compress_encypted()` into `compress_encrypted()`

### Removed

- Removed deprecated `FolderDecoder`. Use `BlockDecoder` instead
- Removed deprecated `SevenZWriter::create_archive_entry()`. Use `SevenZArchiveEntry::from_path()` instead

### 0.6.1 - 2024-07-17

- Fixed 'unsafe precondition(s) violated'. Closed #63

### 0.6.0 - 2024-04-05

- Added support for encrypted headers - close #55
- Return a consistent error in case the password is invalid - close #53

### 0.5.4 - 2023-12-13

- Added docs
- Renamed `FolderDecoder` to `BlockDecoder`
- Added method to compress paths in non-solid mode
- Fixed entry's compressed_size is always 0 when reading archives.

### 0.5.3

Fixed 'Too many open files'
Reduce unnecessary public items #37

### 0.5.2 - 2023-08-24

Fixed file separator issue on Windows system #35

### 0.5.1 - 2023-08-23

Sub crate `lzma-rust` code optimization

### 0.5.0 - 2023-08-19

- Added support for BCJ2.
- Added multi-thread decompress example

### 0.4.3 - 2023-06-16

- Support write encoded header
- Added `LZMAWriter`

### 0.4.2 - 2023-06-10

- Removed unsafe code
- Changed `SevenZWriter.finish` method return inner writer
- Added wasm compress function
- Updates bzip dependency to the patch version of 0.4.4([#23](https://github.com/dyz1990/sevenz-rust/pull/23))

### 0.4.1 - 2023-06-07

- Fixed unable to build without default features

### 0.4.0 - 2023-06-03 - Solid compression

### 0.3.0 - 2023-06-02 - Encrypted compression

- Added Encrypted compression

### 0.2.11 - 2023-05-24

- Fixed numerical overflow

### 0.2.10 - 2023-04-18

- Change to use nt-time crate([#20](https://github.com/dyz1990/sevenz-rust/pull/20))
- Fix typo([#18](https://github.com/dyz1990/sevenz-rust/pull/18))
- make function generics less restrictive ([#17](https://github.com/dyz1990/sevenz-rust/pull/17))
- Solve warnings ([#16](https://github.com/dyz1990/sevenz-rust/pull/16))
- run rustfmt on code ([#15](https://github.com/dyz1990/sevenz-rust/pull/15))

### 0.2.9 - 2023-03-16

- Added bzip2 support([#14](https://github.com/dyz1990/sevenz-rust/pull/14))

### 0.2.8 - 2023-03-06

- Fixed write bitset bugs

### 0.2.7 - 2023-03-05

- Fixed bug while read files info

### 0.2.6 - 2023-02-23

- Added zstd support and use enhanced filetime lib([#11](https://github.com/dyz1990/sevenz-rust/pull/11))
- Fixed lzma encoder bugs

### 0.2.4 - 2023-02-16

- Changed return entry ref when pushing to writer([#10](https://github.com/dyz1990/sevenz-rust/pull/10))

### 0.2.3 - 2023-02-07

- Fixed incorrect handling of 7z time

### 0.2.2 - 2023-01-31 - Create sub crate `lzma-rust`

- Move mod `lzma` to sub crate `lzma-rust`
- Modify GitHub Actions to run tests with --all-features

### 0.2.0 - 2023-01-08 - Added compression supporting

- Added compression supporting

### 0.1.5 - 2022-11-01 - Encrypted 7z files decompression supported

- Added aes256sha256 decode method
- Added wasm support
- Added new tests (for Delta and Copy) and GitHub Actions CI([#5](https://github.com/dyz1990/sevenz-rust/pull/5))
  by [bfrazho](https://github.com/bfrazho)

### 0.1.4 - 2022-09-20 - Replace lzma/lzma2 decoder

- Chnaged new lzma/lzma2 decoder

### 0.1.3 - 2022-09-18 - add more bcj filters

- Added bcj arm/ppc/sparc and delta filters
- Added test for bcj x86 ([#3](https://github.com/dyz1990/sevenz-rust/pull/3)) by [bfrazho](https://github.com/bfrazho)

### 0.1.2 - 2022-09-14 - bcj x86 filter supported

- Added bcj x86 filter
- Added LZMA tests ([#2](https://github.com/dyz1990/sevenz-rust/pull/2)) by [bfrazho](https://github.com/bfrazho)
- Fixed extract empty file

### 0.1.1 - 2022-08-10 - Modify decompression function

### 0.1.0 - 2022-08-10 - Decompression
