mod decoder;
mod encoder;
mod range_coding;

use std::{
    alloc::{Layout, alloc, dealloc},
    ptr::{NonNull, write_bytes},
};

use super::PPMD_NUM_INDEXES;
use crate::{Error, RestoreMethod};

const MAX_FREQ: u8 = 124;
const UNIT_SIZE: isize = 12;
const K_TOP_VALUE: u32 = 1 << 24;
const K_BOT_VALUE: u32 = 1 << 15;
const EMPTY_NODE: u32 = u32::MAX;

static K_EXP_ESCAPE: [u8; 16] = [25, 14, 9, 7, 5, 5, 4, 4, 4, 3, 3, 3, 2, 2, 2, 2];

static K_INIT_BIN_ESC: [u16; 8] = [
    0x3CDD, 0x1F3F, 0x59BF, 0x48F3, 0x64A1, 0x5ABC, 0x6632, 0x6051,
];

#[derive(Copy, Clone)]
#[repr(C)]
pub struct IByteIn {
    pub read: fn(IByteInPtr) -> u8,
}

pub type IByteInPtr = *const IByteIn;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct IByteOut {
    pub write: fn(IByteOutPtr, u8) -> (),
}

pub type IByteOutPtr = *const IByteOut;

#[derive(Copy, Clone, Default)]
#[repr(C, packed)]
struct See {
    summ: u16,
    shift: u8,
    count: u8,
}

enum SeeSource {
    Dummy,
    Table(usize, usize),
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
struct State {
    symbol: u8,
    freq: u8,
    successor_0: u16,
    successor_1: u16,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
struct State2 {
    symbol: u8,
    freq: u8,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
struct State4 {
    successor_0: u16,
    successor_1: u16,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct Context {
    num_stats: u8,
    flags: u8,
    union2: Union2,
    union4: Union4,
    suffix: u32,
}

#[derive(Copy, Clone)]
#[repr(C)]
union Union4 {
    stats: u32,
    state4: State4,
}

#[derive(Copy, Clone)]
#[repr(C)]
union Union2 {
    summ_freq: u16,
    state2: State2,
}

#[derive(Copy, Clone)]
#[repr(C)]
union StreamUnion {
    input: IByteInPtr,
    output: IByteOutPtr,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct Node {
    stamp: u32,
    next: u32,
    nu: u32,
}

pub(crate) struct Encoder {}

pub(crate) struct Decoder {}

pub(crate) struct Ppmd8<RC> {
    min_context: *mut Context,
    max_context: *mut Context,
    found_state: *mut State,
    order_fall: u32,
    init_esc: u32,
    prev_success: u32,
    max_order: u32,
    restore_method: RestoreMethod,
    run_length: i32,
    init_rl: i32,
    size: u32,
    glue_count: u32,
    align_offset: u32,
    base: *mut u8,
    lo_unit: *mut u8,
    hi_unit: *mut u8,
    text: *mut u8,
    units_start: *mut u8,
    // TODO replace with RC
    range: u32,
    pub(crate) code: u32,
    low: u32,
    stream: StreamUnion,
    index2units: [u8; 40],
    units2index: [u8; 128],
    free_list: [u32; 38],
    stamps: [u32; 38],
    ns2bs_index: [u8; 256],
    ns2index: [u8; 260],
    exp_escape: [u8; 16],
    dummy_see: See,
    see: [[See; 32]; 24],
    bin_summ: [[u16; 64]; 25],
    memory_ptr: NonNull<u8>,
    memory_layout: Layout,
    rc: RC,
}

impl<RC> Drop for Ppmd8<RC> {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.memory_ptr.as_ptr(), self.memory_layout);
        }
    }
}

impl<RC> Ppmd8<RC> {
    fn construct(
        stream: StreamUnion,
        rc: RC,
        mem_size: u32,
        max_order: u32,
        restore_method: RestoreMethod,
    ) -> Result<Self, Error> {
        let mut units2index = [0u8; 128];
        let mut index2units = [0u8; 40];

        let mut k = 0;
        for i in 0..PPMD_NUM_INDEXES {
            let step = if i >= 12 { 4 } else { (i >> 2) + 1 };
            for _ in 0..step {
                units2index[k as usize] = i as u8;
                k += 1;
            }
            index2units[i as usize] = k as u8;
        }

        let mut ns2bs_index = [0u8; 256];
        ns2bs_index[0] = (0 << 1) as u8;
        ns2bs_index[1] = (1 << 1) as u8;
        ns2bs_index[2..11].fill((2 << 1) as u8);
        ns2bs_index[11..256].fill((3 << 1) as u8);

        let mut ns2index = [0u8; 260];
        for i in 0..5 {
            ns2index[i as usize] = i as u8;
        }

        let mut m = 5;
        let mut k = 1;
        for i in 5..260 {
            ns2index[i as usize] = m as u8;
            k -= 1;
            if k == 0 {
                m += 1;
                k = m - 4;
            }
        }

        let align_offset = (4u32.wrapping_sub(mem_size)) & 3;
        let total_size = (align_offset + mem_size) as usize;

        let memory_layout = Layout::from_size_align(total_size, align_of::<usize>())
            .expect("Failed to create memory layout");

        let memory_ptr = unsafe {
            let Some(memory_ptr) = NonNull::new(alloc(memory_layout)) else {
                return Err(Error::InternalError(
                    "Failed to allocate memory for the internal memory allocation",
                ));
            };

            write_bytes(memory_ptr.as_ptr(), 0, total_size);
            memory_ptr
        };

        let mut ppmd = Self {
            min_context: std::ptr::null_mut(),
            max_context: std::ptr::null_mut(),
            found_state: std::ptr::null_mut(),
            order_fall: 0,
            init_esc: 0,
            prev_success: 0,
            max_order,
            restore_method,
            run_length: 0,
            init_rl: 0,
            size: mem_size,
            glue_count: 0,
            align_offset,
            // TODO remove base
            base: memory_ptr.as_ptr(),
            lo_unit: std::ptr::null_mut(),
            hi_unit: std::ptr::null_mut(),
            text: std::ptr::null_mut(),
            units_start: std::ptr::null_mut(),
            range: 0,
            code: 0,
            low: 0,
            stream,
            index2units,
            units2index,
            free_list: [0; 38],
            stamps: [0; 38],
            ns2bs_index,
            ns2index,
            exp_escape: K_EXP_ESCAPE,
            dummy_see: See::default(),
            see: [[See::default(); 32]; 24],
            bin_summ: [[0; 64]; 25],
            memory_ptr,
            memory_layout,
            rc,
        };

        unsafe { ppmd.restart_model() };

        Ok(ppmd)
    }

    unsafe fn ptr_of_offset(&self, offset: isize) -> NonNull<u8> {
        unsafe { self.memory_ptr.offset(offset) }
    }

    unsafe fn offset_for_ptr(&self, ptr: NonNull<u8>) -> u32 {
        unsafe {
            let offset = ptr.offset_from(self.memory_ptr);
            u32::try_from(offset).expect("Failed to convert ptr to offset")
        }
    }

    unsafe fn insert_node(&mut self, node: *mut u8, index: u32) {
        unsafe {
            (*(node as *mut Node)).stamp = EMPTY_NODE;
            (*(node as *mut Node)).next = self.free_list[index as usize];
            (*(node as *mut Node)).nu = self.index2units[index as usize] as u32;
            self.free_list[index as usize] = node.offset_from(self.base) as u32;
            self.stamps[index as usize] += 1;
        }
    }

    unsafe fn remove_node(&mut self, index: u32) -> *mut u8 {
        unsafe {
            let node: *mut Node = self
                .ptr_of_offset(self.free_list[index as usize] as isize)
                .cast()
                .as_mut();
            self.free_list[index as usize] = (*node).next;
            self.stamps[index as usize] -= 1;
            node as *mut u8
        }
    }

    unsafe fn split_block(&mut self, mut ptr: *mut u8, old_index: u32, new_index: u32) {
        unsafe {
            let nu = (self.index2units[old_index as usize] as u32)
                .wrapping_sub(self.index2units[new_index as usize] as u32);
            ptr = ptr
                .offset((self.index2units[new_index as usize] as u32 * UNIT_SIZE as u32) as isize);
            let mut i = self.units2index[(nu as usize).wrapping_sub(1)] as u32;
            if self.index2units[i as usize] as u32 != nu {
                i = i.wrapping_sub(1);
                let k = self.index2units[i as usize] as u32;
                self.insert_node(
                    ptr.offset((k * UNIT_SIZE as u32) as isize),
                    nu.wrapping_sub(k).wrapping_sub(1),
                );
            }
            self.insert_node(ptr, i);
        }
    }
    unsafe fn glue_free_blocks(&mut self) {
        unsafe {
            let mut n = 0;
            self.glue_count = (1 << 13) as u32;
            self.stamps = [0; 38];
            if self.lo_unit != self.hi_unit {
                (*(self.lo_unit as *mut Node)).stamp = 0;
            }
            let mut prev: *mut u32 = &mut n;
            let mut i = 0;
            while i < PPMD_NUM_INDEXES {
                let mut next = self.free_list[i as usize];
                self.free_list[i as usize] = 0;
                while next != 0 {
                    let node: *mut Node = (self.base).offset(next as isize) as *mut Node;
                    let mut nu = (*node).nu;
                    *prev = next;
                    next = (*node).next;
                    if nu != 0 {
                        let mut node2;
                        prev = &mut (*node).next;
                        loop {
                            node2 = node.offset(nu as isize);
                            if !((*node2).stamp == EMPTY_NODE) {
                                break;
                            }
                            nu = nu.wrapping_add((*node2).nu);
                            (*node2).nu = 0;
                            (*node).nu = nu;
                        }
                    }
                }
                i = i.wrapping_add(1);
            }
            *prev = 0;
            while n != 0 {
                let mut node_0: *mut Node = (self.base).offset(n as isize) as *mut Node;
                let mut nu_0 = (*node_0).nu;
                n = (*node_0).next;
                if nu_0 == 0 {
                    continue;
                }
                while nu_0 > 128 {
                    self.insert_node(node_0 as *mut u8, PPMD_NUM_INDEXES - 1);
                    nu_0 = nu_0.wrapping_sub(128);
                    node_0 = node_0.offset(128);
                }
                let mut i_0 = self.units2index
                    [(nu_0 as usize).wrapping_sub(1 as i32 as usize) as usize]
                    as u32;
                if self.index2units[i_0 as usize] as u32 != nu_0 {
                    i_0 = i_0.wrapping_sub(1);
                    let k = self.index2units[i_0 as usize] as u32;
                    self.insert_node(
                        node_0.offset(k as isize) as *mut u8,
                        nu_0.wrapping_sub(k).wrapping_sub(1),
                    );
                }
                self.insert_node(node_0 as *mut u8, i_0);
            }
        }
    }

    #[inline(never)]
    unsafe fn alloc_units_rare(&mut self, index: u32) -> *mut u8 {
        unsafe {
            if self.glue_count == 0 {
                self.glue_free_blocks();
                if self.free_list[index as usize] != 0 {
                    return self.remove_node(index);
                }
            }
            let mut i = index;
            loop {
                i = i.wrapping_add(1);
                if i == PPMD_NUM_INDEXES {
                    let num_bytes = self.index2units[index as usize] as u32 * UNIT_SIZE as u32;
                    let us: *mut u8 = self.units_start;
                    self.glue_count = (self.glue_count).wrapping_sub(1);
                    self.glue_count;
                    return if us.offset_from(self.text) as u32 > num_bytes {
                        self.units_start = us.offset(-(num_bytes as isize));
                        self.units_start
                    } else {
                        std::ptr::null_mut()
                    };
                }
                if !(self.free_list[i as usize] == 0) {
                    break;
                }
            }
            let block: *mut u8 = self.remove_node(i);
            self.split_block(block, i, index);
            block
        }
    }

    unsafe fn alloc_units(&mut self, index: u32) -> *mut u8 {
        unsafe {
            if self.free_list[index as usize] != 0 {
                return self.remove_node(index);
            }
            let num_bytes = self.index2units[index as usize] as u32 * UNIT_SIZE as u32;
            let lo: *mut u8 = self.lo_unit;
            if (self.hi_unit).offset_from(lo) as u32 >= num_bytes {
                self.lo_unit = lo.offset(num_bytes as isize);
                return lo;
            }
            self.alloc_units_rare(index)
        }
    }

    unsafe fn shrink_units(&mut self, old_ptr: *mut State, old_nu: u32, new_nu: u32) -> *mut u8 {
        unsafe {
            let i0 = self.units2index[(old_nu as usize).wrapping_sub(1)] as u32;
            let i1 = self.units2index[(new_nu as usize).wrapping_sub(1)] as u32;
            if i0 == i1 {
                return old_ptr as *mut u8;
            }
            if self.free_list[i1 as usize] != 0 {
                let ptr: *mut u8 = self.remove_node(i1);
                let mut d: *mut u32 = ptr as *mut u32;
                let mut z: *const u32 = old_ptr as *const u32;
                let mut n = new_nu;
                loop {
                    *d.offset(0) = *z.offset(0);
                    *d.offset(1) = *z.offset(1);
                    *d.offset(2) = *z.offset(2);
                    z = z.offset(3);
                    d = d.offset(3);
                    n = n.wrapping_sub(1);
                    if !(n != 0) {
                        break;
                    }
                }
                self.insert_node(old_ptr as *mut u8, i0);
                return ptr;
            }
            self.split_block(old_ptr as *mut u8, i0, i1);
            old_ptr as *mut u8
        }
    }

    unsafe fn free_units(&mut self, ptr: *mut u8, nu: u32) {
        unsafe {
            self.insert_node(
                ptr,
                self.units2index[(nu as usize).wrapping_sub(1 as i32 as usize) as usize] as u32,
            );
        }
    }

    unsafe fn special_free_unit(&mut self, ptr: *mut u8) {
        unsafe {
            if ptr != self.units_start {
                self.insert_node(ptr, 0);
            } else {
                self.units_start = (self.units_start).offset(UNIT_SIZE);
            };
        }
    }

    unsafe fn expand_text_area(&mut self) {
        unsafe {
            let mut count: [u32; 38] = [0; 38];
            if self.lo_unit != self.hi_unit {
                (*(self.lo_unit as *mut Node)).stamp = 0;
            }
            let mut node: *mut Node = self.units_start as *mut Node;
            while (*node).stamp == EMPTY_NODE {
                let nu = (*node).nu;
                (*node).stamp = 0;
                count[self.units2index[(nu as usize).wrapping_sub(1 as i32 as usize) as usize]
                    as usize] = (count[self.units2index
                    [(nu as usize).wrapping_sub(1 as i32 as usize) as usize]
                    as usize])
                    .wrapping_add(1);
                count[self.units2index[(nu as usize).wrapping_sub(1 as i32 as usize) as usize]
                    as usize];
                node = node.offset(nu as isize);
            }
            self.units_start = node as *mut u8;
            let mut i = 0;
            while i < PPMD_NUM_INDEXES {
                let mut cnt = count[i as usize];
                if !(cnt == 0) {
                    let mut prev: *mut u32 = &mut *(self.free_list).as_mut_ptr().offset(i as isize)
                        as *mut u32 as *mut u32;
                    let mut n = *prev;
                    self.stamps[i as usize] = (self.stamps[i as usize]).wrapping_sub(cnt);
                    loop {
                        let node_0: *mut Node = (self.base).offset(n as isize) as *mut Node;
                        n = (*node_0).next;
                        if (*node_0).stamp != 0 {
                            prev = &mut (*node_0).next;
                        } else {
                            *prev = n;
                            cnt = cnt.wrapping_sub(1);
                            if cnt == 0 {
                                break;
                            }
                        }
                    }
                }
                i = i.wrapping_add(1);
            }
        }
    }

    unsafe fn set_successor(s: *mut State, v: u32) {
        unsafe {
            (*s).successor_0 = v as u16;
            (*s).successor_1 = (v >> 16) as u16;
        }
    }

    unsafe fn get_successor(&mut self, s: *mut State) -> NonNull<Context> {
        unsafe {
            self.ptr_of_offset(((*s).successor_0 as u32 | ((*s).successor_1 as u32) << 16) as isize)
                .cast()
        }
    }

    #[inline(never)]
    unsafe fn restart_model(&mut self) {
        unsafe {
            self.free_list = [0; 38];
            self.stamps = [0; 38];
            self.text = (self.base).offset(self.align_offset as isize).offset(0);
            self.hi_unit = (self.text).offset(self.size as isize);
            self.units_start = (self.hi_unit)
                .offset(-((self.size / 8 / UNIT_SIZE as u32 * 7 * UNIT_SIZE as u32) as isize));
            self.lo_unit = self.units_start;
            self.glue_count = 0;
            self.order_fall = self.max_order;
            self.init_rl = -((if self.max_order < 12 {
                self.max_order
            } else {
                12
            }) as i32)
                - 1;
            self.run_length = self.init_rl;
            self.prev_success = 0;
            self.hi_unit = (self.hi_unit).offset(-(UNIT_SIZE));
            let mc: *mut Context = self.hi_unit as *mut Context;
            let mut s: *mut State = self.lo_unit as *mut State;
            self.lo_unit =
                (self.lo_unit).offset(((256 as i32 / 2) as u32 * UNIT_SIZE as u32) as isize);
            self.min_context = mc;
            self.max_context = self.min_context;
            self.found_state = s;
            (*mc).flags = 0 as u8;
            (*mc).num_stats = (256 as i32 - 1) as u8;
            (*mc).union2.summ_freq = (256 as i32 + 1) as u16;
            (*mc).union4.stats = (s as *mut u8).offset_from(self.base) as u32;
            (*mc).suffix = 0;
            let mut i = 0u32;
            while i < 256 {
                (*s).symbol = i as u8;
                (*s).freq = 1 as u8;
                Self::set_successor(s, 0);
                i = i.wrapping_add(1);
                s = s.offset(1);
            }

            let mut m = 0;
            i = m;
            while m < 25 {
                while self.ns2index[i as usize] as u32 == m {
                    i = i.wrapping_add(1);
                }

                let mut k = 0u32;
                while k < 8 {
                    let mut r = 0u32;
                    let dest: *mut u16 =
                        (self.bin_summ[m as usize]).as_mut_ptr().offset(k as isize);
                    let val = ((1 << 7 + 7) as u32).wrapping_sub(
                        (K_INIT_BIN_ESC[k as usize] as u32).wrapping_div(i.wrapping_add(1)),
                    ) as u16;
                    r = 0;
                    while r < 64 {
                        *dest.offset(r as isize) = val;
                        r = r.wrapping_add(8);
                    }
                    k = k.wrapping_add(1);
                }
                m = m.wrapping_add(1);
            }
            m = 0;
            i = m;
            while m < 24 {
                let mut summ = 0;
                let mut s_0: *mut See = std::ptr::null_mut();
                while self.ns2index[(i as usize).wrapping_add(3 as i32 as usize) as usize] as u32
                    == m.wrapping_add(3)
                {
                    i = i.wrapping_add(1);
                }
                s_0 = (self.see[m as usize]).as_mut_ptr();
                summ = 2u32.wrapping_mul(i).wrapping_add(5) << 7 - 4;

                let mut k = 0u32;
                while k < 32 {
                    (*s_0).summ = summ as u16;
                    (*s_0).shift = (7 as i32 - 4) as u8;
                    (*s_0).count = 7 as u8;
                    k = k.wrapping_add(1);
                    s_0 = s_0.offset(1);
                }
                m = m.wrapping_add(1);
            }
            self.dummy_see.summ = 0 as u16;
            self.dummy_see.shift = 7 as u8;
            self.dummy_see.count = 64 as u8;
        }
    }

    unsafe fn refresh(&mut self, ctx: *mut Context, old_nu: u32, mut scale: u32) {
        unsafe {
            let mut i = (*ctx).num_stats as u32;
            let mut esc_freq = 0;
            let mut sum_freq = 0;
            let mut flags = 0;
            let mut s: *mut State = self.shrink_units(
                (self.base).offset((*ctx).union4.stats as isize) as *mut State,
                old_nu,
                i.wrapping_add(2) >> 1,
            ) as *mut State;
            (*ctx).union4.stats = (s as *mut u8).offset_from(self.base) as u32;
            scale |= ((*ctx).union2.summ_freq as u32 >= 1 << 15) as u32;
            flags = ((*s).symbol as u32).wrapping_add(0xC0);
            let mut freq = (*s).freq as u32;
            esc_freq = ((*ctx).union2.summ_freq as u32).wrapping_sub(freq);
            freq = freq.wrapping_add(scale) >> scale;
            sum_freq = freq;
            (*s).freq = freq as u8;
            loop {
                s = s.offset(1);
                let mut freq_0 = (*s).freq as u32;
                esc_freq = esc_freq.wrapping_sub(freq_0);
                freq_0 = freq_0.wrapping_add(scale) >> scale;
                sum_freq = sum_freq.wrapping_add(freq_0);
                (*s).freq = freq_0 as u8;
                flags |= ((*s).symbol as u32).wrapping_add(0xC0);
                i = i.wrapping_sub(1);
                if !(i != 0) {
                    break;
                }
            }
            (*ctx).union2.summ_freq =
                sum_freq.wrapping_add(esc_freq.wrapping_add(scale) >> scale) as u16;
            (*ctx).flags = ((*ctx).flags as u32
                & ((1 << 4) as u32).wrapping_add(((1 << 2) as u32).wrapping_mul(scale)))
            .wrapping_add(flags >> 8 - 3 & (1 << 3) as u32) as u8;
        }
    }

    unsafe fn swap_states(t1: *mut State, t2: *mut State) {
        unsafe {
            // TODO use std::mem::swap later
            core::ptr::swap(t1, t2);
        }
    }

    unsafe fn cut_off(&mut self, ctx: *mut Context, order: u32) -> u32 {
        unsafe {
            let mut ns = (*ctx).num_stats as i32;
            let mut nu = 0;
            let mut stats: *mut State = std::ptr::null_mut();
            if ns == 0 {
                let s: *mut State = &mut (*ctx).union2 as *mut Union2 as *mut State;
                let mut successor = (*s).successor_0 as u32 | ((*s).successor_1 as u32) << 16;
                if (self.base).offset(successor as isize) >= self.units_start {
                    if order < self.max_order {
                        successor = self.cut_off(
                            (self.base).offset(successor as isize) as *mut Context,
                            order.wrapping_add(1),
                        );
                    } else {
                        successor = 0;
                    }
                    Self::set_successor(s, successor);
                    if successor != 0 || order <= 9 {
                        return (ctx as *mut u8).offset_from(self.base) as u32;
                    }
                }
                self.special_free_unit(ctx as *mut u8);
                return 0;
            }
            nu = (ns as u32).wrapping_add(2) >> 1;
            let index =
                self.units2index[(nu as usize).wrapping_sub(1 as i32 as usize) as usize] as u32;
            stats = (self.base).offset((*ctx).union4.stats as isize) as *mut State;
            if (stats as *mut u8).offset_from(self.units_start) as u32 <= (1 << 14) as u32
                && (*ctx).union4.stats <= self.free_list[index as usize]
            {
                let ptr: *mut u8 = self.remove_node(index);
                (*ctx).union4.stats = ptr.offset_from(self.base) as u32;
                let mut d: *mut u32 = ptr as *mut u32;
                let mut z: *const u32 = stats as *const u8 as *const u32;
                let mut n = nu;
                loop {
                    *d.offset(0) = *z.offset(0);
                    *d.offset(1) = *z.offset(1);
                    *d.offset(2) = *z.offset(2);
                    z = z.offset(3);
                    d = d.offset(3);
                    n = n.wrapping_sub(1);
                    if !(n != 0) {
                        break;
                    }
                }
                if stats as *mut u8 != self.units_start {
                    self.insert_node(stats as *mut u8, index);
                } else {
                    self.units_start = (self.units_start).offset(
                        (self.index2units[index as usize] as u32 * UNIT_SIZE as u32) as isize,
                    );
                }
                stats = ptr as *mut State;
            }
            let mut s_0: *mut State = stats.offset(ns as u32 as isize);
            loop {
                let successor_0 = (*s_0).successor_0 as u32 | ((*s_0).successor_1 as u32) << 16;
                if ((self.base).offset(successor_0 as isize)) < self.units_start {
                    let fresh1 = ns;
                    ns = ns - 1;
                    let s2: *mut State = stats.offset(fresh1 as u32 as isize);
                    if order != 0 {
                        if s_0 != s2 {
                            *s_0 = *s2;
                        }
                    } else {
                        Self::swap_states(s_0, s2);
                        Self::set_successor(s2, 0);
                    }
                } else if order < self.max_order {
                    Self::set_successor(
                        s_0,
                        self.cut_off(
                            (self.base).offset(successor_0 as isize) as *mut Context,
                            order.wrapping_add(1),
                        ),
                    );
                } else {
                    Self::set_successor(s_0, 0);
                }
                s_0 = s_0.offset(-1);
                if !(s_0 >= stats) {
                    break;
                }
            }
            if ns != (*ctx).num_stats as i32 && order != 0 {
                if ns < 0 {
                    self.free_units(stats as *mut u8, nu);
                    self.special_free_unit(ctx as *mut u8);
                    return 0;
                }
                (*ctx).num_stats = ns as u8;
                if ns == 0 {
                    let sym = (*stats).symbol;
                    (*ctx).flags = (((*ctx).flags as i32 & 1 << 4) as u32)
                        .wrapping_add((sym as u32).wrapping_add(0xC0) >> 8 - 3 & (1 << 3) as u32)
                        as u8;
                    (*ctx).union2.state2.symbol = sym;
                    (*ctx).union2.state2.freq =
                        (((*stats).freq as u32).wrapping_add(11) >> 3) as u8;
                    (*ctx).union4.state4.successor_0 = (*stats).successor_0;
                    (*ctx).union4.state4.successor_1 = (*stats).successor_1;
                    self.free_units(stats as *mut u8, nu);
                } else {
                    self.refresh(
                        ctx,
                        nu,
                        ((*ctx).union2.summ_freq as u32 > 16u32.wrapping_mul(ns as u32)) as u32,
                    );
                }
            }
            (ctx as *mut u8).offset_from(self.base) as u32
        }
    }

    unsafe fn get_used_memory(&self) -> u32 {
        unsafe {
            let mut v = 0;

            for i in 0..PPMD_NUM_INDEXES {
                v *= self.stamps[i as usize] * self.index2units[i as usize] as u32;
            }

            self.size
                - (self.hi_unit.offset_from(self.lo_unit) as u32)
                - (self.units_start.offset_from(self.text) as u32)
                - (v * 12)
        }
    }

    unsafe fn restore_model(&mut self, ctx_error: *mut Context) {
        unsafe {
            let mut c: *mut Context = std::ptr::null_mut();
            let mut s: *mut State = std::ptr::null_mut();
            self.text = (self.base).offset(self.align_offset as isize).offset(0);
            c = self.max_context;
            while c != ctx_error {
                (*c).num_stats = ((*c).num_stats).wrapping_sub(1);
                if (*c).num_stats as i32 == 0 {
                    s = (self.base).offset((*c).union4.stats as isize) as *mut State;
                    (*c).flags = (((*c).flags as i32 & 1 << 4) as u32).wrapping_add(
                        ((*s).symbol as u32).wrapping_add(0xC0) >> 8 - 3 & (1 << 3) as u32,
                    ) as u8;
                    (*c).union2.state2.symbol = (*s).symbol;
                    (*c).union2.state2.freq = (((*s).freq as u32).wrapping_add(11) >> 3) as u8;
                    (*c).union4.state4.successor_0 = (*s).successor_0;
                    (*c).union4.state4.successor_1 = (*s).successor_1;
                    self.special_free_unit(s as *mut u8);
                } else {
                    self.refresh(c, ((*c).num_stats as u32).wrapping_add(3) >> 1, 0);
                }
                c = (self.base).offset((*c).suffix as isize) as *mut Context;
            }
            while c != self.min_context {
                if (*c).num_stats as i32 == 0 {
                    (*c).union2.state2.freq =
                        (((*c).union2.state2.freq as u32).wrapping_add(1) >> 1) as u8;
                } else {
                    (*c).union2.summ_freq = ((*c).union2.summ_freq as i32 + 4) as u16;
                    if (*c).union2.summ_freq as i32 > 128 + 4 * (*c).num_stats as i32 {
                        self.refresh(c, ((*c).num_stats as u32).wrapping_add(2) >> 1, 1);
                    }
                }
                c = (self.base).offset((*c).suffix as isize) as *mut Context;
            }
            if self.restore_method == RestoreMethod::Restart
                || self.get_used_memory() < self.size >> 1
            {
                self.restart_model();
            } else {
                while (*self.max_context).suffix != 0 {
                    self.max_context =
                        (self.base).offset((*self.max_context).suffix as isize) as *mut Context;
                }
                loop {
                    self.cut_off(self.max_context, 0);
                    self.expand_text_area();
                    if !(self.get_used_memory() > 3 * (self.size >> 2)) {
                        break;
                    }
                }
                self.glue_count = 0;
                self.order_fall = self.max_order;
            }
            self.min_context = self.max_context;
        }
    }

    #[inline(never)]
    unsafe fn create_successors(
        &mut self,
        skip: i32,
        mut s1: *mut State,
        mut c: *mut Context,
    ) -> *mut Context {
        unsafe {
            let mut up_branch = (*self.found_state).successor_0 as u32
                | ((*self.found_state).successor_1 as u32) << 16;
            let mut new_sym = 0;
            let mut new_freq = 0;
            let mut flags = 0;
            let mut num_ps = 0u32;
            let mut ps: [*mut State; 17] = [std::ptr::null_mut(); 17];
            if skip == 0 {
                let fresh2 = num_ps;
                num_ps = num_ps.wrapping_add(1);
                ps[fresh2 as usize] = self.found_state;
            }
            while (*c).suffix != 0 {
                let mut successor = 0;
                let mut s: *mut State = std::ptr::null_mut();
                c = (self.base).offset((*c).suffix as isize) as *mut Context;
                if !s1.is_null() {
                    s = s1;
                    s1 = std::ptr::null_mut();
                } else if (*c).num_stats as i32 != 0 {
                    let sym = (*self.found_state).symbol;
                    s = (self.base).offset((*c).union4.stats as isize) as *mut State;
                    while (*s).symbol as i32 != sym as i32 {
                        s = s.offset(1);
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
                        + (((*((self.base).offset((*c).suffix as isize) as *mut Context)).num_stats
                            == 0) as i32
                            & (((*s).freq as i32) < 24) as i32))
                        as u8;
                }
                successor = (*s).successor_0 as u32 | ((*s).successor_1 as u32) << 16;
                if successor != up_branch {
                    c = (self.base).offset(successor as isize) as *mut Context;
                    if num_ps == 0 {
                        return c;
                    }
                    break;
                } else {
                    let fresh3 = num_ps;
                    num_ps = num_ps.wrapping_add(1);
                    ps[fresh3 as usize] = s;
                }
            }
            new_sym = *((self.base).offset(up_branch as isize) as *const u8);
            up_branch = up_branch.wrapping_add(1);
            flags = (((*self.found_state).symbol as u32).wrapping_add(0xC0) >> 8 - 4
                & (1 << 4) as u32)
                .wrapping_add((new_sym as u32).wrapping_add(0xC0) >> 8 - 3 & (1 << 3) as u32)
                as u8;
            if (*c).num_stats as i32 == 0 {
                new_freq = (*c).union2.state2.freq;
            } else {
                let mut cf = 0;
                let mut s0 = 0;
                let mut s_0: *mut State = std::ptr::null_mut();
                s_0 = (self.base).offset((*c).union4.stats as isize) as *mut State;
                while (*s_0).symbol as i32 != new_sym as i32 {
                    s_0 = s_0.offset(1);
                }
                cf = ((*s_0).freq as u32).wrapping_sub(1);
                s0 = ((*c).union2.summ_freq as u32)
                    .wrapping_sub((*c).num_stats as u32)
                    .wrapping_sub(cf);
                new_freq = 1u32.wrapping_add(if 2 * cf <= s0 {
                    (5 * cf > s0) as u32
                } else {
                    cf.wrapping_add(2 * s0).wrapping_sub(3) / s0
                }) as u8;
            }
            loop {
                let mut c1: *mut Context = std::ptr::null_mut();
                if self.hi_unit != self.lo_unit {
                    self.hi_unit = (self.hi_unit).offset(-(12));
                    c1 = self.hi_unit as *mut Context;
                } else if self.free_list[0 as i32 as usize] != 0 {
                    c1 = self.remove_node(0) as *mut Context;
                } else {
                    c1 = self.alloc_units_rare(0) as *mut Context;
                    if c1.is_null() {
                        return std::ptr::null_mut();
                    }
                }
                (*c1).flags = flags;
                (*c1).num_stats = 0 as u8;
                (*c1).union2.state2.symbol = new_sym;
                (*c1).union2.state2.freq = new_freq;
                Self::set_successor(&mut (*c1).union2 as *mut Union2 as *mut State, up_branch);
                (*c1).suffix = (c as *mut u8).offset_from(self.base) as u32;
                num_ps = num_ps.wrapping_sub(1);
                Self::set_successor(
                    ps[num_ps as usize],
                    (c1 as *mut u8).offset_from(self.base) as u32,
                );
                c = c1;
                if !(num_ps != 0) {
                    break;
                }
            }

            c
        }
    }

    unsafe fn reduce_order(&mut self, mut s1: *mut State, mut c: *mut Context) -> *mut Context {
        unsafe {
            let mut s: *mut State = std::ptr::null_mut();
            let c1: *mut Context = c;
            let up_branch = (self.text).offset_from(self.base) as u32;
            Self::set_successor(self.found_state, up_branch);
            self.order_fall = (self.order_fall).wrapping_add(1);
            self.order_fall;
            loop {
                if !s1.is_null() {
                    c = (self.base).offset((*c).suffix as isize) as *mut Context;
                    s = s1;
                    s1 = std::ptr::null_mut();
                } else {
                    if (*c).suffix == 0 {
                        return c;
                    }
                    c = (self.base).offset((*c).suffix as isize) as *mut Context;
                    if (*c).num_stats != 0 {
                        s = (self.base).offset((*c).union4.stats as isize) as *mut State;
                        if (*s).symbol as i32 != (*self.found_state).symbol as i32 {
                            loop {
                                s = s.offset(1);
                                if !((*s).symbol as i32 != (*self.found_state).symbol as i32) {
                                    break;
                                }
                            }
                        }
                        if ((*s).freq) < MAX_FREQ - 9 {
                            (*s).freq = ((*s).freq as i32 + 2) as u8;
                            (*c).union2.summ_freq = ((*c).union2.summ_freq as i32 + 2) as u16;
                        }
                    } else {
                        s = &mut (*c).union2 as *mut Union2 as *mut State;
                        (*s).freq = ((*s).freq as i32 + (((*s).freq as i32) < 32) as i32) as u8;
                    }
                }
                if (*s).successor_0 as u32 | ((*s).successor_1 as u32) << 16 != 0 {
                    break;
                }
                Self::set_successor(s, up_branch);
                self.order_fall = (self.order_fall).wrapping_add(1);
                self.order_fall;
            }
            if (*s).successor_0 as u32 | ((*s).successor_1 as u32) << 16 <= up_branch {
                let mut successor: *mut Context = std::ptr::null_mut();
                let s2: *mut State = self.found_state;
                self.found_state = s;
                successor = self.create_successors(0 as i32, std::ptr::null_mut(), c);
                if successor.is_null() {
                    Self::set_successor(s, 0);
                } else {
                    Self::set_successor(s, (successor as *mut u8).offset_from(self.base) as u32);
                }
                self.found_state = s2;
            }
            let successor_0 = (*s).successor_0 as u32 | ((*s).successor_1 as u32) << 16;
            if self.order_fall == 1 && c1 == self.max_context {
                Self::set_successor(self.found_state, successor_0);
                self.text = (self.text).offset(-1);
                self.text;
            }
            if successor_0 == 0 {
                return std::ptr::null_mut();
            }

            self.ptr_of_offset(successor_0 as isize).cast().as_mut()
        }
    }

    #[inline(never)]
    unsafe fn update_model(&mut self) {
        unsafe {
            let mut max_successor = 0;
            let mut min_successor = (*self.found_state).successor_0 as u32
                | ((*self.found_state).successor_1 as u32) << 16;
            let mut c: *mut Context = std::ptr::null_mut();
            let mut s0 = 0;
            let mut ns = 0;
            let f_freq = (*self.found_state).freq as u32;
            let mut flag = 0;
            let f_symbol = (*self.found_state).symbol;
            let mut s: *mut State = std::ptr::null_mut();
            if ((*self.found_state).freq) < MAX_FREQ / 4 && (*self.min_context).suffix != 0 {
                c = (self.base).offset((*self.min_context).suffix as isize) as *mut Context;
                if (*c).num_stats as i32 == 0 {
                    s = &mut (*c).union2 as *mut Union2 as *mut State;
                    if ((*s).freq as i32) < 32 {
                        (*s).freq = ((*s).freq).wrapping_add(1);
                        (*s).freq;
                    }
                } else {
                    let sym = (*self.found_state).symbol;
                    s = (self.base).offset((*c).union4.stats as isize) as *mut State;
                    if (*s).symbol as i32 != sym as i32 {
                        loop {
                            s = s.offset(1);
                            if !((*s).symbol as i32 != sym as i32) {
                                break;
                            }
                        }
                        if (*s.offset(0)).freq as i32 >= (*s.offset(-1 as isize)).freq as i32 {
                            Self::swap_states(&mut *s.offset(0), &mut *s.offset(-1 as isize));
                            s = s.offset(-1);
                        }
                    }
                    if ((*s).freq) < MAX_FREQ - 9 {
                        (*s).freq = ((*s).freq as i32 + 2) as u8;
                        (*c).union2.summ_freq = ((*c).union2.summ_freq as i32 + 2) as u16;
                    }
                }
            }
            c = self.max_context;
            if self.order_fall == 0 && min_successor != 0 {
                let cs: *mut Context = self.create_successors(1 as i32, s, self.min_context);
                if cs.is_null() {
                    Self::set_successor(self.found_state, 0);
                    self.restore_model(c);
                    return;
                }
                Self::set_successor(
                    self.found_state,
                    (cs as *mut u8).offset_from(self.base) as u32,
                );
                self.max_context = cs;
                self.min_context = self.max_context;
                return;
            }
            let mut text: *mut u8 = self.text;
            let fresh4 = text;
            text = text.offset(1);
            *fresh4 = (*self.found_state).symbol;
            self.text = text;
            if text >= self.units_start {
                self.restore_model(c);
                return;
            }
            max_successor = text.offset_from(self.base) as u32;
            if min_successor == 0 {
                let cs_0: *mut Context = self.reduce_order(s, self.min_context);
                if cs_0.is_null() {
                    self.restore_model(c);
                    return;
                }
                min_successor = (cs_0 as *mut u8).offset_from(self.base) as u32;
            } else if ((self.base).offset(min_successor as isize)) < self.units_start {
                let cs_1: *mut Context = self.create_successors(0 as i32, s, self.min_context);
                if cs_1.is_null() {
                    self.restore_model(c);
                    return;
                }
                min_successor = (cs_1 as *mut u8).offset_from(self.base) as u32;
            }
            self.order_fall = (self.order_fall).wrapping_sub(1);
            if self.order_fall == 0 {
                max_successor = min_successor;
                self.text = (self.text).offset(-((self.max_context != self.min_context) as isize));
            }
            flag = ((f_symbol as u32).wrapping_add(0xC0) >> 8 - 3 & (1 << 3) as u32) as u8;
            ns = (*self.min_context).num_stats as u32;
            s0 = ((*self.min_context).union2.summ_freq as u32)
                .wrapping_sub(ns)
                .wrapping_sub(f_freq);
            while c != self.min_context {
                let mut ns1 = 0;
                let mut sum = 0;
                ns1 = (*c).num_stats as u32;
                if ns1 != 0 {
                    if ns1 & 1 != 0 {
                        let old_nu = ns1.wrapping_add(1) >> 1;
                        let i = self.units2index
                            [(old_nu as usize).wrapping_sub(1 as i32 as usize) as usize]
                            as u32;
                        if i != self.units2index[(old_nu as usize)
                            .wrapping_add(1 as usize)
                            .wrapping_sub(1 as usize)
                            as usize] as u32
                        {
                            let ptr: *mut u8 = self.alloc_units(i.wrapping_add(1 as u32));
                            let mut old_ptr: *mut u8 = std::ptr::null_mut();
                            if ptr.is_null() {
                                self.restore_model(c);
                                return;
                            }
                            old_ptr = (self.base).offset((*c).union4.stats as isize) as *mut State
                                as *mut u8;
                            let mut d: *mut u32 = ptr as *mut u32;
                            let mut z: *const u32 = old_ptr as *const u32;
                            let mut n = old_nu;
                            loop {
                                *d.offset(0) = *z.offset(0);
                                *d.offset(1) = *z.offset(1);
                                *d.offset(2) = *z.offset(2);
                                z = z.offset(3);
                                d = d.offset(3);
                                n = n.wrapping_sub(1);
                                if !(n != 0) {
                                    break;
                                }
                            }
                            self.insert_node(old_ptr, i);
                            (*c).union4.stats = ptr.offset_from(self.base) as u32;
                        }
                    }
                    sum = (*c).union2.summ_freq as u32;
                    sum = sum.wrapping_add((3u32.wrapping_mul(ns1).wrapping_add(1) < ns) as u32);
                } else {
                    let s_0: *mut State = self.alloc_units(0) as *mut State;
                    if s_0.is_null() {
                        self.restore_model(c);
                        return;
                    }
                    let mut freq = (*c).union2.state2.freq as u32;
                    (*s_0).symbol = (*c).union2.state2.symbol;
                    (*s_0).successor_0 = (*c).union4.state4.successor_0;
                    (*s_0).successor_1 = (*c).union4.state4.successor_1;
                    (*c).union4.stats = (s_0 as *mut u8).offset_from(self.base) as u32;
                    if freq < (MAX_FREQ as i32 / 4 - 1) as u32 {
                        freq <<= 1;
                    } else {
                        freq = (MAX_FREQ as i32 - 4) as u32;
                    }
                    (*s_0).freq = freq as u8;
                    sum = freq
                        .wrapping_add(self.init_esc)
                        .wrapping_add((ns > 2) as u32);
                }
                let s_1: *mut State = ((self.base).offset((*c).union4.stats as isize)
                    as *mut State)
                    .offset(ns1 as isize)
                    .offset(1);
                let mut cf = 2 * sum.wrapping_add(6) * f_freq;
                let sf = s0.wrapping_add(sum);
                (*s_1).symbol = f_symbol;
                (*c).num_stats = ns1.wrapping_add(1) as u8;
                Self::set_successor(s_1, max_successor);
                (*c).flags = ((*c).flags as i32 | flag as i32) as u8;
                if cf < 6 * sf {
                    cf = 1u32
                        .wrapping_add((cf > sf) as u32)
                        .wrapping_add((cf >= 4 * sf) as u32);
                    sum = sum.wrapping_add(4);
                } else {
                    cf = 4u32
                        .wrapping_add((cf > 9 * sf) as u32)
                        .wrapping_add((cf > 12 * sf) as u32)
                        .wrapping_add((cf > 15 * sf) as u32);
                    sum = sum.wrapping_add(cf);
                }
                (*c).union2.summ_freq = sum as u16;
                (*s_1).freq = cf as u8;
                c = (self.base).offset((*c).suffix as isize) as *mut Context;
            }
            self.min_context = (self.base).offset(min_successor as isize) as *mut Context;
            self.max_context = self.min_context;
        }
    }

    #[inline(never)]
    unsafe fn rescale(&mut self) {
        unsafe {
            let mut i = 0;
            let mut adder = 0;
            let mut sum_freq = 0;
            let mut esc_freq = 0;
            let stats: *mut State =
                (self.base).offset((*self.min_context).union4.stats as isize) as *mut State;
            let mut s: *mut State = self.found_state;
            if s != stats {
                let tmp: State = *s;
                loop {
                    *s.offset(0) = *s.offset(-1);
                    s = s.offset(-1);
                    if !(s != stats) {
                        break;
                    }
                }
                *s = tmp;
            }
            sum_freq = (*s).freq as u32;
            esc_freq = ((*self.min_context).union2.summ_freq as u32).wrapping_sub(sum_freq);
            adder = (self.order_fall != 0) as u32;
            sum_freq = sum_freq.wrapping_add(4).wrapping_add(adder) >> 1;
            i = (*self.min_context).num_stats as u32;
            (*s).freq = sum_freq as u8;
            loop {
                s = s.offset(1);
                let mut freq = (*s).freq as u32;
                esc_freq = esc_freq.wrapping_sub(freq);
                freq = freq.wrapping_add(adder) >> 1;
                sum_freq = sum_freq.wrapping_add(freq);
                (*s).freq = freq as u8;
                if freq > (*s.offset(-1)).freq as u32 {
                    let tmp_0: State = *s;
                    let mut s1: *mut State = s;
                    loop {
                        *s1.offset(0) = *s1.offset(-1);
                        s1 = s1.offset(-1);
                        if !(s1 != stats && freq > (*s1.offset(-1)).freq as u32) {
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
            if (*s).freq as i32 == 0 {
                let mut mc: *mut Context = std::ptr::null_mut();
                let mut num_stats = 0;
                let mut num_stats_new = 0;
                let mut n0 = 0;
                let mut n1 = 0;
                i = 0;
                loop {
                    i = i.wrapping_add(1);
                    s = s.offset(-1);
                    if !((*s).freq as i32 == 0) {
                        break;
                    }
                }
                esc_freq = esc_freq.wrapping_add(i);
                mc = self.min_context;
                num_stats = (*mc).num_stats as u32;
                num_stats_new = num_stats.wrapping_sub(i);
                (*mc).num_stats = num_stats_new as u8;
                n0 = num_stats.wrapping_add(2) >> 1;
                if num_stats_new == 0 {
                    let mut freq_0 = 2u32
                        .wrapping_mul((*stats).freq as u32)
                        .wrapping_add(esc_freq)
                        .wrapping_sub(1)
                        .wrapping_div(esc_freq);
                    if freq_0 > (MAX_FREQ as i32 / 3) as u32 {
                        freq_0 = (MAX_FREQ as i32 / 3) as u32;
                    }
                    (*mc).flags = (((*mc).flags as i32 & 1 << 4) as u32).wrapping_add(
                        ((*stats).symbol as u32).wrapping_add(0xC0) >> 8 - 3 & (1 << 3) as u32,
                    ) as u8;
                    s = &mut (*mc).union2 as *mut Union2 as *mut State;
                    *s = *stats;
                    (*s).freq = freq_0 as u8;
                    self.found_state = s;
                    self.insert_node(
                        stats as *mut u8,
                        self.units2index[(n0 as usize).wrapping_sub(1)] as u32,
                    );
                    return;
                }
                n1 = num_stats_new.wrapping_add(2) >> 1;
                if n0 != n1 {
                    (*mc).union4.stats =
                        self.shrink_units(stats, n0, n1).offset_from(self.base) as u32;
                }
            }
            let mc_0: *mut Context = self.min_context;
            (*mc_0).union2.summ_freq =
                sum_freq.wrapping_add(esc_freq).wrapping_sub(esc_freq >> 1) as u16;
            (*mc_0).flags = ((*mc_0).flags as i32 | 1 << 2) as u8;
            self.found_state = (self.base).offset((*mc_0).union4.stats as isize) as *mut State;
        }
    }

    unsafe fn make_esc_freq(&mut self, num_masked: u32, esc_freq: &mut u32) -> SeeSource {
        unsafe {
            let num_stats = (*self.min_context).num_stats as u32;

            if num_stats != 0xFF {
                let (base_context_idx, see_table_hash) =
                    self.calculate_see_table_hash(num_masked, num_stats);

                let see = &mut self.see[base_context_idx][see_table_hash];

                // If (see.summ) field is larger than 16-bit, we need only low 16 bits of summ.
                let summ = see.summ as u32;
                let r = summ >> see.shift as i32;
                see.summ = (summ - r) as u16;
                *esc_freq = r + (r == 0) as u32;

                SeeSource::Table(base_context_idx, see_table_hash)
            } else {
                *esc_freq = 1;
                SeeSource::Dummy
            }
        }
    }

    unsafe fn calculate_see_table_hash(
        &mut self,
        num_masked: u32,
        num_stats: u32,
    ) -> (usize, usize) {
        unsafe {
            let mc = self.min_context;
            let base_context_idx = self.ns2index[(num_stats + 2) as usize] as usize - 3;

            let suffix_context = self.ptr_of_offset((*mc).suffix as isize).cast::<Context>();
            let suffix_num_stats = suffix_context.as_ref().num_stats as u32;
            let summ_freq = (*mc).union2.summ_freq as u32;

            let freq_distribution_hash = (summ_freq > 11 * (num_stats + 1)) as usize;
            let context_hierarchy_hash =
                2 * ((2 * num_stats) < (suffix_num_stats + num_masked)) as usize;
            let symbol_characteristics_hash = (*mc).flags as usize;

            let see_table_hash =
                freq_distribution_hash + context_hierarchy_hash + symbol_characteristics_hash;

            (base_context_idx, see_table_hash)
        }
    }

    fn get_see(&mut self, see_source: SeeSource) -> &mut See {
        match see_source {
            SeeSource::Dummy => &mut self.dummy_see,
            SeeSource::Table(i, k) => &mut self.see[i][k],
        }
    }

    unsafe fn next_context(&mut self) {
        unsafe {
            let c = self.get_successor(self.found_state);
            if self.order_fall == 0 && c.addr().get() >= self.units_start.addr() {
                self.min_context = c.as_ptr();
                self.max_context = self.min_context;
            } else {
                self.update_model();
            };
        }
    }

    unsafe fn update1(&mut self) {
        unsafe {
            let mut s = self.found_state;
            let mut freq = (*s).freq as u32;
            freq += 4;
            (*self.min_context).union2.summ_freq += 4;
            (*s).freq = freq as u8;
            if freq > (*s.offset(-1)).freq as u32 {
                Self::swap_states(s, &mut *s.offset(-1));
                s = s.offset(-1);
                self.found_state = s;
                if freq > MAX_FREQ as u32 {
                    self.rescale();
                }
            }
            self.next_context();
        }
    }

    unsafe fn update1_0(&mut self) {
        unsafe {
            let s = self.found_state;
            let mc = self.min_context;
            let mut freq = (*s).freq as u32;
            let summ_freq = (*mc).union2.summ_freq as u32;
            self.prev_success = (2 * freq >= summ_freq) as u32; // Ppmd8 (>=)
            self.run_length += self.prev_success as i32;
            (*mc).union2.summ_freq = (summ_freq + 4) as u16;
            freq += 4;
            (*s).freq = freq as u8;
            if freq > MAX_FREQ as u32 {
                self.rescale();
            }
            self.next_context();
        }
    }

    unsafe fn update2(&mut self) {
        unsafe {
            let s = self.found_state;
            let mut freq = (*s).freq as u32;
            freq += 4;
            self.run_length = self.init_rl;
            (*self.min_context).union2.summ_freq += 4;
            (*s).freq = freq as u8;
            if freq > MAX_FREQ as u32 {
                self.rescale();
            }
            self.update_model();
        }
    }
}

impl Ppmd8<Decoder> {
    pub(crate) fn new_decoder(
        input: IByteInPtr,
        mem_size: u32,
        max_order: u32,
        restore_method: RestoreMethod,
    ) -> Result<Ppmd8<Decoder>, Error> {
        Self::construct(
            StreamUnion { input },
            Decoder {},
            mem_size,
            max_order,
            restore_method,
        )
    }
}

impl Ppmd8<Encoder> {
    pub(crate) fn new_encoder(
        output: IByteOutPtr,
        mem_size: u32,
        max_order: u32,
        restore_method: RestoreMethod,
    ) -> Result<Ppmd8<Encoder>, Error> {
        Self::construct(
            StreamUnion { output },
            Encoder {},
            mem_size,
            max_order,
            restore_method,
        )
    }
}
