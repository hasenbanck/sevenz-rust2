//! Tests that run the crate under an allocator which refuses allocations above a per-thread
//! size limit. A refused allocation must make the crate return an error.

use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
    io::Cursor,
};

use sevenz_rust2::{Archive, ArchiveReader, Password};

thread_local! {
    static LIMIT: Cell<usize> = const { Cell::new(usize::MAX) };
}

/// The system allocator with a per-thread limit on the size of one allocation.
struct LimitedAllocator;

unsafe impl GlobalAlloc for LimitedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.size() > LIMIT.with(Cell::get) {
            return std::ptr::null_mut();
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if new_size > LIMIT.with(Cell::get) {
            return std::ptr::null_mut();
        }
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: LimitedAllocator = LimitedAllocator;

/// Runs `f` with allocations larger than `limit` bytes refused on this thread.
fn with_allocation_limit<T>(limit: usize, f: impl FnOnce() -> T) -> T {
    LIMIT.with(|l| l.set(limit));
    let result = f();
    LIMIT.with(|l| l.set(usize::MAX));
    result
}

/// Encodes a value using the 7z NUMBER varint format.
fn write_number(out: &mut Vec<u8>, value: u64) {
    let mut first: u8 = 0;
    let mut mask: u8 = 0x80;
    let mut low = Vec::new();
    let mut i = 0u32;
    while i < 8 {
        if value < (1u64 << (7 * (i + 1))) {
            first |= (value >> (8 * i)) as u8;
            break;
        }
        first |= mask;
        mask >>= 1;
        low.push((value >> (8 * i)) as u8);
        i += 1;
    }
    out.push(first);
    out.extend_from_slice(&low);
}

/// Builds an archive whose start header describes `next_header` after it.
fn raw_7z_exact(next_header: &[u8]) -> Vec<u8> {
    let mut start = Vec::new();
    start.extend_from_slice(&0u64.to_le_bytes());
    start.extend_from_slice(&(next_header.len() as u64).to_le_bytes());
    start.extend_from_slice(&crc32fast::hash(next_header).to_le_bytes());
    let mut out = vec![0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C, 0x00, 0x04];
    out.extend_from_slice(&crc32fast::hash(&start).to_le_bytes());
    out.extend_from_slice(&start);
    out.extend_from_slice(next_header);
    out
}

/// A plain header with `num_files` empty entries with unique two-character names.
fn unique_names_header(num_files: u32) -> Vec<u8> {
    let mut nh = vec![0x01, 0x05]; // kHeader, kFilesInfo
    write_number(&mut nh, num_files as u64);
    let bits = num_files.div_ceil(8) as u64;
    nh.push(0x0E); // kEmptyStream
    write_number(&mut nh, bits);
    nh.resize(nh.len() + bits as usize, 0xFF);
    nh.push(0x11); // kName
    write_number(&mut nh, 6 * num_files as u64 + 1);
    nh.push(0x00); // external
    for i in 0..num_files {
        nh.extend_from_slice(&(0x100 + (i / 40000) as u16).to_le_bytes());
        nh.extend_from_slice(&(0x100 + (i % 40000) as u16).to_le_bytes());
        nh.extend_from_slice(&[0, 0]);
    }
    nh.extend_from_slice(&[0x00, 0x00]); // files_info props end, header end
    nh
}

/// The name index of 100,000 entries needs more than 1 MiB in one allocation.
/// `ArchiveReader::from_archive` cannot return an error. Reading an entry must return one.
#[test]
fn name_index_allocation_failure_returns_an_error() {
    let mut source = Cursor::new(raw_7z_exact(&unique_names_header(100_000)));
    let archive = Archive::read(&mut source, &Password::empty()).unwrap();
    let name = archive.files[0].name().to_string();

    let result = with_allocation_limit(1 << 20, || {
        let mut reader = ArchiveReader::from_archive(archive, source, Password::empty());
        reader.read_file(&name).map(|_| ())
    });

    assert!(
        result.is_err(),
        "a name index that does not fit must be an error"
    );
}
