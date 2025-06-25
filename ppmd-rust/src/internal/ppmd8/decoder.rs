use super::*;

impl Ppmd8<Decoder> {
    pub unsafe fn init_range_dec(&mut self) -> i32 {
        unsafe {
            self.code = 0;
            self.range = 0xFFFFFFFF;
            self.low = 0;

            for _ in 0..4 {
                self.code = self.code << 8 | ((*self.stream.input).read)(self.stream.input) as u32;
            }

            (self.code < 0xFFFFFFFF) as i32
        }
    }

    #[inline(always)]
    unsafe fn rd_decode(&mut self, mut start: u32, size: u32) {
        start *= self.range;
        self.low += start;
        self.code -= start;
        self.range *= size;
    }

    pub unsafe fn decode_symbol(&mut self) -> i32 {
        unsafe {
            let mut char_mask: [u8; 256];
            if (*self.min_context).num_stats != 0 {
                let mut s = self
                    .base
                    .offset((*self.min_context).data.multi_state.stats as isize)
                    .cast::<State>();
                let mut summ_freq = (*self.min_context).data.multi_state.summ_freq as u32;
                if summ_freq > self.range {
                    summ_freq = self.range;
                }
                self.range /= summ_freq;
                let mut count = self.code / self.range;
                let mut hi_cnt = count;
                count = count.wrapping_sub((*s).freq as u32);
                if (count as i32) < 0 {
                    self.rd_decode(0, (*s).freq as u32);
                    while self.low ^ self.low.wrapping_add(self.range) < K_TOP_VALUE
                        || self.range < K_BOT_VALUE && {
                            self.range = 0u32.wrapping_sub(self.low) & (K_BOT_VALUE - 1);
                            1 != 0
                        }
                    {
                        self.code =
                            self.code << 8 | ((*self.stream.input).read)(self.stream.input) as u32;
                        self.range <<= 8;
                        self.low <<= 8;
                    }
                    self.found_state = s;
                    let sym = (*s).symbol;
                    self.update1_0();
                    return sym as i32;
                }
                self.prev_success = 0;
                let mut i = (*self.min_context).num_stats as u32;
                loop {
                    s = s.offset(1);
                    count = count.wrapping_sub((*s).freq as u32);
                    if (count as i32) < 0 {
                        self.rd_decode(
                            hi_cnt.wrapping_sub(count).wrapping_sub((*s).freq as u32),
                            (*s).freq as u32,
                        );
                        while self.low ^ self.low.wrapping_add(self.range) < K_TOP_VALUE
                            || self.range < K_BOT_VALUE && {
                                self.range = 0u32.wrapping_sub(self.low) & (K_BOT_VALUE - 1);
                                1 != 0
                            }
                        {
                            self.code = self.code << 8
                                | ((*self.stream.input).read)(self.stream.input) as u32;
                            self.range <<= 8;
                            self.low <<= 8;
                        }
                        self.found_state = s;
                        let sym = (*s).symbol;
                        self.update1();
                        return sym as i32;
                    }
                    i = i.wrapping_sub(1);
                    if !(i != 0) {
                        break;
                    }
                }
                if hi_cnt >= summ_freq {
                    return -2;
                }
                hi_cnt = hi_cnt.wrapping_sub(count);
                self.rd_decode(hi_cnt, summ_freq.wrapping_sub(hi_cnt));

                char_mask = [u8::MAX; 256];

                let mut s2 = self
                    .base
                    .offset((*self.min_context).data.multi_state.stats as isize)
                    .cast::<State>();
                *char_mask.as_mut_ptr().offset((*s).symbol as isize) = 0;
                loop {
                    let sym0 = (*s2.offset(0)).symbol as u32;
                    let sym1 = (*s2.offset(1)).symbol as u32;
                    s2 = s2.offset(2);
                    *char_mask.as_mut_ptr().offset(sym0 as isize) = 0;
                    *char_mask.as_mut_ptr().offset(sym1 as isize) = 0;
                    if !(s2 < s) {
                        break;
                    }
                }
            } else {
                let s = &mut (*self.min_context).data.single_state;
                let prob =
                    &mut *(*self
                        .bin_summ
                        .as_mut_ptr()
                        .offset(*self.ns2index.as_mut_ptr().offset(
                            ((*self.min_context).data.single_state.freq as usize).wrapping_sub(1)
                                as isize,
                        ) as isize))
                    .as_mut_ptr()
                    .offset(
                        self.prev_success
                            .wrapping_add((self.run_length >> 26 & 0x20) as u32)
                            .wrapping_add(
                                *self.ns2bs_index.as_mut_ptr().offset(
                                    (*(self
                                        .base
                                        .offset((*self.min_context).suffix as isize)
                                        .cast::<Context>()))
                                    .num_stats as isize,
                                ) as u32,
                            )
                            .wrapping_add((*self.min_context).flags as i32 as u32)
                            as isize,
                    )
                    .cast::<u16>();
                let mut pr = *prob as u32;
                let size = (self.range >> 14) * pr;
                pr = pr.wrapping_sub(pr.wrapping_add((1 << (7 - 2)) as u32) >> 7);
                if self.code < size {
                    *prob = pr.wrapping_add((1 << 7) as u32) as u16;
                    self.range = size;
                    while self.low ^ self.low.wrapping_add(self.range) < K_TOP_VALUE
                        || self.range < K_BOT_VALUE && {
                            self.range = 0u32.wrapping_sub(self.low) & (K_BOT_VALUE - 1);
                            1 != 0
                        }
                    {
                        self.code =
                            self.code << 8 | ((*self.stream.input).read)(self.stream.input) as u32;
                        self.range <<= 8;
                        self.low <<= 8;
                    }
                    let freq = (*s).freq as u32;
                    let c = self.get_successor(s);
                    let sym = (*s).symbol;
                    self.found_state = s;
                    self.prev_success = 1;
                    self.run_length += 1;
                    (*s).freq = freq.wrapping_add((freq < 196) as u32) as u8;
                    if self.order_fall == 0 && c.addr() >= self.units_start.addr() {
                        self.min_context = c;
                        self.max_context = self.min_context;
                    } else {
                        self.update_model();
                    }
                    return sym as i32;
                }
                *prob = pr as u16;
                self.init_esc = self.exp_escape[(pr >> 10) as usize] as u32;
                self.low = self.low.wrapping_add(size);
                self.code = self.code.wrapping_sub(size);
                self.range =
                    (self.range & !((1 << (7 + 7)) as u32).wrapping_sub(1)).wrapping_sub(size);

                char_mask = [u8::MAX; 256];

                *char_mask
                    .as_mut_ptr()
                    .offset((*self.min_context).data.single_state.symbol as isize) = 0;
                self.prev_success = 0;
            }
            loop {
                let mut freq_sum = 0;
                while self.low ^ self.low.wrapping_add(self.range) < K_TOP_VALUE
                    || self.range < K_BOT_VALUE && {
                        self.range = 0u32.wrapping_sub(self.low) & (K_BOT_VALUE - 1);
                        1 != 0
                    }
                {
                    self.code =
                        self.code << 8 | ((*self.stream.input).read)(self.stream.input) as u32;
                    self.range <<= 8;
                    self.low <<= 8;
                }
                let mut mc = self.min_context;
                let num_masked = (*mc).num_stats as u32;
                loop {
                    self.order_fall = self.order_fall.wrapping_add(1);
                    if (*mc).suffix == 0 {
                        return -1;
                    }
                    mc = self.base.offset((*mc).suffix as isize).cast();
                    if !((*mc).num_stats as u32 == num_masked) {
                        break;
                    }
                }
                let s = self
                    .ptr_of_offset((*mc).data.multi_state.stats)
                    .cast::<State>();
                let mut num = ((*mc).num_stats as u32).wrapping_add(1);
                let mut num2 = num.wrapping_div(2);
                num &= 1;
                let mut hi_cnt = (*s).freq as u32
                    & *char_mask.as_mut_ptr().offset((*s).symbol as isize) as u32
                    & 0u32.wrapping_sub(num);
                let mut s = s.offset(num as isize);
                self.min_context = mc;
                loop {
                    let sym0 = (*s.offset(0)).symbol as u32;
                    let sym1 = (*s.offset(1)).symbol as u32;
                    s = s.offset(2);
                    hi_cnt = hi_cnt.wrapping_add(
                        (*s.offset(-2)).freq as u32
                            & *char_mask.as_mut_ptr().offset(sym0 as isize) as u32,
                    );
                    hi_cnt = hi_cnt.wrapping_add(
                        (*s.offset(-1)).freq as u32
                            & *char_mask.as_mut_ptr().offset(sym1 as isize) as u32,
                    );
                    num2 = num2.wrapping_sub(1);
                    if !(num2 != 0) {
                        break;
                    }
                }
                let see_source = self.make_esc_freq(num_masked, &mut freq_sum);
                freq_sum = freq_sum.wrapping_add(hi_cnt);
                let mut freq_sum2 = freq_sum;
                if freq_sum2 > self.range {
                    freq_sum2 = self.range;
                }
                self.range /= freq_sum2;
                let mut count = self.code / self.range;
                if count < hi_cnt {
                    s = self
                        .ptr_of_offset((*self.min_context).data.multi_state.stats)
                        .cast();
                    hi_cnt = count;
                    loop {
                        count = count.wrapping_sub(
                            (*s).freq as u32
                                & *(char_mask.as_mut_ptr()).offset((*s).symbol as isize) as u32,
                        );
                        s = s.offset(1);
                        if (count as i32) < 0 as i32 {
                            break;
                        }
                    }
                    s = s.offset(-1);
                    self.rd_decode(
                        hi_cnt.wrapping_sub(count).wrapping_sub((*s).freq as u32),
                        (*s).freq as u32,
                    );
                    while self.low ^ self.low.wrapping_add(self.range) < K_TOP_VALUE
                        || self.range < K_BOT_VALUE && {
                            self.range = 0u32.wrapping_sub(self.low) & (K_BOT_VALUE - 1);
                            1 != 0
                        }
                    {
                        self.code =
                            self.code << 8 | ((*self.stream.input).read)(self.stream.input) as u32;
                        self.range <<= 8;
                        self.low <<= 8;
                    }

                    let see = self.get_see(see_source);
                    if (see.shift as i32) < 7 && {
                        see.count = see.count.wrapping_sub(1);
                        see.count as i32 == 0
                    } {
                        see.summ = ((see.summ as i32) << 1) as u16;
                        let fresh0 = see.shift;
                        see.shift = see.shift.wrapping_add(1);
                        see.count = (3 << fresh0 as i32) as u8;
                    }
                    self.found_state = s;
                    let sym = (*s).symbol;
                    self.update2();
                    return sym as i32;
                }
                if count >= freq_sum2 {
                    return -2;
                }
                self.rd_decode(hi_cnt, freq_sum2.wrapping_sub(hi_cnt));
                let see = self.get_see(see_source);
                see.summ = (see.summ as u32).wrapping_add(freq_sum) as u16;
                s = self
                    .ptr_of_offset((*self.min_context).data.multi_state.stats)
                    .cast();
                let s2 = s.offset((*self.min_context).num_stats as isize).offset(1);
                loop {
                    *char_mask.as_mut_ptr().offset((*s).symbol as isize) = 0;
                    s = s.offset(1);
                    if !(s != s2) {
                        break;
                    }
                }
            }
        }
    }
}
