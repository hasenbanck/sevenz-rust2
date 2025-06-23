use super::*;

impl Ppmd8<Encoder> {
    pub fn init_range_enc(&mut self) {
        self.low = 0;
        self.range = 0xFFFFFFFF;
    }

    pub unsafe fn flush_range_enc(&mut self) {
        unsafe {
            for _ in 0..4 {
                ((*self.stream.output).write)(self.stream.output, (self.low >> 24) as u8);
                self.low <<= 8;
            }
        }
    }

    #[inline(always)]
    fn range_enc_encode(&mut self, start: u32, size: u32, total: u32) {
        self.range /= total;
        self.low += start * self.range;
        self.range *= size;
    }

    pub unsafe fn encode_symbol(&mut self, symbol: i32) {
        unsafe {
            let mut char_mask: [u8; 256];
            if (*self.min_context).num_stats != 0 {
                let mut s =
                    self.base.offset((*self.min_context).union4.stats as isize) as *mut State;
                let mut sum = 0;
                let mut i = 0;
                let mut summ_freq = (*self.min_context).union2.summ_freq as u32;
                if summ_freq > self.range {
                    summ_freq = self.range;
                }
                if (*s).symbol as i32 == symbol {
                    self.range_enc_encode(0, (*s).freq as u32, summ_freq);
                    while self.low ^ (self.low).wrapping_add(self.range) < K_TOP_VALUE
                        || self.range < K_BOT_VALUE && {
                            self.range = 0u32.wrapping_sub(self.low) & (K_BOT_VALUE - 1);
                            1 != 0
                        }
                    {
                        ((*self.stream.output).write)(self.stream.output, (self.low >> 24) as u8);
                        self.range <<= 8;
                        self.low <<= 8;
                    }
                    self.found_state = s;
                    self.update1_0();
                    return;
                }
                self.prev_success = 0;
                sum = (*s).freq as u32;
                i = (*self.min_context).num_stats as u32;
                loop {
                    s = s.offset(1);
                    if (*s).symbol as i32 == symbol {
                        self.range_enc_encode(sum, (*s).freq as u32, summ_freq);
                        while self.low ^ (self.low).wrapping_add(self.range) < K_TOP_VALUE
                            || self.range < K_BOT_VALUE && {
                                self.range = 0u32.wrapping_sub(self.low) & (K_BOT_VALUE - 1);
                                1 != 0
                            }
                        {
                            ((*self.stream.output).write)(
                                self.stream.output,
                                (self.low >> 24) as u8,
                            );
                            self.range <<= 8;
                            self.low <<= 8;
                        }
                        self.found_state = s;
                        self.update1();
                        return;
                    }
                    sum = sum.wrapping_add((*s).freq as u32);
                    i = i.wrapping_sub(1);
                    if !(i != 0) {
                        break;
                    }
                }
                self.range_enc_encode(sum, summ_freq.wrapping_sub(sum), summ_freq);

                char_mask = [u8::MAX; 256];

                let mut s2: *mut State = (self.base)
                    .offset((*self.min_context).union4.stats as isize)
                    as *mut u8 as *mut State;
                *(char_mask.as_mut_ptr() as *mut u8).offset((*s).symbol as isize) = 0 as i32 as u8;
                loop {
                    let sym0: u32 = (*s2.offset(0 as i32 as isize)).symbol as u32;
                    let sym1: u32 = (*s2.offset(1 as i32 as isize)).symbol as u32;
                    s2 = s2.offset(2 as i32 as isize);
                    *(char_mask.as_mut_ptr() as *mut u8).offset(sym0 as isize) = 0 as i32 as u8;
                    *(char_mask.as_mut_ptr() as *mut u8).offset(sym1 as isize) = 0 as i32 as u8;
                    if !(s2 < s) {
                        break;
                    }
                }
            } else {
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
                let s_0: *mut State = &mut (*self.min_context).union2 as *mut Union2 as *mut State;
                let mut pr: u32 = *prob as u32;
                let bound: u32 = (self.range >> 14 as i32) * pr;
                pr = pr.wrapping_sub(
                    pr.wrapping_add(((1 as i32) << 7 as i32 - 2 as i32) as u32) >> 7 as i32,
                );
                if (*s_0).symbol as i32 == symbol {
                    *prob = pr.wrapping_add(((1 as i32) << 7 as i32) as u32) as u16;
                    self.range = bound;
                    while self.low ^ (self.low).wrapping_add(self.range) < K_TOP_VALUE
                        || self.range < K_BOT_VALUE && {
                            self.range =
                                (0 as i32 as u32).wrapping_sub(self.low) & (K_BOT_VALUE - 1);
                            1 as i32 != 0
                        }
                    {
                        ((*self.stream.output).write)(
                            self.stream.output,
                            (self.low >> 24 as i32) as u8,
                        );
                        self.range <<= 8 as i32;
                        self.low <<= 8 as i32;
                    }
                    let freq: u32 = (*s_0).freq as u32;
                    let c: *mut Context = (self.base).offset(
                        ((*s_0).successor_0 as u32 | ((*s_0).successor_1 as u32) << 16 as i32)
                            as isize,
                    ) as *mut u8 as *mut Context;
                    self.found_state = s_0;
                    self.prev_success = 1 as i32 as u32;
                    self.run_length += 1;
                    self.run_length;
                    (*s_0).freq = freq.wrapping_add((freq < 196 as i32 as u32) as i32 as u32) as u8;
                    if self.order_fall == 0 as i32 as u32
                        && c as *const u8 >= self.units_start as *const u8
                    {
                        self.min_context = c;
                        self.max_context = self.min_context;
                    } else {
                        self.update_model();
                    }
                    return;
                }
                *prob = pr as u16;
                self.init_esc = self.exp_escape[(pr >> 10 as i32) as usize] as u32;
                self.low = (self.low).wrapping_add(bound);
                self.range = (self.range
                    & !(((1 as i32) << 7 as i32 + 7 as i32) as u32).wrapping_sub(1 as i32 as u32))
                .wrapping_sub(bound);

                char_mask = [u8::MAX; 256];
                char_mask[(*s_0).symbol as usize] = 0;
                self.prev_success = 0 as i32 as u32;
            }
            loop {
                let mut s_1: *mut State = 0 as *mut State;
                let mut sum_0: u32 = 0;
                let mut esc_freq: u32 = 0;
                let mut mc: *mut Context = 0 as *mut Context;
                let mut i_0: u32 = 0;
                let mut num_masked: u32 = 0;
                while self.low ^ (self.low).wrapping_add(self.range) < K_TOP_VALUE
                    || self.range < K_BOT_VALUE && {
                        self.range = (0 as i32 as u32).wrapping_sub(self.low) & (K_BOT_VALUE - 1);
                        1 as i32 != 0
                    }
                {
                    ((*self.stream.output).write)(
                        self.stream.output,
                        (self.low >> 24 as i32) as u8,
                    );
                    self.range <<= 8 as i32;
                    self.low <<= 8 as i32;
                }
                mc = self.min_context;
                num_masked = (*mc).num_stats as u32;
                loop {
                    self.order_fall = (self.order_fall).wrapping_add(1);
                    self.order_fall;
                    if (*mc).suffix == 0 {
                        return;
                    }
                    mc = (self.base).offset((*mc).suffix as isize) as *mut u8 as *mut Context;
                    if !((*mc).num_stats as u32 == num_masked) {
                        break;
                    }
                }
                self.min_context = mc;
                let see_source = self.make_esc_freq(num_masked, &mut esc_freq);
                s_1 = (self.base).offset((*self.min_context).union4.stats as isize) as *mut u8
                    as *mut State;
                sum_0 = 0 as i32 as u32;
                i_0 = ((*self.min_context).num_stats as u32).wrapping_add(1 as i32 as u32);
                loop {
                    let cur: u32 = (*s_1).symbol as u32;
                    if cur as i32 == symbol {
                        let low: u32 = sum_0;
                        let freq_0: u32 = (*s_1).freq as u32;
                        let mut num2: u32 = 0;

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
                        self.found_state = s_1;
                        sum_0 = sum_0.wrapping_add(esc_freq);
                        num2 = i_0.wrapping_div(2 as i32 as u32);
                        i_0 &= 1 as i32 as u32;
                        sum_0 = sum_0.wrapping_add(freq_0 & (0 as i32 as u32).wrapping_sub(i_0));
                        if num2 != 0 as i32 as u32 {
                            s_1 = s_1.offset(i_0 as isize);
                            loop {
                                let sym0_0: u32 = (*s_1.offset(0 as i32 as isize)).symbol as u32;
                                let sym1_0: u32 = (*s_1.offset(1 as i32 as isize)).symbol as u32;
                                s_1 = s_1.offset(2 as i32 as isize);
                                sum_0 = (sum_0 as u32).wrapping_add(
                                    (*s_1.offset(-(2 as i32) as isize)).freq as u32
                                        & *(char_mask.as_mut_ptr() as *mut u8)
                                            .offset(sym0_0 as isize)
                                            as u32,
                                ) as u32 as u32;
                                sum_0 = (sum_0 as u32).wrapping_add(
                                    (*s_1.offset(-(1 as i32) as isize)).freq as u32
                                        & *(char_mask.as_mut_ptr() as *mut u8)
                                            .offset(sym1_0 as isize)
                                            as u32,
                                ) as u32 as u32;
                                num2 = num2.wrapping_sub(1);
                                if !(num2 != 0) {
                                    break;
                                }
                            }
                        }
                        if sum_0 > self.range {
                            sum_0 = self.range;
                        }
                        self.range_enc_encode(low, freq_0, sum_0);
                        while self.low ^ (self.low).wrapping_add(self.range) < K_TOP_VALUE
                            || self.range < K_BOT_VALUE && {
                                self.range =
                                    (0 as i32 as u32).wrapping_sub(self.low) & K_BOT_VALUE - 1;
                                1 as i32 != 0
                            }
                        {
                            ((*self.stream.output).write)(
                                self.stream.output,
                                (self.low >> 24 as i32) as u8,
                            );
                            self.range <<= 8 as i32;
                            self.low <<= 8 as i32;
                        }
                        self.update2();
                        return;
                    }
                    sum_0 = (sum_0 as u32).wrapping_add(
                        (*s_1).freq as u32
                            & *(char_mask.as_mut_ptr() as *mut u8).offset(cur as isize) as u32,
                    ) as u32 as u32;
                    s_1 = s_1.offset(1);
                    s_1;
                    i_0 = i_0.wrapping_sub(1);
                    if !(i_0 != 0) {
                        break;
                    }
                }
                let mut total: u32 = sum_0.wrapping_add(esc_freq);
                let see = self.get_see(see_source);
                (*see).summ = ((*see).summ as u32).wrapping_add(total) as u16;
                if total > self.range {
                    total = self.range;
                }
                self.range_enc_encode(sum_0, total.wrapping_sub(sum_0), total);
                let mut s2_0: *const State = (self.base)
                    .offset((*self.min_context).union4.stats as isize)
                    as *mut u8 as *mut State;
                s_1 = s_1.offset(-1);
                s_1;
                *(char_mask.as_mut_ptr() as *mut u8).offset((*s_1).symbol as isize) =
                    0 as i32 as u8;
                loop {
                    let sym0_1: u32 = (*s2_0.offset(0 as i32 as isize)).symbol as u32;
                    let sym1_1: u32 = (*s2_0.offset(1 as i32 as isize)).symbol as u32;
                    s2_0 = s2_0.offset(2 as i32 as isize);
                    *(char_mask.as_mut_ptr() as *mut u8).offset(sym0_1 as isize) = 0 as i32 as u8;
                    *(char_mask.as_mut_ptr() as *mut u8).offset(sym1_1 as isize) = 0 as i32 as u8;
                    if !(s2_0 < s_1 as *const State) {
                        break;
                    }
                }
            }
        }
    }
}
