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
                let mut s =
                    self.base.offset((*self.min_context).union4.stats as isize) as *mut State;
                let mut summ_freq = (*self.min_context).union2.summ_freq as u32;
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
                            self.code = self.code << 8 as i32
                                | ((*self.stream.input).read)(self.stream.input) as u32;
                            self.range <<= 8 as i32;
                            self.low <<= 8 as i32;
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
                    return -(2 as i32);
                }
                hi_cnt = hi_cnt.wrapping_sub(count);
                self.rd_decode(hi_cnt, summ_freq.wrapping_sub(hi_cnt));

                char_mask = [u8::MAX; 256];

                let mut s2: *mut State = (self.base)
                    .offset((*self.min_context).union4.stats as isize)
                    as *mut u8 as *mut State;
                *(char_mask.as_mut_ptr() as *mut u8).offset((*s).symbol as isize) = 0 as i32 as u8;
                loop {
                    let sym0 = (*s2.offset(0 as i32 as isize)).symbol as u32;
                    let sym1 = (*s2.offset(1 as i32 as isize)).symbol as u32;
                    s2 = s2.offset(2 as i32 as isize);
                    *(char_mask.as_mut_ptr() as *mut u8).offset(sym0 as isize) = 0 as i32 as u8;
                    *(char_mask.as_mut_ptr() as *mut u8).offset(sym1 as isize) = 0 as i32 as u8;
                    if !(s2 < s) {
                        break;
                    }
                }
            } else {
                let s_0: *mut State = &mut (*self.min_context).union2 as *mut Union2 as *mut State;
                let prob: *mut u16 = &mut *(*(self.bin_summ).as_mut_ptr().offset(
                    *(self.ns2index).as_mut_ptr().offset(
                        ((*(&mut (*self.min_context).union2 as *mut Union2 as *mut State)).freq
                            as usize)
                            .wrapping_sub(1 as i32 as usize) as isize,
                    ) as isize,
                ))
                .as_mut_ptr()
                .offset(
                    (self.prev_success)
                        .wrapping_add((self.run_length >> 26 as i32 & 0x20 as i32) as u32)
                        .wrapping_add(
                            *(self.ns2bs_index).as_mut_ptr().offset(
                                (*((self.base).offset((*self.min_context).suffix as isize)
                                    as *mut u8 as *mut Context))
                                    .num_stats as isize,
                            ) as u32,
                        )
                        .wrapping_add((*self.min_context).flags as i32 as u32)
                        as isize,
                ) as *mut u16;
                let mut pr = *prob as u32;
                let size0 = (self.range >> 14 as i32) * pr;
                pr = pr.wrapping_sub(
                    pr.wrapping_add(((1 as i32) << 7 as i32 - 2 as i32) as u32) >> 7 as i32,
                );
                if self.code < size0 {
                    *prob = pr.wrapping_add(((1 as i32) << 7 as i32) as u32) as u16;
                    self.range = size0;
                    while self.low ^ self.low.wrapping_add(self.range) < K_TOP_VALUE
                        || self.range < K_BOT_VALUE && {
                            self.range =
                                (0 as i32 as u32).wrapping_sub(self.low) & (K_BOT_VALUE - 1);
                            1 as i32 != 0
                        }
                    {
                        self.code = self.code << 8 as i32
                            | ((*self.stream.input).read)(self.stream.input) as u32;
                        self.range <<= 8 as i32;
                        self.low <<= 8 as i32;
                    }
                    let freq = (*s_0).freq as u32;
                    let c = self.get_successor(s_0);
                    let sym = (*s_0).symbol;
                    self.found_state = s_0;
                    self.prev_success = 1 as i32 as u32;
                    self.run_length += 1;
                    self.run_length;
                    (*s_0).freq = freq.wrapping_add((freq < 196 as i32 as u32) as i32 as u32) as u8;
                    if self.order_fall == 0 as i32 as u32
                        && c.as_ptr() as *const u8 >= self.units_start as *const u8
                    {
                        self.min_context = c.as_ptr();
                        self.max_context = self.min_context;
                    } else {
                        self.update_model();
                    }
                    return sym as i32;
                }
                *prob = pr as u16;
                self.init_esc = self.exp_escape[(pr >> 10 as i32) as usize] as u32;
                self.low = (self.low).wrapping_add(size0);
                self.code = (self.code).wrapping_sub(size0);
                self.range = (self.range
                    & !(((1 as i32) << 7 as i32 + 7 as i32) as u32).wrapping_sub(1 as i32 as u32))
                .wrapping_sub(size0);

                char_mask = [u8::MAX; 256];

                *char_mask.as_mut_ptr().offset(
                    (*(&mut (*self.min_context).union2 as *mut Union2 as *mut State)).symbol
                        as isize,
                ) = 0;
                self.prev_success = 0;
            }
            loop {
                let mut freq_sum = 0;
                let mut count_0 = 0;
                let mut hi_cnt_0 = 0;
                let mut freq_sum2 = 0;
                let mut num_masked = 0;
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
                num_masked = (*mc).num_stats as u32;
                loop {
                    self.order_fall = (self.order_fall).wrapping_add(1);
                    self.order_fall;
                    if (*mc).suffix == 0 {
                        return -(1 as i32);
                    }
                    mc = (self.base).offset((*mc).suffix as isize) as *mut Context;
                    if !((*mc).num_stats as u32 == num_masked) {
                        break;
                    }
                }
                let mut s_1 = (self.base).offset((*mc).union4.stats as isize) as *mut State;
                let mut num = ((*mc).num_stats as u32).wrapping_add(1 as i32 as u32);
                let mut num2 = num.wrapping_div(2 as i32 as u32);
                num &= 1 as i32 as u32;
                hi_cnt_0 = (*s_1).freq as u32
                    & *char_mask.as_mut_ptr().offset((*s_1).symbol as isize) as u32
                    & (0 as i32 as u32).wrapping_sub(num);
                let mut s = s_1.offset(num as isize);
                self.min_context = mc;
                loop {
                    let sym0_0 = (*s.offset(0 as i32 as isize)).symbol as u32;
                    let sym1_0 = (*s.offset(1 as i32 as isize)).symbol as u32;
                    s = s.offset(2 as i32 as isize);
                    hi_cnt_0 = hi_cnt_0.wrapping_add(
                        (*s.offset(-(2 as i32) as isize)).freq as u32
                            & *char_mask.as_mut_ptr().offset(sym0_0 as isize) as u32,
                    );
                    hi_cnt_0 = hi_cnt_0.wrapping_add(
                        (*s.offset(-(1 as i32) as isize)).freq as u32
                            & *char_mask.as_mut_ptr().offset(sym1_0 as isize) as u32,
                    );
                    num2 = num2.wrapping_sub(1);
                    if !(num2 != 0) {
                        break;
                    }
                }
                let see_source = self.make_esc_freq(num_masked, &mut freq_sum);
                freq_sum = freq_sum.wrapping_add(hi_cnt_0);
                freq_sum2 = freq_sum;
                if freq_sum2 > self.range {
                    freq_sum2 = self.range;
                }
                self.range /= freq_sum2;
                count_0 = self.code / self.range;
                if count_0 < hi_cnt_0 {
                    s = (self.base).offset((*self.min_context).union4.stats as isize) as *mut u8
                        as *mut State;
                    hi_cnt_0 = count_0;
                    loop {
                        count_0 = count_0.wrapping_sub(
                            (*s).freq as u32
                                & *(char_mask.as_mut_ptr() as *mut u8).offset((*s).symbol as isize)
                                    as u32,
                        );
                        s = s.offset(1);
                        if (count_0 as i32) < 0 as i32 {
                            break;
                        }
                    }
                    s = s.offset(-1);
                    self.rd_decode(
                        hi_cnt_0
                            .wrapping_sub(count_0)
                            .wrapping_sub((*s).freq as u32),
                        (*s).freq as u32,
                    );
                    while self.low ^ self.low.wrapping_add(self.range) < K_TOP_VALUE
                        || self.range < K_BOT_VALUE && {
                            self.range = (0 as i32 as u32).wrapping_sub(self.low) & K_BOT_VALUE - 1;
                            1 as i32 != 0
                        }
                    {
                        self.code = self.code << 8 as i32
                            | ((*self.stream.input).read)(self.stream.input) as u32;
                        self.range <<= 8 as i32;
                        self.low <<= 8 as i32;
                    }

                    let see = self.get_see(see_source);
                    if ((*see).shift as i32) < 7 as i32 && {
                        (*see).count = ((*see).count).wrapping_sub(1);
                        (*see).count as i32 == 0 as i32
                    } {
                        (*see).summ = (((*see).summ as i32) << 1 as i32) as u16;
                        let fresh0 = (*see).shift;
                        (*see).shift = ((*see).shift).wrapping_add(1);
                        (*see).count = ((3 as i32) << fresh0 as i32) as u8;
                    }
                    self.found_state = s;
                    let sym = (*s).symbol;
                    self.update2();
                    return sym as i32;
                }
                if count_0 >= freq_sum2 {
                    return -(2 as i32);
                }
                self.rd_decode(hi_cnt_0, freq_sum2.wrapping_sub(hi_cnt_0));
                let see = self.get_see(see_source);
                (*see).summ = ((*see).summ as u32).wrapping_add(freq_sum) as u16;
                s = (self.base).offset((*self.min_context).union4.stats as isize) as *mut u8
                    as *mut State;
                let s2 = s
                    .offset((*self.min_context).num_stats as i32 as isize)
                    .offset(1 as i32 as isize);
                loop {
                    *char_mask.as_mut_ptr().offset((*s).symbol as isize) = 0;
                    s = s.offset(1);
                    s;
                    if !(s != s2) {
                        break;
                    }
                }
            }
        }
    }
}
