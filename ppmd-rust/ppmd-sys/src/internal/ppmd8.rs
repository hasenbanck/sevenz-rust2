#![allow(
    dead_code,
    mutable_transmutes,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_assignments,
    unused_mut
)]

mod decoder;
mod encoder;
mod range_coding;

pub type size_t = std::ffi::c_ulong;
pub type Byte = std::ffi::c_uchar;
pub type UInt16 = std::ffi::c_ushort;
pub type Int32 = std::ffi::c_int;
pub type UInt32 = std::ffi::c_uint;
pub type BoolInt = std::ffi::c_int;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct IByteIn_ {
    pub Read: Option<unsafe extern "C" fn(IByteInPtr) -> Byte>,
}

pub type IByteInPtr = *const IByteIn_;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct IByteOut_ {
    pub Write: Option<unsafe extern "C" fn(IByteOutPtr, Byte) -> ()>,
}

pub type IByteOutPtr = *const IByteOut_;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ISzAlloc {
    pub Alloc: Option<unsafe extern "C" fn(ISzAllocPtr, size_t) -> *mut std::ffi::c_void>,
    pub Free: Option<unsafe extern "C" fn(ISzAllocPtr, *mut std::ffi::c_void) -> ()>,
}

pub type ISzAllocPtr = *const ISzAlloc;

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct CPpmd_See {
    pub Summ: UInt16,
    pub Shift: Byte,
    pub Count: Byte,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct CPpmd_State {
    pub Symbol: Byte,
    pub Freq: Byte,
    pub Successor_0: UInt16,
    pub Successor_1: UInt16,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct CPpmd_State2_ {
    pub Symbol: Byte,
    pub Freq: Byte,
}

pub type CPpmd_State2 = CPpmd_State2_;

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct CPpmd_State4_ {
    pub Successor_0: UInt16,
    pub Successor_1: UInt16,
}

pub type CPpmd_State4 = CPpmd_State4_;

pub type CPpmd_State_Ref = UInt32;

pub type CPpmd_Void_Ref = UInt32;

pub type CPpmd_Byte_Ref = UInt32;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CPpmd8_Context_ {
    pub NumStats: Byte,
    pub Flags: Byte,
    pub Union2: C2RustUnnamed_0,
    pub Union4: C2RustUnnamed,
    pub Suffix: CPpmd8_Context_Ref,
}

pub type CPpmd8_Context_Ref = UInt32;

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed {
    pub Stats: CPpmd_State_Ref,
    pub State4: CPpmd_State4,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_0 {
    pub SummFreq: UInt16,
    pub State2: CPpmd_State2,
}

pub type CPpmd8_Context = CPpmd8_Context_;

pub type C2RustUnnamed_1 = std::ffi::c_uint;

pub const PPMD8_RESTORE_METHOD_UNSUPPPORTED: C2RustUnnamed_1 = 2;
pub const PPMD8_RESTORE_METHOD_CUT_OFF: C2RustUnnamed_1 = 1;
pub const PPMD8_RESTORE_METHOD_RESTART: C2RustUnnamed_1 = 0;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CPpmd8 {
    pub MinContext: *mut CPpmd8_Context,
    pub MaxContext: *mut CPpmd8_Context,
    pub FoundState: *mut CPpmd_State,
    pub OrderFall: std::ffi::c_uint,
    pub InitEsc: std::ffi::c_uint,
    pub PrevSuccess: std::ffi::c_uint,
    pub MaxOrder: std::ffi::c_uint,
    pub RestoreMethod: std::ffi::c_uint,
    pub RunLength: Int32,
    pub InitRL: Int32,
    pub Size: UInt32,
    pub GlueCount: UInt32,
    pub AlignOffset: UInt32,
    pub Base: *mut Byte,
    pub LoUnit: *mut Byte,
    pub HiUnit: *mut Byte,
    pub Text: *mut Byte,
    pub UnitsStart: *mut Byte,
    pub Range: UInt32,
    pub Code: UInt32,
    pub Low: UInt32,
    pub Stream: C2RustUnnamed_2,
    pub Indx2Units: [Byte; 40],
    pub Units2Indx: [Byte; 128],
    pub FreeList: [CPpmd_Void_Ref; 38],
    pub Stamps: [UInt32; 38],
    pub NS2BSIndx: [Byte; 256],
    pub NS2Indx: [Byte; 260],
    pub ExpEscape: [Byte; 16],
    pub DummySee: CPpmd_See,
    pub See: [[CPpmd_See; 32]; 24],
    pub BinSumm: [[UInt16; 64]; 25],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_2 {
    pub In: IByteInPtr,
    pub Out: IByteOutPtr,
}

pub type PPMD8_CTX_PTR = *mut CPpmd8_Context;
pub type CPpmd8_Node = CPpmd8_Node_;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CPpmd8_Node_ {
    pub Stamp: UInt32,
    pub Next: CPpmd8_Node_Ref,
    pub NU: UInt32,
}

pub type CPpmd8_Node_Ref = UInt32;

static mut PPMD8_kExpEscape: [Byte; 16] = [25, 14, 9, 7, 5, 5, 4, 4, 4, 3, 3, 3, 2, 2, 2, 2];

static mut PPMD8_kInitBinEsc: [UInt16; 8] = [
    0x3cdd, 0x1f3f, 0x59bf, 0x48f3, 0x64a1, 0x5abc, 0x6632, 0x6051,
];

pub fn Ppmd8_Construct(mut p: *mut CPpmd8) {
    let mut i: std::ffi::c_uint = 0;
    let mut k: std::ffi::c_uint = 0;
    let mut m: std::ffi::c_uint = 0;
    (*p).Base = 0 as *mut Byte;
    i = 0 as std::ffi::c_int as std::ffi::c_uint;
    k = 0 as std::ffi::c_int as std::ffi::c_uint;
    while i
        < (4 as std::ffi::c_int
            + 4 as std::ffi::c_int
            + 4 as std::ffi::c_int
            + (128 as std::ffi::c_int + 3 as std::ffi::c_int
                - 1 as std::ffi::c_int * 4 as std::ffi::c_int
                - 2 as std::ffi::c_int * 4 as std::ffi::c_int
                - 3 as std::ffi::c_int * 4 as std::ffi::c_int)
                / 4 as std::ffi::c_int) as std::ffi::c_uint
    {
        let mut step: std::ffi::c_uint = if i >= 12 as std::ffi::c_int as std::ffi::c_uint {
            4 as std::ffi::c_int as std::ffi::c_uint
        } else {
            (i >> 2 as std::ffi::c_int).wrapping_add(1 as std::ffi::c_int as std::ffi::c_uint)
        };
        loop {
            let fresh0 = k;
            k = k.wrapping_add(1);
            (*p).Units2Indx[fresh0 as usize] = i as Byte;
            step = step.wrapping_sub(1);
            if !(step != 0) {
                break;
            }
        }
        (*p).Indx2Units[i as usize] = k as Byte;
        i = i.wrapping_add(1);
        i;
    }
    (*p).NS2BSIndx[0 as std::ffi::c_int as usize] =
        ((0 as std::ffi::c_int) << 1 as std::ffi::c_int) as Byte;
    (*p).NS2BSIndx[1 as std::ffi::c_int as usize] =
        ((1 as std::ffi::c_int) << 1 as std::ffi::c_int) as Byte;
    memset(
        ((*p).NS2BSIndx)
            .as_mut_ptr()
            .offset(2 as std::ffi::c_int as isize) as *mut std::ffi::c_void,
        (2 as std::ffi::c_int) << 1 as std::ffi::c_int,
        9 as std::ffi::c_int as std::ffi::c_ulong,
    );
    memset(
        ((*p).NS2BSIndx)
            .as_mut_ptr()
            .offset(11 as std::ffi::c_int as isize) as *mut std::ffi::c_void,
        (3 as std::ffi::c_int) << 1 as std::ffi::c_int,
        (256 as std::ffi::c_int - 11 as std::ffi::c_int) as std::ffi::c_ulong,
    );
    i = 0 as std::ffi::c_int as std::ffi::c_uint;
    while i < 5 as std::ffi::c_int as std::ffi::c_uint {
        (*p).NS2Indx[i as usize] = i as Byte;
        i = i.wrapping_add(1);
        i;
    }
    m = i;
    k = 1 as std::ffi::c_int as std::ffi::c_uint;
    while i < 260 as std::ffi::c_int as std::ffi::c_uint {
        (*p).NS2Indx[i as usize] = m as Byte;
        k = k.wrapping_sub(1);
        if k == 0 as std::ffi::c_int as std::ffi::c_uint {
            m = m.wrapping_add(1);
            k = m.wrapping_sub(4 as std::ffi::c_int as std::ffi::c_uint);
        }
        i = i.wrapping_add(1);
        i;
    }
    memcpy(
        ((*p).ExpEscape).as_mut_ptr() as *mut std::ffi::c_void,
        PPMD8_kExpEscape.as_ptr() as *const std::ffi::c_void,
        16 as std::ffi::c_int as std::ffi::c_ulong,
    );
}

pub fn Ppmd8_Free(mut p: *mut CPpmd8, mut alloc: ISzAllocPtr) {
    ((*alloc).Free).expect("non-null function pointer")(alloc, (*p).Base as *mut std::ffi::c_void);
    (*p).Size = 0 as std::ffi::c_int as UInt32;
    (*p).Base = 0 as *mut Byte;
}

pub fn Ppmd8_Alloc(mut p: *mut CPpmd8, mut size: UInt32, mut alloc: ISzAllocPtr) -> BoolInt {
    if ((*p).Base).is_null() || (*p).Size != size {
        Ppmd8_Free(p, alloc);
        (*p).AlignOffset =
            (4 as std::ffi::c_int as UInt32).wrapping_sub(size) & 3 as std::ffi::c_int as UInt32;
        (*p).Base = ((*alloc).Alloc).expect("non-null function pointer")(
            alloc,
            ((*p).AlignOffset).wrapping_add(size) as size_t,
        ) as *mut Byte;
        if ((*p).Base).is_null() {
            return 0 as std::ffi::c_int;
        }
        (*p).Size = size;
    }
    return 1 as std::ffi::c_int;
}
fn Ppmd8_InsertNode(
    mut p: *mut CPpmd8,
    mut node: *mut std::ffi::c_void,
    mut indx: std::ffi::c_uint,
) {
    (*(node as *mut CPpmd8_Node)).Stamp = 0xffffffff as std::ffi::c_uint;
    (*(node as *mut CPpmd8_Node)).Next = (*p).FreeList[indx as usize];
    (*(node as *mut CPpmd8_Node)).NU = (*p).Indx2Units[indx as usize] as std::ffi::c_uint;
    (*p).FreeList[indx as usize] =
        (node as *mut Byte).offset_from((*p).Base) as std::ffi::c_long as UInt32;
    (*p).Stamps[indx as usize] = ((*p).Stamps[indx as usize]).wrapping_add(1);
    (*p).Stamps[indx as usize];
}
fn Ppmd8_RemoveNode(mut p: *mut CPpmd8, mut indx: std::ffi::c_uint) -> *mut std::ffi::c_void {
    let mut node: *mut CPpmd8_Node = ((*p).Base).offset((*p).FreeList[indx as usize] as isize)
        as *mut std::ffi::c_void as *mut CPpmd8_Node;
    (*p).FreeList[indx as usize] = (*node).Next;
    (*p).Stamps[indx as usize] = ((*p).Stamps[indx as usize]).wrapping_sub(1);
    (*p).Stamps[indx as usize];
    return node as *mut std::ffi::c_void;
}
fn Ppmd8_SplitBlock(
    mut p: *mut CPpmd8,
    mut ptr: *mut std::ffi::c_void,
    mut oldIndx: std::ffi::c_uint,
    mut newIndx: std::ffi::c_uint,
) {
    let mut i: std::ffi::c_uint = 0;
    let mut nu: std::ffi::c_uint = ((*p).Indx2Units[oldIndx as usize] as std::ffi::c_uint)
        .wrapping_sub((*p).Indx2Units[newIndx as usize] as std::ffi::c_uint);
    ptr = (ptr as *mut Byte).offset(
        ((*p).Indx2Units[newIndx as usize] as std::ffi::c_uint * 12 as std::ffi::c_int as UInt32)
            as isize,
    ) as *mut std::ffi::c_void;
    i = (*p).Units2Indx[(nu as size_t).wrapping_sub(1 as std::ffi::c_int as size_t) as usize]
        as std::ffi::c_uint;
    if (*p).Indx2Units[i as usize] as std::ffi::c_uint != nu {
        i = i.wrapping_sub(1);
        let mut k: std::ffi::c_uint = (*p).Indx2Units[i as usize] as std::ffi::c_uint;
        Ppmd8_InsertNode(
            p,
            (ptr as *mut Byte).offset((k * 12 as std::ffi::c_int as UInt32) as isize)
                as *mut std::ffi::c_void,
            nu.wrapping_sub(k)
                .wrapping_sub(1 as std::ffi::c_int as std::ffi::c_uint),
        );
    }
    Ppmd8_InsertNode(p, ptr, i);
}
fn Ppmd8_GlueFreeBlocks(mut p: *mut CPpmd8) {
    let mut n: CPpmd8_Node_Ref = 0;
    (*p).GlueCount = ((1 as std::ffi::c_int) << 13 as std::ffi::c_int) as UInt32;
    memset(
        ((*p).Stamps).as_mut_ptr() as *mut std::ffi::c_void,
        0 as std::ffi::c_int,
        ::core::mem::size_of::<[UInt32; 38]>() as std::ffi::c_ulong,
    );
    if (*p).LoUnit != (*p).HiUnit {
        (*((*p).LoUnit as *mut std::ffi::c_void as *mut CPpmd8_Node)).Stamp =
            0 as std::ffi::c_int as UInt32;
    }
    let mut prev: *mut CPpmd8_Node_Ref = &mut n;
    let mut i: std::ffi::c_uint = 0;
    i = 0 as std::ffi::c_int as std::ffi::c_uint;
    while i
        < (4 as std::ffi::c_int
            + 4 as std::ffi::c_int
            + 4 as std::ffi::c_int
            + (128 as std::ffi::c_int + 3 as std::ffi::c_int
                - 1 as std::ffi::c_int * 4 as std::ffi::c_int
                - 2 as std::ffi::c_int * 4 as std::ffi::c_int
                - 3 as std::ffi::c_int * 4 as std::ffi::c_int)
                / 4 as std::ffi::c_int) as std::ffi::c_uint
    {
        let mut next: CPpmd8_Node_Ref = (*p).FreeList[i as usize];
        (*p).FreeList[i as usize] = 0 as std::ffi::c_int as CPpmd_Void_Ref;
        while next != 0 as std::ffi::c_int as CPpmd8_Node_Ref {
            let mut node: *mut CPpmd8_Node =
                ((*p).Base).offset(next as isize) as *mut std::ffi::c_void as *mut CPpmd8_Node;
            let mut nu: UInt32 = (*node).NU;
            *prev = next;
            next = (*node).Next;
            if nu != 0 as std::ffi::c_int as UInt32 {
                let mut node2: *mut CPpmd8_Node = 0 as *mut CPpmd8_Node;
                prev = &mut (*node).Next;
                loop {
                    node2 = node.offset(nu as isize);
                    if !((*node2).Stamp == 0xffffffff as std::ffi::c_uint) {
                        break;
                    }
                    nu = nu.wrapping_add((*node2).NU);
                    (*node2).NU = 0 as std::ffi::c_int as UInt32;
                    (*node).NU = nu;
                }
            }
        }
        i = i.wrapping_add(1);
        i;
    }
    *prev = 0 as std::ffi::c_int as CPpmd8_Node_Ref;
    while n != 0 as std::ffi::c_int as CPpmd8_Node_Ref {
        let mut node_0: *mut CPpmd8_Node =
            ((*p).Base).offset(n as isize) as *mut std::ffi::c_void as *mut CPpmd8_Node;
        let mut nu_0: UInt32 = (*node_0).NU;
        let mut i_0: std::ffi::c_uint = 0;
        n = (*node_0).Next;
        if nu_0 == 0 as std::ffi::c_int as UInt32 {
            continue;
        }
        while nu_0 > 128 as std::ffi::c_int as UInt32 {
            Ppmd8_InsertNode(
                p,
                node_0 as *mut std::ffi::c_void,
                (4 as std::ffi::c_int
                    + 4 as std::ffi::c_int
                    + 4 as std::ffi::c_int
                    + (128 as std::ffi::c_int + 3 as std::ffi::c_int
                        - 1 as std::ffi::c_int * 4 as std::ffi::c_int
                        - 2 as std::ffi::c_int * 4 as std::ffi::c_int
                        - 3 as std::ffi::c_int * 4 as std::ffi::c_int)
                        / 4 as std::ffi::c_int
                    - 1 as std::ffi::c_int) as std::ffi::c_uint,
            );
            nu_0 = nu_0.wrapping_sub(128 as std::ffi::c_int as UInt32);
            node_0 = node_0.offset(128 as std::ffi::c_int as isize);
        }
        i_0 = (*p).Units2Indx
            [(nu_0 as size_t).wrapping_sub(1 as std::ffi::c_int as size_t) as usize]
            as std::ffi::c_uint;
        if (*p).Indx2Units[i_0 as usize] as std::ffi::c_uint != nu_0 {
            i_0 = i_0.wrapping_sub(1);
            let mut k: std::ffi::c_uint = (*p).Indx2Units[i_0 as usize] as std::ffi::c_uint;
            Ppmd8_InsertNode(
                p,
                node_0.offset(k as isize) as *mut std::ffi::c_void,
                nu_0.wrapping_sub(k)
                    .wrapping_sub(1 as std::ffi::c_int as std::ffi::c_uint),
            );
        }
        Ppmd8_InsertNode(p, node_0 as *mut std::ffi::c_void, i_0);
    }
}

#[inline(never)]
fn Ppmd8_AllocUnitsRare(mut p: *mut CPpmd8, mut indx: std::ffi::c_uint) -> *mut std::ffi::c_void {
    let mut i: std::ffi::c_uint = 0;
    if (*p).GlueCount == 0 as std::ffi::c_int as UInt32 {
        Ppmd8_GlueFreeBlocks(p);
        if (*p).FreeList[indx as usize] != 0 as std::ffi::c_int as CPpmd_Void_Ref {
            return Ppmd8_RemoveNode(p, indx);
        }
    }
    i = indx;
    loop {
        i = i.wrapping_add(1);
        if i == (4 as std::ffi::c_int
            + 4 as std::ffi::c_int
            + 4 as std::ffi::c_int
            + (128 as std::ffi::c_int + 3 as std::ffi::c_int
                - 1 as std::ffi::c_int * 4 as std::ffi::c_int
                - 2 as std::ffi::c_int * 4 as std::ffi::c_int
                - 3 as std::ffi::c_int * 4 as std::ffi::c_int)
                / 4 as std::ffi::c_int) as std::ffi::c_uint
        {
            let mut numBytes: UInt32 = (*p).Indx2Units[indx as usize] as std::ffi::c_uint
                * 12 as std::ffi::c_int as UInt32;
            let mut us: *mut Byte = (*p).UnitsStart;
            (*p).GlueCount = ((*p).GlueCount).wrapping_sub(1);
            (*p).GlueCount;
            return (if us.offset_from((*p).Text) as std::ffi::c_long as UInt32 > numBytes {
                (*p).UnitsStart = us.offset(-(numBytes as isize));
                (*p).UnitsStart
            } else {
                0 as *mut Byte
            }) as *mut std::ffi::c_void;
        }
        if !((*p).FreeList[i as usize] == 0 as std::ffi::c_int as CPpmd_Void_Ref) {
            break;
        }
    }
    let mut block: *mut std::ffi::c_void = Ppmd8_RemoveNode(p, i);
    Ppmd8_SplitBlock(p, block, i, indx);
    return block;
}

fn Ppmd8_AllocUnits(mut p: *mut CPpmd8, mut indx: std::ffi::c_uint) -> *mut std::ffi::c_void {
    if (*p).FreeList[indx as usize] != 0 as std::ffi::c_int as CPpmd_Void_Ref {
        return Ppmd8_RemoveNode(p, indx);
    }
    let mut numBytes: UInt32 =
        (*p).Indx2Units[indx as usize] as std::ffi::c_uint * 12 as std::ffi::c_int as UInt32;
    let mut lo: *mut Byte = (*p).LoUnit;
    if ((*p).HiUnit).offset_from(lo) as std::ffi::c_long as UInt32 >= numBytes {
        (*p).LoUnit = lo.offset(numBytes as isize);
        return lo as *mut std::ffi::c_void;
    }
    Ppmd8_AllocUnitsRare(p, indx)
}

fn ShrinkUnits(
    mut p: *mut CPpmd8,
    mut oldPtr: *mut std::ffi::c_void,
    mut oldNU: std::ffi::c_uint,
    mut newNU: std::ffi::c_uint,
) -> *mut std::ffi::c_void {
    let mut i0: std::ffi::c_uint = (*p).Units2Indx
        [(oldNU as size_t).wrapping_sub(1 as std::ffi::c_int as size_t) as usize]
        as std::ffi::c_uint;
    let mut i1: std::ffi::c_uint = (*p).Units2Indx
        [(newNU as size_t).wrapping_sub(1 as std::ffi::c_int as size_t) as usize]
        as std::ffi::c_uint;
    if i0 == i1 {
        return oldPtr;
    }
    if (*p).FreeList[i1 as usize] != 0 as std::ffi::c_int as CPpmd_Void_Ref {
        let mut ptr: *mut std::ffi::c_void = Ppmd8_RemoveNode(p, i1);
        let mut d: *mut UInt32 = ptr as *mut UInt32;
        let mut z: *const UInt32 = oldPtr as *const UInt32;
        let mut n: std::ffi::c_uint = newNU;
        loop {
            *d.offset(0 as std::ffi::c_int as isize) = *z.offset(0 as std::ffi::c_int as isize);
            *d.offset(1 as std::ffi::c_int as isize) = *z.offset(1 as std::ffi::c_int as isize);
            *d.offset(2 as std::ffi::c_int as isize) = *z.offset(2 as std::ffi::c_int as isize);
            z = z.offset(3 as std::ffi::c_int as isize);
            d = d.offset(3 as std::ffi::c_int as isize);
            n = n.wrapping_sub(1);
            if !(n != 0) {
                break;
            }
        }
        Ppmd8_InsertNode(p, oldPtr, i0);
        return ptr;
    }
    Ppmd8_SplitBlock(p, oldPtr, i0, i1);
    return oldPtr;
}

fn FreeUnits(mut p: *mut CPpmd8, mut ptr: *mut std::ffi::c_void, mut nu: std::ffi::c_uint) {
    Ppmd8_InsertNode(
        p,
        ptr,
        (*p).Units2Indx[(nu as size_t).wrapping_sub(1 as std::ffi::c_int as size_t) as usize]
            as std::ffi::c_uint,
    );
}

fn SpecialFreeUnit(mut p: *mut CPpmd8, mut ptr: *mut std::ffi::c_void) {
    if ptr as *mut Byte != (*p).UnitsStart {
        Ppmd8_InsertNode(p, ptr, 0 as std::ffi::c_int as std::ffi::c_uint);
    } else {
        (*p).UnitsStart = ((*p).UnitsStart).offset(12 as std::ffi::c_int as isize);
    };
}

fn ExpandTextArea(mut p: *mut CPpmd8) {
    let mut count: [UInt32; 38] = [0; 38];
    let mut i: std::ffi::c_uint = 0;
    memset(
        count.as_mut_ptr() as *mut std::ffi::c_void,
        0 as std::ffi::c_int,
        ::core::mem::size_of::<[UInt32; 38]>() as std::ffi::c_ulong,
    );
    if (*p).LoUnit != (*p).HiUnit {
        (*((*p).LoUnit as *mut std::ffi::c_void as *mut CPpmd8_Node)).Stamp =
            0 as std::ffi::c_int as UInt32;
    }
    let mut node: *mut CPpmd8_Node = (*p).UnitsStart as *mut std::ffi::c_void as *mut CPpmd8_Node;
    while (*node).Stamp == 0xffffffff as std::ffi::c_uint {
        let mut nu: UInt32 = (*node).NU;
        (*node).Stamp = 0 as std::ffi::c_int as UInt32;
        count[(*p).Units2Indx[(nu as size_t).wrapping_sub(1 as std::ffi::c_int as size_t) as usize]
            as usize] = (count[(*p).Units2Indx
            [(nu as size_t).wrapping_sub(1 as std::ffi::c_int as size_t) as usize]
            as usize])
            .wrapping_add(1);
        count[(*p).Units2Indx[(nu as size_t).wrapping_sub(1 as std::ffi::c_int as size_t) as usize]
            as usize];
        node = node.offset(nu as isize);
    }
    (*p).UnitsStart = node as *mut Byte;
    i = 0 as std::ffi::c_int as std::ffi::c_uint;
    while i
        < (4 as std::ffi::c_int
            + 4 as std::ffi::c_int
            + 4 as std::ffi::c_int
            + (128 as std::ffi::c_int + 3 as std::ffi::c_int
                - 1 as std::ffi::c_int * 4 as std::ffi::c_int
                - 2 as std::ffi::c_int * 4 as std::ffi::c_int
                - 3 as std::ffi::c_int * 4 as std::ffi::c_int)
                / 4 as std::ffi::c_int) as std::ffi::c_uint
    {
        let mut cnt: UInt32 = count[i as usize];
        if !(cnt == 0 as std::ffi::c_int as UInt32) {
            let mut prev: *mut CPpmd8_Node_Ref =
                &mut *((*p).FreeList).as_mut_ptr().offset(i as isize) as *mut CPpmd_Void_Ref
                    as *mut CPpmd8_Node_Ref;
            let mut n: CPpmd8_Node_Ref = *prev;
            (*p).Stamps[i as usize] = ((*p).Stamps[i as usize]).wrapping_sub(cnt);
            loop {
                let mut node_0: *mut CPpmd8_Node =
                    ((*p).Base).offset(n as isize) as *mut std::ffi::c_void as *mut CPpmd8_Node;
                n = (*node_0).Next;
                if (*node_0).Stamp != 0 as std::ffi::c_int as UInt32 {
                    prev = &mut (*node_0).Next;
                } else {
                    *prev = n;
                    cnt = cnt.wrapping_sub(1);
                    if cnt == 0 as std::ffi::c_int as UInt32 {
                        break;
                    }
                }
            }
        }
        i = i.wrapping_add(1);
        i;
    }
}

fn Ppmd8State_SetSuccessor(mut p: *mut CPpmd_State, mut v: CPpmd_Void_Ref) {
    (*p).Successor_0 = v as UInt16;
    (*p).Successor_1 = (v >> 16 as std::ffi::c_int) as UInt16;
}

#[inline(never)]
fn Ppmd8_RestartModel(mut p: *mut CPpmd8) {
    let mut i: std::ffi::c_uint = 0;
    let mut k: std::ffi::c_uint = 0;
    let mut m: std::ffi::c_uint = 0;
    memset(
        ((*p).FreeList).as_mut_ptr() as *mut std::ffi::c_void,
        0 as std::ffi::c_int,
        ::core::mem::size_of::<[CPpmd_Void_Ref; 38]>() as std::ffi::c_ulong,
    );
    memset(
        ((*p).Stamps).as_mut_ptr() as *mut std::ffi::c_void,
        0 as std::ffi::c_int,
        ::core::mem::size_of::<[UInt32; 38]>() as std::ffi::c_ulong,
    );
    (*p).Text = ((*p).Base)
        .offset((*p).AlignOffset as isize)
        .offset(0 as std::ffi::c_int as isize);
    (*p).HiUnit = ((*p).Text).offset((*p).Size as isize);
    (*p).UnitsStart = ((*p).HiUnit).offset(
        -(((*p).Size / 8 as std::ffi::c_int as UInt32 / 12 as std::ffi::c_int as UInt32
            * 7 as std::ffi::c_int as UInt32
            * 12 as std::ffi::c_int as UInt32) as isize),
    );
    (*p).LoUnit = (*p).UnitsStart;
    (*p).GlueCount = 0 as std::ffi::c_int as UInt32;
    (*p).OrderFall = (*p).MaxOrder;
    (*p).InitRL = -((if (*p).MaxOrder < 12 as std::ffi::c_int as std::ffi::c_uint {
        (*p).MaxOrder
    } else {
        12 as std::ffi::c_int as std::ffi::c_uint
    }) as Int32)
        - 1 as std::ffi::c_int;
    (*p).RunLength = (*p).InitRL;
    (*p).PrevSuccess = 0 as std::ffi::c_int as std::ffi::c_uint;
    (*p).HiUnit = ((*p).HiUnit).offset(-(12 as std::ffi::c_int as isize));
    let mut mc: *mut CPpmd8_Context = (*p).HiUnit as *mut std::ffi::c_void as PPMD8_CTX_PTR;
    let mut s: *mut CPpmd_State = (*p).LoUnit as *mut CPpmd_State;
    (*p).LoUnit = ((*p).LoUnit).offset(
        ((256 as std::ffi::c_int / 2 as std::ffi::c_int) as UInt32
            * 12 as std::ffi::c_int as UInt32) as isize,
    );
    (*p).MinContext = mc;
    (*p).MaxContext = (*p).MinContext;
    (*p).FoundState = s;
    (*mc).Flags = 0 as std::ffi::c_int as Byte;
    (*mc).NumStats = (256 as std::ffi::c_int - 1 as std::ffi::c_int) as Byte;
    (*mc).Union2.SummFreq = (256 as std::ffi::c_int + 1 as std::ffi::c_int) as UInt16;
    (*mc).Union4.Stats = (s as *mut Byte).offset_from((*p).Base) as std::ffi::c_long as UInt32;
    (*mc).Suffix = 0 as std::ffi::c_int as CPpmd8_Context_Ref;
    i = 0 as std::ffi::c_int as std::ffi::c_uint;
    while i < 256 as std::ffi::c_int as std::ffi::c_uint {
        (*s).Symbol = i as Byte;
        (*s).Freq = 1 as std::ffi::c_int as Byte;
        Ppmd8State_SetSuccessor(s, 0 as std::ffi::c_int as CPpmd_Void_Ref);
        i = i.wrapping_add(1);
        i;
        s = s.offset(1);
        s;
    }
    m = 0 as std::ffi::c_int as std::ffi::c_uint;
    i = m;
    while m < 25 as std::ffi::c_int as std::ffi::c_uint {
        while (*p).NS2Indx[i as usize] as std::ffi::c_uint == m {
            i = i.wrapping_add(1);
            i;
        }
        k = 0 as std::ffi::c_int as std::ffi::c_uint;
        while k < 8 as std::ffi::c_int as std::ffi::c_uint {
            let mut r: std::ffi::c_uint = 0;
            let mut dest: *mut UInt16 = ((*p).BinSumm[m as usize]).as_mut_ptr().offset(k as isize);
            let val: UInt16 = (((1 as std::ffi::c_int)
                << 7 as std::ffi::c_int + 7 as std::ffi::c_int)
                as std::ffi::c_uint)
                .wrapping_sub(
                    (PPMD8_kInitBinEsc[k as usize] as std::ffi::c_uint)
                        .wrapping_div(i.wrapping_add(1 as std::ffi::c_int as std::ffi::c_uint)),
                ) as UInt16;
            r = 0 as std::ffi::c_int as std::ffi::c_uint;
            while r < 64 as std::ffi::c_int as std::ffi::c_uint {
                *dest.offset(r as isize) = val;
                r = r.wrapping_add(8 as std::ffi::c_int as std::ffi::c_uint);
            }
            k = k.wrapping_add(1);
            k;
        }
        m = m.wrapping_add(1);
        m;
    }
    m = 0 as std::ffi::c_int as std::ffi::c_uint;
    i = m;
    while m < 24 as std::ffi::c_int as std::ffi::c_uint {
        let mut summ: std::ffi::c_uint = 0;
        let mut s_0: *mut CPpmd_See = 0 as *mut CPpmd_See;
        while (*p).NS2Indx[(i as size_t).wrapping_add(3 as std::ffi::c_int as size_t) as usize]
            as std::ffi::c_uint
            == m.wrapping_add(3 as std::ffi::c_int as std::ffi::c_uint)
        {
            i = i.wrapping_add(1);
            i;
        }
        s_0 = ((*p).See[m as usize]).as_mut_ptr();
        summ = (2 as std::ffi::c_int as std::ffi::c_uint)
            .wrapping_mul(i)
            .wrapping_add(5 as std::ffi::c_int as std::ffi::c_uint)
            << 7 as std::ffi::c_int - 4 as std::ffi::c_int;
        k = 0 as std::ffi::c_int as std::ffi::c_uint;
        while k < 32 as std::ffi::c_int as std::ffi::c_uint {
            (*s_0).Summ = summ as UInt16;
            (*s_0).Shift = (7 as std::ffi::c_int - 4 as std::ffi::c_int) as Byte;
            (*s_0).Count = 7 as std::ffi::c_int as Byte;
            k = k.wrapping_add(1);
            k;
            s_0 = s_0.offset(1);
            s_0;
        }
        m = m.wrapping_add(1);
        m;
    }
    (*p).DummySee.Summ = 0 as std::ffi::c_int as UInt16;
    (*p).DummySee.Shift = 7 as std::ffi::c_int as Byte;
    (*p).DummySee.Count = 64 as std::ffi::c_int as Byte;
}

pub fn Ppmd8_Init(
    mut p: *mut CPpmd8,
    mut maxOrder: std::ffi::c_uint,
    mut restoreMethod: std::ffi::c_uint,
) {
    (*p).MaxOrder = maxOrder;
    (*p).RestoreMethod = restoreMethod;
    Ppmd8_RestartModel(p);
}

fn Refresh(
    mut p: *mut CPpmd8,
    mut ctx: PPMD8_CTX_PTR,
    mut oldNU: std::ffi::c_uint,
    mut scale: std::ffi::c_uint,
) {
    let mut i: std::ffi::c_uint = (*ctx).NumStats as std::ffi::c_uint;
    let mut escFreq: std::ffi::c_uint = 0;
    let mut sumFreq: std::ffi::c_uint = 0;
    let mut flags: std::ffi::c_uint = 0;
    let mut s: *mut CPpmd_State = ShrinkUnits(
        p,
        ((*p).Base).offset((*ctx).Union4.Stats as isize) as *mut std::ffi::c_void
            as *mut CPpmd_State as *mut std::ffi::c_void,
        oldNU,
        i.wrapping_add(2 as std::ffi::c_int as std::ffi::c_uint) >> 1 as std::ffi::c_int,
    ) as *mut CPpmd_State;
    (*ctx).Union4.Stats = (s as *mut Byte).offset_from((*p).Base) as std::ffi::c_long as UInt32;
    scale |= ((*ctx).Union2.SummFreq as UInt32
        >= (1 as std::ffi::c_int as UInt32) << 15 as std::ffi::c_int)
        as std::ffi::c_int as std::ffi::c_uint;
    flags =
        ((*s).Symbol as std::ffi::c_uint).wrapping_add(0xc0 as std::ffi::c_int as std::ffi::c_uint);
    let mut freq: std::ffi::c_uint = (*s).Freq as std::ffi::c_uint;
    escFreq = ((*ctx).Union2.SummFreq as std::ffi::c_uint).wrapping_sub(freq);
    freq = freq.wrapping_add(scale) >> scale;
    sumFreq = freq;
    (*s).Freq = freq as Byte;
    loop {
        s = s.offset(1);
        let mut freq_0: std::ffi::c_uint = (*s).Freq as std::ffi::c_uint;
        escFreq = escFreq.wrapping_sub(freq_0);
        freq_0 = freq_0.wrapping_add(scale) >> scale;
        sumFreq = sumFreq.wrapping_add(freq_0);
        (*s).Freq = freq_0 as Byte;
        flags |= ((*s).Symbol as std::ffi::c_uint)
            .wrapping_add(0xc0 as std::ffi::c_int as std::ffi::c_uint);
        i = i.wrapping_sub(1);
        if !(i != 0) {
            break;
        }
    }
    (*ctx).Union2.SummFreq = sumFreq.wrapping_add(escFreq.wrapping_add(scale) >> scale) as UInt16;
    (*ctx).Flags = ((*ctx).Flags as std::ffi::c_uint
        & (((1 as std::ffi::c_int) << 4 as std::ffi::c_int) as std::ffi::c_uint).wrapping_add(
            (((1 as std::ffi::c_int) << 2 as std::ffi::c_int) as std::ffi::c_uint)
                .wrapping_mul(scale),
        ))
    .wrapping_add(
        flags >> 8 as std::ffi::c_int - 3 as std::ffi::c_int
            & ((1 as std::ffi::c_int) << 3 as std::ffi::c_int) as std::ffi::c_uint,
    ) as Byte;
}

fn SWAP_STATES(mut t1: *mut CPpmd_State, mut t2: *mut CPpmd_State) {
    let mut tmp: CPpmd_State = *t1;
    *t1 = *t2;
    *t2 = tmp;
}

fn CutOff(
    mut p: *mut CPpmd8,
    mut ctx: PPMD8_CTX_PTR,
    mut order: std::ffi::c_uint,
) -> CPpmd_Void_Ref {
    let mut ns: std::ffi::c_int = (*ctx).NumStats as std::ffi::c_int;
    let mut nu: std::ffi::c_uint = 0;
    let mut stats: *mut CPpmd_State = 0 as *mut CPpmd_State;
    if ns == 0 as std::ffi::c_int {
        let mut s: *mut CPpmd_State =
            &mut (*ctx).Union2 as *mut C2RustUnnamed_0 as *mut CPpmd_State;
        let mut successor: CPpmd_Void_Ref =
            (*s).Successor_0 as UInt32 | ((*s).Successor_1 as UInt32) << 16 as std::ffi::c_int;
        if ((*p).Base).offset(successor as isize) as *mut std::ffi::c_void as *mut Byte
            >= (*p).UnitsStart
        {
            if order < (*p).MaxOrder {
                successor = CutOff(
                    p,
                    ((*p).Base).offset(successor as isize) as *mut std::ffi::c_void
                        as *mut CPpmd8_Context,
                    order.wrapping_add(1 as std::ffi::c_int as std::ffi::c_uint),
                );
            } else {
                successor = 0 as std::ffi::c_int as CPpmd_Void_Ref;
            }
            Ppmd8State_SetSuccessor(s, successor);
            if successor != 0 || order <= 9 as std::ffi::c_int as std::ffi::c_uint {
                return (ctx as *mut Byte).offset_from((*p).Base) as std::ffi::c_long as UInt32;
            }
        }
        SpecialFreeUnit(p, ctx as *mut std::ffi::c_void);
        return 0 as std::ffi::c_int as CPpmd_Void_Ref;
    }
    nu = (ns as std::ffi::c_uint).wrapping_add(2 as std::ffi::c_int as std::ffi::c_uint)
        >> 1 as std::ffi::c_int;
    let mut indx: std::ffi::c_uint = (*p).Units2Indx
        [(nu as size_t).wrapping_sub(1 as std::ffi::c_int as size_t) as usize]
        as std::ffi::c_uint;
    stats = ((*p).Base).offset((*ctx).Union4.Stats as isize) as *mut std::ffi::c_void
        as *mut CPpmd_State;
    if (stats as *mut Byte).offset_from((*p).UnitsStart) as std::ffi::c_long as UInt32
        <= ((1 as std::ffi::c_int) << 14 as std::ffi::c_int) as UInt32
        && (*ctx).Union4.Stats <= (*p).FreeList[indx as usize]
    {
        let mut ptr: *mut std::ffi::c_void = Ppmd8_RemoveNode(p, indx);
        (*ctx).Union4.Stats =
            (ptr as *mut Byte).offset_from((*p).Base) as std::ffi::c_long as UInt32;
        let mut d: *mut UInt32 = ptr as *mut UInt32;
        let mut z: *const UInt32 = stats as *const std::ffi::c_void as *const UInt32;
        let mut n: std::ffi::c_uint = nu;
        loop {
            *d.offset(0 as std::ffi::c_int as isize) = *z.offset(0 as std::ffi::c_int as isize);
            *d.offset(1 as std::ffi::c_int as isize) = *z.offset(1 as std::ffi::c_int as isize);
            *d.offset(2 as std::ffi::c_int as isize) = *z.offset(2 as std::ffi::c_int as isize);
            z = z.offset(3 as std::ffi::c_int as isize);
            d = d.offset(3 as std::ffi::c_int as isize);
            n = n.wrapping_sub(1);
            if !(n != 0) {
                break;
            }
        }
        if stats as *mut Byte != (*p).UnitsStart {
            Ppmd8_InsertNode(p, stats as *mut std::ffi::c_void, indx);
        } else {
            (*p).UnitsStart = ((*p).UnitsStart).offset(
                ((*p).Indx2Units[indx as usize] as std::ffi::c_uint
                    * 12 as std::ffi::c_int as UInt32) as isize,
            );
        }
        stats = ptr as *mut CPpmd_State;
    }
    let mut s_0: *mut CPpmd_State = stats.offset(ns as std::ffi::c_uint as isize);
    loop {
        let mut successor_0: CPpmd_Void_Ref =
            (*s_0).Successor_0 as UInt32 | ((*s_0).Successor_1 as UInt32) << 16 as std::ffi::c_int;
        if (((*p).Base).offset(successor_0 as isize) as *mut std::ffi::c_void as *mut Byte)
            < (*p).UnitsStart
        {
            let fresh1 = ns;
            ns = ns - 1;
            let mut s2: *mut CPpmd_State = stats.offset(fresh1 as std::ffi::c_uint as isize);
            if order != 0 {
                if s_0 != s2 {
                    *s_0 = *s2;
                }
            } else {
                SWAP_STATES(s_0, s2);
                Ppmd8State_SetSuccessor(s2, 0 as std::ffi::c_int as CPpmd_Void_Ref);
            }
        } else if order < (*p).MaxOrder {
            Ppmd8State_SetSuccessor(
                s_0,
                CutOff(
                    p,
                    ((*p).Base).offset(successor_0 as isize) as *mut std::ffi::c_void
                        as *mut CPpmd8_Context,
                    order.wrapping_add(1 as std::ffi::c_int as std::ffi::c_uint),
                ),
            );
        } else {
            Ppmd8State_SetSuccessor(s_0, 0 as std::ffi::c_int as CPpmd_Void_Ref);
        }
        s_0 = s_0.offset(-1);
        if !(s_0 >= stats) {
            break;
        }
    }
    if ns != (*ctx).NumStats as std::ffi::c_int && order != 0 {
        if ns < 0 as std::ffi::c_int {
            FreeUnits(p, stats as *mut std::ffi::c_void, nu);
            SpecialFreeUnit(p, ctx as *mut std::ffi::c_void);
            return 0 as std::ffi::c_int as CPpmd_Void_Ref;
        }
        (*ctx).NumStats = ns as Byte;
        if ns == 0 as std::ffi::c_int {
            let sym: Byte = (*stats).Symbol;
            (*ctx).Flags = (((*ctx).Flags as std::ffi::c_int
                & (1 as std::ffi::c_int) << 4 as std::ffi::c_int)
                as std::ffi::c_uint)
                .wrapping_add(
                    (sym as std::ffi::c_uint)
                        .wrapping_add(0xc0 as std::ffi::c_int as std::ffi::c_uint)
                        >> 8 as std::ffi::c_int - 3 as std::ffi::c_int
                        & ((1 as std::ffi::c_int) << 3 as std::ffi::c_int) as std::ffi::c_uint,
                ) as Byte;
            (*ctx).Union2.State2.Symbol = sym;
            (*ctx).Union2.State2.Freq = (((*stats).Freq as std::ffi::c_uint)
                .wrapping_add(11 as std::ffi::c_int as std::ffi::c_uint)
                >> 3 as std::ffi::c_int) as Byte;
            (*ctx).Union4.State4.Successor_0 = (*stats).Successor_0;
            (*ctx).Union4.State4.Successor_1 = (*stats).Successor_1;
            FreeUnits(p, stats as *mut std::ffi::c_void, nu);
        } else {
            Refresh(
                p,
                ctx,
                nu,
                ((*ctx).Union2.SummFreq as std::ffi::c_uint
                    > (16 as std::ffi::c_int as std::ffi::c_uint)
                        .wrapping_mul(ns as std::ffi::c_uint)) as std::ffi::c_int
                    as std::ffi::c_uint,
            );
        }
    }
    return (ctx as *mut Byte).offset_from((*p).Base) as std::ffi::c_long as UInt32;
}

fn GetUsedMemory(mut p: *const CPpmd8) -> UInt32 {
    let mut v: UInt32 = 0 as std::ffi::c_int as UInt32;
    let mut i: std::ffi::c_uint = 0;
    i = 0 as std::ffi::c_int as std::ffi::c_uint;
    while i
        < (4 as std::ffi::c_int
            + 4 as std::ffi::c_int
            + 4 as std::ffi::c_int
            + (128 as std::ffi::c_int + 3 as std::ffi::c_int
                - 1 as std::ffi::c_int * 4 as std::ffi::c_int
                - 2 as std::ffi::c_int * 4 as std::ffi::c_int
                - 3 as std::ffi::c_int * 4 as std::ffi::c_int)
                / 4 as std::ffi::c_int) as std::ffi::c_uint
    {
        v = (v as std::ffi::c_uint).wrapping_add(
            ((*p).Stamps[i as usize]).wrapping_mul((*p).Indx2Units[i as usize] as std::ffi::c_uint),
        ) as UInt32 as UInt32;
        i = i.wrapping_add(1);
        i;
    }
    return ((*p).Size)
        .wrapping_sub(((*p).HiUnit).offset_from((*p).LoUnit) as std::ffi::c_long as UInt32)
        .wrapping_sub(((*p).UnitsStart).offset_from((*p).Text) as std::ffi::c_long as UInt32)
        .wrapping_sub(v * 12 as std::ffi::c_int as UInt32);
}

fn RestoreModel(mut p: *mut CPpmd8, mut ctxError: PPMD8_CTX_PTR) {
    let mut c: PPMD8_CTX_PTR = 0 as *mut CPpmd8_Context;
    let mut s: *mut CPpmd_State = 0 as *mut CPpmd_State;
    (*p).Text = ((*p).Base)
        .offset((*p).AlignOffset as isize)
        .offset(0 as std::ffi::c_int as isize);
    c = (*p).MaxContext;
    while c != ctxError {
        (*c).NumStats = ((*c).NumStats).wrapping_sub(1);
        if (*c).NumStats as std::ffi::c_int == 0 as std::ffi::c_int {
            s = ((*p).Base).offset((*c).Union4.Stats as isize) as *mut std::ffi::c_void
                as *mut CPpmd_State;
            (*c).Flags = (((*c).Flags as std::ffi::c_int
                & (1 as std::ffi::c_int) << 4 as std::ffi::c_int)
                as std::ffi::c_uint)
                .wrapping_add(
                    ((*s).Symbol as std::ffi::c_uint)
                        .wrapping_add(0xc0 as std::ffi::c_int as std::ffi::c_uint)
                        >> 8 as std::ffi::c_int - 3 as std::ffi::c_int
                        & ((1 as std::ffi::c_int) << 3 as std::ffi::c_int) as std::ffi::c_uint,
                ) as Byte;
            (*c).Union2.State2.Symbol = (*s).Symbol;
            (*c).Union2.State2.Freq = (((*s).Freq as std::ffi::c_uint)
                .wrapping_add(11 as std::ffi::c_int as std::ffi::c_uint)
                >> 3 as std::ffi::c_int) as Byte;
            (*c).Union4.State4.Successor_0 = (*s).Successor_0;
            (*c).Union4.State4.Successor_1 = (*s).Successor_1;
            SpecialFreeUnit(p, s as *mut std::ffi::c_void);
        } else {
            Refresh(
                p,
                c,
                ((*c).NumStats as std::ffi::c_uint)
                    .wrapping_add(3 as std::ffi::c_int as std::ffi::c_uint)
                    >> 1 as std::ffi::c_int,
                0 as std::ffi::c_int as std::ffi::c_uint,
            );
        }
        c = ((*p).Base).offset((*c).Suffix as isize) as *mut std::ffi::c_void
            as *mut CPpmd8_Context;
    }
    while c != (*p).MinContext {
        if (*c).NumStats as std::ffi::c_int == 0 as std::ffi::c_int {
            (*c).Union2.State2.Freq = (((*c).Union2.State2.Freq as std::ffi::c_uint)
                .wrapping_add(1 as std::ffi::c_int as std::ffi::c_uint)
                >> 1 as std::ffi::c_int) as Byte;
        } else {
            (*c).Union2.SummFreq =
                ((*c).Union2.SummFreq as std::ffi::c_int + 4 as std::ffi::c_int) as UInt16;
            if (*c).Union2.SummFreq as std::ffi::c_int
                > 128 as std::ffi::c_int + 4 as std::ffi::c_int * (*c).NumStats as std::ffi::c_int
            {
                Refresh(
                    p,
                    c,
                    ((*c).NumStats as std::ffi::c_uint)
                        .wrapping_add(2 as std::ffi::c_int as std::ffi::c_uint)
                        >> 1 as std::ffi::c_int,
                    1 as std::ffi::c_int as std::ffi::c_uint,
                );
            }
        }
        c = ((*p).Base).offset((*c).Suffix as isize) as *mut std::ffi::c_void
            as *mut CPpmd8_Context;
    }
    if (*p).RestoreMethod == PPMD8_RESTORE_METHOD_RESTART as std::ffi::c_int as std::ffi::c_uint
        || GetUsedMemory(p) < (*p).Size >> 1 as std::ffi::c_int
    {
        Ppmd8_RestartModel(p);
    } else {
        while (*(*p).MaxContext).Suffix != 0 {
            (*p).MaxContext = ((*p).Base).offset((*(*p).MaxContext).Suffix as isize)
                as *mut std::ffi::c_void as *mut CPpmd8_Context;
        }
        loop {
            CutOff(p, (*p).MaxContext, 0 as std::ffi::c_int as std::ffi::c_uint);
            ExpandTextArea(p);
            if !(GetUsedMemory(p)
                > 3 as std::ffi::c_int as UInt32 * ((*p).Size >> 2 as std::ffi::c_int))
            {
                break;
            }
        }
        (*p).GlueCount = 0 as std::ffi::c_int as UInt32;
        (*p).OrderFall = (*p).MaxOrder;
    }
    (*p).MinContext = (*p).MaxContext;
}

#[inline(never)]
fn Ppmd8_CreateSuccessors(
    mut p: *mut CPpmd8,
    mut skip: BoolInt,
    mut s1: *mut CPpmd_State,
    mut c: PPMD8_CTX_PTR,
) -> PPMD8_CTX_PTR {
    let mut upBranch: CPpmd_Byte_Ref = (*(*p).FoundState).Successor_0 as UInt32
        | ((*(*p).FoundState).Successor_1 as UInt32) << 16 as std::ffi::c_int;
    let mut newSym: Byte = 0;
    let mut newFreq: Byte = 0;
    let mut flags: Byte = 0;
    let mut numPs: std::ffi::c_uint = 0 as std::ffi::c_int as std::ffi::c_uint;
    let mut ps: [*mut CPpmd_State; 17] = [0 as *mut CPpmd_State; 17];
    if skip == 0 {
        let fresh2 = numPs;
        numPs = numPs.wrapping_add(1);
        ps[fresh2 as usize] = (*p).FoundState;
    }
    while (*c).Suffix != 0 {
        let mut successor: CPpmd_Void_Ref = 0;
        let mut s: *mut CPpmd_State = 0 as *mut CPpmd_State;
        c = ((*p).Base).offset((*c).Suffix as isize) as *mut std::ffi::c_void
            as *mut CPpmd8_Context;
        if !s1.is_null() {
            s = s1;
            s1 = 0 as *mut CPpmd_State;
        } else if (*c).NumStats as std::ffi::c_int != 0 as std::ffi::c_int {
            let mut sym: Byte = (*(*p).FoundState).Symbol;
            s = ((*p).Base).offset((*c).Union4.Stats as isize) as *mut std::ffi::c_void
                as *mut CPpmd_State;
            while (*s).Symbol as std::ffi::c_int != sym as std::ffi::c_int {
                s = s.offset(1);
                s;
            }
            if ((*s).Freq as std::ffi::c_int) < 124 as std::ffi::c_int - 9 as std::ffi::c_int {
                (*s).Freq = ((*s).Freq).wrapping_add(1);
                (*s).Freq;
                (*c).Union2.SummFreq = ((*c).Union2.SummFreq).wrapping_add(1);
                (*c).Union2.SummFreq;
            }
        } else {
            s = &mut (*c).Union2 as *mut C2RustUnnamed_0 as *mut CPpmd_State;
            (*s).Freq = ((*s).Freq as std::ffi::c_int
                + (((*(((*p).Base).offset((*c).Suffix as isize) as *mut std::ffi::c_void
                    as *mut CPpmd8_Context))
                    .NumStats
                    == 0) as std::ffi::c_int
                    & (((*s).Freq as std::ffi::c_int) < 24 as std::ffi::c_int) as std::ffi::c_int))
                as Byte;
        }
        successor =
            (*s).Successor_0 as UInt32 | ((*s).Successor_1 as UInt32) << 16 as std::ffi::c_int;
        if successor != upBranch {
            c = ((*p).Base).offset(successor as isize) as *mut std::ffi::c_void
                as *mut CPpmd8_Context;
            if numPs == 0 as std::ffi::c_int as std::ffi::c_uint {
                return c;
            }
            break;
        } else {
            let fresh3 = numPs;
            numPs = numPs.wrapping_add(1);
            ps[fresh3 as usize] = s;
        }
    }
    newSym = *(((*p).Base).offset(upBranch as isize) as *mut std::ffi::c_void as *const Byte);
    upBranch = upBranch.wrapping_add(1);
    upBranch;
    flags = (((*(*p).FoundState).Symbol as std::ffi::c_uint)
        .wrapping_add(0xc0 as std::ffi::c_int as std::ffi::c_uint)
        >> 8 as std::ffi::c_int - 4 as std::ffi::c_int
        & ((1 as std::ffi::c_int) << 4 as std::ffi::c_int) as std::ffi::c_uint)
        .wrapping_add(
            (newSym as std::ffi::c_uint).wrapping_add(0xc0 as std::ffi::c_int as std::ffi::c_uint)
                >> 8 as std::ffi::c_int - 3 as std::ffi::c_int
                & ((1 as std::ffi::c_int) << 3 as std::ffi::c_int) as std::ffi::c_uint,
        ) as Byte;
    if (*c).NumStats as std::ffi::c_int == 0 as std::ffi::c_int {
        newFreq = (*c).Union2.State2.Freq;
    } else {
        let mut cf: UInt32 = 0;
        let mut s0: UInt32 = 0;
        let mut s_0: *mut CPpmd_State = 0 as *mut CPpmd_State;
        s_0 = ((*p).Base).offset((*c).Union4.Stats as isize) as *mut std::ffi::c_void
            as *mut CPpmd_State;
        while (*s_0).Symbol as std::ffi::c_int != newSym as std::ffi::c_int {
            s_0 = s_0.offset(1);
            s_0;
        }
        cf = ((*s_0).Freq as UInt32).wrapping_sub(1 as std::ffi::c_int as UInt32);
        s0 = ((*c).Union2.SummFreq as UInt32)
            .wrapping_sub((*c).NumStats as UInt32)
            .wrapping_sub(cf);
        newFreq = (1 as std::ffi::c_int as UInt32).wrapping_add(
            (if 2 as std::ffi::c_int as UInt32 * cf <= s0 {
                (5 as std::ffi::c_int as UInt32 * cf > s0) as std::ffi::c_int as UInt32
            } else {
                cf.wrapping_add(2 as std::ffi::c_int as UInt32 * s0)
                    .wrapping_sub(3 as std::ffi::c_int as UInt32)
                    / s0
            }),
        ) as Byte;
    }
    loop {
        let mut c1: PPMD8_CTX_PTR = 0 as *mut CPpmd8_Context;
        if (*p).HiUnit != (*p).LoUnit {
            (*p).HiUnit = ((*p).HiUnit).offset(-(12 as std::ffi::c_int as isize));
            c1 = (*p).HiUnit as *mut std::ffi::c_void as PPMD8_CTX_PTR;
        } else if (*p).FreeList[0 as std::ffi::c_int as usize]
            != 0 as std::ffi::c_int as CPpmd_Void_Ref
        {
            c1 = Ppmd8_RemoveNode(p, 0 as std::ffi::c_int as std::ffi::c_uint) as PPMD8_CTX_PTR;
        } else {
            c1 = Ppmd8_AllocUnitsRare(p, 0 as std::ffi::c_int as std::ffi::c_uint) as PPMD8_CTX_PTR;
            if c1.is_null() {
                return 0 as PPMD8_CTX_PTR;
            }
        }
        (*c1).Flags = flags;
        (*c1).NumStats = 0 as std::ffi::c_int as Byte;
        (*c1).Union2.State2.Symbol = newSym;
        (*c1).Union2.State2.Freq = newFreq;
        Ppmd8State_SetSuccessor(
            &mut (*c1).Union2 as *mut C2RustUnnamed_0 as *mut CPpmd_State,
            upBranch,
        );
        (*c1).Suffix = (c as *mut Byte).offset_from((*p).Base) as std::ffi::c_long as UInt32;
        numPs = numPs.wrapping_sub(1);
        Ppmd8State_SetSuccessor(
            ps[numPs as usize],
            (c1 as *mut Byte).offset_from((*p).Base) as std::ffi::c_long as UInt32,
        );
        c = c1;
        if !(numPs != 0 as std::ffi::c_int as std::ffi::c_uint) {
            break;
        }
    }
    return c;
}

fn ReduceOrder(
    mut p: *mut CPpmd8,
    mut s1: *mut CPpmd_State,
    mut c: PPMD8_CTX_PTR,
) -> PPMD8_CTX_PTR {
    let mut s: *mut CPpmd_State = 0 as *mut CPpmd_State;
    let mut c1: PPMD8_CTX_PTR = c;
    let mut upBranch: CPpmd_Void_Ref =
        ((*p).Text).offset_from((*p).Base) as std::ffi::c_long as UInt32;
    Ppmd8State_SetSuccessor((*p).FoundState, upBranch);
    (*p).OrderFall = ((*p).OrderFall).wrapping_add(1);
    (*p).OrderFall;
    loop {
        if !s1.is_null() {
            c = ((*p).Base).offset((*c).Suffix as isize) as *mut std::ffi::c_void
                as *mut CPpmd8_Context;
            s = s1;
            s1 = 0 as *mut CPpmd_State;
        } else {
            if (*c).Suffix == 0 {
                return c;
            }
            c = ((*p).Base).offset((*c).Suffix as isize) as *mut std::ffi::c_void
                as *mut CPpmd8_Context;
            if (*c).NumStats != 0 {
                s = ((*p).Base).offset((*c).Union4.Stats as isize) as *mut std::ffi::c_void
                    as *mut CPpmd_State;
                if (*s).Symbol as std::ffi::c_int != (*(*p).FoundState).Symbol as std::ffi::c_int {
                    loop {
                        s = s.offset(1);
                        s;
                        if !((*s).Symbol as std::ffi::c_int
                            != (*(*p).FoundState).Symbol as std::ffi::c_int)
                        {
                            break;
                        }
                    }
                }
                if ((*s).Freq as std::ffi::c_int) < 124 as std::ffi::c_int - 9 as std::ffi::c_int {
                    (*s).Freq = ((*s).Freq as std::ffi::c_int + 2 as std::ffi::c_int) as Byte;
                    (*c).Union2.SummFreq =
                        ((*c).Union2.SummFreq as std::ffi::c_int + 2 as std::ffi::c_int) as UInt16;
                }
            } else {
                s = &mut (*c).Union2 as *mut C2RustUnnamed_0 as *mut CPpmd_State;
                (*s).Freq = ((*s).Freq as std::ffi::c_int
                    + (((*s).Freq as std::ffi::c_int) < 32 as std::ffi::c_int) as std::ffi::c_int)
                    as Byte;
            }
        }
        if (*s).Successor_0 as UInt32 | ((*s).Successor_1 as UInt32) << 16 as std::ffi::c_int != 0 {
            break;
        }
        Ppmd8State_SetSuccessor(s, upBranch);
        (*p).OrderFall = ((*p).OrderFall).wrapping_add(1);
        (*p).OrderFall;
    }
    if (*s).Successor_0 as UInt32 | ((*s).Successor_1 as UInt32) << 16 as std::ffi::c_int
        <= upBranch
    {
        let mut successor: PPMD8_CTX_PTR = 0 as *mut CPpmd8_Context;
        let mut s2: *mut CPpmd_State = (*p).FoundState;
        (*p).FoundState = s;
        successor = Ppmd8_CreateSuccessors(p, 0 as std::ffi::c_int, 0 as *mut CPpmd_State, c);
        if successor.is_null() {
            Ppmd8State_SetSuccessor(s, 0 as std::ffi::c_int as CPpmd_Void_Ref);
        } else {
            Ppmd8State_SetSuccessor(
                s,
                (successor as *mut Byte).offset_from((*p).Base) as std::ffi::c_long as UInt32,
            );
        }
        (*p).FoundState = s2;
    }
    let mut successor_0: CPpmd_Void_Ref =
        (*s).Successor_0 as UInt32 | ((*s).Successor_1 as UInt32) << 16 as std::ffi::c_int;
    if (*p).OrderFall == 1 as std::ffi::c_int as std::ffi::c_uint && c1 == (*p).MaxContext {
        Ppmd8State_SetSuccessor((*p).FoundState, successor_0);
        (*p).Text = ((*p).Text).offset(-1);
        (*p).Text;
    }
    if successor_0 == 0 as std::ffi::c_int as CPpmd_Void_Ref {
        return 0 as PPMD8_CTX_PTR;
    }
    return ((*p).Base).offset(successor_0 as isize) as *mut std::ffi::c_void
        as *mut CPpmd8_Context;
}

#[inline(never)]
pub fn Ppmd8_UpdateModel(mut p: *mut CPpmd8) {
    let mut maxSuccessor: CPpmd_Void_Ref = 0;
    let mut minSuccessor: CPpmd_Void_Ref = (*(*p).FoundState).Successor_0 as UInt32
        | ((*(*p).FoundState).Successor_1 as UInt32) << 16 as std::ffi::c_int;
    let mut c: PPMD8_CTX_PTR = 0 as *mut CPpmd8_Context;
    let mut s0: std::ffi::c_uint = 0;
    let mut ns: std::ffi::c_uint = 0;
    let mut fFreq: std::ffi::c_uint = (*(*p).FoundState).Freq as std::ffi::c_uint;
    let mut flag: Byte = 0;
    let mut fSymbol: Byte = (*(*p).FoundState).Symbol;
    let mut s: *mut CPpmd_State = 0 as *mut CPpmd_State;
    if ((*(*p).FoundState).Freq as std::ffi::c_int) < 124 as std::ffi::c_int / 4 as std::ffi::c_int
        && (*(*p).MinContext).Suffix != 0 as std::ffi::c_int as CPpmd8_Context_Ref
    {
        c = ((*p).Base).offset((*(*p).MinContext).Suffix as isize) as *mut std::ffi::c_void
            as *mut CPpmd8_Context;
        if (*c).NumStats as std::ffi::c_int == 0 as std::ffi::c_int {
            s = &mut (*c).Union2 as *mut C2RustUnnamed_0 as *mut CPpmd_State;
            if ((*s).Freq as std::ffi::c_int) < 32 as std::ffi::c_int {
                (*s).Freq = ((*s).Freq).wrapping_add(1);
                (*s).Freq;
            }
        } else {
            let mut sym: Byte = (*(*p).FoundState).Symbol;
            s = ((*p).Base).offset((*c).Union4.Stats as isize) as *mut std::ffi::c_void
                as *mut CPpmd_State;
            if (*s).Symbol as std::ffi::c_int != sym as std::ffi::c_int {
                loop {
                    s = s.offset(1);
                    s;
                    if !((*s).Symbol as std::ffi::c_int != sym as std::ffi::c_int) {
                        break;
                    }
                }
                if (*s.offset(0 as std::ffi::c_int as isize)).Freq as std::ffi::c_int
                    >= (*s.offset(-(1 as std::ffi::c_int) as isize)).Freq as std::ffi::c_int
                {
                    SWAP_STATES(
                        &mut *s.offset(0 as std::ffi::c_int as isize),
                        &mut *s.offset(-(1 as std::ffi::c_int) as isize),
                    );
                    s = s.offset(-1);
                    s;
                }
            }
            if ((*s).Freq as std::ffi::c_int) < 124 as std::ffi::c_int - 9 as std::ffi::c_int {
                (*s).Freq = ((*s).Freq as std::ffi::c_int + 2 as std::ffi::c_int) as Byte;
                (*c).Union2.SummFreq =
                    ((*c).Union2.SummFreq as std::ffi::c_int + 2 as std::ffi::c_int) as UInt16;
            }
        }
    }
    c = (*p).MaxContext;
    if (*p).OrderFall == 0 as std::ffi::c_int as std::ffi::c_uint && minSuccessor != 0 {
        let mut cs: PPMD8_CTX_PTR =
            Ppmd8_CreateSuccessors(p, 1 as std::ffi::c_int, s, (*p).MinContext);
        if cs.is_null() {
            Ppmd8State_SetSuccessor((*p).FoundState, 0 as std::ffi::c_int as CPpmd_Void_Ref);
            RestoreModel(p, c);
            return;
        }
        Ppmd8State_SetSuccessor(
            (*p).FoundState,
            (cs as *mut Byte).offset_from((*p).Base) as std::ffi::c_long as UInt32,
        );
        (*p).MaxContext = cs;
        (*p).MinContext = (*p).MaxContext;
        return;
    }
    let mut text: *mut Byte = (*p).Text;
    let fresh4 = text;
    text = text.offset(1);
    *fresh4 = (*(*p).FoundState).Symbol;
    (*p).Text = text;
    if text >= (*p).UnitsStart {
        RestoreModel(p, c);
        return;
    }
    maxSuccessor = text.offset_from((*p).Base) as std::ffi::c_long as UInt32;
    if minSuccessor == 0 {
        let mut cs_0: PPMD8_CTX_PTR = ReduceOrder(p, s, (*p).MinContext);
        if cs_0.is_null() {
            RestoreModel(p, c);
            return;
        }
        minSuccessor = (cs_0 as *mut Byte).offset_from((*p).Base) as std::ffi::c_long as UInt32;
    } else if (((*p).Base).offset(minSuccessor as isize) as *mut std::ffi::c_void as *mut Byte)
        < (*p).UnitsStart
    {
        let mut cs_1: PPMD8_CTX_PTR =
            Ppmd8_CreateSuccessors(p, 0 as std::ffi::c_int, s, (*p).MinContext);
        if cs_1.is_null() {
            RestoreModel(p, c);
            return;
        }
        minSuccessor = (cs_1 as *mut Byte).offset_from((*p).Base) as std::ffi::c_long as UInt32;
    }
    (*p).OrderFall = ((*p).OrderFall).wrapping_sub(1);
    if (*p).OrderFall == 0 as std::ffi::c_int as std::ffi::c_uint {
        maxSuccessor = minSuccessor;
        (*p).Text =
            ((*p).Text).offset(-(((*p).MaxContext != (*p).MinContext) as std::ffi::c_int as isize));
    }
    flag = ((fSymbol as std::ffi::c_uint).wrapping_add(0xc0 as std::ffi::c_int as std::ffi::c_uint)
        >> 8 as std::ffi::c_int - 3 as std::ffi::c_int
        & ((1 as std::ffi::c_int) << 3 as std::ffi::c_int) as std::ffi::c_uint) as Byte;
    ns = (*(*p).MinContext).NumStats as std::ffi::c_uint;
    s0 = ((*(*p).MinContext).Union2.SummFreq as std::ffi::c_uint)
        .wrapping_sub(ns)
        .wrapping_sub(fFreq);
    while c != (*p).MinContext {
        let mut ns1: std::ffi::c_uint = 0;
        let mut sum: UInt32 = 0;
        ns1 = (*c).NumStats as std::ffi::c_uint;
        if ns1 != 0 as std::ffi::c_int as std::ffi::c_uint {
            if ns1 & 1 as std::ffi::c_int as std::ffi::c_uint
                != 0 as std::ffi::c_int as std::ffi::c_uint
            {
                let oldNU: std::ffi::c_uint = ns1
                    .wrapping_add(1 as std::ffi::c_int as std::ffi::c_uint)
                    >> 1 as std::ffi::c_int;
                let i: std::ffi::c_uint = (*p).Units2Indx
                    [(oldNU as size_t).wrapping_sub(1 as std::ffi::c_int as size_t) as usize]
                    as std::ffi::c_uint;
                if i != (*p).Units2Indx[(oldNU as size_t)
                    .wrapping_add(1 as std::ffi::c_int as size_t)
                    .wrapping_sub(1 as std::ffi::c_int as size_t)
                    as usize] as std::ffi::c_uint
                {
                    let mut ptr: *mut std::ffi::c_void = Ppmd8_AllocUnits(
                        p,
                        i.wrapping_add(1 as std::ffi::c_int as std::ffi::c_uint),
                    );
                    let mut oldPtr: *mut std::ffi::c_void = 0 as *mut std::ffi::c_void;
                    if ptr.is_null() {
                        RestoreModel(p, c);
                        return;
                    }
                    oldPtr = ((*p).Base).offset((*c).Union4.Stats as isize) as *mut std::ffi::c_void
                        as *mut CPpmd_State as *mut std::ffi::c_void;
                    let mut d: *mut UInt32 = ptr as *mut UInt32;
                    let mut z: *const UInt32 = oldPtr as *const UInt32;
                    let mut n: std::ffi::c_uint = oldNU;
                    loop {
                        *d.offset(0 as std::ffi::c_int as isize) =
                            *z.offset(0 as std::ffi::c_int as isize);
                        *d.offset(1 as std::ffi::c_int as isize) =
                            *z.offset(1 as std::ffi::c_int as isize);
                        *d.offset(2 as std::ffi::c_int as isize) =
                            *z.offset(2 as std::ffi::c_int as isize);
                        z = z.offset(3 as std::ffi::c_int as isize);
                        d = d.offset(3 as std::ffi::c_int as isize);
                        n = n.wrapping_sub(1);
                        if !(n != 0) {
                            break;
                        }
                    }
                    Ppmd8_InsertNode(p, oldPtr, i);
                    (*c).Union4.Stats =
                        (ptr as *mut Byte).offset_from((*p).Base) as std::ffi::c_long as UInt32;
                }
            }
            sum = (*c).Union2.SummFreq as UInt32;
            sum = sum.wrapping_add(
                ((3 as std::ffi::c_int as std::ffi::c_uint)
                    .wrapping_mul(ns1)
                    .wrapping_add(1 as std::ffi::c_int as std::ffi::c_uint)
                    < ns) as std::ffi::c_int as std::ffi::c_uint,
            );
        } else {
            let mut s_0: *mut CPpmd_State =
                Ppmd8_AllocUnits(p, 0 as std::ffi::c_int as std::ffi::c_uint) as *mut CPpmd_State;
            if s_0.is_null() {
                RestoreModel(p, c);
                return;
            }
            let mut freq: std::ffi::c_uint = (*c).Union2.State2.Freq as std::ffi::c_uint;
            (*s_0).Symbol = (*c).Union2.State2.Symbol;
            (*s_0).Successor_0 = (*c).Union4.State4.Successor_0;
            (*s_0).Successor_1 = (*c).Union4.State4.Successor_1;
            (*c).Union4.Stats =
                (s_0 as *mut Byte).offset_from((*p).Base) as std::ffi::c_long as UInt32;
            if freq
                < (124 as std::ffi::c_int / 4 as std::ffi::c_int - 1 as std::ffi::c_int)
                    as std::ffi::c_uint
            {
                freq <<= 1 as std::ffi::c_int;
            } else {
                freq = (124 as std::ffi::c_int - 4 as std::ffi::c_int) as std::ffi::c_uint;
            }
            (*s_0).Freq = freq as Byte;
            sum = freq.wrapping_add((*p).InitEsc).wrapping_add(
                (ns > 2 as std::ffi::c_int as std::ffi::c_uint) as std::ffi::c_int
                    as std::ffi::c_uint,
            );
        }
        let mut s_1: *mut CPpmd_State = (((*p).Base).offset((*c).Union4.Stats as isize)
            as *mut std::ffi::c_void as *mut CPpmd_State)
            .offset(ns1 as isize)
            .offset(1 as std::ffi::c_int as isize);
        let mut cf: UInt32 = 2 as std::ffi::c_int as UInt32
            * sum.wrapping_add(6 as std::ffi::c_int as UInt32)
            * fFreq;
        let mut sf: UInt32 = s0.wrapping_add(sum);
        (*s_1).Symbol = fSymbol;
        (*c).NumStats = ns1.wrapping_add(1 as std::ffi::c_int as std::ffi::c_uint) as Byte;
        Ppmd8State_SetSuccessor(s_1, maxSuccessor);
        (*c).Flags = ((*c).Flags as std::ffi::c_int | flag as std::ffi::c_int) as Byte;
        if cf < 6 as std::ffi::c_int as UInt32 * sf {
            cf = (1 as std::ffi::c_int as std::ffi::c_uint)
                .wrapping_add((cf > sf) as std::ffi::c_int as std::ffi::c_uint)
                .wrapping_add(
                    (cf >= 4 as std::ffi::c_int as UInt32 * sf) as std::ffi::c_int
                        as std::ffi::c_uint,
                );
            sum = sum.wrapping_add(4 as std::ffi::c_int as UInt32);
        } else {
            cf = (4 as std::ffi::c_int as std::ffi::c_uint)
                .wrapping_add(
                    (cf > 9 as std::ffi::c_int as UInt32 * sf) as std::ffi::c_int
                        as std::ffi::c_uint,
                )
                .wrapping_add(
                    (cf > 12 as std::ffi::c_int as UInt32 * sf) as std::ffi::c_int
                        as std::ffi::c_uint,
                )
                .wrapping_add(
                    (cf > 15 as std::ffi::c_int as UInt32 * sf) as std::ffi::c_int
                        as std::ffi::c_uint,
                );
            sum = sum.wrapping_add(cf);
        }
        (*c).Union2.SummFreq = sum as UInt16;
        (*s_1).Freq = cf as Byte;
        c = ((*p).Base).offset((*c).Suffix as isize) as *mut std::ffi::c_void
            as *mut CPpmd8_Context;
    }
    (*p).MinContext =
        ((*p).Base).offset(minSuccessor as isize) as *mut std::ffi::c_void as *mut CPpmd8_Context;
    (*p).MaxContext = (*p).MinContext;
}

#[inline(never)]
fn Ppmd8_Rescale(mut p: *mut CPpmd8) {
    let mut i: std::ffi::c_uint = 0;
    let mut adder: std::ffi::c_uint = 0;
    let mut sumFreq: std::ffi::c_uint = 0;
    let mut escFreq: std::ffi::c_uint = 0;
    let mut stats: *mut CPpmd_State = ((*p).Base).offset((*(*p).MinContext).Union4.Stats as isize)
        as *mut std::ffi::c_void as *mut CPpmd_State;
    let mut s: *mut CPpmd_State = (*p).FoundState;
    if s != stats {
        let mut tmp: CPpmd_State = *s;
        loop {
            *s.offset(0 as std::ffi::c_int as isize) = *s.offset(-(1 as std::ffi::c_int) as isize);
            s = s.offset(-1);
            if !(s != stats) {
                break;
            }
        }
        *s = tmp;
    }
    sumFreq = (*s).Freq as std::ffi::c_uint;
    escFreq = ((*(*p).MinContext).Union2.SummFreq as std::ffi::c_uint).wrapping_sub(sumFreq);
    adder = ((*p).OrderFall != 0 as std::ffi::c_int as std::ffi::c_uint) as std::ffi::c_int
        as std::ffi::c_uint;
    sumFreq = sumFreq
        .wrapping_add(4 as std::ffi::c_int as std::ffi::c_uint)
        .wrapping_add(adder)
        >> 1 as std::ffi::c_int;
    i = (*(*p).MinContext).NumStats as std::ffi::c_uint;
    (*s).Freq = sumFreq as Byte;
    loop {
        s = s.offset(1);
        let mut freq: std::ffi::c_uint = (*s).Freq as std::ffi::c_uint;
        escFreq = escFreq.wrapping_sub(freq);
        freq = freq.wrapping_add(adder) >> 1 as std::ffi::c_int;
        sumFreq = sumFreq.wrapping_add(freq);
        (*s).Freq = freq as Byte;
        if freq > (*s.offset(-(1 as std::ffi::c_int) as isize)).Freq as std::ffi::c_uint {
            let mut tmp_0: CPpmd_State = *s;
            let mut s1: *mut CPpmd_State = s;
            loop {
                *s1.offset(0 as std::ffi::c_int as isize) =
                    *s1.offset(-(1 as std::ffi::c_int) as isize);
                s1 = s1.offset(-1);
                if !(s1 != stats
                    && freq
                        > (*s1.offset(-(1 as std::ffi::c_int) as isize)).Freq as std::ffi::c_uint)
                {
                    break;
                }
            }
            *s1 = tmp_0;
        }
        i = i.wrapping_sub(1);
        if !(i != 0) {
            break;
        }
    }
    if (*s).Freq as std::ffi::c_int == 0 as std::ffi::c_int {
        let mut mc: *mut CPpmd8_Context = 0 as *mut CPpmd8_Context;
        let mut numStats: std::ffi::c_uint = 0;
        let mut numStatsNew: std::ffi::c_uint = 0;
        let mut n0: std::ffi::c_uint = 0;
        let mut n1: std::ffi::c_uint = 0;
        i = 0 as std::ffi::c_int as std::ffi::c_uint;
        loop {
            i = i.wrapping_add(1);
            i;
            s = s.offset(-1);
            if !((*s).Freq as std::ffi::c_int == 0 as std::ffi::c_int) {
                break;
            }
        }
        escFreq = escFreq.wrapping_add(i);
        mc = (*p).MinContext;
        numStats = (*mc).NumStats as std::ffi::c_uint;
        numStatsNew = numStats.wrapping_sub(i);
        (*mc).NumStats = numStatsNew as Byte;
        n0 =
            numStats.wrapping_add(2 as std::ffi::c_int as std::ffi::c_uint) >> 1 as std::ffi::c_int;
        if numStatsNew == 0 as std::ffi::c_int as std::ffi::c_uint {
            let mut freq_0: std::ffi::c_uint = (2 as std::ffi::c_int as std::ffi::c_uint)
                .wrapping_mul((*stats).Freq as std::ffi::c_uint)
                .wrapping_add(escFreq)
                .wrapping_sub(1 as std::ffi::c_int as std::ffi::c_uint)
                .wrapping_div(escFreq);
            if freq_0 > (124 as std::ffi::c_int / 3 as std::ffi::c_int) as std::ffi::c_uint {
                freq_0 = (124 as std::ffi::c_int / 3 as std::ffi::c_int) as std::ffi::c_uint;
            }
            (*mc).Flags = (((*mc).Flags as std::ffi::c_int
                & (1 as std::ffi::c_int) << 4 as std::ffi::c_int)
                as std::ffi::c_uint)
                .wrapping_add(
                    ((*stats).Symbol as std::ffi::c_uint)
                        .wrapping_add(0xc0 as std::ffi::c_int as std::ffi::c_uint)
                        >> 8 as std::ffi::c_int - 3 as std::ffi::c_int
                        & ((1 as std::ffi::c_int) << 3 as std::ffi::c_int) as std::ffi::c_uint,
                ) as Byte;
            s = &mut (*mc).Union2 as *mut C2RustUnnamed_0 as *mut CPpmd_State;
            *s = *stats;
            (*s).Freq = freq_0 as Byte;
            (*p).FoundState = s;
            Ppmd8_InsertNode(
                p,
                stats as *mut std::ffi::c_void,
                (*p).Units2Indx
                    [(n0 as size_t).wrapping_sub(1 as std::ffi::c_int as size_t) as usize]
                    as std::ffi::c_uint,
            );
            return;
        }
        n1 = numStatsNew.wrapping_add(2 as std::ffi::c_int as std::ffi::c_uint)
            >> 1 as std::ffi::c_int;
        if n0 != n1 {
            (*mc).Union4.Stats =
                (ShrinkUnits(p, stats as *mut std::ffi::c_void, n0, n1) as *mut Byte)
                    .offset_from((*p).Base) as std::ffi::c_long as UInt32;
        }
    }
    let mut mc_0: *mut CPpmd8_Context = (*p).MinContext;
    (*mc_0).Union2.SummFreq = sumFreq
        .wrapping_add(escFreq)
        .wrapping_sub(escFreq >> 1 as std::ffi::c_int) as UInt16;
    (*mc_0).Flags =
        ((*mc_0).Flags as std::ffi::c_int | (1 as std::ffi::c_int) << 2 as std::ffi::c_int) as Byte;
    (*p).FoundState = ((*p).Base).offset((*mc_0).Union4.Stats as isize) as *mut std::ffi::c_void
        as *mut CPpmd_State;
}

pub fn Ppmd8_MakeEscFreq(
    mut p: *mut CPpmd8,
    mut numMasked1: std::ffi::c_uint,
    mut escFreq: *mut UInt32,
) -> *mut CPpmd_See {
    let mut see: *mut CPpmd_See = 0 as *mut CPpmd_See;
    let mut mc: *const CPpmd8_Context = (*p).MinContext;
    let mut numStats: std::ffi::c_uint = (*mc).NumStats as std::ffi::c_uint;
    if numStats != 0xff as std::ffi::c_int as std::ffi::c_uint {
        see = ((*p).See[((*p).NS2Indx
            [(numStats as size_t).wrapping_add(2 as std::ffi::c_int as size_t) as usize]
            as std::ffi::c_uint as size_t)
            .wrapping_sub(3 as std::ffi::c_int as size_t) as usize])
            .as_mut_ptr()
            .offset(
                ((*mc).Union2.SummFreq as std::ffi::c_uint
                    > (11 as std::ffi::c_int as std::ffi::c_uint).wrapping_mul(
                        numStats.wrapping_add(1 as std::ffi::c_int as std::ffi::c_uint),
                    )) as std::ffi::c_int as isize,
            )
            .offset(
                (2 as std::ffi::c_int as std::ffi::c_uint).wrapping_mul(
                    ((2 as std::ffi::c_int as std::ffi::c_uint).wrapping_mul(numStats)
                        < ((*(((*p).Base).offset((*mc).Suffix as isize) as *mut std::ffi::c_void
                            as *mut CPpmd8_Context))
                            .NumStats as std::ffi::c_uint)
                            .wrapping_add(numMasked1)) as std::ffi::c_int
                        as std::ffi::c_uint,
                ) as isize,
            )
            .offset((*mc).Flags as std::ffi::c_int as isize);
        let summ: std::ffi::c_uint = (*see).Summ as std::ffi::c_uint;
        let r: std::ffi::c_uint = summ >> (*see).Shift as std::ffi::c_int;
        (*see).Summ = summ.wrapping_sub(r) as UInt16;
        *escFreq = r.wrapping_add(
            (r == 0 as std::ffi::c_int as std::ffi::c_uint) as std::ffi::c_int as std::ffi::c_uint,
        );
    } else {
        see = &mut (*p).DummySee;
        *escFreq = 1 as std::ffi::c_int as UInt32;
    }
    return see;
}

fn Ppmd8_NextContext(mut p: *mut CPpmd8) {
    let mut c: PPMD8_CTX_PTR = ((*p).Base).offset(
        ((*(*p).FoundState).Successor_0 as UInt32
            | ((*(*p).FoundState).Successor_1 as UInt32) << 16 as std::ffi::c_int) as isize,
    ) as *mut std::ffi::c_void as *mut CPpmd8_Context;
    if (*p).OrderFall == 0 as std::ffi::c_int as std::ffi::c_uint
        && c as *const Byte >= (*p).UnitsStart as *const Byte
    {
        (*p).MinContext = c;
        (*p).MaxContext = (*p).MinContext;
    } else {
        Ppmd8_UpdateModel(p);
    };
}

pub fn Ppmd8_Update1(mut p: *mut CPpmd8) {
    let mut s: *mut CPpmd_State = (*p).FoundState;
    let mut freq: std::ffi::c_uint = (*s).Freq as std::ffi::c_uint;
    freq = freq.wrapping_add(4 as std::ffi::c_int as std::ffi::c_uint);
    (*(*p).MinContext).Union2.SummFreq =
        ((*(*p).MinContext).Union2.SummFreq as std::ffi::c_int + 4 as std::ffi::c_int) as UInt16;
    (*s).Freq = freq as Byte;
    if freq > (*s.offset(-(1 as std::ffi::c_int) as isize)).Freq as std::ffi::c_uint {
        SWAP_STATES(s, &mut *s.offset(-(1 as std::ffi::c_int) as isize));
        s = s.offset(-1);
        (*p).FoundState = s;
        if freq > 124 as std::ffi::c_int as std::ffi::c_uint {
            Ppmd8_Rescale(p);
        }
    }
    Ppmd8_NextContext(p);
}

pub fn Ppmd8_Update1_0(mut p: *mut CPpmd8) {
    let mut s: *mut CPpmd_State = (*p).FoundState;
    let mut mc: *mut CPpmd8_Context = (*p).MinContext;
    let mut freq: std::ffi::c_uint = (*s).Freq as std::ffi::c_uint;
    let summFreq: std::ffi::c_uint = (*mc).Union2.SummFreq as std::ffi::c_uint;
    (*p).PrevSuccess = ((2 as std::ffi::c_int as std::ffi::c_uint).wrapping_mul(freq) >= summFreq)
        as std::ffi::c_int as std::ffi::c_uint;
    (*p).RunLength += (*p).PrevSuccess as Int32;
    (*mc).Union2.SummFreq =
        summFreq.wrapping_add(4 as std::ffi::c_int as std::ffi::c_uint) as UInt16;
    freq = freq.wrapping_add(4 as std::ffi::c_int as std::ffi::c_uint);
    (*s).Freq = freq as Byte;
    if freq > 124 as std::ffi::c_int as std::ffi::c_uint {
        Ppmd8_Rescale(p);
    }
    Ppmd8_NextContext(p);
}

pub fn Ppmd8_Update2(mut p: *mut CPpmd8) {
    let mut s: *mut CPpmd_State = (*p).FoundState;
    let mut freq: std::ffi::c_uint = (*s).Freq as std::ffi::c_uint;
    freq = freq.wrapping_add(4 as std::ffi::c_int as std::ffi::c_uint);
    (*p).RunLength = (*p).InitRL;
    (*(*p).MinContext).Union2.SummFreq =
        ((*(*p).MinContext).Union2.SummFreq as std::ffi::c_int + 4 as std::ffi::c_int) as UInt16;
    (*s).Freq = freq as Byte;
    if freq > 124 as std::ffi::c_int as std::ffi::c_uint {
        Ppmd8_Rescale(p);
    }
    Ppmd8_UpdateModel(p);
}
