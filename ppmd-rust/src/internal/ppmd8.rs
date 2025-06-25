mod decoder;
mod encoder;
mod range_coding;

use std::{
    alloc::{Layout, alloc, dealloc},
    ptr::{NonNull, write_bytes},
};

use super::{PPMD_BIN_SCALE, PPMD_NUM_INDEXES, PPMD_PERIOD_BITS};
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

#[derive(Copy, Clone, Default)]
#[repr(C, packed)]
struct State {
    symbol: u8,
    freq: u8,
    successor_0: u16,
    successor_1: u16,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
struct MultiState {
    summ_freq: u16,
    stats: u32,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
union ContextData {
    single_state: State,
    multi_state: MultiState,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct Context {
    num_stats: u8,
    flags: u8,
    data: ContextData,
    suffix: u32,
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

    unsafe fn ptr_of_offset(&self, offset: u32) -> *mut u8 {
        unsafe { self.memory_ptr.offset(offset as isize).as_ptr() }
    }

    unsafe fn offset_for_ptr(&self, ptr: *mut u8) -> u32 {
        unsafe {
            let offset = ptr.offset_from(self.memory_ptr.as_ptr());
            u32::try_from(offset).expect("Failed to convert ptr to offset")
        }
    }

    unsafe fn insert_node(&mut self, node: *mut u8, index: u32) {
        unsafe {
            {
                let node = node.cast::<Node>();
                (*(node)).stamp = EMPTY_NODE;
                (*(node)).next = self.free_list[index as usize];
                (*(node)).nu = self.index2units[index as usize] as u32;
            }
            self.free_list[index as usize] = self.offset_for_ptr(node);
            self.stamps[index as usize] += 1;
        }
    }

    unsafe fn remove_node(&mut self, index: u32) -> *mut u8 {
        unsafe {
            let node = self
                .ptr_of_offset(self.free_list[index as usize])
                .cast::<Node>();
            self.free_list[index as usize] = (*node).next;
            self.stamps[index as usize] -= 1;
            node.cast()
        }
    }

    unsafe fn split_block(&mut self, mut ptr: *mut u8, old_index: u32, new_index: u32) {
        unsafe {
            let nu = self.index2units[old_index as usize] as u32
                - self.index2units[new_index as usize] as u32;
            ptr = ptr.offset(self.index2units[new_index as usize] as isize * UNIT_SIZE);
            let mut i = self.units2index[(nu as usize) - 1] as u32;
            if self.index2units[i as usize] as u32 != nu {
                i -= 1;
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
                (*(self.lo_unit.cast::<Node>())).stamp = 0;
            }
            let mut prev = &mut n;
            let mut i = 0;
            while i < PPMD_NUM_INDEXES {
                let mut next = self.free_list[i as usize];
                self.free_list[i as usize] = 0;
                while next != 0 {
                    let node = self.ptr_of_offset(next).cast::<Node>();
                    let mut nu = (*node).nu;
                    *prev = next;
                    next = (*node).next;
                    if nu != 0 {
                        prev = &mut (*node).next;
                        loop {
                            let node2 = node.offset(nu as isize);
                            if (*node2).stamp != EMPTY_NODE {
                                break;
                            }
                            nu += (*node2).nu;
                            (*node).nu = nu;
                            (*node2).nu = 0;
                        }
                    }
                }
                i = i.wrapping_add(1);
            }
            *prev = 0;
            while n != 0 {
                let mut node = self.ptr_of_offset(n).cast::<Node>();
                let mut nu = (*node).nu;
                n = (*node).next;
                if nu == 0 {
                    continue;
                }
                while nu > 128 {
                    self.insert_node(node.cast(), PPMD_NUM_INDEXES - 1);
                    nu = nu.wrapping_sub(128);
                    node = node.offset(128);
                }
                let mut i_0 = self.units2index[(nu as usize) - 1] as u32;
                if self.index2units[i_0 as usize] as u32 != nu {
                    i_0 = i_0.wrapping_sub(1);
                    let k = self.index2units[i_0 as usize] as u32;
                    self.insert_node(
                        node.offset(k as isize).cast(),
                        nu.wrapping_sub(k).wrapping_sub(1),
                    );
                }
                self.insert_node(node.cast(), i_0);
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
                    let us = self.units_start;
                    self.glue_count = (self.glue_count).wrapping_sub(1);
                    self.glue_count;
                    return if us.offset_from(self.text) as u32 > num_bytes {
                        self.units_start = us.offset(-(num_bytes as isize));
                        self.units_start
                    } else {
                        std::ptr::null_mut()
                    };
                }
                if self.free_list[i as usize] != 0 {
                    break;
                }
            }
            let block = self.remove_node(i);
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
            let lo = self.lo_unit;
            if self.hi_unit.offset_from(lo) as u32 >= num_bytes {
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
                return old_ptr.cast();
            }
            if self.free_list[i1 as usize] != 0 {
                let ptr = self.remove_node(i1);
                let mut d = ptr.cast::<u32>();
                let mut z = old_ptr.cast::<u32>();
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
                self.insert_node(old_ptr.cast(), i0);
                return ptr;
            }
            self.split_block(old_ptr.cast(), i0, i1);
            old_ptr.cast()
        }
    }

    unsafe fn free_units(&mut self, ptr: *mut u8, nu: u32) {
        unsafe {
            self.insert_node(
                ptr,
                self.units2index[(nu as usize).wrapping_sub(1 as i32 as usize)] as u32,
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
                (*(self.lo_unit.cast::<Node>())).stamp = 0;
            }
            let mut node = self.units_start.cast::<Node>();
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
            self.units_start = node.cast();
            let mut i = 0;
            while i < PPMD_NUM_INDEXES {
                let mut cnt = count[i as usize];
                if !(cnt == 0) {
                    let mut prev = &mut *(self.free_list)
                        .as_mut_ptr()
                        .offset(i as isize)
                        .cast::<u32>();
                    let mut n = *prev;
                    self.stamps[i as usize] = (self.stamps[i as usize]).wrapping_sub(cnt);
                    loop {
                        let node = self.ptr_of_offset(n).cast::<Node>();
                        n = (*node).next;
                        if (*node).stamp != 0 {
                            prev = &mut (*node).next;
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

    unsafe fn get_successor(&mut self, s: *mut State) -> *mut Context {
        unsafe {
            self.ptr_of_offset((*s).successor_0 as u32 | ((*s).successor_1 as u32) << 16)
                .cast()
        }
    }

    #[inline(never)]
    unsafe fn restart_model(&mut self) {
        unsafe {
            self.free_list = [0; 38];

            self.stamps = [0; 38];
            self.text = self.ptr_of_offset(self.align_offset);
            self.hi_unit = self.text.offset(self.size as isize);
            self.units_start = self
                .hi_unit
                .offset(-(self.size as isize / 8 / UNIT_SIZE * 7 * UNIT_SIZE));
            self.lo_unit = self.units_start;
            self.glue_count = 0;

            self.order_fall = self.max_order;
            self.init_rl = -(if self.max_order < 12 {
                self.max_order as i32
            } else {
                12
            }) - 1;
            self.run_length = self.init_rl;
            self.prev_success = 0;

            self.hi_unit = self.hi_unit.offset(-UNIT_SIZE);
            let mc = self.hi_unit.cast::<Context>();
            let s = self.lo_unit.cast::<State>();

            self.lo_unit = self.lo_unit.offset((256 / 2) * UNIT_SIZE);
            self.min_context = mc;
            self.max_context = self.min_context;
            self.found_state = s;

            {
                (*mc).flags = 0;
                (*mc).num_stats = (256 - 1) as u8;
                (*mc).data.multi_state.summ_freq = (256 + 1) as u16;
                (*mc).data.multi_state.stats = self.offset_for_ptr(s.cast());
                (*mc).suffix = 0;
            }

            (0..256).for_each(|i| {
                let s = s.offset(i);
                (*s).symbol = i as u8;
                (*s).freq = 1;
                Self::set_successor(s, 0);
            });

            let mut i = 0;
            (0..25).for_each(|m| {
                while self.ns2index[i as usize] as usize == m {
                    i += 1;
                }

                (0..8).for_each(|k| {
                    let val = PPMD_BIN_SCALE - (K_INIT_BIN_ESC[k] as u32) / (i + 1);

                    (0..64).step_by(8).for_each(|r| {
                        self.bin_summ[m][k + r] = val as u16;
                    });
                });
            });

            let mut i = 0;
            (0..24).for_each(|m| {
                while self.ns2index[(i + 3) as usize] as usize == m + 3 {
                    i += 1;
                }

                let summ = (2 * i + 5) << (PPMD_PERIOD_BITS - 4);

                (0..32).for_each(|k| {
                    let see = &mut self.see[m][k];
                    see.summ = summ as u16;
                    see.shift = (PPMD_PERIOD_BITS - 4) as u8;
                    see.count = 7;
                });
            });

            self.dummy_see.summ = 0;
            self.dummy_see.shift = PPMD_PERIOD_BITS as u8;
            self.dummy_see.count = 64;
        }
    }

    unsafe fn refresh(&mut self, ctx: *mut Context, old_nu: u32, mut scale: u32) {
        unsafe {
            let mut i = (*ctx).num_stats as u32;

            let mut s = self
                .shrink_units(
                    self.ptr_of_offset((*ctx).data.multi_state.stats)
                        .cast::<State>(),
                    old_nu,
                    (i + 2) >> 1,
                )
                .cast::<State>();
            (*ctx).data.multi_state.stats = self.offset_for_ptr(s.cast());
            scale |= ((*ctx).data.multi_state.summ_freq as u32 >= 1 << 15) as u32;
            let mut flags = ((*s).symbol as u32).wrapping_add(0xC0);
            let mut freq = (*s).freq as u32;
            let mut esc_freq = ((*ctx).data.multi_state.summ_freq as u32).wrapping_sub(freq);
            freq = freq.wrapping_add(scale) >> scale;
            let mut sum_freq = freq;
            (*s).freq = freq as u8;
            loop {
                s = s.offset(1);
                let mut freq = (*s).freq as u32;
                esc_freq = esc_freq.wrapping_sub(freq);
                freq = freq.wrapping_add(scale) >> scale;
                sum_freq = sum_freq.wrapping_add(freq);
                (*s).freq = freq as u8;
                flags |= ((*s).symbol as u32).wrapping_add(0xC0);
                i = i.wrapping_sub(1);
                if !(i != 0) {
                    break;
                }
            }
            (*ctx).data.multi_state.summ_freq =
                sum_freq.wrapping_add(esc_freq.wrapping_add(scale) >> scale) as u16;
            (*ctx).flags = ((*ctx).flags as u32
                & ((1 << 4) as u32).wrapping_add(((1 << 2) as u32).wrapping_mul(scale)))
            .wrapping_add(flags >> (8 - 3) & (1 << 3) as u32) as u8;
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

            if ns == 0 {
                let s = &mut (*ctx).data.single_state as *mut State;
                let mut successor = (*s).successor_0 as u32 | ((*s).successor_1 as u32) << 16;
                if self.ptr_of_offset(successor) >= self.units_start {
                    if order < self.max_order {
                        successor = self
                            .cut_off(self.ptr_of_offset(successor).cast(), order.wrapping_add(1));
                    } else {
                        successor = 0;
                    }
                    Self::set_successor(s, successor);
                    if successor != 0 || order <= 9 {
                        return self.offset_for_ptr(ctx.cast());
                    }
                }
                self.special_free_unit(ctx.cast());
                return 0;
            }
            let nu = (ns as u32).wrapping_add(2) >> 1;
            let index = self.units2index[(nu as usize) - 1] as u32;
            let mut stats = self
                .ptr_of_offset((*ctx).data.multi_state.stats)
                .cast::<State>();
            if stats.cast::<u8>().offset_from(self.units_start) as u32 <= (1 << 14) as u32
                && (*ctx).data.multi_state.stats <= self.free_list[index as usize]
            {
                let ptr = self.remove_node(index);
                (*ctx).data.multi_state.stats = self.offset_for_ptr(ptr.cast());
                let mut d = ptr.cast::<u32>();
                let mut z = stats.cast::<u32>();
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
                if stats.addr() != self.units_start.addr() {
                    self.insert_node(stats.cast(), index);
                } else {
                    self.units_start = self.units_start.offset(
                        (self.index2units[index as usize] as u32 * UNIT_SIZE as u32) as isize,
                    );
                }
                stats = ptr.cast();
            }
            let mut s = stats.offset(ns as isize);
            loop {
                let successor_0 = (*s).successor_0 as u32 | ((*s).successor_1 as u32) << 16;
                if self.ptr_of_offset(successor_0).addr() < self.units_start.addr() {
                    let fresh = ns;
                    ns -= 1;
                    let s2 = stats.offset(fresh as u32 as isize);
                    if order != 0 {
                        if s != s2 {
                            *s = *s2;
                        }
                    } else {
                        Self::swap_states(s, s2);
                        Self::set_successor(s2, 0);
                    }
                } else if order < self.max_order {
                    Self::set_successor(
                        s,
                        self.cut_off(self.ptr_of_offset(successor_0).cast::<Context>(), order + 1),
                    );
                } else {
                    Self::set_successor(s, 0);
                }
                s = s.offset(-1);
                if !(s >= stats) {
                    break;
                }
            }
            if ns != (*ctx).num_stats as i32 && order != 0 {
                if ns < 0 {
                    self.free_units(stats.cast(), nu);
                    self.special_free_unit(ctx.cast());
                    return 0;
                }
                (*ctx).num_stats = ns as u8;
                if ns == 0 {
                    let sym = (*stats).symbol;
                    (*ctx).flags = (((*ctx).flags as i32 & 1 << 4) as u32)
                        .wrapping_add((sym as u32 + 0xC0) >> (8 - 3) & (1 << 3))
                        as u8;
                    (*ctx).data.single_state.symbol = sym;
                    (*ctx).data.single_state.freq = ((((*stats).freq as u32) + 11) >> 3) as u8;
                    (*ctx).data.single_state.successor_0 = (*stats).successor_0;
                    (*ctx).data.single_state.successor_1 = (*stats).successor_1;
                    self.free_units(stats.cast(), nu);
                } else {
                    self.refresh(
                        ctx,
                        nu,
                        ((*ctx).data.multi_state.summ_freq as u32 > 16 * (ns as u32)) as u32,
                    );
                }
            }
            self.offset_for_ptr(ctx.cast())
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
            self.text = self.ptr_of_offset(self.align_offset).offset(0);

            let mut s;
            let mut c = self.max_context;
            while c != ctx_error {
                (*c).num_stats -= 1;
                if (*c).num_stats as i32 == 0 {
                    s = self
                        .ptr_of_offset((*c).data.multi_state.stats)
                        .cast::<State>();
                    (*c).flags = ((((*c).flags as i32 & (1 << 4)) as u32)
                        + ((((*s).symbol as u32) + 0xC0) >> (8 - 3) & (1 << 3)))
                        as u8;
                    (*c).data.single_state.symbol = (*s).symbol;
                    (*c).data.single_state.freq = ((((*s).freq as u32) + 11) >> 3) as u8;
                    (*c).data.single_state.successor_0 = (*s).successor_0;
                    (*c).data.single_state.successor_1 = (*s).successor_1;
                    self.special_free_unit(s.cast());
                } else {
                    self.refresh(c, (((*c).num_stats as u32) + 3) >> 1, 0);
                }
                c = self.ptr_of_offset((*c).suffix).cast::<Context>();
            }
            while c != self.min_context {
                if (*c).num_stats as i32 == 0 {
                    (*c).data.single_state.freq =
                        (((*c).data.single_state.freq as u32).wrapping_add(1) >> 1) as u8;
                } else {
                    (*c).data.multi_state.summ_freq =
                        ((*c).data.multi_state.summ_freq as i32 + 4) as u16;
                    if (*c).data.multi_state.summ_freq as i32 > 128 + 4 * (*c).num_stats as i32 {
                        self.refresh(c, ((*c).num_stats as u32).wrapping_add(2) >> 1, 1);
                    }
                }
                c = self.ptr_of_offset((*c).suffix).cast::<Context>();
            }
            if self.restore_method == RestoreMethod::Restart
                || self.get_used_memory() < self.size >> 1
            {
                self.restart_model();
            } else {
                while (*self.max_context).suffix != 0 {
                    self.max_context = self
                        .ptr_of_offset((*self.max_context).suffix)
                        .cast::<Context>();
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
            let mut num_ps = 0u32;
            let mut ps: [*mut State; 17] = [std::ptr::null_mut(); 17];
            if skip == 0 {
                let fresh = num_ps;
                num_ps = num_ps.wrapping_add(1);
                ps[fresh as usize] = self.found_state;
            }
            while (*c).suffix != 0 {
                let mut s;
                c = self.ptr_of_offset((*c).suffix).cast::<Context>();
                if !s1.is_null() {
                    s = s1;
                    s1 = std::ptr::null_mut();
                } else if (*c).num_stats as i32 != 0 {
                    let sym = (*self.found_state).symbol;
                    s = self
                        .ptr_of_offset((*c).data.multi_state.stats)
                        .cast::<State>();
                    while (*s).symbol as i32 != sym as i32 {
                        s = s.offset(1);
                    }
                    if ((*s).freq) < MAX_FREQ - 9 {
                        (*s).freq = (*s).freq.wrapping_add(1);
                        (*s).freq;
                        (*c).data.multi_state.summ_freq =
                            ((*c).data.multi_state.summ_freq).wrapping_add(1);
                        (*c).data.multi_state.summ_freq;
                    }
                } else {
                    s = &mut (*c).data.single_state as *mut State;
                    (*s).freq = ((*s).freq as i32
                        + (((*(self.ptr_of_offset((*c).suffix).cast::<Context>())).num_stats == 0)
                            as i32
                            & (((*s).freq as i32) < 24) as i32))
                        as u8;
                }
                let successor = (*s).successor_0 as u32 | ((*s).successor_1 as u32) << 16;
                if successor != up_branch {
                    c = self.ptr_of_offset(successor).cast::<Context>();
                    if num_ps == 0 {
                        return c;
                    }
                    break;
                } else {
                    let fresh = num_ps;
                    num_ps = num_ps.wrapping_add(1);
                    ps[fresh as usize] = s;
                }
            }

            let new_sym = *(self.ptr_of_offset(up_branch));
            up_branch = up_branch.wrapping_add(1);
            let flags = (((*self.found_state).symbol as u32).wrapping_add(0xC0) >> 8 - 4
                & (1 << 4) as u32)
                .wrapping_add((new_sym as u32).wrapping_add(0xC0) >> 8 - 3 & (1 << 3) as u32)
                as u8;
            let new_freq = if (*c).num_stats as i32 == 0 {
                (*c).data.single_state.freq
            } else {
                let mut s = self
                    .ptr_of_offset((*c).data.multi_state.stats)
                    .cast::<State>();
                while (*s).symbol as i32 != new_sym as i32 {
                    s = s.offset(1);
                }
                let cf = ((*s).freq as u32).wrapping_sub(1);
                let s0 = ((*c).data.multi_state.summ_freq as u32)
                    .wrapping_sub((*c).num_stats as u32)
                    .wrapping_sub(cf);
                1u32.wrapping_add(if 2 * cf <= s0 {
                    (5 * cf > s0) as u32
                } else {
                    cf.wrapping_add(2 * s0).wrapping_sub(3) / s0
                }) as u8
            };
            loop {
                let c1 = if self.hi_unit != self.lo_unit {
                    self.hi_unit = (self.hi_unit).offset(-(12));
                    self.hi_unit.cast::<Context>()
                } else if self.free_list[0] != 0 {
                    self.remove_node(0).cast()
                } else {
                    // TODO simplify with Option
                    let c1: *mut Context = self.alloc_units_rare(0).cast();
                    if c1.is_null() {
                        return std::ptr::null_mut();
                    }
                    c1
                };
                (*c1).flags = flags;
                (*c1).num_stats = 0;
                (*c1).data.single_state.symbol = new_sym;
                (*c1).data.single_state.freq = new_freq;
                Self::set_successor(&mut (*c1).data.single_state as *mut State, up_branch);
                (*c1).suffix = self.offset_for_ptr(c.cast());
                num_ps = num_ps.wrapping_sub(1);
                Self::set_successor(ps[num_ps as usize], self.offset_for_ptr(c1.cast()));
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
            let mut s;
            let c1 = c;
            let up_branch = self.offset_for_ptr(self.text.cast());
            Self::set_successor(self.found_state, up_branch);
            self.order_fall = (self.order_fall).wrapping_add(1);
            loop {
                if !s1.is_null() {
                    c = self.ptr_of_offset((*c).suffix).cast::<Context>();
                    s = s1;
                    s1 = std::ptr::null_mut();
                } else {
                    if (*c).suffix == 0 {
                        return c;
                    }
                    c = self.ptr_of_offset((*c).suffix).cast::<Context>();
                    if (*c).num_stats != 0 {
                        s = self
                            .ptr_of_offset((*c).data.multi_state.stats)
                            .cast::<State>();
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
                            (*c).data.multi_state.summ_freq =
                                ((*c).data.multi_state.summ_freq as i32 + 2) as u16;
                        }
                    } else {
                        s = &mut (*c).data.single_state as *mut State;
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
                let s2 = self.found_state;
                self.found_state = s;
                let successor = self.create_successors(0 as i32, std::ptr::null_mut(), c);
                if successor.is_null() {
                    Self::set_successor(s, 0);
                } else {
                    Self::set_successor(s, self.offset_for_ptr(successor.cast()));
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

            self.ptr_of_offset(successor_0).cast()
        }
    }

    #[inline(never)]
    unsafe fn update_model(&mut self) {
        unsafe {
            let mut max_successor;
            let mut min_successor = (*self.found_state).successor_0 as u32
                | ((*self.found_state).successor_1 as u32) << 16;
            let mut c: *mut Context;
            let f_freq = (*self.found_state).freq as u32;
            let f_symbol = (*self.found_state).symbol;
            let mut s = std::ptr::null_mut();
            if ((*self.found_state).freq) < MAX_FREQ / 4 && (*self.min_context).suffix != 0 {
                c = self.ptr_of_offset((*self.min_context).suffix).cast();
                if (*c).num_stats as i32 == 0 {
                    s = &mut (*c).data.single_state as *mut State;
                    if ((*s).freq as i32) < 32 {
                        (*s).freq = (*s).freq.wrapping_add(1);
                    }
                } else {
                    let sym = (*self.found_state).symbol;
                    s = self
                        .ptr_of_offset((*c).data.multi_state.stats)
                        .cast::<State>();
                    if (*s).symbol as i32 != sym as i32 {
                        loop {
                            s = s.offset(1);
                            if !((*s).symbol as i32 != sym as i32) {
                                break;
                            }
                        }
                        if (*s.offset(0)).freq as i32 >= (*s.offset(-1)).freq as i32 {
                            Self::swap_states(&mut *s.offset(0), &mut *s.offset(-1));
                            s = s.offset(-1);
                        }
                    }
                    if ((*s).freq) < MAX_FREQ - 9 {
                        (*s).freq = ((*s).freq as i32 + 2) as u8;
                        (*c).data.multi_state.summ_freq =
                            ((*c).data.multi_state.summ_freq as i32 + 2) as u16;
                    }
                }
            }
            c = self.max_context;
            if self.order_fall == 0 && min_successor != 0 {
                let cs = self.create_successors(1 as i32, s, self.min_context);
                if cs.is_null() {
                    Self::set_successor(self.found_state, 0);
                    self.restore_model(c);
                    return;
                }
                Self::set_successor(self.found_state, self.offset_for_ptr(cs.cast()));
                self.max_context = cs;
                self.min_context = self.max_context;
                return;
            }
            let mut text = self.text;
            let fresh = text;
            text = text.offset(1);
            *fresh = (*self.found_state).symbol;
            self.text = text;
            if text >= self.units_start {
                self.restore_model(c);
                return;
            }
            max_successor = self.offset_for_ptr(text);
            if min_successor == 0 {
                let cs = self.reduce_order(s, self.min_context);
                if cs.is_null() {
                    self.restore_model(c);
                    return;
                }
                min_successor = self.offset_for_ptr(cs.cast());
            } else if (self.ptr_of_offset(min_successor)).addr() < self.units_start.addr() {
                let cs = self.create_successors(0 as i32, s, self.min_context);
                if cs.is_null() {
                    self.restore_model(c);
                    return;
                }
                min_successor = self.offset_for_ptr(cs.cast());
            }
            self.order_fall = self.order_fall.wrapping_sub(1);
            if self.order_fall == 0 {
                max_successor = min_successor;
                self.text = (self.text).offset(-((self.max_context != self.min_context) as isize));
            }
            let flag = ((f_symbol as u32).wrapping_add(0xC0) >> 8 - 3 & (1 << 3) as u32) as u8;
            let ns = (*self.min_context).num_stats as u32;
            let s0 = ((*self.min_context).data.multi_state.summ_freq as u32)
                .wrapping_sub(ns)
                .wrapping_sub(f_freq);
            while c != self.min_context {
                let mut sum;
                let ns1 = (*c).num_stats as u32;
                if ns1 != 0 {
                    if ns1 & 1 != 0 {
                        let old_nu = ns1.wrapping_add(1) >> 1;
                        let i = self.units2index[(old_nu as usize).wrapping_sub(1 as i32 as usize)]
                            as u32;
                        if i != self.units2index[(old_nu as usize)
                            .wrapping_add(1 as usize)
                            .wrapping_sub(1 as usize)] as u32
                        {
                            let ptr = self.alloc_units(i.wrapping_add(1));
                            if ptr.is_null() {
                                self.restore_model(c);
                                return;
                            }
                            let old_ptr = self.ptr_of_offset((*c).data.multi_state.stats);
                            let mut d = ptr.cast::<u32>();
                            let mut z = old_ptr.cast::<u32>();
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
                            (*c).data.multi_state.stats = self.offset_for_ptr(ptr);
                        }
                    }
                    sum = (*c).data.multi_state.summ_freq as u32;
                    sum = sum.wrapping_add((3u32.wrapping_mul(ns1).wrapping_add(1) < ns) as u32);
                } else {
                    let s = self.alloc_units(0).cast::<State>();
                    if s.is_null() {
                        self.restore_model(c);
                        return;
                    }
                    let mut freq = (*c).data.single_state.freq as u32;
                    (*s).symbol = (*c).data.single_state.symbol;
                    (*s).successor_0 = (*c).data.single_state.successor_0;
                    (*s).successor_1 = (*c).data.single_state.successor_1;
                    (*c).data.multi_state.stats = self.offset_for_ptr(s.cast());
                    if freq < (MAX_FREQ as i32 / 4 - 1) as u32 {
                        freq <<= 1;
                    } else {
                        freq = (MAX_FREQ as i32 - 4) as u32;
                    }
                    (*s).freq = freq as u8;
                    sum = freq
                        .wrapping_add(self.init_esc)
                        .wrapping_add((ns > 2) as u32);
                }
                let s = self
                    .ptr_of_offset((*c).data.multi_state.stats)
                    .cast::<State>()
                    .offset(ns1 as isize)
                    .offset(1);
                let mut cf = 2 * sum.wrapping_add(6) * f_freq;
                let sf = s0.wrapping_add(sum);
                (*s).symbol = f_symbol;
                (*c).num_stats = ns1.wrapping_add(1) as u8;
                Self::set_successor(s, max_successor);
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
                (*c).data.multi_state.summ_freq = sum as u16;
                (*s).freq = cf as u8;
                c = self.ptr_of_offset((*c).suffix).cast::<Context>();
            }
            self.min_context = self.ptr_of_offset(min_successor).cast::<Context>();
            self.max_context = self.min_context;
        }
    }

    #[inline(never)]
    unsafe fn rescale(&mut self) {
        unsafe {
            let stats = self
                .ptr_of_offset((*self.min_context).data.multi_state.stats)
                .cast::<State>();
            let mut s = self.found_state;
            if s != stats {
                let tmp = *s;
                while s != stats {
                    *s.offset(0) = *s.offset(-1);
                    s = s.offset(-1);
                }
                *s = tmp;
            }
            let mut sum_freq = (*s).freq as u32;
            let mut esc_freq = ((*self.min_context).data.multi_state.summ_freq as u32) - sum_freq;
            let adder = (self.order_fall != 0) as u32;
            sum_freq = (sum_freq + 4 + (adder)) >> 1;
            (*s).freq = sum_freq as u8;

            let num_stats = (*self.min_context).num_stats as u32;
            for _ in 0..num_stats {
                s = s.offset(1);
                let mut freq = (*s).freq as u32;
                esc_freq -= freq;
                freq = freq.wrapping_add(adder) >> 1;
                sum_freq += freq;
                (*s).freq = freq as u8;
                if freq > (*s.offset(-1)).freq as u32 {
                    let tmp = *s;
                    let mut s1 = s;
                    loop {
                        *s1.offset(0) = *s1.offset(-1);
                        s1 = s1.offset(-1);
                        if !(s1 != stats && freq > (*s1.offset(-1)).freq as u32) {
                            break;
                        }
                    }
                    *s1 = tmp;
                }
            }
            if (*s).freq as i32 == 0 {
                let mut i = 0;
                loop {
                    i += 1;
                    s = s.offset(-1);
                    if (*s).freq as i32 != 0 {
                        break;
                    }
                }
                esc_freq += i;
                let mc = self.min_context;
                let num_stats = (*mc).num_stats as u32;
                let num_stats_new = num_stats.wrapping_sub(i);
                (*mc).num_stats = num_stats_new as u8;
                let n0 = (num_stats + 2) >> 1;
                if num_stats_new == 0 {
                    let mut freq = 2u32 + ((*stats).freq as u32) + esc_freq - 1 / (esc_freq);
                    if freq > (MAX_FREQ as i32 / 3) as u32 {
                        freq = (MAX_FREQ as i32 / 3) as u32;
                    }
                    (*mc).flags = ((((*mc).flags as i32 & 1 << 4) as u32)
                        + ((((*stats).symbol as u32) + 0xC0) >> (8 - 3) & (1 << 3)))
                        as u8;
                    s = &mut (*mc).data.single_state as *mut State;
                    *s = *stats;
                    (*s).freq = freq as u8;
                    self.found_state = s;
                    self.insert_node(stats.cast(), self.units2index[(n0 as usize) - 1] as u32);
                    return;
                }
                let n1 = (num_stats_new + 2) >> 1;
                if n0 != n1 {
                    let shrunk = self.shrink_units(stats, n0, n1);
                    (*mc).data.multi_state.stats = self.offset_for_ptr(shrunk.cast());
                }
            }
            let mc = self.min_context;
            (*mc).data.multi_state.summ_freq =
                sum_freq.wrapping_add(esc_freq).wrapping_sub(esc_freq >> 1) as u16;
            (*mc).flags = ((*mc).flags as i32 | 1 << 2) as u8;
            self.found_state = self.ptr_of_offset((*mc).data.multi_state.stats).cast();
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

            let suffix_context = self.ptr_of_offset((*mc).suffix).cast::<Context>();
            let suffix_num_stats = (*suffix_context).num_stats as u32;
            let summ_freq = (*mc).data.multi_state.summ_freq as u32;

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
            if self.order_fall == 0 && c.addr() >= self.units_start.addr() {
                self.min_context = c;
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
            (*self.min_context).data.multi_state.summ_freq += 4;
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
            let summ_freq = (*mc).data.multi_state.summ_freq as u32;
            self.prev_success = (2 * freq >= summ_freq) as u32; // Ppmd8 (>=)
            self.run_length += self.prev_success as i32;
            (*mc).data.multi_state.summ_freq = (summ_freq + 4) as u16;
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
            (*self.min_context).data.multi_state.summ_freq += 4;
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
