mod decoder;
mod encoder;
mod range_coding;

pub(crate) use decoder::{decode_symbol, init_range_dec};
pub(crate) use encoder::{encode_symbol, flush_range_enc};

use super::PPMD_NUM_INDEXES;
use crate::RestoreMethod;

const MAX_FREQ: u8 = 124;
const UNIT_SIZE: isize = 12;
const K_TOP_VALUE: u32 = 1 << 24;
const EMPTY_NODE: u16 = 0;

static K_EXP_ESCAPE: [u8; 16] = [25, 14, 9, 7, 5, 5, 4, 4, 4, 3, 3, 3, 2, 2, 2, 2];

static K_INIT_BIN_ESC: [u16; 8] = [
    0x3CDD, 0x1F3F, 0x59BF, 0x48F3, 0x64A1, 0x5ABC, 0x6632, 0x6051,
];

#[derive(Copy, Clone)]
#[repr(C)]
pub struct IByteIn {
    pub read: Option<unsafe extern "C" fn(IByteInPtr) -> u8>,
}

pub type IByteInPtr = *const IByteIn;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct IByteOut {
    pub write: Option<unsafe extern "C" fn(IByteOutPtr, u8) -> ()>,
}

pub type IByteOutPtr = *const IByteOut;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ISzAlloc {
    pub alloc: Option<fn(ISzAllocPtr, usize) -> *mut u8>,
    pub free: Option<fn(ISzAllocPtr, *mut u8) -> ()>,
}

pub type ISzAllocPtr = *const ISzAlloc;

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct See {
    pub summ: u16,
    pub shift: u8,
    pub count: u8,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct State {
    pub symbol: u8,
    pub freq: u8,
    pub successor_0: u16,
    pub successor_1: u16,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct State2 {
    pub symbol: u8,
    pub freq: u8,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct State4 {
    pub successor_0: u16,
    pub successor_1: u16,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Context {
    pub num_stats: u8,
    pub flags: u8,
    pub union2: Union2,
    pub union4: Union4,
    pub suffix: u32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union Union4 {
    pub stats: u32,
    pub state4: State4,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union Union2 {
    pub summ_freq: u16,
    pub state2: State2,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union StreamUnion {
    pub input: IByteInPtr,
    pub output: IByteOutPtr,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Node {
    pub stamp: u32,
    pub next: u32,
    pub nu: u32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Ppmd8 {
    pub min_context: *mut Context,
    pub max_context: *mut Context,
    pub found_state: *mut State,
    pub order_fall: u32,
    pub init_esc: u32,
    pub prev_success: u32,
    pub max_order: u32,
    pub restore_method: RestoreMethod,
    pub run_length: i32,
    pub init_rl: i32,
    pub size: u32,
    pub glue_count: u32,
    pub align_offset: u32,
    pub base: *mut u8,
    pub lo_unit: *mut u8,
    pub hi_unit: *mut u8,
    pub text: *mut u8,
    pub units_start: *mut u8,
    pub range: u32,
    pub code: u32,
    pub low: u32,
    pub stream: StreamUnion,
    pub index2units: [u8; 40],
    pub units2index: [u8; 128],
    pub free_list: [u32; 38],
    pub stamps: [u32; 38],
    pub ns2bs_index: [u8; 256],
    pub ns2index: [u8; 260],
    pub exp_escape: [u8; 16],
    pub dummy_see: See,
    pub see: [[See; 32]; 24],
    pub bin_summ: [[u16; 64]; 25],
}

pub unsafe fn construct(p: *mut Ppmd8) {
    unsafe {
        let mut i: u32 = 0;
        let mut k: u32 = 0;
        let mut m: u32 = 0;
        (*p).base = 0 as *mut u8;
        i = 0 as i32 as u32;
        k = 0 as i32 as u32;
        while i < PPMD_NUM_INDEXES {
            let mut step: u32 = if i >= 12 as i32 as u32 {
                4 as i32 as u32
            } else {
                (i >> 2 as i32).wrapping_add(1 as i32 as u32)
            };
            loop {
                let fresh0 = k;
                k = k.wrapping_add(1);
                (*p).units2index[fresh0 as usize] = i as u8;
                step = step.wrapping_sub(1);
                if !(step != 0) {
                    break;
                }
            }
            (*p).index2units[i as usize] = k as u8;
            i = i.wrapping_add(1);
            i;
        }

        (*p).ns2bs_index[0] = (0 << 1) as u8;
        (*p).ns2bs_index[1] = (1 << 1) as u8;
        (*p).ns2bs_index[2..11].fill((2 << 1) as u8);
        (*p).ns2bs_index[11..256].fill((3 << 1) as u8);

        i = 0 as i32 as u32;
        while i < 5 as i32 as u32 {
            (*p).ns2index[i as usize] = i as u8;
            i = i.wrapping_add(1);
            i;
        }
        m = i;
        k = 1 as i32 as u32;
        while i < 260 as i32 as u32 {
            (*p).ns2index[i as usize] = m as u8;
            k = k.wrapping_sub(1);
            if k == 0 as i32 as u32 {
                m = m.wrapping_add(1);
                k = m.wrapping_sub(4 as i32 as u32);
            }
            i = i.wrapping_add(1);
            i;
        }
        (*p).exp_escape.copy_from_slice(&K_EXP_ESCAPE);
    }
}

pub unsafe fn free(p: *mut Ppmd8, alloc: ISzAllocPtr) {
    unsafe {
        ((*alloc).free).expect("non-null function pointer")(alloc, (*p).base as *mut u8);
        (*p).size = 0 as i32 as u32;
        (*p).base = 0 as *mut u8;
    }
}

pub unsafe fn alloc(p: *mut Ppmd8, size: u32, alloc: ISzAllocPtr) -> i32 {
    unsafe {
        if ((*p).base).is_null() || (*p).size != size {
            free(p, alloc);
            (*p).align_offset = (4 as i32 as u32).wrapping_sub(size) & 3 as i32 as u32;
            (*p).base = ((*alloc).alloc).expect("non-null function pointer")(
                alloc,
                ((*p).align_offset).wrapping_add(size) as usize,
            ) as *mut u8;
            if ((*p).base).is_null() {
                return 0 as i32;
            }
            (*p).size = size;
        }
        return 1 as i32;
    }
}

unsafe fn insert_node(p: *mut Ppmd8, node: *mut u8, index: u32) {
    unsafe {
        (*(node as *mut Node)).stamp = 0xFFFFFFFF as u32;
        (*(node as *mut Node)).next = (*p).free_list[index as usize];
        (*(node as *mut Node)).nu = (*p).index2units[index as usize] as u32;
        (*p).free_list[index as usize] = (node as *mut u8).offset_from((*p).base) as isize as u32;
        (*p).stamps[index as usize] = ((*p).stamps[index as usize]).wrapping_add(1);
        (*p).stamps[index as usize];
    }
}

unsafe fn remove_node(p: *mut Ppmd8, index: u32) -> *mut u8 {
    unsafe {
        let node: *mut Node =
            ((*p).base).offset((*p).free_list[index as usize] as isize) as *mut u8 as *mut Node;
        (*p).free_list[index as usize] = (*node).next;
        (*p).stamps[index as usize] = ((*p).stamps[index as usize]).wrapping_sub(1);
        (*p).stamps[index as usize];
        node as *mut u8
    }
}

unsafe fn split_block(p: *mut Ppmd8, mut ptr: *mut u8, oldindex: u32, newindex: u32) {
    unsafe {
        let mut i: u32 = 0;
        let nu: u32 = ((*p).index2units[oldindex as usize] as u32)
            .wrapping_sub((*p).index2units[newindex as usize] as u32);
        ptr = (ptr as *mut u8)
            .offset(((*p).index2units[newindex as usize] as u32 * 12 as i32 as u32) as isize)
            as *mut u8;
        i = (*p).units2index[(nu as usize).wrapping_sub(1 as i32 as usize) as usize] as u32;
        if (*p).index2units[i as usize] as u32 != nu {
            i = i.wrapping_sub(1);
            let k: u32 = (*p).index2units[i as usize] as u32;
            insert_node(
                p,
                (ptr as *mut u8).offset((k * 12 as i32 as u32) as isize) as *mut u8,
                nu.wrapping_sub(k).wrapping_sub(1 as i32 as u32),
            );
        }
        insert_node(p, ptr, i);
    }
}
unsafe fn glue_free_blocks(p: *mut Ppmd8) {
    unsafe {
        let mut n: u32 = 0;
        (*p).glue_count = ((1 as i32) << 13 as i32) as u32;
        (*p).stamps = [0; 38];
        if (*p).lo_unit != (*p).hi_unit {
            (*((*p).lo_unit as *mut u8 as *mut Node)).stamp = 0 as i32 as u32;
        }
        let mut prev: *mut u32 = &mut n;
        let mut i: u32 = 0;
        i = 0 as i32 as u32;
        while i < PPMD_NUM_INDEXES {
            let mut next: u32 = (*p).free_list[i as usize];
            (*p).free_list[i as usize] = 0 as i32 as u32;
            while next != 0 as i32 as u32 {
                let node: *mut Node = ((*p).base).offset(next as isize) as *mut u8 as *mut Node;
                let mut nu: u32 = (*node).nu;
                *prev = next;
                next = (*node).next;
                if nu != 0 as i32 as u32 {
                    let mut node2: *mut Node = 0 as *mut Node;
                    prev = &mut (*node).next;
                    loop {
                        node2 = node.offset(nu as isize);
                        if !((*node2).stamp == 0xFFFFFFFF as u32) {
                            break;
                        }
                        nu = nu.wrapping_add((*node2).nu);
                        (*node2).nu = 0 as i32 as u32;
                        (*node).nu = nu;
                    }
                }
            }
            i = i.wrapping_add(1);
            i;
        }
        *prev = 0 as i32 as u32;
        while n != 0 as i32 as u32 {
            let mut node_0: *mut Node = ((*p).base).offset(n as isize) as *mut u8 as *mut Node;
            let mut nu_0: u32 = (*node_0).nu;
            let mut i_0: u32 = 0;
            n = (*node_0).next;
            if nu_0 == 0 as i32 as u32 {
                continue;
            }
            while nu_0 > 128 as i32 as u32 {
                insert_node(p, node_0 as *mut u8, PPMD_NUM_INDEXES - 1);
                nu_0 = nu_0.wrapping_sub(128 as i32 as u32);
                node_0 = node_0.offset(128 as i32 as isize);
            }
            i_0 = (*p).units2index[(nu_0 as usize).wrapping_sub(1 as i32 as usize) as usize] as u32;
            if (*p).index2units[i_0 as usize] as u32 != nu_0 {
                i_0 = i_0.wrapping_sub(1);
                let k: u32 = (*p).index2units[i_0 as usize] as u32;
                insert_node(
                    p,
                    node_0.offset(k as isize) as *mut u8,
                    nu_0.wrapping_sub(k).wrapping_sub(1 as i32 as u32),
                );
            }
            insert_node(p, node_0 as *mut u8, i_0);
        }
    }
}

#[inline(never)]
unsafe fn alloc_units_rare(p: *mut Ppmd8, index: u32) -> *mut u8 {
    unsafe {
        let mut i: u32 = 0;
        if (*p).glue_count == 0 as i32 as u32 {
            glue_free_blocks(p);
            if (*p).free_list[index as usize] != 0 as i32 as u32 {
                return remove_node(p, index);
            }
        }
        i = index;
        loop {
            i = i.wrapping_add(1);
            if i == PPMD_NUM_INDEXES {
                let numBytes: u32 = (*p).index2units[index as usize] as u32 * 12 as i32 as u32;
                let us: *mut u8 = (*p).units_start;
                (*p).glue_count = ((*p).glue_count).wrapping_sub(1);
                (*p).glue_count;
                return (if us.offset_from((*p).text) as isize as u32 > numBytes {
                    (*p).units_start = us.offset(-(numBytes as isize));
                    (*p).units_start
                } else {
                    0 as *mut u8
                }) as *mut u8;
            }
            if !((*p).free_list[i as usize] == 0 as i32 as u32) {
                break;
            }
        }
        let block: *mut u8 = remove_node(p, i);
        split_block(p, block, i, index);
        block
    }
}

unsafe fn alloc_units(p: *mut Ppmd8, index: u32) -> *mut u8 {
    unsafe {
        if (*p).free_list[index as usize] != 0 as i32 as u32 {
            return remove_node(p, index);
        }
        let numBytes: u32 = (*p).index2units[index as usize] as u32 * 12 as i32 as u32;
        let lo: *mut u8 = (*p).lo_unit;
        if ((*p).hi_unit).offset_from(lo) as isize as u32 >= numBytes {
            (*p).lo_unit = lo.offset(numBytes as isize);
            return lo as *mut u8;
        }
        alloc_units_rare(p, index)
    }
}

unsafe fn shrink_units(p: *mut Ppmd8, oldPtr: *mut u8, oldNU: u32, newNU: u32) -> *mut u8 {
    unsafe {
        let i0: u32 =
            (*p).units2index[(oldNU as usize).wrapping_sub(1 as i32 as usize) as usize] as u32;
        let i1: u32 =
            (*p).units2index[(newNU as usize).wrapping_sub(1 as i32 as usize) as usize] as u32;
        if i0 == i1 {
            return oldPtr;
        }
        if (*p).free_list[i1 as usize] != 0 as i32 as u32 {
            let ptr: *mut u8 = remove_node(p, i1);
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
            insert_node(p, oldPtr, i0);
            return ptr;
        }
        split_block(p, oldPtr, i0, i1);
        oldPtr
    }
}

unsafe fn free_units(p: *mut Ppmd8, ptr: *mut u8, nu: u32) {
    unsafe {
        insert_node(
            p,
            ptr,
            (*p).units2index[(nu as usize).wrapping_sub(1 as i32 as usize) as usize] as u32,
        );
    }
}

unsafe fn special_free_unit(p: *mut Ppmd8, ptr: *mut u8) {
    unsafe {
        if ptr != (*p).units_start {
            insert_node(p, ptr, 0 as i32 as u32);
        } else {
            (*p).units_start = ((*p).units_start).offset(12 as i32 as isize);
        };
    }
}

unsafe fn expand_text_area(p: *mut Ppmd8) {
    unsafe {
        let mut count: [u32; 38] = [0; 38];
        let mut i: u32 = 0;
        count = [0; 38];
        if (*p).lo_unit != (*p).hi_unit {
            (*((*p).lo_unit as *mut u8 as *mut Node)).stamp = 0 as i32 as u32;
        }
        let mut node: *mut Node = (*p).units_start as *mut u8 as *mut Node;
        while (*node).stamp == 0xFFFFFFFF as u32 {
            let nu: u32 = (*node).nu;
            (*node).stamp = 0 as i32 as u32;
            count[(*p).units2index[(nu as usize).wrapping_sub(1 as i32 as usize) as usize]
                as usize] = (count[(*p).units2index
                [(nu as usize).wrapping_sub(1 as i32 as usize) as usize]
                as usize])
                .wrapping_add(1);
            count
                [(*p).units2index[(nu as usize).wrapping_sub(1 as i32 as usize) as usize] as usize];
            node = node.offset(nu as isize);
        }
        (*p).units_start = node as *mut u8;
        i = 0 as i32 as u32;
        while i < PPMD_NUM_INDEXES {
            let mut cnt: u32 = count[i as usize];
            if !(cnt == 0 as i32 as u32) {
                let mut prev: *mut u32 =
                    &mut *((*p).free_list).as_mut_ptr().offset(i as isize) as *mut u32 as *mut u32;
                let mut n: u32 = *prev;
                (*p).stamps[i as usize] = ((*p).stamps[i as usize]).wrapping_sub(cnt);
                loop {
                    let node_0: *mut Node = ((*p).base).offset(n as isize) as *mut u8 as *mut Node;
                    n = (*node_0).next;
                    if (*node_0).stamp != 0 as i32 as u32 {
                        prev = &mut (*node_0).next;
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
}

unsafe fn set_successor(p: *mut State, v: u32) {
    unsafe {
        (*p).successor_0 = v as u16;
        (*p).successor_1 = (v >> 16 as i32) as u16;
    }
}

#[inline(never)]
unsafe fn restart_model(p: *mut Ppmd8) {
    unsafe {
        let mut i: u32 = 0;
        let mut k: u32 = 0;
        let mut m: u32 = 0;
        (*p).free_list = [0; 38];
        (*p).stamps = [0; 38];
        (*p).text = ((*p).base)
            .offset((*p).align_offset as isize)
            .offset(0 as i32 as isize);
        (*p).hi_unit = ((*p).text).offset((*p).size as isize);
        (*p).units_start = ((*p).hi_unit).offset(
            -(((*p).size / 8 as i32 as u32 / 12 as i32 as u32 * 7 as i32 as u32 * 12 as i32 as u32)
                as isize),
        );
        (*p).lo_unit = (*p).units_start;
        (*p).glue_count = 0 as i32 as u32;
        (*p).order_fall = (*p).max_order;
        (*p).init_rl = -((if (*p).max_order < 12 as i32 as u32 {
            (*p).max_order
        } else {
            12 as i32 as u32
        }) as i32)
            - 1 as i32;
        (*p).run_length = (*p).init_rl;
        (*p).prev_success = 0 as i32 as u32;
        (*p).hi_unit = ((*p).hi_unit).offset(-(12 as i32 as isize));
        let mc: *mut Context = (*p).hi_unit as *mut u8 as *mut Context;
        let mut s: *mut State = (*p).lo_unit as *mut State;
        (*p).lo_unit =
            ((*p).lo_unit).offset(((256 as i32 / 2 as i32) as u32 * 12 as i32 as u32) as isize);
        (*p).min_context = mc;
        (*p).max_context = (*p).min_context;
        (*p).found_state = s;
        (*mc).flags = 0 as i32 as u8;
        (*mc).num_stats = (256 as i32 - 1 as i32) as u8;
        (*mc).union2.summ_freq = (256 as i32 + 1 as i32) as u16;
        (*mc).union4.stats = (s as *mut u8).offset_from((*p).base) as isize as u32;
        (*mc).suffix = 0 as i32 as u32;
        i = 0 as i32 as u32;
        while i < 256 as i32 as u32 {
            (*s).symbol = i as u8;
            (*s).freq = 1 as i32 as u8;
            set_successor(s, 0 as i32 as u32);
            i = i.wrapping_add(1);
            i;
            s = s.offset(1);
            s;
        }
        m = 0 as i32 as u32;
        i = m;
        while m < 25 as i32 as u32 {
            while (*p).ns2index[i as usize] as u32 == m {
                i = i.wrapping_add(1);
                i;
            }
            k = 0 as i32 as u32;
            while k < 8 as i32 as u32 {
                let mut r: u32 = 0;
                let dest: *mut u16 = ((*p).bin_summ[m as usize]).as_mut_ptr().offset(k as isize);
                let val: u16 = (((1 as i32) << 7 as i32 + 7 as i32) as u32).wrapping_sub(
                    (K_INIT_BIN_ESC[k as usize] as u32)
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
            let mut s_0: *mut See = 0 as *mut See;
            while (*p).ns2index[(i as usize).wrapping_add(3 as i32 as usize) as usize] as u32
                == m.wrapping_add(3 as i32 as u32)
            {
                i = i.wrapping_add(1);
                i;
            }
            s_0 = ((*p).see[m as usize]).as_mut_ptr();
            summ = (2 as i32 as u32)
                .wrapping_mul(i)
                .wrapping_add(5 as i32 as u32)
                << 7 as i32 - 4 as i32;
            k = 0 as i32 as u32;
            while k < 32 as i32 as u32 {
                (*s_0).summ = summ as u16;
                (*s_0).shift = (7 as i32 - 4 as i32) as u8;
                (*s_0).count = 7 as i32 as u8;
                k = k.wrapping_add(1);
                k;
                s_0 = s_0.offset(1);
                s_0;
            }
            m = m.wrapping_add(1);
            m;
        }
        (*p).dummy_see.summ = 0 as i32 as u16;
        (*p).dummy_see.shift = 7 as i32 as u8;
        (*p).dummy_see.count = 64 as i32 as u8;
    }
}

pub unsafe fn init(p: *mut Ppmd8, max_order: u32, restore_method: RestoreMethod) {
    unsafe {
        (*p).max_order = max_order;
        (*p).restore_method = restore_method;
        restart_model(p);
    }
}

unsafe fn refresh(p: *mut Ppmd8, ctx: *mut Context, oldNU: u32, mut scale: u32) {
    unsafe {
        let mut i: u32 = (*ctx).num_stats as u32;
        let mut escFreq: u32 = 0;
        let mut sumFreq: u32 = 0;
        let mut flags: u32 = 0;
        let mut s: *mut State = shrink_units(
            p,
            ((*p).base).offset((*ctx).union4.stats as isize) as *mut u8 as *mut State as *mut u8,
            oldNU,
            i.wrapping_add(2 as i32 as u32) >> 1 as i32,
        ) as *mut State;
        (*ctx).union4.stats = (s as *mut u8).offset_from((*p).base) as isize as u32;
        scale |= ((*ctx).union2.summ_freq as u32 >= (1 as i32 as u32) << 15 as i32) as i32 as u32;
        flags = ((*s).symbol as u32).wrapping_add(0xC0 as i32 as u32);
        let mut freq: u32 = (*s).freq as u32;
        escFreq = ((*ctx).union2.summ_freq as u32).wrapping_sub(freq);
        freq = freq.wrapping_add(scale) >> scale;
        sumFreq = freq;
        (*s).freq = freq as u8;
        loop {
            s = s.offset(1);
            let mut freq_0: u32 = (*s).freq as u32;
            escFreq = escFreq.wrapping_sub(freq_0);
            freq_0 = freq_0.wrapping_add(scale) >> scale;
            sumFreq = sumFreq.wrapping_add(freq_0);
            (*s).freq = freq_0 as u8;
            flags |= ((*s).symbol as u32).wrapping_add(0xC0 as i32 as u32);
            i = i.wrapping_sub(1);
            if !(i != 0) {
                break;
            }
        }
        (*ctx).union2.summ_freq = sumFreq.wrapping_add(escFreq.wrapping_add(scale) >> scale) as u16;
        (*ctx).flags = ((*ctx).flags as u32
            & (((1 as i32) << 4 as i32) as u32)
                .wrapping_add((((1 as i32) << 2 as i32) as u32).wrapping_mul(scale)))
        .wrapping_add(flags >> 8 as i32 - 3 as i32 & ((1 as i32) << 3 as i32) as u32)
            as u8;
    }
}

unsafe fn swap_states(t1: *mut State, t2: *mut State) {
    unsafe {
        let tmp: State = *t1;
        *t1 = *t2;
        *t2 = tmp;
    }
}

unsafe fn cut_off(p: *mut Ppmd8, ctx: *mut Context, order: u32) -> u32 {
    unsafe {
        let mut ns: i32 = (*ctx).num_stats as i32;
        let mut nu: u32 = 0;
        let mut stats: *mut State = 0 as *mut State;
        if ns == 0 as i32 {
            let s: *mut State = &mut (*ctx).union2 as *mut Union2 as *mut State;
            let mut successor: u32 =
                (*s).successor_0 as u32 | ((*s).successor_1 as u32) << 16 as i32;
            if ((*p).base).offset(successor as isize) as *mut u8 as *mut u8 >= (*p).units_start {
                if order < (*p).max_order {
                    successor = cut_off(
                        p,
                        ((*p).base).offset(successor as isize) as *mut u8 as *mut Context,
                        order.wrapping_add(1 as i32 as u32),
                    );
                } else {
                    successor = 0 as i32 as u32;
                }
                set_successor(s, successor);
                if successor != 0 || order <= 9 as i32 as u32 {
                    return (ctx as *mut u8).offset_from((*p).base) as isize as u32;
                }
            }
            special_free_unit(p, ctx as *mut u8);
            return 0 as i32 as u32;
        }
        nu = (ns as u32).wrapping_add(2 as i32 as u32) >> 1 as i32;
        let index: u32 =
            (*p).units2index[(nu as usize).wrapping_sub(1 as i32 as usize) as usize] as u32;
        stats = ((*p).base).offset((*ctx).union4.stats as isize) as *mut u8 as *mut State;
        if (stats as *mut u8).offset_from((*p).units_start) as isize as u32
            <= ((1 as i32) << 14 as i32) as u32
            && (*ctx).union4.stats <= (*p).free_list[index as usize]
        {
            let ptr: *mut u8 = remove_node(p, index);
            (*ctx).union4.stats = (ptr as *mut u8).offset_from((*p).base) as isize as u32;
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
            if stats as *mut u8 != (*p).units_start {
                insert_node(p, stats as *mut u8, index);
            } else {
                (*p).units_start = ((*p).units_start)
                    .offset(((*p).index2units[index as usize] as u32 * 12 as i32 as u32) as isize);
            }
            stats = ptr as *mut State;
        }
        let mut s_0: *mut State = stats.offset(ns as u32 as isize);
        loop {
            let successor_0: u32 =
                (*s_0).successor_0 as u32 | ((*s_0).successor_1 as u32) << 16 as i32;
            if (((*p).base).offset(successor_0 as isize) as *mut u8 as *mut u8) < (*p).units_start {
                let fresh1 = ns;
                ns = ns - 1;
                let s2: *mut State = stats.offset(fresh1 as u32 as isize);
                if order != 0 {
                    if s_0 != s2 {
                        *s_0 = *s2;
                    }
                } else {
                    swap_states(s_0, s2);
                    set_successor(s2, 0 as i32 as u32);
                }
            } else if order < (*p).max_order {
                set_successor(
                    s_0,
                    cut_off(
                        p,
                        ((*p).base).offset(successor_0 as isize) as *mut u8 as *mut Context,
                        order.wrapping_add(1 as i32 as u32),
                    ),
                );
            } else {
                set_successor(s_0, 0 as i32 as u32);
            }
            s_0 = s_0.offset(-1);
            if !(s_0 >= stats) {
                break;
            }
        }
        if ns != (*ctx).num_stats as i32 && order != 0 {
            if ns < 0 as i32 {
                free_units(p, stats as *mut u8, nu);
                special_free_unit(p, ctx as *mut u8);
                return 0 as i32 as u32;
            }
            (*ctx).num_stats = ns as u8;
            if ns == 0 as i32 {
                let sym: u8 = (*stats).symbol;
                (*ctx).flags = (((*ctx).flags as i32 & (1 as i32) << 4 as i32) as u32).wrapping_add(
                    (sym as u32).wrapping_add(0xC0 as i32 as u32) >> 8 as i32 - 3 as i32
                        & ((1 as i32) << 3 as i32) as u32,
                ) as u8;
                (*ctx).union2.state2.symbol = sym;
                (*ctx).union2.state2.freq =
                    (((*stats).freq as u32).wrapping_add(11 as i32 as u32) >> 3 as i32) as u8;
                (*ctx).union4.state4.successor_0 = (*stats).successor_0;
                (*ctx).union4.state4.successor_1 = (*stats).successor_1;
                free_units(p, stats as *mut u8, nu);
            } else {
                refresh(
                    p,
                    ctx,
                    nu,
                    ((*ctx).union2.summ_freq as u32 > (16 as i32 as u32).wrapping_mul(ns as u32))
                        as i32 as u32,
                );
            }
        }
        return (ctx as *mut u8).offset_from((*p).base) as isize as u32;
    }
}

unsafe fn get_used_memory(p: *const Ppmd8) -> u32 {
    unsafe {
        let mut v: u32 = 0 as i32 as u32;
        let mut i: u32 = 0;
        i = 0 as i32 as u32;
        while i < PPMD_NUM_INDEXES {
            v = (v as u32).wrapping_add(
                ((*p).stamps[i as usize]).wrapping_mul((*p).index2units[i as usize] as u32),
            ) as u32 as u32;
            i = i.wrapping_add(1);
            i;
        }
        return ((*p).size)
            .wrapping_sub(((*p).hi_unit).offset_from((*p).lo_unit) as isize as u32)
            .wrapping_sub(((*p).units_start).offset_from((*p).text) as isize as u32)
            .wrapping_sub(v * 12 as i32 as u32);
    }
}

unsafe fn restore_model(p: *mut Ppmd8, ctxError: *mut Context) {
    unsafe {
        let mut c: *mut Context = 0 as *mut Context;
        let mut s: *mut State = 0 as *mut State;
        (*p).text = ((*p).base)
            .offset((*p).align_offset as isize)
            .offset(0 as i32 as isize);
        c = (*p).max_context;
        while c != ctxError {
            (*c).num_stats = ((*c).num_stats).wrapping_sub(1);
            if (*c).num_stats as i32 == 0 as i32 {
                s = ((*p).base).offset((*c).union4.stats as isize) as *mut u8 as *mut State;
                (*c).flags = (((*c).flags as i32 & (1 as i32) << 4 as i32) as u32).wrapping_add(
                    ((*s).symbol as u32).wrapping_add(0xC0 as i32 as u32) >> 8 as i32 - 3 as i32
                        & ((1 as i32) << 3 as i32) as u32,
                ) as u8;
                (*c).union2.state2.symbol = (*s).symbol;
                (*c).union2.state2.freq =
                    (((*s).freq as u32).wrapping_add(11 as i32 as u32) >> 3 as i32) as u8;
                (*c).union4.state4.successor_0 = (*s).successor_0;
                (*c).union4.state4.successor_1 = (*s).successor_1;
                special_free_unit(p, s as *mut u8);
            } else {
                refresh(
                    p,
                    c,
                    ((*c).num_stats as u32).wrapping_add(3 as i32 as u32) >> 1 as i32,
                    0 as i32 as u32,
                );
            }
            c = ((*p).base).offset((*c).suffix as isize) as *mut u8 as *mut Context;
        }
        while c != (*p).min_context {
            if (*c).num_stats as i32 == 0 as i32 {
                (*c).union2.state2.freq = (((*c).union2.state2.freq as u32)
                    .wrapping_add(1 as i32 as u32)
                    >> 1 as i32) as u8;
            } else {
                (*c).union2.summ_freq = ((*c).union2.summ_freq as i32 + 4 as i32) as u16;
                if (*c).union2.summ_freq as i32 > 128 as i32 + 4 as i32 * (*c).num_stats as i32 {
                    refresh(
                        p,
                        c,
                        ((*c).num_stats as u32).wrapping_add(2 as i32 as u32) >> 1 as i32,
                        1 as i32 as u32,
                    );
                }
            }
            c = ((*p).base).offset((*c).suffix as isize) as *mut u8 as *mut Context;
        }
        if (*p).restore_method == RestoreMethod::Restart
            || get_used_memory(p) < (*p).size >> 1 as i32
        {
            restart_model(p);
        } else {
            while (*(*p).max_context).suffix != 0 {
                (*p).max_context = ((*p).base).offset((*(*p).max_context).suffix as isize)
                    as *mut u8 as *mut Context;
            }
            loop {
                cut_off(p, (*p).max_context, 0 as i32 as u32);
                expand_text_area(p);
                if !(get_used_memory(p) > 3 as i32 as u32 * ((*p).size >> 2 as i32)) {
                    break;
                }
            }
            (*p).glue_count = 0 as i32 as u32;
            (*p).order_fall = (*p).max_order;
        }
        (*p).min_context = (*p).max_context;
    }
}

#[inline(never)]
unsafe fn create_successors(
    p: *mut Ppmd8,
    skip: i32,
    mut s1: *mut State,
    mut c: *mut Context,
) -> *mut Context {
    unsafe {
        let mut upBranch: u32 = (*(*p).found_state).successor_0 as u32
            | ((*(*p).found_state).successor_1 as u32) << 16 as i32;
        let mut newSym: u8 = 0;
        let mut newFreq: u8 = 0;
        let mut flags: u8 = 0;
        let mut numPs: u32 = 0 as i32 as u32;
        let mut ps: [*mut State; 17] = [0 as *mut State; 17];
        if skip == 0 {
            let fresh2 = numPs;
            numPs = numPs.wrapping_add(1);
            ps[fresh2 as usize] = (*p).found_state;
        }
        while (*c).suffix != 0 {
            let mut successor: u32 = 0;
            let mut s: *mut State = 0 as *mut State;
            c = ((*p).base).offset((*c).suffix as isize) as *mut u8 as *mut Context;
            if !s1.is_null() {
                s = s1;
                s1 = 0 as *mut State;
            } else if (*c).num_stats as i32 != 0 as i32 {
                let sym: u8 = (*(*p).found_state).symbol;
                s = ((*p).base).offset((*c).union4.stats as isize) as *mut u8 as *mut State;
                while (*s).symbol as i32 != sym as i32 {
                    s = s.offset(1);
                    s;
                }
                if ((*s).freq) < MAX_FREQ - 9 {
                    (*s).freq = ((*s).freq).wrapping_add(1);
                    (*s).freq;
                    (*c).union2.summ_freq = ((*c).union2.summ_freq).wrapping_add(1);
                    (*c).union2.summ_freq;
                }
            } else {
                s = &mut (*c).union2 as *mut Union2 as *mut State;
                (*s).freq = ((*s).freq as i32
                    + (((*(((*p).base).offset((*c).suffix as isize) as *mut u8 as *mut Context))
                        .num_stats
                        == 0) as i32
                        & (((*s).freq as i32) < 24 as i32) as i32))
                    as u8;
            }
            successor = (*s).successor_0 as u32 | ((*s).successor_1 as u32) << 16 as i32;
            if successor != upBranch {
                c = ((*p).base).offset(successor as isize) as *mut u8 as *mut Context;
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
        newSym = *(((*p).base).offset(upBranch as isize) as *mut u8 as *const u8);
        upBranch = upBranch.wrapping_add(1);
        upBranch;
        flags = (((*(*p).found_state).symbol as u32).wrapping_add(0xC0 as i32 as u32)
            >> 8 as i32 - 4 as i32
            & ((1 as i32) << 4 as i32) as u32)
            .wrapping_add(
                (newSym as u32).wrapping_add(0xC0 as i32 as u32) >> 8 as i32 - 3 as i32
                    & ((1 as i32) << 3 as i32) as u32,
            ) as u8;
        if (*c).num_stats as i32 == 0 as i32 {
            newFreq = (*c).union2.state2.freq;
        } else {
            let mut cf: u32 = 0;
            let mut s0: u32 = 0;
            let mut s_0: *mut State = 0 as *mut State;
            s_0 = ((*p).base).offset((*c).union4.stats as isize) as *mut u8 as *mut State;
            while (*s_0).symbol as i32 != newSym as i32 {
                s_0 = s_0.offset(1);
                s_0;
            }
            cf = ((*s_0).freq as u32).wrapping_sub(1 as i32 as u32);
            s0 = ((*c).union2.summ_freq as u32)
                .wrapping_sub((*c).num_stats as u32)
                .wrapping_sub(cf);
            newFreq = (1 as i32 as u32).wrapping_add(if 2 as i32 as u32 * cf <= s0 {
                (5 as i32 as u32 * cf > s0) as i32 as u32
            } else {
                cf.wrapping_add(2 as i32 as u32 * s0)
                    .wrapping_sub(3 as i32 as u32)
                    / s0
            }) as u8;
        }
        loop {
            let mut c1: *mut Context = 0 as *mut Context;
            if (*p).hi_unit != (*p).lo_unit {
                (*p).hi_unit = ((*p).hi_unit).offset(-(12 as i32 as isize));
                c1 = (*p).hi_unit as *mut u8 as *mut Context;
            } else if (*p).free_list[0 as i32 as usize] != 0 as i32 as u32 {
                c1 = remove_node(p, 0 as i32 as u32) as *mut Context;
            } else {
                c1 = alloc_units_rare(p, 0 as i32 as u32) as *mut Context;
                if c1.is_null() {
                    return 0 as *mut Context;
                }
            }
            (*c1).flags = flags;
            (*c1).num_stats = 0 as i32 as u8;
            (*c1).union2.state2.symbol = newSym;
            (*c1).union2.state2.freq = newFreq;
            set_successor(&mut (*c1).union2 as *mut Union2 as *mut State, upBranch);
            (*c1).suffix = (c as *mut u8).offset_from((*p).base) as isize as u32;
            numPs = numPs.wrapping_sub(1);
            set_successor(
                ps[numPs as usize],
                (c1 as *mut u8).offset_from((*p).base) as isize as u32,
            );
            c = c1;
            if !(numPs != 0 as i32 as u32) {
                break;
            }
        }
        return c;
    }
}

unsafe fn reduce_order(p: *mut Ppmd8, mut s1: *mut State, mut c: *mut Context) -> *mut Context {
    unsafe {
        let mut s: *mut State = 0 as *mut State;
        let c1: *mut Context = c;
        let upBranch: u32 = ((*p).text).offset_from((*p).base) as isize as u32;
        set_successor((*p).found_state, upBranch);
        (*p).order_fall = ((*p).order_fall).wrapping_add(1);
        (*p).order_fall;
        loop {
            if !s1.is_null() {
                c = ((*p).base).offset((*c).suffix as isize) as *mut u8 as *mut Context;
                s = s1;
                s1 = 0 as *mut State;
            } else {
                if (*c).suffix == 0 {
                    return c;
                }
                c = ((*p).base).offset((*c).suffix as isize) as *mut u8 as *mut Context;
                if (*c).num_stats != 0 {
                    s = ((*p).base).offset((*c).union4.stats as isize) as *mut u8 as *mut State;
                    if (*s).symbol as i32 != (*(*p).found_state).symbol as i32 {
                        loop {
                            s = s.offset(1);
                            s;
                            if !((*s).symbol as i32 != (*(*p).found_state).symbol as i32) {
                                break;
                            }
                        }
                    }
                    if ((*s).freq) < MAX_FREQ - 9 {
                        (*s).freq = ((*s).freq as i32 + 2 as i32) as u8;
                        (*c).union2.summ_freq = ((*c).union2.summ_freq as i32 + 2 as i32) as u16;
                    }
                } else {
                    s = &mut (*c).union2 as *mut Union2 as *mut State;
                    (*s).freq = ((*s).freq as i32 + (((*s).freq as i32) < 32 as i32) as i32) as u8;
                }
            }
            if (*s).successor_0 as u32 | ((*s).successor_1 as u32) << 16 as i32 != 0 {
                break;
            }
            set_successor(s, upBranch);
            (*p).order_fall = ((*p).order_fall).wrapping_add(1);
            (*p).order_fall;
        }
        if (*s).successor_0 as u32 | ((*s).successor_1 as u32) << 16 as i32 <= upBranch {
            let mut successor: *mut Context = 0 as *mut Context;
            let s2: *mut State = (*p).found_state;
            (*p).found_state = s;
            successor = create_successors(p, 0 as i32, 0 as *mut State, c);
            if successor.is_null() {
                set_successor(s, 0 as i32 as u32);
            } else {
                set_successor(
                    s,
                    (successor as *mut u8).offset_from((*p).base) as isize as u32,
                );
            }
            (*p).found_state = s2;
        }
        let successor_0: u32 = (*s).successor_0 as u32 | ((*s).successor_1 as u32) << 16 as i32;
        if (*p).order_fall == 1 as i32 as u32 && c1 == (*p).max_context {
            set_successor((*p).found_state, successor_0);
            (*p).text = ((*p).text).offset(-1);
            (*p).text;
        }
        if successor_0 == 0 as i32 as u32 {
            return 0 as *mut Context;
        }
        return ((*p).base).offset(successor_0 as isize) as *mut u8 as *mut Context;
    }
}

#[inline(never)]
pub unsafe fn update_model(p: *mut Ppmd8) {
    unsafe {
        let mut maxSuccessor: u32 = 0;
        let mut minSuccessor: u32 = (*(*p).found_state).successor_0 as u32
            | ((*(*p).found_state).successor_1 as u32) << 16 as i32;
        let mut c: *mut Context = 0 as *mut Context;
        let mut s0: u32 = 0;
        let mut ns: u32 = 0;
        let fFreq: u32 = (*(*p).found_state).freq as u32;
        let mut flag: u8 = 0;
        let fSymbol: u8 = (*(*p).found_state).symbol;
        let mut s: *mut State = 0 as *mut State;
        if ((*(*p).found_state).freq) < MAX_FREQ / 4
            && (*(*p).min_context).suffix != 0 as i32 as u32
        {
            c = ((*p).base).offset((*(*p).min_context).suffix as isize) as *mut u8 as *mut Context;
            if (*c).num_stats as i32 == 0 as i32 {
                s = &mut (*c).union2 as *mut Union2 as *mut State;
                if ((*s).freq as i32) < 32 as i32 {
                    (*s).freq = ((*s).freq).wrapping_add(1);
                    (*s).freq;
                }
            } else {
                let sym: u8 = (*(*p).found_state).symbol;
                s = ((*p).base).offset((*c).union4.stats as isize) as *mut u8 as *mut State;
                if (*s).symbol as i32 != sym as i32 {
                    loop {
                        s = s.offset(1);
                        s;
                        if !((*s).symbol as i32 != sym as i32) {
                            break;
                        }
                    }
                    if (*s.offset(0 as i32 as isize)).freq as i32
                        >= (*s.offset(-(1 as i32) as isize)).freq as i32
                    {
                        swap_states(
                            &mut *s.offset(0 as i32 as isize),
                            &mut *s.offset(-(1 as i32) as isize),
                        );
                        s = s.offset(-1);
                        s;
                    }
                }
                if ((*s).freq) < MAX_FREQ - 9 {
                    (*s).freq = ((*s).freq as i32 + 2 as i32) as u8;
                    (*c).union2.summ_freq = ((*c).union2.summ_freq as i32 + 2 as i32) as u16;
                }
            }
        }
        c = (*p).max_context;
        if (*p).order_fall == 0 as i32 as u32 && minSuccessor != 0 {
            let cs: *mut Context = create_successors(p, 1 as i32, s, (*p).min_context);
            if cs.is_null() {
                set_successor((*p).found_state, 0 as i32 as u32);
                restore_model(p, c);
                return;
            }
            set_successor(
                (*p).found_state,
                (cs as *mut u8).offset_from((*p).base) as isize as u32,
            );
            (*p).max_context = cs;
            (*p).min_context = (*p).max_context;
            return;
        }
        let mut text: *mut u8 = (*p).text;
        let fresh4 = text;
        text = text.offset(1);
        *fresh4 = (*(*p).found_state).symbol;
        (*p).text = text;
        if text >= (*p).units_start {
            restore_model(p, c);
            return;
        }
        maxSuccessor = text.offset_from((*p).base) as isize as u32;
        if minSuccessor == 0 {
            let cs_0: *mut Context = reduce_order(p, s, (*p).min_context);
            if cs_0.is_null() {
                restore_model(p, c);
                return;
            }
            minSuccessor = (cs_0 as *mut u8).offset_from((*p).base) as isize as u32;
        } else if (((*p).base).offset(minSuccessor as isize) as *mut u8 as *mut u8)
            < (*p).units_start
        {
            let cs_1: *mut Context = create_successors(p, 0 as i32, s, (*p).min_context);
            if cs_1.is_null() {
                restore_model(p, c);
                return;
            }
            minSuccessor = (cs_1 as *mut u8).offset_from((*p).base) as isize as u32;
        }
        (*p).order_fall = ((*p).order_fall).wrapping_sub(1);
        if (*p).order_fall == 0 as i32 as u32 {
            maxSuccessor = minSuccessor;
            (*p).text =
                ((*p).text).offset(-(((*p).max_context != (*p).min_context) as i32 as isize));
        }
        flag = ((fSymbol as u32).wrapping_add(0xC0 as i32 as u32) >> 8 as i32 - 3 as i32
            & ((1 as i32) << 3 as i32) as u32) as u8;
        ns = (*(*p).min_context).num_stats as u32;
        s0 = ((*(*p).min_context).union2.summ_freq as u32)
            .wrapping_sub(ns)
            .wrapping_sub(fFreq);
        while c != (*p).min_context {
            let mut ns1: u32 = 0;
            let mut sum: u32 = 0;
            ns1 = (*c).num_stats as u32;
            if ns1 != 0 as i32 as u32 {
                if ns1 & 1 as i32 as u32 != 0 as i32 as u32 {
                    let oldNU: u32 = ns1.wrapping_add(1 as i32 as u32) >> 1 as i32;
                    let i: u32 = (*p).units2index
                        [(oldNU as usize).wrapping_sub(1 as i32 as usize) as usize]
                        as u32;
                    if i != (*p).units2index[(oldNU as usize)
                        .wrapping_add(1 as i32 as usize)
                        .wrapping_sub(1 as i32 as usize)
                        as usize] as u32
                    {
                        let ptr: *mut u8 = alloc_units(p, i.wrapping_add(1 as i32 as u32));
                        let mut oldPtr: *mut u8 = 0 as *mut u8;
                        if ptr.is_null() {
                            restore_model(p, c);
                            return;
                        }
                        oldPtr = ((*p).base).offset((*c).union4.stats as isize) as *mut u8
                            as *mut State as *mut u8;
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
                        insert_node(p, oldPtr, i);
                        (*c).union4.stats = (ptr as *mut u8).offset_from((*p).base) as isize as u32;
                    }
                }
                sum = (*c).union2.summ_freq as u32;
                sum = sum.wrapping_add(
                    ((3 as i32 as u32)
                        .wrapping_mul(ns1)
                        .wrapping_add(1 as i32 as u32)
                        < ns) as i32 as u32,
                );
            } else {
                let s_0: *mut State = alloc_units(p, 0 as i32 as u32) as *mut State;
                if s_0.is_null() {
                    restore_model(p, c);
                    return;
                }
                let mut freq: u32 = (*c).union2.state2.freq as u32;
                (*s_0).symbol = (*c).union2.state2.symbol;
                (*s_0).successor_0 = (*c).union4.state4.successor_0;
                (*s_0).successor_1 = (*c).union4.state4.successor_1;
                (*c).union4.stats = (s_0 as *mut u8).offset_from((*p).base) as isize as u32;
                if freq < (MAX_FREQ as i32 / 4 as i32 - 1 as i32) as u32 {
                    freq <<= 1 as i32;
                } else {
                    freq = (MAX_FREQ as i32 - 4 as i32) as u32;
                }
                (*s_0).freq = freq as u8;
                sum = freq
                    .wrapping_add((*p).init_esc)
                    .wrapping_add((ns > 2 as i32 as u32) as i32 as u32);
            }
            let s_1: *mut State = (((*p).base).offset((*c).union4.stats as isize) as *mut u8
                as *mut State)
                .offset(ns1 as isize)
                .offset(1 as i32 as isize);
            let mut cf: u32 = 2 as i32 as u32 * sum.wrapping_add(6 as i32 as u32) * fFreq;
            let sf: u32 = s0.wrapping_add(sum);
            (*s_1).symbol = fSymbol;
            (*c).num_stats = ns1.wrapping_add(1 as i32 as u32) as u8;
            set_successor(s_1, maxSuccessor);
            (*c).flags = ((*c).flags as i32 | flag as i32) as u8;
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
            (*c).union2.summ_freq = sum as u16;
            (*s_1).freq = cf as u8;
            c = ((*p).base).offset((*c).suffix as isize) as *mut u8 as *mut Context;
        }
        (*p).min_context = ((*p).base).offset(minSuccessor as isize) as *mut u8 as *mut Context;
        (*p).max_context = (*p).min_context;
    }
}

#[inline(never)]
unsafe fn rescale(p: *mut Ppmd8) {
    unsafe {
        let mut i: u32 = 0;
        let mut adder: u32 = 0;
        let mut sumFreq: u32 = 0;
        let mut escFreq: u32 = 0;
        let stats: *mut State =
            ((*p).base).offset((*(*p).min_context).union4.stats as isize) as *mut u8 as *mut State;
        let mut s: *mut State = (*p).found_state;
        if s != stats {
            let tmp: State = *s;
            loop {
                *s.offset(0 as i32 as isize) = *s.offset(-(1 as i32) as isize);
                s = s.offset(-1);
                if !(s != stats) {
                    break;
                }
            }
            *s = tmp;
        }
        sumFreq = (*s).freq as u32;
        escFreq = ((*(*p).min_context).union2.summ_freq as u32).wrapping_sub(sumFreq);
        adder = ((*p).order_fall != 0 as i32 as u32) as i32 as u32;
        sumFreq = sumFreq.wrapping_add(4 as i32 as u32).wrapping_add(adder) >> 1 as i32;
        i = (*(*p).min_context).num_stats as u32;
        (*s).freq = sumFreq as u8;
        loop {
            s = s.offset(1);
            let mut freq: u32 = (*s).freq as u32;
            escFreq = escFreq.wrapping_sub(freq);
            freq = freq.wrapping_add(adder) >> 1 as i32;
            sumFreq = sumFreq.wrapping_add(freq);
            (*s).freq = freq as u8;
            if freq > (*s.offset(-(1 as i32) as isize)).freq as u32 {
                let tmp_0: State = *s;
                let mut s1: *mut State = s;
                loop {
                    *s1.offset(0 as i32 as isize) = *s1.offset(-(1 as i32) as isize);
                    s1 = s1.offset(-1);
                    if !(s1 != stats && freq > (*s1.offset(-(1 as i32) as isize)).freq as u32) {
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
        if (*s).freq as i32 == 0 as i32 {
            let mut mc: *mut Context = 0 as *mut Context;
            let mut numStats: u32 = 0;
            let mut numStatsNew: u32 = 0;
            let mut n0: u32 = 0;
            let mut n1: u32 = 0;
            i = 0 as i32 as u32;
            loop {
                i = i.wrapping_add(1);
                i;
                s = s.offset(-1);
                if !((*s).freq as i32 == 0 as i32) {
                    break;
                }
            }
            escFreq = escFreq.wrapping_add(i);
            mc = (*p).min_context;
            numStats = (*mc).num_stats as u32;
            numStatsNew = numStats.wrapping_sub(i);
            (*mc).num_stats = numStatsNew as u8;
            n0 = numStats.wrapping_add(2 as i32 as u32) >> 1 as i32;
            if numStatsNew == 0 as i32 as u32 {
                let mut freq_0: u32 = (2 as i32 as u32)
                    .wrapping_mul((*stats).freq as u32)
                    .wrapping_add(escFreq)
                    .wrapping_sub(1 as i32 as u32)
                    .wrapping_div(escFreq);
                if freq_0 > (MAX_FREQ as i32 / 3 as i32) as u32 {
                    freq_0 = (MAX_FREQ as i32 / 3 as i32) as u32;
                }
                (*mc).flags = (((*mc).flags as i32 & (1 as i32) << 4 as i32) as u32).wrapping_add(
                    ((*stats).symbol as u32).wrapping_add(0xC0 as i32 as u32)
                        >> 8 as i32 - 3 as i32
                        & ((1 as i32) << 3 as i32) as u32,
                ) as u8;
                s = &mut (*mc).union2 as *mut Union2 as *mut State;
                *s = *stats;
                (*s).freq = freq_0 as u8;
                (*p).found_state = s;
                insert_node(
                    p,
                    stats as *mut u8,
                    (*p).units2index[(n0 as usize).wrapping_sub(1 as i32 as usize) as usize] as u32,
                );
                return;
            }
            n1 = numStatsNew.wrapping_add(2 as i32 as u32) >> 1 as i32;
            if n0 != n1 {
                (*mc).union4.stats = (shrink_units(p, stats as *mut u8, n0, n1) as *mut u8)
                    .offset_from((*p).base) as isize as u32;
            }
        }
        let mc_0: *mut Context = (*p).min_context;
        (*mc_0).union2.summ_freq = sumFreq
            .wrapping_add(escFreq)
            .wrapping_sub(escFreq >> 1 as i32) as u16;
        (*mc_0).flags = ((*mc_0).flags as i32 | (1 as i32) << 2 as i32) as u8;
        (*p).found_state =
            ((*p).base).offset((*mc_0).union4.stats as isize) as *mut u8 as *mut State;
    }
}

pub unsafe fn make_esc_freq(p: *mut Ppmd8, numMasked1: u32, escFreq: *mut u32) -> *mut See {
    unsafe {
        let mut see: *mut See = 0 as *mut See;
        let mc: *const Context = (*p).min_context;
        let numStats: u32 = (*mc).num_stats as u32;
        if numStats != 0xFF as i32 as u32 {
            see = ((*p).see[((*p).ns2index
                [(numStats as usize).wrapping_add(2 as i32 as usize) as usize]
                as u32 as usize)
                .wrapping_sub(3 as i32 as usize) as usize])
                .as_mut_ptr()
                .offset(
                    ((*mc).union2.summ_freq as u32
                        > (11 as i32 as u32).wrapping_mul(numStats.wrapping_add(1 as i32 as u32)))
                        as i32 as isize,
                )
                .offset(
                    (2 as i32 as u32).wrapping_mul(
                        ((2 as i32 as u32).wrapping_mul(numStats)
                            < ((*(((*p).base).offset((*mc).suffix as isize) as *mut u8
                                as *mut Context))
                                .num_stats as u32)
                                .wrapping_add(numMasked1)) as i32 as u32,
                    ) as isize,
                )
                .offset((*mc).flags as i32 as isize);
            let summ: u32 = (*see).summ as u32;
            let r: u32 = summ >> (*see).shift as i32;
            (*see).summ = summ.wrapping_sub(r) as u16;
            *escFreq = r.wrapping_add((r == 0 as i32 as u32) as i32 as u32);
        } else {
            see = &mut (*p).dummy_see;
            *escFreq = 1 as i32 as u32;
        }
        return see;
    }
}

unsafe fn next_context(p: *mut Ppmd8) {
    unsafe {
        let c: *mut Context = ((*p).base).offset(
            ((*(*p).found_state).successor_0 as u32
                | ((*(*p).found_state).successor_1 as u32) << 16 as i32) as isize,
        ) as *mut u8 as *mut Context;
        if (*p).order_fall == 0 as i32 as u32 && c as *const u8 >= (*p).units_start as *const u8 {
            (*p).min_context = c;
            (*p).max_context = (*p).min_context;
        } else {
            update_model(p);
        };
    }
}

pub unsafe fn update1(p: *mut Ppmd8) {
    unsafe {
        let mut s = (*p).found_state;
        let mut freq = (*s).freq as u32;
        freq += 4;
        (*(*p).min_context).union2.summ_freq += 4;
        (*s).freq = freq as u8;
        if freq > (*s.offset(-1)).freq as u32 {
            swap_states(s, &mut *s.offset(-1));
            s = s.offset(-1);
            (*p).found_state = s;
            if freq > MAX_FREQ as u32 {
                rescale(p);
            }
        }
        next_context(p);
    }
}

pub unsafe fn update1_0(p: *mut Ppmd8) {
    unsafe {
        let s = (*p).found_state;
        let mc = (*p).min_context;
        let mut freq = (*s).freq as u32;
        let summ_freq = (*mc).union2.summ_freq as u32;
        (*p).prev_success = (2 * freq >= summ_freq) as u32; // Ppmd8 (>=)
        (*p).run_length += (*p).prev_success as i32;
        (*mc).union2.summ_freq = (summ_freq + 4) as u16;
        freq += 4;
        (*s).freq = freq as u8;
        if freq > MAX_FREQ as u32 {
            rescale(p);
        }
        next_context(p);
    }
}

pub unsafe fn update2(p: *mut Ppmd8) {
    unsafe {
        let s = (*p).found_state;
        let mut freq = (*s).freq as u32;
        freq += 4;
        (*p).run_length = (*p).init_rl;
        (*(*p).min_context).union2.summ_freq += 4;
        (*s).freq = freq as u8;
        if freq > MAX_FREQ as u32 {
            rescale(p);
        }
        update_model(p);
    }
}
