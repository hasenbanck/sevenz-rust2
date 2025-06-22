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

pub(crate) use decoder::{Ppmd8_DecodeSymbol, Ppmd8_Init_RangeDec};
pub(crate) use encoder::{Ppmd8_EncodeSymbol, Ppmd8_Flush_RangeEnc};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct IByteIn {
    pub Read: Option<unsafe extern "C" fn(IByteInPtr) -> u8>,
}

pub type IByteInPtr = *const IByteIn;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct IByteOut {
    pub Write: Option<unsafe extern "C" fn(IByteOutPtr, u8) -> ()>,
}

pub type IByteOutPtr = *const IByteOut;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ISzAlloc {
    pub Alloc: Option<fn(ISzAllocPtr, usize) -> *mut u8>,
    pub Free: Option<fn(ISzAllocPtr, *mut u8) -> ()>,
}

pub type ISzAllocPtr = *const ISzAlloc;

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct CPpmd_See {
    pub Summ: u16,
    pub Shift: u8,
    pub Count: u8,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct CPpmd_State {
    pub Symbol: u8,
    pub Freq: u8,
    pub Successor_0: u16,
    pub Successor_1: u16,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct CPpmd_State2 {
    pub Symbol: u8,
    pub Freq: u8,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct CPpmd_State4 {
    pub Successor_0: u16,
    pub Successor_1: u16,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CPpmd8_Context {
    pub NumStats: u8,
    pub Flags: u8,
    pub Union2: Union2,
    pub Union4: Union4,
    pub Suffix: u32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union Union4 {
    pub Stats: u32,
    pub State4: CPpmd_State4,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union Union2 {
    pub SummFreq: u16,
    pub State2: CPpmd_State2,
}

pub const PPMD8_RESTORE_METHOD_UNSUPPPORTED: u32 = 2;
pub const PPMD8_RESTORE_METHOD_CUT_OFF: u32 = 1;
pub const PPMD8_RESTORE_METHOD_RESTART: u32 = 0;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CPpmd8 {
    pub MinContext: *mut CPpmd8_Context,
    pub MaxContext: *mut CPpmd8_Context,
    pub FoundState: *mut CPpmd_State,
    pub OrderFall: u32,
    pub InitEsc: u32,
    pub PrevSuccess: u32,
    pub MaxOrder: u32,
    pub RestoreMethod: u32,
    pub RunLength: i32,
    pub InitRL: i32,
    pub Size: u32,
    pub GlueCount: u32,
    pub AlignOffset: u32,
    pub Base: *mut u8,
    pub LoUnit: *mut u8,
    pub HiUnit: *mut u8,
    pub Text: *mut u8,
    pub UnitsStart: *mut u8,
    pub Range: u32,
    pub Code: u32,
    pub Low: u32,
    pub Stream: StreamUnion,
    pub Indx2Units: [u8; 40],
    pub Units2Indx: [u8; 128],
    pub FreeList: [u32; 38],
    pub Stamps: [u32; 38],
    pub NS2BSIndx: [u8; 256],
    pub NS2Indx: [u8; 260],
    pub ExpEscape: [u8; 16],
    pub DummySee: CPpmd_See,
    pub See: [[CPpmd_See; 32]; 24],
    pub BinSumm: [[u16; 64]; 25],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union StreamUnion {
    pub In: IByteInPtr,
    pub Out: IByteOutPtr,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CPpmd8_Node {
    pub Stamp: u32,
    pub Next: u32,
    pub NU: u32,
}

static PPMD8_kExpEscape: [u8; 16] = [25, 14, 9, 7, 5, 5, 4, 4, 4, 3, 3, 3, 2, 2, 2, 2];

static PPMD8_kInitBinEsc: [u16; 8] = [
    0x3cdd, 0x1f3f, 0x59bf, 0x48f3, 0x64a1, 0x5abc, 0x6632, 0x6051,
];

pub unsafe fn Ppmd8_Construct(mut p: *mut CPpmd8) {
    let mut i: u32 = 0;
    let mut k: u32 = 0;
    let mut m: u32 = 0;
    (*p).Base = 0 as *mut u8;
    i = 0 as i32 as u32;
    k = 0 as i32 as u32;
    while i
        < (4 as i32
            + 4 as i32
            + 4 as i32
            + (128 as i32 + 3 as i32
                - 1 as i32 * 4 as i32
                - 2 as i32 * 4 as i32
                - 3 as i32 * 4 as i32)
                / 4 as i32) as u32
    {
        let mut step: u32 = if i >= 12 as i32 as u32 {
            4 as i32 as u32
        } else {
            (i >> 2 as i32).wrapping_add(1 as i32 as u32)
        };
        loop {
            let fresh0 = k;
            k = k.wrapping_add(1);
            (*p).Units2Indx[fresh0 as usize] = i as u8;
            step = step.wrapping_sub(1);
            if !(step != 0) {
                break;
            }
        }
        (*p).Indx2Units[i as usize] = k as u8;
        i = i.wrapping_add(1);
        i;
    }

    (*p).NS2BSIndx[0] = (0 << 1) as u8;
    (*p).NS2BSIndx[1] = (1 << 1) as u8;
    (*p).NS2BSIndx[2..11].fill((2 << 1) as u8);
    (*p).NS2BSIndx[11..256].fill((3 << 1) as u8);

    i = 0 as i32 as u32;
    while i < 5 as i32 as u32 {
        (*p).NS2Indx[i as usize] = i as u8;
        i = i.wrapping_add(1);
        i;
    }
    m = i;
    k = 1 as i32 as u32;
    while i < 260 as i32 as u32 {
        (*p).NS2Indx[i as usize] = m as u8;
        k = k.wrapping_sub(1);
        if k == 0 as i32 as u32 {
            m = m.wrapping_add(1);
            k = m.wrapping_sub(4 as i32 as u32);
        }
        i = i.wrapping_add(1);
        i;
    }
    (*p).ExpEscape.copy_from_slice(&PPMD8_kExpEscape);
}

pub unsafe fn Ppmd8_Free(mut p: *mut CPpmd8, mut alloc: ISzAllocPtr) {
    ((*alloc).Free).expect("non-null function pointer")(alloc, (*p).Base as *mut u8);
    (*p).Size = 0 as i32 as u32;
    (*p).Base = 0 as *mut u8;
}

pub unsafe fn Ppmd8_Alloc(mut p: *mut CPpmd8, mut size: u32, mut alloc: ISzAllocPtr) -> i32 {
    if ((*p).Base).is_null() || (*p).Size != size {
        Ppmd8_Free(p, alloc);
        (*p).AlignOffset = (4 as i32 as u32).wrapping_sub(size) & 3 as i32 as u32;
        (*p).Base = ((*alloc).Alloc).expect("non-null function pointer")(
            alloc,
            ((*p).AlignOffset).wrapping_add(size) as usize,
        ) as *mut u8;
        if ((*p).Base).is_null() {
            return 0 as i32;
        }
        (*p).Size = size;
    }
    return 1 as i32;
}

unsafe fn Ppmd8_InsertNode(mut p: *mut CPpmd8, mut node: *mut u8, mut indx: u32) {
    (*(node as *mut CPpmd8_Node)).Stamp = 0xffffffff as u32;
    (*(node as *mut CPpmd8_Node)).Next = (*p).FreeList[indx as usize];
    (*(node as *mut CPpmd8_Node)).NU = (*p).Indx2Units[indx as usize] as u32;
    (*p).FreeList[indx as usize] = (node as *mut u8).offset_from((*p).Base) as isize as u32;
    (*p).Stamps[indx as usize] = ((*p).Stamps[indx as usize]).wrapping_add(1);
    (*p).Stamps[indx as usize];
}

unsafe fn Ppmd8_RemoveNode(mut p: *mut CPpmd8, mut indx: u32) -> *mut u8 {
    let mut node: *mut CPpmd8_Node =
        ((*p).Base).offset((*p).FreeList[indx as usize] as isize) as *mut u8 as *mut CPpmd8_Node;
    (*p).FreeList[indx as usize] = (*node).Next;
    (*p).Stamps[indx as usize] = ((*p).Stamps[indx as usize]).wrapping_sub(1);
    (*p).Stamps[indx as usize];
    node as *mut u8
}

unsafe fn Ppmd8_SplitBlock(
    mut p: *mut CPpmd8,
    mut ptr: *mut u8,
    mut oldIndx: u32,
    mut newIndx: u32,
) {
    let mut i: u32 = 0;
    let mut nu: u32 = ((*p).Indx2Units[oldIndx as usize] as u32)
        .wrapping_sub((*p).Indx2Units[newIndx as usize] as u32);
    ptr = (ptr as *mut u8)
        .offset(((*p).Indx2Units[newIndx as usize] as u32 * 12 as i32 as u32) as isize)
        as *mut u8;
    i = (*p).Units2Indx[(nu as usize).wrapping_sub(1 as i32 as usize) as usize] as u32;
    if (*p).Indx2Units[i as usize] as u32 != nu {
        i = i.wrapping_sub(1);
        let mut k: u32 = (*p).Indx2Units[i as usize] as u32;
        Ppmd8_InsertNode(
            p,
            (ptr as *mut u8).offset((k * 12 as i32 as u32) as isize) as *mut u8,
            nu.wrapping_sub(k).wrapping_sub(1 as i32 as u32),
        );
    }
    Ppmd8_InsertNode(p, ptr, i);
}
unsafe fn Ppmd8_GlueFreeBlocks(mut p: *mut CPpmd8) {
    let mut n: u32 = 0;
    (*p).GlueCount = ((1 as i32) << 13 as i32) as u32;
    (*p).Stamps = [0; 38];
    if (*p).LoUnit != (*p).HiUnit {
        (*((*p).LoUnit as *mut u8 as *mut CPpmd8_Node)).Stamp = 0 as i32 as u32;
    }
    let mut prev: *mut u32 = &mut n;
    let mut i: u32 = 0;
    i = 0 as i32 as u32;
    while i
        < (4 as i32
            + 4 as i32
            + 4 as i32
            + (128 as i32 + 3 as i32
                - 1 as i32 * 4 as i32
                - 2 as i32 * 4 as i32
                - 3 as i32 * 4 as i32)
                / 4 as i32) as u32
    {
        let mut next: u32 = (*p).FreeList[i as usize];
        (*p).FreeList[i as usize] = 0 as i32 as u32;
        while next != 0 as i32 as u32 {
            let mut node: *mut CPpmd8_Node =
                ((*p).Base).offset(next as isize) as *mut u8 as *mut CPpmd8_Node;
            let mut nu: u32 = (*node).NU;
            *prev = next;
            next = (*node).Next;
            if nu != 0 as i32 as u32 {
                let mut node2: *mut CPpmd8_Node = 0 as *mut CPpmd8_Node;
                prev = &mut (*node).Next;
                loop {
                    node2 = node.offset(nu as isize);
                    if !((*node2).Stamp == 0xffffffff as u32) {
                        break;
                    }
                    nu = nu.wrapping_add((*node2).NU);
                    (*node2).NU = 0 as i32 as u32;
                    (*node).NU = nu;
                }
            }
        }
        i = i.wrapping_add(1);
        i;
    }
    *prev = 0 as i32 as u32;
    while n != 0 as i32 as u32 {
        let mut node_0: *mut CPpmd8_Node =
            ((*p).Base).offset(n as isize) as *mut u8 as *mut CPpmd8_Node;
        let mut nu_0: u32 = (*node_0).NU;
        let mut i_0: u32 = 0;
        n = (*node_0).Next;
        if nu_0 == 0 as i32 as u32 {
            continue;
        }
        while nu_0 > 128 as i32 as u32 {
            Ppmd8_InsertNode(
                p,
                node_0 as *mut u8,
                (4 as i32
                    + 4 as i32
                    + 4 as i32
                    + (128 as i32 + 3 as i32
                        - 1 as i32 * 4 as i32
                        - 2 as i32 * 4 as i32
                        - 3 as i32 * 4 as i32)
                        / 4 as i32
                    - 1 as i32) as u32,
            );
            nu_0 = nu_0.wrapping_sub(128 as i32 as u32);
            node_0 = node_0.offset(128 as i32 as isize);
        }
        i_0 = (*p).Units2Indx[(nu_0 as usize).wrapping_sub(1 as i32 as usize) as usize] as u32;
        if (*p).Indx2Units[i_0 as usize] as u32 != nu_0 {
            i_0 = i_0.wrapping_sub(1);
            let mut k: u32 = (*p).Indx2Units[i_0 as usize] as u32;
            Ppmd8_InsertNode(
                p,
                node_0.offset(k as isize) as *mut u8,
                nu_0.wrapping_sub(k).wrapping_sub(1 as i32 as u32),
            );
        }
        Ppmd8_InsertNode(p, node_0 as *mut u8, i_0);
    }
}

#[inline(never)]
unsafe fn Ppmd8_AllocUnitsRare(mut p: *mut CPpmd8, mut indx: u32) -> *mut u8 {
    let mut i: u32 = 0;
    if (*p).GlueCount == 0 as i32 as u32 {
        Ppmd8_GlueFreeBlocks(p);
        if (*p).FreeList[indx as usize] != 0 as i32 as u32 {
            return Ppmd8_RemoveNode(p, indx);
        }
    }
    i = indx;
    loop {
        i = i.wrapping_add(1);
        if i == (4 as i32
            + 4 as i32
            + 4 as i32
            + (128 as i32 + 3 as i32
                - 1 as i32 * 4 as i32
                - 2 as i32 * 4 as i32
                - 3 as i32 * 4 as i32)
                / 4 as i32) as u32
        {
            let mut numBytes: u32 = (*p).Indx2Units[indx as usize] as u32 * 12 as i32 as u32;
            let mut us: *mut u8 = (*p).UnitsStart;
            (*p).GlueCount = ((*p).GlueCount).wrapping_sub(1);
            (*p).GlueCount;
            return (if us.offset_from((*p).Text) as isize as u32 > numBytes {
                (*p).UnitsStart = us.offset(-(numBytes as isize));
                (*p).UnitsStart
            } else {
                0 as *mut u8
            }) as *mut u8;
        }
        if !((*p).FreeList[i as usize] == 0 as i32 as u32) {
            break;
        }
    }
    let mut block: *mut u8 = Ppmd8_RemoveNode(p, i);
    Ppmd8_SplitBlock(p, block, i, indx);
    block
}

unsafe fn Ppmd8_AllocUnits(mut p: *mut CPpmd8, mut indx: u32) -> *mut u8 {
    if (*p).FreeList[indx as usize] != 0 as i32 as u32 {
        return Ppmd8_RemoveNode(p, indx);
    }
    let mut numBytes: u32 = (*p).Indx2Units[indx as usize] as u32 * 12 as i32 as u32;
    let mut lo: *mut u8 = (*p).LoUnit;
    if ((*p).HiUnit).offset_from(lo) as isize as u32 >= numBytes {
        (*p).LoUnit = lo.offset(numBytes as isize);
        return lo as *mut u8;
    }
    Ppmd8_AllocUnitsRare(p, indx)
}

unsafe fn ShrinkUnits(
    mut p: *mut CPpmd8,
    mut oldPtr: *mut u8,
    mut oldNU: u32,
    mut newNU: u32,
) -> *mut u8 {
    let mut i0: u32 =
        (*p).Units2Indx[(oldNU as usize).wrapping_sub(1 as i32 as usize) as usize] as u32;
    let mut i1: u32 =
        (*p).Units2Indx[(newNU as usize).wrapping_sub(1 as i32 as usize) as usize] as u32;
    if i0 == i1 {
        return oldPtr;
    }
    if (*p).FreeList[i1 as usize] != 0 as i32 as u32 {
        let mut ptr: *mut u8 = Ppmd8_RemoveNode(p, i1);
        let mut d: *mut u32 = ptr as *mut u32;
        let mut z: *const u32 = oldPtr as *const u32;
        let mut n: u32 = newNU;
        loop {
            *d.offset(0 as i32 as isize) = *z.offset(0 as i32 as isize);
            *d.offset(1 as i32 as isize) = *z.offset(1 as i32 as isize);
            *d.offset(2 as i32 as isize) = *z.offset(2 as i32 as isize);
            z = z.offset(3 as i32 as isize);
            d = d.offset(3 as i32 as isize);
            n = n.wrapping_sub(1);
            if !(n != 0) {
                break;
            }
        }
        Ppmd8_InsertNode(p, oldPtr, i0);
        return ptr;
    }
    Ppmd8_SplitBlock(p, oldPtr, i0, i1);
    oldPtr
}

unsafe fn FreeUnits(mut p: *mut CPpmd8, mut ptr: *mut u8, mut nu: u32) {
    Ppmd8_InsertNode(
        p,
        ptr,
        (*p).Units2Indx[(nu as usize).wrapping_sub(1 as i32 as usize) as usize] as u32,
    );
}

unsafe fn SpecialFreeUnit(mut p: *mut CPpmd8, mut ptr: *mut u8) {
    if ptr != (*p).UnitsStart {
        Ppmd8_InsertNode(p, ptr, 0 as i32 as u32);
    } else {
        (*p).UnitsStart = ((*p).UnitsStart).offset(12 as i32 as isize);
    };
}

unsafe fn ExpandTextArea(mut p: *mut CPpmd8) {
    let mut count: [u32; 38] = [0; 38];
    let mut i: u32 = 0;
    count = [0; 38];
    if (*p).LoUnit != (*p).HiUnit {
        (*((*p).LoUnit as *mut u8 as *mut CPpmd8_Node)).Stamp = 0 as i32 as u32;
    }
    let mut node: *mut CPpmd8_Node = (*p).UnitsStart as *mut u8 as *mut CPpmd8_Node;
    while (*node).Stamp == 0xffffffff as u32 {
        let mut nu: u32 = (*node).NU;
        (*node).Stamp = 0 as i32 as u32;
        count[(*p).Units2Indx[(nu as usize).wrapping_sub(1 as i32 as usize) as usize] as usize] =
            (count
                [(*p).Units2Indx[(nu as usize).wrapping_sub(1 as i32 as usize) as usize] as usize])
                .wrapping_add(1);
        count[(*p).Units2Indx[(nu as usize).wrapping_sub(1 as i32 as usize) as usize] as usize];
        node = node.offset(nu as isize);
    }
    (*p).UnitsStart = node as *mut u8;
    i = 0 as i32 as u32;
    while i
        < (4 as i32
            + 4 as i32
            + 4 as i32
            + (128 as i32 + 3 as i32
                - 1 as i32 * 4 as i32
                - 2 as i32 * 4 as i32
                - 3 as i32 * 4 as i32)
                / 4 as i32) as u32
    {
        let mut cnt: u32 = count[i as usize];
        if !(cnt == 0 as i32 as u32) {
            let mut prev: *mut u32 =
                &mut *((*p).FreeList).as_mut_ptr().offset(i as isize) as *mut u32 as *mut u32;
            let mut n: u32 = *prev;
            (*p).Stamps[i as usize] = ((*p).Stamps[i as usize]).wrapping_sub(cnt);
            loop {
                let mut node_0: *mut CPpmd8_Node =
                    ((*p).Base).offset(n as isize) as *mut u8 as *mut CPpmd8_Node;
                n = (*node_0).Next;
                if (*node_0).Stamp != 0 as i32 as u32 {
                    prev = &mut (*node_0).Next;
                } else {
                    *prev = n;
                    cnt = cnt.wrapping_sub(1);
                    if cnt == 0 as i32 as u32 {
                        break;
                    }
                }
            }
        }
        i = i.wrapping_add(1);
        i;
    }
}

unsafe fn Ppmd8State_SetSuccessor(mut p: *mut CPpmd_State, mut v: u32) {
    (*p).Successor_0 = v as u16;
    (*p).Successor_1 = (v >> 16 as i32) as u16;
}

#[inline(never)]
unsafe fn Ppmd8_RestartModel(mut p: *mut CPpmd8) {
    let mut i: u32 = 0;
    let mut k: u32 = 0;
    let mut m: u32 = 0;
    (*p).FreeList = [0; 38];
    (*p).Stamps = [0; 38];
    (*p).Text = ((*p).Base)
        .offset((*p).AlignOffset as isize)
        .offset(0 as i32 as isize);
    (*p).HiUnit = ((*p).Text).offset((*p).Size as isize);
    (*p).UnitsStart = ((*p).HiUnit).offset(
        -(((*p).Size / 8 as i32 as u32 / 12 as i32 as u32 * 7 as i32 as u32 * 12 as i32 as u32)
            as isize),
    );
    (*p).LoUnit = (*p).UnitsStart;
    (*p).GlueCount = 0 as i32 as u32;
    (*p).OrderFall = (*p).MaxOrder;
    (*p).InitRL = -((if (*p).MaxOrder < 12 as i32 as u32 {
        (*p).MaxOrder
    } else {
        12 as i32 as u32
    }) as i32)
        - 1 as i32;
    (*p).RunLength = (*p).InitRL;
    (*p).PrevSuccess = 0 as i32 as u32;
    (*p).HiUnit = ((*p).HiUnit).offset(-(12 as i32 as isize));
    let mut mc: *mut CPpmd8_Context = (*p).HiUnit as *mut u8 as *mut CPpmd8_Context;
    let mut s: *mut CPpmd_State = (*p).LoUnit as *mut CPpmd_State;
    (*p).LoUnit =
        ((*p).LoUnit).offset(((256 as i32 / 2 as i32) as u32 * 12 as i32 as u32) as isize);
    (*p).MinContext = mc;
    (*p).MaxContext = (*p).MinContext;
    (*p).FoundState = s;
    (*mc).Flags = 0 as i32 as u8;
    (*mc).NumStats = (256 as i32 - 1 as i32) as u8;
    (*mc).Union2.SummFreq = (256 as i32 + 1 as i32) as u16;
    (*mc).Union4.Stats = (s as *mut u8).offset_from((*p).Base) as isize as u32;
    (*mc).Suffix = 0 as i32 as u32;
    i = 0 as i32 as u32;
    while i < 256 as i32 as u32 {
        (*s).Symbol = i as u8;
        (*s).Freq = 1 as i32 as u8;
        Ppmd8State_SetSuccessor(s, 0 as i32 as u32);
        i = i.wrapping_add(1);
        i;
        s = s.offset(1);
        s;
    }
    m = 0 as i32 as u32;
    i = m;
    while m < 25 as i32 as u32 {
        while (*p).NS2Indx[i as usize] as u32 == m {
            i = i.wrapping_add(1);
            i;
        }
        k = 0 as i32 as u32;
        while k < 8 as i32 as u32 {
            let mut r: u32 = 0;
            let mut dest: *mut u16 = ((*p).BinSumm[m as usize]).as_mut_ptr().offset(k as isize);
            let val: u16 = (((1 as i32) << 7 as i32 + 7 as i32) as u32).wrapping_sub(
                (PPMD8_kInitBinEsc[k as usize] as u32)
                    .wrapping_div(i.wrapping_add(1 as i32 as u32)),
            ) as u16;
            r = 0 as i32 as u32;
            while r < 64 as i32 as u32 {
                *dest.offset(r as isize) = val;
                r = r.wrapping_add(8 as i32 as u32);
            }
            k = k.wrapping_add(1);
            k;
        }
        m = m.wrapping_add(1);
        m;
    }
    m = 0 as i32 as u32;
    i = m;
    while m < 24 as i32 as u32 {
        let mut summ: u32 = 0;
        let mut s_0: *mut CPpmd_See = 0 as *mut CPpmd_See;
        while (*p).NS2Indx[(i as usize).wrapping_add(3 as i32 as usize) as usize] as u32
            == m.wrapping_add(3 as i32 as u32)
        {
            i = i.wrapping_add(1);
            i;
        }
        s_0 = ((*p).See[m as usize]).as_mut_ptr();
        summ = (2 as i32 as u32)
            .wrapping_mul(i)
            .wrapping_add(5 as i32 as u32)
            << 7 as i32 - 4 as i32;
        k = 0 as i32 as u32;
        while k < 32 as i32 as u32 {
            (*s_0).Summ = summ as u16;
            (*s_0).Shift = (7 as i32 - 4 as i32) as u8;
            (*s_0).Count = 7 as i32 as u8;
            k = k.wrapping_add(1);
            k;
            s_0 = s_0.offset(1);
            s_0;
        }
        m = m.wrapping_add(1);
        m;
    }
    (*p).DummySee.Summ = 0 as i32 as u16;
    (*p).DummySee.Shift = 7 as i32 as u8;
    (*p).DummySee.Count = 64 as i32 as u8;
}

pub unsafe fn Ppmd8_Init(mut p: *mut CPpmd8, mut maxOrder: u32, mut restoreMethod: u32) {
    (*p).MaxOrder = maxOrder;
    (*p).RestoreMethod = restoreMethod;
    Ppmd8_RestartModel(p);
}

unsafe fn Refresh(
    mut p: *mut CPpmd8,
    mut ctx: *mut CPpmd8_Context,
    mut oldNU: u32,
    mut scale: u32,
) {
    let mut i: u32 = (*ctx).NumStats as u32;
    let mut escFreq: u32 = 0;
    let mut sumFreq: u32 = 0;
    let mut flags: u32 = 0;
    let mut s: *mut CPpmd_State = ShrinkUnits(
        p,
        ((*p).Base).offset((*ctx).Union4.Stats as isize) as *mut u8 as *mut CPpmd_State as *mut u8,
        oldNU,
        i.wrapping_add(2 as i32 as u32) >> 1 as i32,
    ) as *mut CPpmd_State;
    (*ctx).Union4.Stats = (s as *mut u8).offset_from((*p).Base) as isize as u32;
    scale |= ((*ctx).Union2.SummFreq as u32 >= (1 as i32 as u32) << 15 as i32) as i32 as u32;
    flags = ((*s).Symbol as u32).wrapping_add(0xc0 as i32 as u32);
    let mut freq: u32 = (*s).Freq as u32;
    escFreq = ((*ctx).Union2.SummFreq as u32).wrapping_sub(freq);
    freq = freq.wrapping_add(scale) >> scale;
    sumFreq = freq;
    (*s).Freq = freq as u8;
    loop {
        s = s.offset(1);
        let mut freq_0: u32 = (*s).Freq as u32;
        escFreq = escFreq.wrapping_sub(freq_0);
        freq_0 = freq_0.wrapping_add(scale) >> scale;
        sumFreq = sumFreq.wrapping_add(freq_0);
        (*s).Freq = freq_0 as u8;
        flags |= ((*s).Symbol as u32).wrapping_add(0xc0 as i32 as u32);
        i = i.wrapping_sub(1);
        if !(i != 0) {
            break;
        }
    }
    (*ctx).Union2.SummFreq = sumFreq.wrapping_add(escFreq.wrapping_add(scale) >> scale) as u16;
    (*ctx).Flags = ((*ctx).Flags as u32
        & (((1 as i32) << 4 as i32) as u32)
            .wrapping_add((((1 as i32) << 2 as i32) as u32).wrapping_mul(scale)))
    .wrapping_add(flags >> 8 as i32 - 3 as i32 & ((1 as i32) << 3 as i32) as u32)
        as u8;
}

unsafe fn SWAP_STATES(mut t1: *mut CPpmd_State, mut t2: *mut CPpmd_State) {
    let mut tmp: CPpmd_State = *t1;
    *t1 = *t2;
    *t2 = tmp;
}

unsafe fn CutOff(mut p: *mut CPpmd8, mut ctx: *mut CPpmd8_Context, mut order: u32) -> u32 {
    let mut ns: i32 = (*ctx).NumStats as i32;
    let mut nu: u32 = 0;
    let mut stats: *mut CPpmd_State = 0 as *mut CPpmd_State;
    if ns == 0 as i32 {
        let mut s: *mut CPpmd_State = &mut (*ctx).Union2 as *mut Union2 as *mut CPpmd_State;
        let mut successor: u32 = (*s).Successor_0 as u32 | ((*s).Successor_1 as u32) << 16 as i32;
        if ((*p).Base).offset(successor as isize) as *mut u8 as *mut u8 >= (*p).UnitsStart {
            if order < (*p).MaxOrder {
                successor = CutOff(
                    p,
                    ((*p).Base).offset(successor as isize) as *mut u8 as *mut CPpmd8_Context,
                    order.wrapping_add(1 as i32 as u32),
                );
            } else {
                successor = 0 as i32 as u32;
            }
            Ppmd8State_SetSuccessor(s, successor);
            if successor != 0 || order <= 9 as i32 as u32 {
                return (ctx as *mut u8).offset_from((*p).Base) as isize as u32;
            }
        }
        SpecialFreeUnit(p, ctx as *mut u8);
        return 0 as i32 as u32;
    }
    nu = (ns as u32).wrapping_add(2 as i32 as u32) >> 1 as i32;
    let mut indx: u32 =
        (*p).Units2Indx[(nu as usize).wrapping_sub(1 as i32 as usize) as usize] as u32;
    stats = ((*p).Base).offset((*ctx).Union4.Stats as isize) as *mut u8 as *mut CPpmd_State;
    if (stats as *mut u8).offset_from((*p).UnitsStart) as isize as u32
        <= ((1 as i32) << 14 as i32) as u32
        && (*ctx).Union4.Stats <= (*p).FreeList[indx as usize]
    {
        let mut ptr: *mut u8 = Ppmd8_RemoveNode(p, indx);
        (*ctx).Union4.Stats = (ptr as *mut u8).offset_from((*p).Base) as isize as u32;
        let mut d: *mut u32 = ptr as *mut u32;
        let mut z: *const u32 = stats as *const u8 as *const u32;
        let mut n: u32 = nu;
        loop {
            *d.offset(0 as i32 as isize) = *z.offset(0 as i32 as isize);
            *d.offset(1 as i32 as isize) = *z.offset(1 as i32 as isize);
            *d.offset(2 as i32 as isize) = *z.offset(2 as i32 as isize);
            z = z.offset(3 as i32 as isize);
            d = d.offset(3 as i32 as isize);
            n = n.wrapping_sub(1);
            if !(n != 0) {
                break;
            }
        }
        if stats as *mut u8 != (*p).UnitsStart {
            Ppmd8_InsertNode(p, stats as *mut u8, indx);
        } else {
            (*p).UnitsStart = ((*p).UnitsStart)
                .offset(((*p).Indx2Units[indx as usize] as u32 * 12 as i32 as u32) as isize);
        }
        stats = ptr as *mut CPpmd_State;
    }
    let mut s_0: *mut CPpmd_State = stats.offset(ns as u32 as isize);
    loop {
        let mut successor_0: u32 =
            (*s_0).Successor_0 as u32 | ((*s_0).Successor_1 as u32) << 16 as i32;
        if (((*p).Base).offset(successor_0 as isize) as *mut u8 as *mut u8) < (*p).UnitsStart {
            let fresh1 = ns;
            ns = ns - 1;
            let mut s2: *mut CPpmd_State = stats.offset(fresh1 as u32 as isize);
            if order != 0 {
                if s_0 != s2 {
                    *s_0 = *s2;
                }
            } else {
                SWAP_STATES(s_0, s2);
                Ppmd8State_SetSuccessor(s2, 0 as i32 as u32);
            }
        } else if order < (*p).MaxOrder {
            Ppmd8State_SetSuccessor(
                s_0,
                CutOff(
                    p,
                    ((*p).Base).offset(successor_0 as isize) as *mut u8 as *mut CPpmd8_Context,
                    order.wrapping_add(1 as i32 as u32),
                ),
            );
        } else {
            Ppmd8State_SetSuccessor(s_0, 0 as i32 as u32);
        }
        s_0 = s_0.offset(-1);
        if !(s_0 >= stats) {
            break;
        }
    }
    if ns != (*ctx).NumStats as i32 && order != 0 {
        if ns < 0 as i32 {
            FreeUnits(p, stats as *mut u8, nu);
            SpecialFreeUnit(p, ctx as *mut u8);
            return 0 as i32 as u32;
        }
        (*ctx).NumStats = ns as u8;
        if ns == 0 as i32 {
            let sym: u8 = (*stats).Symbol;
            (*ctx).Flags = (((*ctx).Flags as i32 & (1 as i32) << 4 as i32) as u32).wrapping_add(
                (sym as u32).wrapping_add(0xc0 as i32 as u32) >> 8 as i32 - 3 as i32
                    & ((1 as i32) << 3 as i32) as u32,
            ) as u8;
            (*ctx).Union2.State2.Symbol = sym;
            (*ctx).Union2.State2.Freq =
                (((*stats).Freq as u32).wrapping_add(11 as i32 as u32) >> 3 as i32) as u8;
            (*ctx).Union4.State4.Successor_0 = (*stats).Successor_0;
            (*ctx).Union4.State4.Successor_1 = (*stats).Successor_1;
            FreeUnits(p, stats as *mut u8, nu);
        } else {
            Refresh(
                p,
                ctx,
                nu,
                ((*ctx).Union2.SummFreq as u32 > (16 as i32 as u32).wrapping_mul(ns as u32)) as i32
                    as u32,
            );
        }
    }
    return (ctx as *mut u8).offset_from((*p).Base) as isize as u32;
}

unsafe fn GetUsedMemory(mut p: *const CPpmd8) -> u32 {
    let mut v: u32 = 0 as i32 as u32;
    let mut i: u32 = 0;
    i = 0 as i32 as u32;
    while i
        < (4 as i32
            + 4 as i32
            + 4 as i32
            + (128 as i32 + 3 as i32
                - 1 as i32 * 4 as i32
                - 2 as i32 * 4 as i32
                - 3 as i32 * 4 as i32)
                / 4 as i32) as u32
    {
        v = (v as u32).wrapping_add(
            ((*p).Stamps[i as usize]).wrapping_mul((*p).Indx2Units[i as usize] as u32),
        ) as u32 as u32;
        i = i.wrapping_add(1);
        i;
    }
    return ((*p).Size)
        .wrapping_sub(((*p).HiUnit).offset_from((*p).LoUnit) as isize as u32)
        .wrapping_sub(((*p).UnitsStart).offset_from((*p).Text) as isize as u32)
        .wrapping_sub(v * 12 as i32 as u32);
}

unsafe fn RestoreModel(mut p: *mut CPpmd8, mut ctxError: *mut CPpmd8_Context) {
    let mut c: *mut CPpmd8_Context = 0 as *mut CPpmd8_Context;
    let mut s: *mut CPpmd_State = 0 as *mut CPpmd_State;
    (*p).Text = ((*p).Base)
        .offset((*p).AlignOffset as isize)
        .offset(0 as i32 as isize);
    c = (*p).MaxContext;
    while c != ctxError {
        (*c).NumStats = ((*c).NumStats).wrapping_sub(1);
        if (*c).NumStats as i32 == 0 as i32 {
            s = ((*p).Base).offset((*c).Union4.Stats as isize) as *mut u8 as *mut CPpmd_State;
            (*c).Flags = (((*c).Flags as i32 & (1 as i32) << 4 as i32) as u32).wrapping_add(
                ((*s).Symbol as u32).wrapping_add(0xc0 as i32 as u32) >> 8 as i32 - 3 as i32
                    & ((1 as i32) << 3 as i32) as u32,
            ) as u8;
            (*c).Union2.State2.Symbol = (*s).Symbol;
            (*c).Union2.State2.Freq =
                (((*s).Freq as u32).wrapping_add(11 as i32 as u32) >> 3 as i32) as u8;
            (*c).Union4.State4.Successor_0 = (*s).Successor_0;
            (*c).Union4.State4.Successor_1 = (*s).Successor_1;
            SpecialFreeUnit(p, s as *mut u8);
        } else {
            Refresh(
                p,
                c,
                ((*c).NumStats as u32).wrapping_add(3 as i32 as u32) >> 1 as i32,
                0 as i32 as u32,
            );
        }
        c = ((*p).Base).offset((*c).Suffix as isize) as *mut u8 as *mut CPpmd8_Context;
    }
    while c != (*p).MinContext {
        if (*c).NumStats as i32 == 0 as i32 {
            (*c).Union2.State2.Freq =
                (((*c).Union2.State2.Freq as u32).wrapping_add(1 as i32 as u32) >> 1 as i32) as u8;
        } else {
            (*c).Union2.SummFreq = ((*c).Union2.SummFreq as i32 + 4 as i32) as u16;
            if (*c).Union2.SummFreq as i32 > 128 as i32 + 4 as i32 * (*c).NumStats as i32 {
                Refresh(
                    p,
                    c,
                    ((*c).NumStats as u32).wrapping_add(2 as i32 as u32) >> 1 as i32,
                    1 as i32 as u32,
                );
            }
        }
        c = ((*p).Base).offset((*c).Suffix as isize) as *mut u8 as *mut CPpmd8_Context;
    }
    if (*p).RestoreMethod == PPMD8_RESTORE_METHOD_RESTART as i32 as u32
        || GetUsedMemory(p) < (*p).Size >> 1 as i32
    {
        Ppmd8_RestartModel(p);
    } else {
        while (*(*p).MaxContext).Suffix != 0 {
            (*p).MaxContext = ((*p).Base).offset((*(*p).MaxContext).Suffix as isize) as *mut u8
                as *mut CPpmd8_Context;
        }
        loop {
            CutOff(p, (*p).MaxContext, 0 as i32 as u32);
            ExpandTextArea(p);
            if !(GetUsedMemory(p) > 3 as i32 as u32 * ((*p).Size >> 2 as i32)) {
                break;
            }
        }
        (*p).GlueCount = 0 as i32 as u32;
        (*p).OrderFall = (*p).MaxOrder;
    }
    (*p).MinContext = (*p).MaxContext;
}

#[inline(never)]
unsafe fn Ppmd8_CreateSuccessors(
    mut p: *mut CPpmd8,
    mut skip: i32,
    mut s1: *mut CPpmd_State,
    mut c: *mut CPpmd8_Context,
) -> *mut CPpmd8_Context {
    let mut upBranch: u32 = (*(*p).FoundState).Successor_0 as u32
        | ((*(*p).FoundState).Successor_1 as u32) << 16 as i32;
    let mut newSym: u8 = 0;
    let mut newFreq: u8 = 0;
    let mut flags: u8 = 0;
    let mut numPs: u32 = 0 as i32 as u32;
    let mut ps: [*mut CPpmd_State; 17] = [0 as *mut CPpmd_State; 17];
    if skip == 0 {
        let fresh2 = numPs;
        numPs = numPs.wrapping_add(1);
        ps[fresh2 as usize] = (*p).FoundState;
    }
    while (*c).Suffix != 0 {
        let mut successor: u32 = 0;
        let mut s: *mut CPpmd_State = 0 as *mut CPpmd_State;
        c = ((*p).Base).offset((*c).Suffix as isize) as *mut u8 as *mut CPpmd8_Context;
        if !s1.is_null() {
            s = s1;
            s1 = 0 as *mut CPpmd_State;
        } else if (*c).NumStats as i32 != 0 as i32 {
            let mut sym: u8 = (*(*p).FoundState).Symbol;
            s = ((*p).Base).offset((*c).Union4.Stats as isize) as *mut u8 as *mut CPpmd_State;
            while (*s).Symbol as i32 != sym as i32 {
                s = s.offset(1);
                s;
            }
            if ((*s).Freq as i32) < 124 as i32 - 9 as i32 {
                (*s).Freq = ((*s).Freq).wrapping_add(1);
                (*s).Freq;
                (*c).Union2.SummFreq = ((*c).Union2.SummFreq).wrapping_add(1);
                (*c).Union2.SummFreq;
            }
        } else {
            s = &mut (*c).Union2 as *mut Union2 as *mut CPpmd_State;
            (*s).Freq = ((*s).Freq as i32
                + (((*(((*p).Base).offset((*c).Suffix as isize) as *mut u8 as *mut CPpmd8_Context))
                    .NumStats
                    == 0) as i32
                    & (((*s).Freq as i32) < 24 as i32) as i32)) as u8;
        }
        successor = (*s).Successor_0 as u32 | ((*s).Successor_1 as u32) << 16 as i32;
        if successor != upBranch {
            c = ((*p).Base).offset(successor as isize) as *mut u8 as *mut CPpmd8_Context;
            if numPs == 0 as i32 as u32 {
                return c;
            }
            break;
        } else {
            let fresh3 = numPs;
            numPs = numPs.wrapping_add(1);
            ps[fresh3 as usize] = s;
        }
    }
    newSym = *(((*p).Base).offset(upBranch as isize) as *mut u8 as *const u8);
    upBranch = upBranch.wrapping_add(1);
    upBranch;
    flags = (((*(*p).FoundState).Symbol as u32).wrapping_add(0xc0 as i32 as u32)
        >> 8 as i32 - 4 as i32
        & ((1 as i32) << 4 as i32) as u32)
        .wrapping_add(
            (newSym as u32).wrapping_add(0xc0 as i32 as u32) >> 8 as i32 - 3 as i32
                & ((1 as i32) << 3 as i32) as u32,
        ) as u8;
    if (*c).NumStats as i32 == 0 as i32 {
        newFreq = (*c).Union2.State2.Freq;
    } else {
        let mut cf: u32 = 0;
        let mut s0: u32 = 0;
        let mut s_0: *mut CPpmd_State = 0 as *mut CPpmd_State;
        s_0 = ((*p).Base).offset((*c).Union4.Stats as isize) as *mut u8 as *mut CPpmd_State;
        while (*s_0).Symbol as i32 != newSym as i32 {
            s_0 = s_0.offset(1);
            s_0;
        }
        cf = ((*s_0).Freq as u32).wrapping_sub(1 as i32 as u32);
        s0 = ((*c).Union2.SummFreq as u32)
            .wrapping_sub((*c).NumStats as u32)
            .wrapping_sub(cf);
        newFreq = (1 as i32 as u32).wrapping_add(
            (if 2 as i32 as u32 * cf <= s0 {
                (5 as i32 as u32 * cf > s0) as i32 as u32
            } else {
                cf.wrapping_add(2 as i32 as u32 * s0)
                    .wrapping_sub(3 as i32 as u32)
                    / s0
            }),
        ) as u8;
    }
    loop {
        let mut c1: *mut CPpmd8_Context = 0 as *mut CPpmd8_Context;
        if (*p).HiUnit != (*p).LoUnit {
            (*p).HiUnit = ((*p).HiUnit).offset(-(12 as i32 as isize));
            c1 = (*p).HiUnit as *mut u8 as *mut CPpmd8_Context;
        } else if (*p).FreeList[0 as i32 as usize] != 0 as i32 as u32 {
            c1 = Ppmd8_RemoveNode(p, 0 as i32 as u32) as *mut CPpmd8_Context;
        } else {
            c1 = Ppmd8_AllocUnitsRare(p, 0 as i32 as u32) as *mut CPpmd8_Context;
            if c1.is_null() {
                return 0 as *mut CPpmd8_Context;
            }
        }
        (*c1).Flags = flags;
        (*c1).NumStats = 0 as i32 as u8;
        (*c1).Union2.State2.Symbol = newSym;
        (*c1).Union2.State2.Freq = newFreq;
        Ppmd8State_SetSuccessor(
            &mut (*c1).Union2 as *mut Union2 as *mut CPpmd_State,
            upBranch,
        );
        (*c1).Suffix = (c as *mut u8).offset_from((*p).Base) as isize as u32;
        numPs = numPs.wrapping_sub(1);
        Ppmd8State_SetSuccessor(
            ps[numPs as usize],
            (c1 as *mut u8).offset_from((*p).Base) as isize as u32,
        );
        c = c1;
        if !(numPs != 0 as i32 as u32) {
            break;
        }
    }
    return c;
}

unsafe fn ReduceOrder(
    mut p: *mut CPpmd8,
    mut s1: *mut CPpmd_State,
    mut c: *mut CPpmd8_Context,
) -> *mut CPpmd8_Context {
    let mut s: *mut CPpmd_State = 0 as *mut CPpmd_State;
    let mut c1: *mut CPpmd8_Context = c;
    let mut upBranch: u32 = ((*p).Text).offset_from((*p).Base) as isize as u32;
    Ppmd8State_SetSuccessor((*p).FoundState, upBranch);
    (*p).OrderFall = ((*p).OrderFall).wrapping_add(1);
    (*p).OrderFall;
    loop {
        if !s1.is_null() {
            c = ((*p).Base).offset((*c).Suffix as isize) as *mut u8 as *mut CPpmd8_Context;
            s = s1;
            s1 = 0 as *mut CPpmd_State;
        } else {
            if (*c).Suffix == 0 {
                return c;
            }
            c = ((*p).Base).offset((*c).Suffix as isize) as *mut u8 as *mut CPpmd8_Context;
            if (*c).NumStats != 0 {
                s = ((*p).Base).offset((*c).Union4.Stats as isize) as *mut u8 as *mut CPpmd_State;
                if (*s).Symbol as i32 != (*(*p).FoundState).Symbol as i32 {
                    loop {
                        s = s.offset(1);
                        s;
                        if !((*s).Symbol as i32 != (*(*p).FoundState).Symbol as i32) {
                            break;
                        }
                    }
                }
                if ((*s).Freq as i32) < 124 as i32 - 9 as i32 {
                    (*s).Freq = ((*s).Freq as i32 + 2 as i32) as u8;
                    (*c).Union2.SummFreq = ((*c).Union2.SummFreq as i32 + 2 as i32) as u16;
                }
            } else {
                s = &mut (*c).Union2 as *mut Union2 as *mut CPpmd_State;
                (*s).Freq = ((*s).Freq as i32 + (((*s).Freq as i32) < 32 as i32) as i32) as u8;
            }
        }
        if (*s).Successor_0 as u32 | ((*s).Successor_1 as u32) << 16 as i32 != 0 {
            break;
        }
        Ppmd8State_SetSuccessor(s, upBranch);
        (*p).OrderFall = ((*p).OrderFall).wrapping_add(1);
        (*p).OrderFall;
    }
    if (*s).Successor_0 as u32 | ((*s).Successor_1 as u32) << 16 as i32 <= upBranch {
        let mut successor: *mut CPpmd8_Context = 0 as *mut CPpmd8_Context;
        let mut s2: *mut CPpmd_State = (*p).FoundState;
        (*p).FoundState = s;
        successor = Ppmd8_CreateSuccessors(p, 0 as i32, 0 as *mut CPpmd_State, c);
        if successor.is_null() {
            Ppmd8State_SetSuccessor(s, 0 as i32 as u32);
        } else {
            Ppmd8State_SetSuccessor(
                s,
                (successor as *mut u8).offset_from((*p).Base) as isize as u32,
            );
        }
        (*p).FoundState = s2;
    }
    let mut successor_0: u32 = (*s).Successor_0 as u32 | ((*s).Successor_1 as u32) << 16 as i32;
    if (*p).OrderFall == 1 as i32 as u32 && c1 == (*p).MaxContext {
        Ppmd8State_SetSuccessor((*p).FoundState, successor_0);
        (*p).Text = ((*p).Text).offset(-1);
        (*p).Text;
    }
    if successor_0 == 0 as i32 as u32 {
        return 0 as *mut CPpmd8_Context;
    }
    return ((*p).Base).offset(successor_0 as isize) as *mut u8 as *mut CPpmd8_Context;
}

#[inline(never)]
pub unsafe fn Ppmd8_UpdateModel(mut p: *mut CPpmd8) {
    let mut maxSuccessor: u32 = 0;
    let mut minSuccessor: u32 = (*(*p).FoundState).Successor_0 as u32
        | ((*(*p).FoundState).Successor_1 as u32) << 16 as i32;
    let mut c: *mut CPpmd8_Context = 0 as *mut CPpmd8_Context;
    let mut s0: u32 = 0;
    let mut ns: u32 = 0;
    let mut fFreq: u32 = (*(*p).FoundState).Freq as u32;
    let mut flag: u8 = 0;
    let mut fSymbol: u8 = (*(*p).FoundState).Symbol;
    let mut s: *mut CPpmd_State = 0 as *mut CPpmd_State;
    if ((*(*p).FoundState).Freq as i32) < 124 as i32 / 4 as i32
        && (*(*p).MinContext).Suffix != 0 as i32 as u32
    {
        c = ((*p).Base).offset((*(*p).MinContext).Suffix as isize) as *mut u8
            as *mut CPpmd8_Context;
        if (*c).NumStats as i32 == 0 as i32 {
            s = &mut (*c).Union2 as *mut Union2 as *mut CPpmd_State;
            if ((*s).Freq as i32) < 32 as i32 {
                (*s).Freq = ((*s).Freq).wrapping_add(1);
                (*s).Freq;
            }
        } else {
            let mut sym: u8 = (*(*p).FoundState).Symbol;
            s = ((*p).Base).offset((*c).Union4.Stats as isize) as *mut u8 as *mut CPpmd_State;
            if (*s).Symbol as i32 != sym as i32 {
                loop {
                    s = s.offset(1);
                    s;
                    if !((*s).Symbol as i32 != sym as i32) {
                        break;
                    }
                }
                if (*s.offset(0 as i32 as isize)).Freq as i32
                    >= (*s.offset(-(1 as i32) as isize)).Freq as i32
                {
                    SWAP_STATES(
                        &mut *s.offset(0 as i32 as isize),
                        &mut *s.offset(-(1 as i32) as isize),
                    );
                    s = s.offset(-1);
                    s;
                }
            }
            if ((*s).Freq as i32) < 124 as i32 - 9 as i32 {
                (*s).Freq = ((*s).Freq as i32 + 2 as i32) as u8;
                (*c).Union2.SummFreq = ((*c).Union2.SummFreq as i32 + 2 as i32) as u16;
            }
        }
    }
    c = (*p).MaxContext;
    if (*p).OrderFall == 0 as i32 as u32 && minSuccessor != 0 {
        let mut cs: *mut CPpmd8_Context = Ppmd8_CreateSuccessors(p, 1 as i32, s, (*p).MinContext);
        if cs.is_null() {
            Ppmd8State_SetSuccessor((*p).FoundState, 0 as i32 as u32);
            RestoreModel(p, c);
            return;
        }
        Ppmd8State_SetSuccessor(
            (*p).FoundState,
            (cs as *mut u8).offset_from((*p).Base) as isize as u32,
        );
        (*p).MaxContext = cs;
        (*p).MinContext = (*p).MaxContext;
        return;
    }
    let mut text: *mut u8 = (*p).Text;
    let fresh4 = text;
    text = text.offset(1);
    *fresh4 = (*(*p).FoundState).Symbol;
    (*p).Text = text;
    if text >= (*p).UnitsStart {
        RestoreModel(p, c);
        return;
    }
    maxSuccessor = text.offset_from((*p).Base) as isize as u32;
    if minSuccessor == 0 {
        let mut cs_0: *mut CPpmd8_Context = ReduceOrder(p, s, (*p).MinContext);
        if cs_0.is_null() {
            RestoreModel(p, c);
            return;
        }
        minSuccessor = (cs_0 as *mut u8).offset_from((*p).Base) as isize as u32;
    } else if (((*p).Base).offset(minSuccessor as isize) as *mut u8 as *mut u8) < (*p).UnitsStart {
        let mut cs_1: *mut CPpmd8_Context = Ppmd8_CreateSuccessors(p, 0 as i32, s, (*p).MinContext);
        if cs_1.is_null() {
            RestoreModel(p, c);
            return;
        }
        minSuccessor = (cs_1 as *mut u8).offset_from((*p).Base) as isize as u32;
    }
    (*p).OrderFall = ((*p).OrderFall).wrapping_sub(1);
    if (*p).OrderFall == 0 as i32 as u32 {
        maxSuccessor = minSuccessor;
        (*p).Text = ((*p).Text).offset(-(((*p).MaxContext != (*p).MinContext) as i32 as isize));
    }
    flag = ((fSymbol as u32).wrapping_add(0xc0 as i32 as u32) >> 8 as i32 - 3 as i32
        & ((1 as i32) << 3 as i32) as u32) as u8;
    ns = (*(*p).MinContext).NumStats as u32;
    s0 = ((*(*p).MinContext).Union2.SummFreq as u32)
        .wrapping_sub(ns)
        .wrapping_sub(fFreq);
    while c != (*p).MinContext {
        let mut ns1: u32 = 0;
        let mut sum: u32 = 0;
        ns1 = (*c).NumStats as u32;
        if ns1 != 0 as i32 as u32 {
            if ns1 & 1 as i32 as u32 != 0 as i32 as u32 {
                let oldNU: u32 = ns1.wrapping_add(1 as i32 as u32) >> 1 as i32;
                let i: u32 = (*p).Units2Indx
                    [(oldNU as usize).wrapping_sub(1 as i32 as usize) as usize]
                    as u32;
                if i != (*p).Units2Indx[(oldNU as usize)
                    .wrapping_add(1 as i32 as usize)
                    .wrapping_sub(1 as i32 as usize)
                    as usize] as u32
                {
                    let mut ptr: *mut u8 = Ppmd8_AllocUnits(p, i.wrapping_add(1 as i32 as u32));
                    let mut oldPtr: *mut u8 = 0 as *mut u8;
                    if ptr.is_null() {
                        RestoreModel(p, c);
                        return;
                    }
                    oldPtr = ((*p).Base).offset((*c).Union4.Stats as isize) as *mut u8
                        as *mut CPpmd_State as *mut u8;
                    let mut d: *mut u32 = ptr as *mut u32;
                    let mut z: *const u32 = oldPtr as *const u32;
                    let mut n: u32 = oldNU;
                    loop {
                        *d.offset(0 as i32 as isize) = *z.offset(0 as i32 as isize);
                        *d.offset(1 as i32 as isize) = *z.offset(1 as i32 as isize);
                        *d.offset(2 as i32 as isize) = *z.offset(2 as i32 as isize);
                        z = z.offset(3 as i32 as isize);
                        d = d.offset(3 as i32 as isize);
                        n = n.wrapping_sub(1);
                        if !(n != 0) {
                            break;
                        }
                    }
                    Ppmd8_InsertNode(p, oldPtr, i);
                    (*c).Union4.Stats = (ptr as *mut u8).offset_from((*p).Base) as isize as u32;
                }
            }
            sum = (*c).Union2.SummFreq as u32;
            sum = sum.wrapping_add(
                ((3 as i32 as u32)
                    .wrapping_mul(ns1)
                    .wrapping_add(1 as i32 as u32)
                    < ns) as i32 as u32,
            );
        } else {
            let mut s_0: *mut CPpmd_State =
                Ppmd8_AllocUnits(p, 0 as i32 as u32) as *mut CPpmd_State;
            if s_0.is_null() {
                RestoreModel(p, c);
                return;
            }
            let mut freq: u32 = (*c).Union2.State2.Freq as u32;
            (*s_0).Symbol = (*c).Union2.State2.Symbol;
            (*s_0).Successor_0 = (*c).Union4.State4.Successor_0;
            (*s_0).Successor_1 = (*c).Union4.State4.Successor_1;
            (*c).Union4.Stats = (s_0 as *mut u8).offset_from((*p).Base) as isize as u32;
            if freq < (124 as i32 / 4 as i32 - 1 as i32) as u32 {
                freq <<= 1 as i32;
            } else {
                freq = (124 as i32 - 4 as i32) as u32;
            }
            (*s_0).Freq = freq as u8;
            sum = freq
                .wrapping_add((*p).InitEsc)
                .wrapping_add((ns > 2 as i32 as u32) as i32 as u32);
        }
        let mut s_1: *mut CPpmd_State = (((*p).Base).offset((*c).Union4.Stats as isize) as *mut u8
            as *mut CPpmd_State)
            .offset(ns1 as isize)
            .offset(1 as i32 as isize);
        let mut cf: u32 = 2 as i32 as u32 * sum.wrapping_add(6 as i32 as u32) * fFreq;
        let mut sf: u32 = s0.wrapping_add(sum);
        (*s_1).Symbol = fSymbol;
        (*c).NumStats = ns1.wrapping_add(1 as i32 as u32) as u8;
        Ppmd8State_SetSuccessor(s_1, maxSuccessor);
        (*c).Flags = ((*c).Flags as i32 | flag as i32) as u8;
        if cf < 6 as i32 as u32 * sf {
            cf = (1 as i32 as u32)
                .wrapping_add((cf > sf) as i32 as u32)
                .wrapping_add((cf >= 4 as i32 as u32 * sf) as i32 as u32);
            sum = sum.wrapping_add(4 as i32 as u32);
        } else {
            cf = (4 as i32 as u32)
                .wrapping_add((cf > 9 as i32 as u32 * sf) as i32 as u32)
                .wrapping_add((cf > 12 as i32 as u32 * sf) as i32 as u32)
                .wrapping_add((cf > 15 as i32 as u32 * sf) as i32 as u32);
            sum = sum.wrapping_add(cf);
        }
        (*c).Union2.SummFreq = sum as u16;
        (*s_1).Freq = cf as u8;
        c = ((*p).Base).offset((*c).Suffix as isize) as *mut u8 as *mut CPpmd8_Context;
    }
    (*p).MinContext = ((*p).Base).offset(minSuccessor as isize) as *mut u8 as *mut CPpmd8_Context;
    (*p).MaxContext = (*p).MinContext;
}

#[inline(never)]
unsafe fn Ppmd8_Rescale(mut p: *mut CPpmd8) {
    let mut i: u32 = 0;
    let mut adder: u32 = 0;
    let mut sumFreq: u32 = 0;
    let mut escFreq: u32 = 0;
    let mut stats: *mut CPpmd_State =
        ((*p).Base).offset((*(*p).MinContext).Union4.Stats as isize) as *mut u8 as *mut CPpmd_State;
    let mut s: *mut CPpmd_State = (*p).FoundState;
    if s != stats {
        let mut tmp: CPpmd_State = *s;
        loop {
            *s.offset(0 as i32 as isize) = *s.offset(-(1 as i32) as isize);
            s = s.offset(-1);
            if !(s != stats) {
                break;
            }
        }
        *s = tmp;
    }
    sumFreq = (*s).Freq as u32;
    escFreq = ((*(*p).MinContext).Union2.SummFreq as u32).wrapping_sub(sumFreq);
    adder = ((*p).OrderFall != 0 as i32 as u32) as i32 as u32;
    sumFreq = sumFreq.wrapping_add(4 as i32 as u32).wrapping_add(adder) >> 1 as i32;
    i = (*(*p).MinContext).NumStats as u32;
    (*s).Freq = sumFreq as u8;
    loop {
        s = s.offset(1);
        let mut freq: u32 = (*s).Freq as u32;
        escFreq = escFreq.wrapping_sub(freq);
        freq = freq.wrapping_add(adder) >> 1 as i32;
        sumFreq = sumFreq.wrapping_add(freq);
        (*s).Freq = freq as u8;
        if freq > (*s.offset(-(1 as i32) as isize)).Freq as u32 {
            let mut tmp_0: CPpmd_State = *s;
            let mut s1: *mut CPpmd_State = s;
            loop {
                *s1.offset(0 as i32 as isize) = *s1.offset(-(1 as i32) as isize);
                s1 = s1.offset(-1);
                if !(s1 != stats && freq > (*s1.offset(-(1 as i32) as isize)).Freq as u32) {
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
    if (*s).Freq as i32 == 0 as i32 {
        let mut mc: *mut CPpmd8_Context = 0 as *mut CPpmd8_Context;
        let mut numStats: u32 = 0;
        let mut numStatsNew: u32 = 0;
        let mut n0: u32 = 0;
        let mut n1: u32 = 0;
        i = 0 as i32 as u32;
        loop {
            i = i.wrapping_add(1);
            i;
            s = s.offset(-1);
            if !((*s).Freq as i32 == 0 as i32) {
                break;
            }
        }
        escFreq = escFreq.wrapping_add(i);
        mc = (*p).MinContext;
        numStats = (*mc).NumStats as u32;
        numStatsNew = numStats.wrapping_sub(i);
        (*mc).NumStats = numStatsNew as u8;
        n0 = numStats.wrapping_add(2 as i32 as u32) >> 1 as i32;
        if numStatsNew == 0 as i32 as u32 {
            let mut freq_0: u32 = (2 as i32 as u32)
                .wrapping_mul((*stats).Freq as u32)
                .wrapping_add(escFreq)
                .wrapping_sub(1 as i32 as u32)
                .wrapping_div(escFreq);
            if freq_0 > (124 as i32 / 3 as i32) as u32 {
                freq_0 = (124 as i32 / 3 as i32) as u32;
            }
            (*mc).Flags = (((*mc).Flags as i32 & (1 as i32) << 4 as i32) as u32).wrapping_add(
                ((*stats).Symbol as u32).wrapping_add(0xc0 as i32 as u32) >> 8 as i32 - 3 as i32
                    & ((1 as i32) << 3 as i32) as u32,
            ) as u8;
            s = &mut (*mc).Union2 as *mut Union2 as *mut CPpmd_State;
            *s = *stats;
            (*s).Freq = freq_0 as u8;
            (*p).FoundState = s;
            Ppmd8_InsertNode(
                p,
                stats as *mut u8,
                (*p).Units2Indx[(n0 as usize).wrapping_sub(1 as i32 as usize) as usize] as u32,
            );
            return;
        }
        n1 = numStatsNew.wrapping_add(2 as i32 as u32) >> 1 as i32;
        if n0 != n1 {
            (*mc).Union4.Stats = (ShrinkUnits(p, stats as *mut u8, n0, n1) as *mut u8)
                .offset_from((*p).Base) as isize as u32;
        }
    }
    let mut mc_0: *mut CPpmd8_Context = (*p).MinContext;
    (*mc_0).Union2.SummFreq = sumFreq
        .wrapping_add(escFreq)
        .wrapping_sub(escFreq >> 1 as i32) as u16;
    (*mc_0).Flags = ((*mc_0).Flags as i32 | (1 as i32) << 2 as i32) as u8;
    (*p).FoundState =
        ((*p).Base).offset((*mc_0).Union4.Stats as isize) as *mut u8 as *mut CPpmd_State;
}

pub unsafe fn Ppmd8_MakeEscFreq(
    mut p: *mut CPpmd8,
    mut numMasked1: u32,
    mut escFreq: *mut u32,
) -> *mut CPpmd_See {
    let mut see: *mut CPpmd_See = 0 as *mut CPpmd_See;
    let mut mc: *const CPpmd8_Context = (*p).MinContext;
    let mut numStats: u32 = (*mc).NumStats as u32;
    if numStats != 0xff as i32 as u32 {
        see = ((*p).See[((*p).NS2Indx[(numStats as usize).wrapping_add(2 as i32 as usize) as usize]
            as u32 as usize)
            .wrapping_sub(3 as i32 as usize) as usize])
            .as_mut_ptr()
            .offset(
                ((*mc).Union2.SummFreq as u32
                    > (11 as i32 as u32).wrapping_mul(numStats.wrapping_add(1 as i32 as u32)))
                    as i32 as isize,
            )
            .offset(
                (2 as i32 as u32).wrapping_mul(
                    ((2 as i32 as u32).wrapping_mul(numStats)
                        < ((*(((*p).Base).offset((*mc).Suffix as isize) as *mut u8
                            as *mut CPpmd8_Context))
                            .NumStats as u32)
                            .wrapping_add(numMasked1)) as i32 as u32,
                ) as isize,
            )
            .offset((*mc).Flags as i32 as isize);
        let summ: u32 = (*see).Summ as u32;
        let r: u32 = summ >> (*see).Shift as i32;
        (*see).Summ = summ.wrapping_sub(r) as u16;
        *escFreq = r.wrapping_add((r == 0 as i32 as u32) as i32 as u32);
    } else {
        see = &mut (*p).DummySee;
        *escFreq = 1 as i32 as u32;
    }
    return see;
}

unsafe fn Ppmd8_NextContext(mut p: *mut CPpmd8) {
    let mut c: *mut CPpmd8_Context = ((*p).Base).offset(
        ((*(*p).FoundState).Successor_0 as u32
            | ((*(*p).FoundState).Successor_1 as u32) << 16 as i32) as isize,
    ) as *mut u8 as *mut CPpmd8_Context;
    if (*p).OrderFall == 0 as i32 as u32 && c as *const u8 >= (*p).UnitsStart as *const u8 {
        (*p).MinContext = c;
        (*p).MaxContext = (*p).MinContext;
    } else {
        Ppmd8_UpdateModel(p);
    };
}

pub unsafe fn Ppmd8_Update1(mut p: *mut CPpmd8) {
    let mut s: *mut CPpmd_State = (*p).FoundState;
    let mut freq: u32 = (*s).Freq as u32;
    freq = freq.wrapping_add(4 as i32 as u32);
    (*(*p).MinContext).Union2.SummFreq =
        ((*(*p).MinContext).Union2.SummFreq as i32 + 4 as i32) as u16;
    (*s).Freq = freq as u8;
    if freq > (*s.offset(-(1 as i32) as isize)).Freq as u32 {
        SWAP_STATES(s, &mut *s.offset(-(1 as i32) as isize));
        s = s.offset(-1);
        (*p).FoundState = s;
        if freq > 124 as i32 as u32 {
            Ppmd8_Rescale(p);
        }
    }
    Ppmd8_NextContext(p);
}

pub unsafe fn Ppmd8_Update1_0(mut p: *mut CPpmd8) {
    let mut s: *mut CPpmd_State = (*p).FoundState;
    let mut mc: *mut CPpmd8_Context = (*p).MinContext;
    let mut freq: u32 = (*s).Freq as u32;
    let summFreq: u32 = (*mc).Union2.SummFreq as u32;
    (*p).PrevSuccess = ((2 as i32 as u32).wrapping_mul(freq) >= summFreq) as i32 as u32;
    (*p).RunLength += (*p).PrevSuccess as i32;
    (*mc).Union2.SummFreq = summFreq.wrapping_add(4 as i32 as u32) as u16;
    freq = freq.wrapping_add(4 as i32 as u32);
    (*s).Freq = freq as u8;
    if freq > 124 as i32 as u32 {
        Ppmd8_Rescale(p);
    }
    Ppmd8_NextContext(p);
}

pub unsafe fn Ppmd8_Update2(mut p: *mut CPpmd8) {
    let mut s: *mut CPpmd_State = (*p).FoundState;
    let mut freq: u32 = (*s).Freq as u32;
    freq = freq.wrapping_add(4 as i32 as u32);
    (*p).RunLength = (*p).InitRL;
    (*(*p).MinContext).Union2.SummFreq =
        ((*(*p).MinContext).Union2.SummFreq as i32 + 4 as i32) as u16;
    (*s).Freq = freq as u8;
    if freq > 124 as i32 as u32 {
        Ppmd8_Rescale(p);
    }
    Ppmd8_UpdateModel(p);
}
