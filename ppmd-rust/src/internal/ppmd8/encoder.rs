use std::ptr::addr_of_mut;

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
            if self.min_context.as_ref().num_stats != 0 {
                let mut s = self
                    .ptr_of_offset(self.min_context.as_ref().data.multi_state.stats)
                    .cast::<State>();
                let mut summ_freq = self.min_context.as_ref().data.multi_state.summ_freq as u32;
                if summ_freq > self.range {
                    summ_freq = self.range;
                }
                if s.as_ref().symbol as i32 == symbol {
                    self.range_enc_encode(0, s.as_ref().freq as u32, summ_freq);
                    while self.low ^ self.low.wrapping_add(self.range) < K_TOP_VALUE
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
                let mut sum = s.as_ref().freq as u32;
                let mut i = self.min_context.as_ref().num_stats as u32;
                loop {
                    s = s.offset(1);
                    if s.as_ref().symbol as i32 == symbol {
                        self.range_enc_encode(sum, s.as_ref().freq as u32, summ_freq);
                        while self.low ^ self.low.wrapping_add(self.range) < K_TOP_VALUE
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
                    sum = sum.wrapping_add(s.as_ref().freq as u32);
                    i = i.wrapping_sub(1);
                    if !(i != 0) {
                        break;
                    }
                }
                self.range_enc_encode(sum, summ_freq.wrapping_sub(sum), summ_freq);

                char_mask = [u8::MAX; 256];

                let mut s2 = self
                    .ptr_of_offset(self.min_context.as_ref().data.multi_state.stats)
                    .cast::<State>();

                char_mask[s.as_ref().symbol as usize] = 0;
                loop {
                    let sym0 = s2.offset(0).as_ref().symbol as u32;
                    let sym1 = s2.offset(1).as_ref().symbol as u32;
                    s2 = s2.offset(2);
                    char_mask[sym0 as usize] = 0;
                    char_mask[sym1 as usize] = 0;
                    if !(s2 < s) {
                        break;
                    }
                }
            } else {
                let prob =
                    &mut *(*self
                        .bin_summ
                        .as_mut_ptr()
                        .offset(*self.ns2index.as_mut_ptr().offset(
                            ((self.min_context.as_ref().data.single_state.freq as usize) - 1)
                                as isize,
                        ) as isize))
                    .as_mut_ptr()
                    .offset(
                        self.prev_success
                            .wrapping_add((self.run_length >> 26 & 0x20) as u32)
                            .wrapping_add(
                                *self.ns2bs_index.as_mut_ptr().offset(
                                    self.ptr_of_offset(self.min_context.as_ref().suffix)
                                        .cast::<Context>()
                                        .as_ref()
                                        .num_stats as isize,
                                ) as u32,
                            )
                            .wrapping_add(self.min_context.as_ref().flags as u32)
                            as isize,
                    )
                    .cast::<u16>();
                let mut s = NonNull::new_unchecked(addr_of_mut!(
                    self.min_context.as_mut().data.single_state
                ));
                let mut pr = *prob as u32;
                let bound = (self.range >> 14) * pr;
                pr = pr.wrapping_sub(pr.wrapping_add((1 << (7 - 2)) as u32) >> 7);
                if s.as_ref().symbol as i32 == symbol {
                    *prob = pr.wrapping_add((1 << 7) as u32) as u16;
                    self.range = bound;
                    while self.low ^ self.low.wrapping_add(self.range) < K_TOP_VALUE
                        || self.range < K_BOT_VALUE && {
                            self.range = 0u32.wrapping_sub(self.low) & (K_BOT_VALUE - 1);
                            1 != 0
                        }
                    {
                        ((*self.stream.output).write)(self.stream.output, (self.low >> 24) as u8);
                        self.range <<= 8;
                        self.low <<= 8;
                    }
                    let freq = s.as_ref().freq as u32;
                    let c = self.ptr_of_offset(s.as_ref().successor).cast::<Context>();
                    self.found_state = s;
                    self.prev_success = 1;
                    self.run_length += 1;
                    s.as_mut().freq = (freq + ((freq < 196) as u32)) as u8;
                    if self.order_fall == 0 && c.addr() >= self.units_start.addr() {
                        self.min_context = c;
                        self.max_context = self.min_context;
                    } else {
                        self.update_model();
                    }
                    return;
                }
                *prob = pr as u16;
                self.init_esc = self.exp_escape[(pr >> 10) as usize] as u32;
                self.low += bound;
                self.range =
                    (self.range & !((1 << (7 + 7)) as u32).wrapping_sub(1)).wrapping_sub(bound);

                char_mask = [u8::MAX; 256];
                char_mask[s.as_ref().symbol as usize] = 0;
                self.prev_success = 0;
            }
            loop {
                let mut esc_freq = 0;
                while self.low ^ self.low.wrapping_add(self.range) < K_TOP_VALUE
                    || self.range < K_BOT_VALUE && {
                        self.range = 0u32.wrapping_sub(self.low) & (K_BOT_VALUE - 1);
                        1 != 0
                    }
                {
                    ((*self.stream.output).write)(self.stream.output, (self.low >> 24) as u8);
                    self.range <<= 8;
                    self.low <<= 8;
                }
                let mut mc = self.min_context;
                let num_masked = mc.as_ref().num_stats as u32;
                loop {
                    self.order_fall += 1;
                    if mc.as_ref().suffix == 0 {
                        return;
                    }
                    mc = self.ptr_of_offset(mc.as_ref().suffix).cast::<Context>();
                    if !(mc.as_ref().num_stats as u32 == num_masked) {
                        break;
                    }
                }
                self.min_context = mc;
                let see_source = self.make_esc_freq(num_masked, &mut esc_freq);
                let mut s = self
                    .ptr_of_offset(self.min_context.as_ref().data.multi_state.stats)
                    .cast::<State>();
                let mut sum = 0u32;
                let mut i = (self.min_context.as_ref().num_stats as u32) + 1;
                loop {
                    let cur = s.as_ref().symbol as u32;
                    if cur as i32 == symbol {
                        let low = sum;
                        let freq = s.as_ref().freq as u32;

                        let see = self.get_see(see_source);
                        if (see.shift as i32) < 7 && {
                            see.count -= 1;
                            see.count as i32 == 0
                        } {
                            see.summ = ((see.summ as i32) << 1) as u16;
                            let fresh = see.shift as u32;
                            see.shift += 1;
                            see.count = (3 << fresh) as u8;
                        }
                        self.found_state = s;
                        sum += esc_freq;
                        let mut num2 = i / 2;
                        i &= 1;
                        sum += freq & 0u32.wrapping_sub(i);
                        if num2 != 0 {
                            s = s.offset(i as isize);
                            loop {
                                let sym0 = s.offset(0).as_ref().symbol as u32;
                                let sym1 = s.offset(1).as_ref().symbol as u32;
                                s = s.offset(2);
                                sum += s.offset(-2).as_ref().freq as u32
                                    & char_mask[sym0 as usize] as u32;
                                sum += s.offset(-1).as_ref().freq as u32
                                    & char_mask[sym1 as usize] as u32;
                                num2 -= 1;
                                if !(num2 != 0) {
                                    break;
                                }
                            }
                        }
                        if sum > self.range {
                            sum = self.range;
                        }
                        self.range_enc_encode(low, freq, sum);
                        while self.low ^ self.low.wrapping_add(self.range) < K_TOP_VALUE
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
                        self.update2();
                        return;
                    }
                    sum += s.as_ref().freq as u32 & char_mask[cur as usize] as u32;
                    s = s.offset(1);
                    i = i.wrapping_sub(1);
                    if !(i != 0) {
                        break;
                    }
                }
                let mut total = sum.wrapping_add(esc_freq);
                let see = self.get_see(see_source);
                see.summ = (see.summ as u32).wrapping_add(total) as u16;
                if total > self.range {
                    total = self.range;
                }
                self.range_enc_encode(sum, total.wrapping_sub(sum), total);
                let mut s2 = self
                    .ptr_of_offset(self.min_context.as_ref().data.multi_state.stats)
                    .cast::<State>();
                s = s.offset(-1);

                char_mask[s.as_ref().symbol as usize] = 0;
                loop {
                    let sym0 = s2.offset(0).as_ref().symbol as u32;
                    let sym1 = s2.offset(1).as_ref().symbol as u32;
                    s2 = s2.offset(2);
                    char_mask[sym0 as usize] = 0;
                    char_mask[sym1 as usize] = 0;
                    if !(s2.addr() < s.addr()) {
                        break;
                    }
                }
            }
        }
    }
}
