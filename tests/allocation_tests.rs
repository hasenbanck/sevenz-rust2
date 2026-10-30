//! Tests that run the crate under an allocator which refuses allocations above a per-thread
//! size limit or beyond a per-thread budget of live bytes. A refused allocation must make the
//! crate return an error.

use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
    io::Cursor,
};

use sevenz_rust2::{Archive, ArchiveReader, Password};

/// No limit and no budget.
const UNLIMITED: usize = usize::MAX;

thread_local! {
    static LIMIT: Cell<usize> = const { Cell::new(UNLIMITED) };
    static BUDGET: Cell<usize> = const { Cell::new(UNLIMITED) };
}

/// Takes `size` bytes from the budget of this thread. Returns false when they do not fit.
fn take_budget(size: usize) -> bool {
    BUDGET.with(|budget| match budget.get() {
        UNLIMITED => true,
        left if size <= left => {
            budget.set(left - size);
            true
        }
        _ => false,
    })
}

/// Returns `size` bytes to the budget of this thread.
fn return_budget(size: usize) {
    BUDGET.with(|budget| {
        if budget.get() != UNLIMITED {
            budget.set(budget.get().saturating_add(size).min(UNLIMITED - 1));
        }
    });
}

/// The system allocator with a per-thread limit on the size of one allocation and a
/// per-thread budget of live bytes.
struct LimitedAllocator;

unsafe impl GlobalAlloc for LimitedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.size() > LIMIT.with(Cell::get) || !take_budget(layout.size()) {
            return std::ptr::null_mut();
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        return_budget(layout.size());
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if new_size > LIMIT.with(Cell::get) {
            return std::ptr::null_mut();
        }
        let old_size = layout.size();
        if new_size > old_size && !take_budget(new_size - old_size) {
            return std::ptr::null_mut();
        }
        let new_ptr = unsafe { System.realloc(ptr, layout, new_size) };
        if new_ptr.is_null() {
            return_budget(new_size.saturating_sub(old_size));
        } else if new_size < old_size {
            return_budget(old_size - new_size);
        }
        new_ptr
    }
}

#[global_allocator]
static GLOBAL: LimitedAllocator = LimitedAllocator;

/// Runs `f` with allocations larger than `limit` bytes refused on this thread.
fn with_allocation_limit<T>(limit: usize, f: impl FnOnce() -> T) -> T {
    LIMIT.with(|l| l.set(limit));
    let result = f();
    LIMIT.with(|l| l.set(UNLIMITED));
    result
}

/// Runs `f` with at most `budget` bytes of new allocations live on this thread.
fn with_allocation_budget<T>(budget: usize, f: impl FnOnce() -> T) -> T {
    BUDGET.with(|b| b.set(budget));
    let result = f();
    BUDGET.with(|b| b.set(UNLIMITED));
    result
}

/// Returns true when `result` failed because an allocation was refused.
fn is_allocation_error<T: std::fmt::Debug>(result: &Result<T, sevenz_rust2::Error>) -> bool {
    matches!(result, Err(e) if e.to_string().contains("cannot allocate"))
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
        matches!(&result, Err(e) if e.to_string().contains("index")),
        "a name index that does not fit must be an error, got {result:?}"
    );
}

/// A plain header with one empty entry whose name holds `units`.
fn one_name_header(units: &[u16]) -> Vec<u8> {
    let mut nh = vec![
        0x01, // kHeader
        0x05, // kFilesInfo
        0x01, // num_files = 1
        0x0E, // kEmptyStream
        0x01, // property size
        0x80, // entry 0 has no stream
        0x11, // kName
    ];
    write_number(&mut nh, 2 * units.len() as u64 + 3);
    nh.push(0x00); // external
    for unit in units {
        nh.extend_from_slice(&unit.to_le_bytes());
    }
    nh.extend_from_slice(&[0x00, 0x00]); // name terminator
    nh.extend_from_slice(&[0x00, 0x00]); // files_info props end, header end
    nh
}

/// A name of 2^20 + 1 ASCII units grows its UTF-16 buffer to 4 MiB. A name of 1,000,000 CJK
/// units needs 3,000,000 bytes as UTF-8. Both allocations exceed the limit, and the header
/// fits under it. Opening the archive must return an error.
#[test]
fn entry_name_allocation_failure_returns_an_error() {
    let cases: [(&str, Vec<u16>, usize); 2] = [
        ("UTF-16 buffer", vec![0x61; (1 << 20) + 1], 3 << 20),
        ("UTF-8 name", vec![0x4E00; 1_000_000], 5 << 19),
    ];
    for (name, units, limit) in cases {
        let bytes = raw_7z_exact(&one_name_header(&units));
        assert!(
            Archive::read(&mut Cursor::new(bytes.clone()), &Password::empty()).is_ok(),
            "{name}: the archive must open without a limit"
        );

        let result = with_allocation_limit(limit, || {
            Archive::read(&mut Cursor::new(bytes), &Password::empty()).map(|_| ())
        });
        assert!(
            matches!(&result, Err(e) if e.to_string().contains("entry name")),
            "{name}: a name that does not fit must be an error, got {result:?}"
        );
    }
}

/// A plain header with three blocks of two COPY coders each. The second coder of a block has
/// properties and feeds the first one through a bind pair. The header ends after the unpack
/// info.
fn small_blocks_header() -> Vec<u8> {
    let mut nh = vec![
        0x01, // kHeader
        0x04, // kMainStreamsInfo
        0x06, // kPackInfo
        0x00, // pack_pos
        0x03, // num_pack_streams
        0x09, // kSize
        0x01, 0x01, 0x01, // pack_sizes
        0x00, // pack_info end
        0x07, // kUnpackInfo
        0x0B, // kFolder
        0x03, // num_blocks
        0x00, // external
    ];
    for _ in 0..3 {
        nh.extend_from_slice(&[
            0x02, // num_coders
            0x01, 0x00, // simple COPY coder
            0x21, 0x00, 0x02, 0xAA, 0xBB, // simple COPY coder with two property bytes
            0x00, 0x01, // bind pair: in_index 0, out_index 1
        ]);
    }
    nh.push(0x0C); // kCodersUnpackSize
    nh.extend_from_slice(&[0x01; 6]);
    nh.push(0x00); // unpack_info end
    nh
}

/// Opens `bytes` under every budget from 0 bytes up until the result is no allocation error.
/// Under each budget, opening must return. That result must equal the result without a
/// budget.
fn assert_returns_under_every_budget(bytes: &[u8]) {
    let unlimited = format!(
        "{:?}",
        Archive::read(&mut Cursor::new(bytes.to_vec()), &Password::empty()).map(|_| ())
    );
    for budget in 0..65536 {
        let mut source = Cursor::new(bytes.to_vec());
        let result = with_allocation_budget(budget, || {
            Archive::read(&mut source, &Password::empty()).map(|_| ())
        });
        if !is_allocation_error(&result) {
            assert_eq!(format!("{result:?}"), unlimited, "budget {budget}");
            assert!(budget > 0, "the smallest budget must be refused");
            return;
        }
    }
    panic!("opening must finish under a budget below 64 KiB");
}

/// Each block allocates its coders, properties, bind pairs and streams. Opening the archive
/// must return under every budget.
#[test]
fn block_allocation_failure_returns_an_error() {
    assert_returns_under_every_budget(&raw_7z_exact(&small_blocks_header()));
}

/// One coder declares 1 MiB of properties. The budget holds the header, which contains them,
/// and little more. Opening the archive must return an allocation error.
#[test]
fn coder_properties_allocation_failure_returns_an_error() {
    const PROPERTIES_SIZE: usize = 1 << 20;
    let mut header = vec![
        0x01, // kHeader
        0x04, // kMainStreamsInfo
        0x07, // kUnpackInfo
        0x0B, // kFolder
        0x01, // num_blocks
        0x00, // external
        0x01, // num_coders
        0x21, // coder flags: id_size=1, simple, has attributes
        0x00, // coder id = COPY
    ];
    write_number(&mut header, PROPERTIES_SIZE as u64);
    header.resize(header.len() + PROPERTIES_SIZE, 0x00);
    let budget = header.len() + 4096;
    let mut source = Cursor::new(raw_7z_exact(&header));

    let result = with_allocation_budget(budget, || {
        Archive::read(&mut source, &Password::empty()).map(|_| ())
    });

    assert!(
        is_allocation_error(&result),
        "coder properties that do not fit must be an allocation error, got {result:?}"
    );
}

/// A plain header with two COPY blocks, CRCs for the pack streams, the blocks and the
/// sub-streams, and six entries. Three entries have streams. The others are an empty file, an
/// anti item and a directory. The entries have names and the time and attribute `properties`.
fn complete_header(properties: &[&[u8]]) -> Vec<u8> {
    let mut nh = vec![
        0x01, // kHeader
        0x04, // kMainStreamsInfo
        0x06, // kPackInfo
        0x00, // pack_pos
        0x02, // num_pack_streams
        0x09, // kSize
        0x02, 0x01, // pack_sizes
        0x0A, // kCRC
        0x01, // all defined
    ];
    nh.extend_from_slice(&[0x11; 8]); // pack stream CRCs
    nh.extend_from_slice(&[
        0x00, // pack_info end
        0x07, // kUnpackInfo
        0x0B, // kFolder
        0x02, // num_blocks
        0x00, // external
        0x01, 0x01, 0x00, // block 0: one simple COPY coder
        0x01, 0x01, 0x00, // block 1: one simple COPY coder
        0x0C, // kCodersUnpackSize
        0x02, 0x01, // unpack sizes
        0x0A, // kCRC
        0x00, // not all defined
        0x40, // block 1 has a CRC
    ]);
    nh.extend_from_slice(&[0x22; 4]); // block 1 CRC
    nh.extend_from_slice(&[
        0x00, // unpack_info end
        0x08, // kSubStreamsInfo
        0x0D, // kNumUnpackStream
        0x02, 0x01, // sub-streams per block
        0x09, // kSize
        0x01, // first sub-stream of block 0
        0x0A, // kCRC
        0x00, // not all defined
        0x80, // the first sub-stream of block 0 has a CRC
    ]);
    nh.extend_from_slice(&[0x33; 4]); // sub-stream CRC
    nh.extend_from_slice(&[
        0x00, // sub_streams_info end
        0x00, // streams_info end
        0x05, // kFilesInfo
        0x06, // num_files
        0x0E, 0x01, 0x1C, // kEmptyStream: entries 3, 4 and 5
        0x0F, 0x01, 0xC0, // kEmptyFile: entries 3 and 4
        0x10, 0x01, 0x40, // kAnti: entry 4
        0x11, 0x19, 0x00, // kName, 25 bytes, external
    ]);
    for name in b"abcdef" {
        nh.extend_from_slice(&[*name, 0x00, 0x00, 0x00]);
    }
    for property in properties {
        nh.extend_from_slice(property);
    }
    nh.extend_from_slice(&[0x00, 0x00]); // files_info props end, header end
    nh
}

/// Time and attribute properties for the six entries of `complete_header`.
fn time_and_attribute_properties() -> [Vec<u8>; 4] {
    let property = |head: &[u8], value: u8, len: usize| {
        let mut property = head.to_vec();
        property.resize(head.len() + len, value);
        property
    };
    [
        property(&[0x12, 0x0B, 0x00, 0x80, 0x00], 0x44, 8), // kCTime: entry 0 defined
        property(&[0x13, 0x32, 0x01, 0x00], 0x55, 48),      // kATime: all defined
        property(&[0x14, 0x13, 0x00, 0xC0, 0x00], 0x66, 16), // kMTime: entries 0 and 1 defined
        property(&[0x15, 0x13, 0x00, 0xF0, 0x00], 0x77, 16), // kWinAttributes: entries 0 to 3
    ]
}

/// The headers set every bit set the reader builds. Each time and attribute property also
/// appears alone. A bit set of the same size as a freed one fits under every budget the freed
/// one fits under. Opening each archive must return under every budget.
#[test]
fn header_allocation_failure_returns_an_error() {
    let properties = time_and_attribute_properties();
    let all: Vec<&[u8]> = properties.iter().map(Vec::as_slice).collect();
    let mut headers = vec![complete_header(&all)];
    headers.extend(all.iter().map(|property| complete_header(&[property])));

    for header in headers {
        let bytes = raw_7z_exact(&header);
        let archive = Archive::read(&mut Cursor::new(bytes.clone()), &Password::empty()).unwrap();
        assert_eq!(archive.files.len(), 6);
        assert!(archive.files[4].is_anti_item());
        assert!(archive.files[5].is_directory());
        assert_eq!(archive.files[5].name(), "f");
        assert_returns_under_every_budget(&bytes);
    }
}
