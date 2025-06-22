use super::*;

pub unsafe fn flush_range_enc(p: *mut Ppmd8) {
    unsafe {
        let mut i: u32 = 0;
        i = 0 as i32 as u32;
        while i < 4 as i32 as u32 {
            ((*(*p).stream.output).write).expect("non-null function pointer")(
                (*p).stream.output,
                ((*p).low >> 24 as i32) as u8,
            );
            i = i.wrapping_add(1);
            i;
            (*p).low <<= 8 as i32;
        }
    }
}

unsafe fn range_enc_encode(p: *mut Ppmd8, start: u32, size: u32, total: u32) {
    unsafe {
        (*p).range = (*p).range / total;
        (*p).low = ((*p).low).wrapping_add(start * (*p).range);
        (*p).range = (*p).range * size;
    }
}

pub unsafe fn encode_symbol(p: *mut Ppmd8, symbol: i32) {
    unsafe {
        let mut charMask: [usize; 32] = [0; 32];
        if (*(*p).min_context).num_stats as i32 != 0 as i32 {
            let mut s: *mut State = ((*p).base).offset((*(*p).min_context).union4.stats as isize)
                as *mut u8 as *mut State;
            let mut sum: u32 = 0;
            let mut i: u32 = 0;
            let mut summFreq: u32 = (*(*p).min_context).union2.summ_freq as u32;
            if summFreq > (*p).range {
                summFreq = (*p).range;
            }
            if (*s).symbol as i32 == symbol {
                range_enc_encode(p, 0 as i32 as u32, (*s).freq as u32, summFreq);
                while (*p).low ^ ((*p).low).wrapping_add((*p).range)
                    < (1 as i32 as u32) << 24 as i32
                    || (*p).range < (1 as i32 as u32) << 15 as i32 && {
                        (*p).range = (0 as i32 as u32).wrapping_sub((*p).low)
                            & ((1 as i32 as u32) << 15 as i32).wrapping_sub(1 as i32 as u32);
                        1 as i32 != 0
                    }
                {
                    ((*(*p).stream.output).write).expect("non-null function pointer")(
                        (*p).stream.output,
                        ((*p).low >> 24 as i32) as u8,
                    );
                    (*p).range <<= 8 as i32;
                    (*p).low <<= 8 as i32;
                }
                (*p).found_state = s;
                update1_0(p);
                return;
            }
            (*p).prev_success = 0 as i32 as u32;
            sum = (*s).freq as u32;
            i = (*(*p).min_context).num_stats as u32;
            loop {
                s = s.offset(1);
                if (*s).symbol as i32 == symbol {
                    range_enc_encode(p, sum, (*s).freq as u32, summFreq);
                    while (*p).low ^ ((*p).low).wrapping_add((*p).range)
                        < (1 as i32 as u32) << 24 as i32
                        || (*p).range < (1 as i32 as u32) << 15 as i32 && {
                            (*p).range = (0 as i32 as u32).wrapping_sub((*p).low)
                                & ((1 as i32 as u32) << 15 as i32).wrapping_sub(1 as i32 as u32);
                            1 as i32 != 0
                        }
                    {
                        ((*(*p).stream.output).write).expect("non-null function pointer")(
                            (*p).stream.output,
                            ((*p).low >> 24 as i32) as u8,
                        );
                        (*p).range <<= 8 as i32;
                        (*p).low <<= 8 as i32;
                    }
                    (*p).found_state = s;
                    update1(p);
                    return;
                }
                sum = sum.wrapping_add((*s).freq as u32);
                i = i.wrapping_sub(1);
                if !(i != 0) {
                    break;
                }
            }
            range_enc_encode(p, sum, summFreq.wrapping_sub(sum), summFreq);
            let mut z: usize = 0;
            z = 0 as i32 as usize;
            while z < (256 as i32 as usize).wrapping_div(::core::mem::size_of::<usize>() as usize) {
                charMask[z.wrapping_add(0 as i32 as usize) as usize] = !(0 as i32 as usize);
                charMask[z.wrapping_add(1 as i32 as usize) as usize] =
                    charMask[z.wrapping_add(0 as i32 as usize) as usize];
                charMask[z.wrapping_add(2 as i32 as usize) as usize] =
                    charMask[z.wrapping_add(1 as i32 as usize) as usize];
                charMask[z.wrapping_add(3 as i32 as usize) as usize] =
                    charMask[z.wrapping_add(2 as i32 as usize) as usize];
                charMask[z.wrapping_add(4 as i32 as usize) as usize] =
                    charMask[z.wrapping_add(3 as i32 as usize) as usize];
                charMask[z.wrapping_add(5 as i32 as usize) as usize] =
                    charMask[z.wrapping_add(4 as i32 as usize) as usize];
                charMask[z.wrapping_add(6 as i32 as usize) as usize] =
                    charMask[z.wrapping_add(5 as i32 as usize) as usize];
                charMask[z.wrapping_add(7 as i32 as usize) as usize] =
                    charMask[z.wrapping_add(6 as i32 as usize) as usize];
                z = z.wrapping_add(8 as i32 as usize);
            }
            let mut s2: *mut State = ((*p).base).offset((*(*p).min_context).union4.stats as isize)
                as *mut u8 as *mut State;
            *(charMask.as_mut_ptr() as *mut u8).offset((*s).symbol as isize) = 0 as i32 as u8;
            loop {
                let sym0: u32 = (*s2.offset(0 as i32 as isize)).symbol as u32;
                let sym1: u32 = (*s2.offset(1 as i32 as isize)).symbol as u32;
                s2 = s2.offset(2 as i32 as isize);
                *(charMask.as_mut_ptr() as *mut u8).offset(sym0 as isize) = 0 as i32 as u8;
                *(charMask.as_mut_ptr() as *mut u8).offset(sym1 as isize) = 0 as i32 as u8;
                if !(s2 < s) {
                    break;
                }
            }
        } else {
            let prob: *mut u16 = &mut *(*((*p).bin_summ).as_mut_ptr().offset(
                *((*p).ns2index).as_mut_ptr().offset(
                    ((*(&mut (*(*p).min_context).union2 as *mut Union2 as *mut State)).freq
                        as usize)
                        .wrapping_sub(1 as i32 as usize) as isize,
                ) as isize,
            ))
            .as_mut_ptr()
            .offset(
                ((*p).prev_success)
                    .wrapping_add(((*p).run_length >> 26 as i32 & 0x20 as i32) as u32)
                    .wrapping_add(
                        *((*p).ns2bs_index).as_mut_ptr().offset(
                            (*(((*p).base).offset((*(*p).min_context).suffix as isize) as *mut u8
                                as *mut Context))
                                .num_stats as isize,
                        ) as u32,
                    )
                    .wrapping_add((*(*p).min_context).flags as i32 as u32) as isize,
            ) as *mut u16;
            let s_0: *mut State = &mut (*(*p).min_context).union2 as *mut Union2 as *mut State;
            let mut pr: u32 = *prob as u32;
            let bound: u32 = ((*p).range >> 14 as i32) * pr;
            pr = pr.wrapping_sub(
                pr.wrapping_add(((1 as i32) << 7 as i32 - 2 as i32) as u32) >> 7 as i32,
            );
            if (*s_0).symbol as i32 == symbol {
                *prob = pr.wrapping_add(((1 as i32) << 7 as i32) as u32) as u16;
                (*p).range = bound;
                while (*p).low ^ ((*p).low).wrapping_add((*p).range)
                    < (1 as i32 as u32) << 24 as i32
                    || (*p).range < (1 as i32 as u32) << 15 as i32 && {
                        (*p).range = (0 as i32 as u32).wrapping_sub((*p).low)
                            & ((1 as i32 as u32) << 15 as i32).wrapping_sub(1 as i32 as u32);
                        1 as i32 != 0
                    }
                {
                    ((*(*p).stream.output).write).expect("non-null function pointer")(
                        (*p).stream.output,
                        ((*p).low >> 24 as i32) as u8,
                    );
                    (*p).range <<= 8 as i32;
                    (*p).low <<= 8 as i32;
                }
                let freq: u32 = (*s_0).freq as u32;
                let c: *mut Context = ((*p).base).offset(
                    ((*s_0).successor_0 as u32 | ((*s_0).successor_1 as u32) << 16 as i32) as isize,
                ) as *mut u8 as *mut Context;
                (*p).found_state = s_0;
                (*p).prev_success = 1 as i32 as u32;
                (*p).run_length += 1;
                (*p).run_length;
                (*s_0).freq = freq.wrapping_add((freq < 196 as i32 as u32) as i32 as u32) as u8;
                if (*p).order_fall == 0 as i32 as u32
                    && c as *const u8 >= (*p).units_start as *const u8
                {
                    (*p).min_context = c;
                    (*p).max_context = (*p).min_context;
                } else {
                    update_model(p);
                }
                return;
            }
            *prob = pr as u16;
            (*p).init_esc = (*p).exp_escape[(pr >> 10 as i32) as usize] as u32;
            (*p).low = ((*p).low).wrapping_add(bound);
            (*p).range = ((*p).range
                & !(((1 as i32) << 7 as i32 + 7 as i32) as u32).wrapping_sub(1 as i32 as u32))
            .wrapping_sub(bound);
            let mut z_0: usize = 0;
            z_0 = 0 as i32 as usize;
            while z_0 < (256 as i32 as usize).wrapping_div(::core::mem::size_of::<usize>() as usize)
            {
                charMask[z_0.wrapping_add(0 as i32 as usize) as usize] = !(0 as i32 as usize);
                charMask[z_0.wrapping_add(1 as i32 as usize) as usize] =
                    charMask[z_0.wrapping_add(0 as i32 as usize) as usize];
                charMask[z_0.wrapping_add(2 as i32 as usize) as usize] =
                    charMask[z_0.wrapping_add(1 as i32 as usize) as usize];
                charMask[z_0.wrapping_add(3 as i32 as usize) as usize] =
                    charMask[z_0.wrapping_add(2 as i32 as usize) as usize];
                charMask[z_0.wrapping_add(4 as i32 as usize) as usize] =
                    charMask[z_0.wrapping_add(3 as i32 as usize) as usize];
                charMask[z_0.wrapping_add(5 as i32 as usize) as usize] =
                    charMask[z_0.wrapping_add(4 as i32 as usize) as usize];
                charMask[z_0.wrapping_add(6 as i32 as usize) as usize] =
                    charMask[z_0.wrapping_add(5 as i32 as usize) as usize];
                charMask[z_0.wrapping_add(7 as i32 as usize) as usize] =
                    charMask[z_0.wrapping_add(6 as i32 as usize) as usize];
                z_0 = z_0.wrapping_add(8 as i32 as usize);
            }
            *(charMask.as_mut_ptr() as *mut u8).offset((*s_0).symbol as isize) = 0 as i32 as u8;
            (*p).prev_success = 0 as i32 as u32;
        }
        loop {
            let mut see: *mut See = 0 as *mut See;
            let mut s_1: *mut State = 0 as *mut State;
            let mut sum_0: u32 = 0;
            let mut escFreq: u32 = 0;
            let mut mc: *mut Context = 0 as *mut Context;
            let mut i_0: u32 = 0;
            let mut numMasked: u32 = 0;
            while (*p).low ^ ((*p).low).wrapping_add((*p).range) < (1 as i32 as u32) << 24 as i32
                || (*p).range < (1 as i32 as u32) << 15 as i32 && {
                    (*p).range = (0 as i32 as u32).wrapping_sub((*p).low)
                        & ((1 as i32 as u32) << 15 as i32).wrapping_sub(1 as i32 as u32);
                    1 as i32 != 0
                }
            {
                ((*(*p).stream.output).write).expect("non-null function pointer")(
                    (*p).stream.output,
                    ((*p).low >> 24 as i32) as u8,
                );
                (*p).range <<= 8 as i32;
                (*p).low <<= 8 as i32;
            }
            mc = (*p).min_context;
            numMasked = (*mc).num_stats as u32;
            loop {
                (*p).order_fall = ((*p).order_fall).wrapping_add(1);
                (*p).order_fall;
                if (*mc).suffix == 0 {
                    return;
                }
                mc = ((*p).base).offset((*mc).suffix as isize) as *mut u8 as *mut Context;
                if !((*mc).num_stats as u32 == numMasked) {
                    break;
                }
            }
            (*p).min_context = mc;
            see = make_esc_freq(p, numMasked, &mut escFreq);
            s_1 = ((*p).base).offset((*(*p).min_context).union4.stats as isize) as *mut u8
                as *mut State;
            sum_0 = 0 as i32 as u32;
            i_0 = ((*(*p).min_context).num_stats as u32).wrapping_add(1 as i32 as u32);
            loop {
                let cur: u32 = (*s_1).symbol as u32;
                if cur as i32 == symbol {
                    let low: u32 = sum_0;
                    let freq_0: u32 = (*s_1).freq as u32;
                    let mut num2: u32 = 0;
                    if ((*see).shift as i32) < 7 as i32 && {
                        (*see).count = ((*see).count).wrapping_sub(1);
                        (*see).count as i32 == 0 as i32
                    } {
                        (*see).summ = (((*see).summ as i32) << 1 as i32) as u16;
                        let fresh0 = (*see).shift;
                        (*see).shift = ((*see).shift).wrapping_add(1);
                        (*see).count = ((3 as i32) << fresh0 as i32) as u8;
                    }
                    (*p).found_state = s_1;
                    sum_0 = sum_0.wrapping_add(escFreq);
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
                                    & *(charMask.as_mut_ptr() as *mut u8).offset(sym0_0 as isize)
                                        as u32,
                            ) as u32 as u32;
                            sum_0 = (sum_0 as u32).wrapping_add(
                                (*s_1.offset(-(1 as i32) as isize)).freq as u32
                                    & *(charMask.as_mut_ptr() as *mut u8).offset(sym1_0 as isize)
                                        as u32,
                            ) as u32 as u32;
                            num2 = num2.wrapping_sub(1);
                            if !(num2 != 0) {
                                break;
                            }
                        }
                    }
                    if sum_0 > (*p).range {
                        sum_0 = (*p).range;
                    }
                    range_enc_encode(p, low, freq_0, sum_0);
                    while (*p).low ^ ((*p).low).wrapping_add((*p).range)
                        < (1 as i32 as u32) << 24 as i32
                        || (*p).range < (1 as i32 as u32) << 15 as i32 && {
                            (*p).range = (0 as i32 as u32).wrapping_sub((*p).low)
                                & ((1 as i32 as u32) << 15 as i32).wrapping_sub(1 as i32 as u32);
                            1 as i32 != 0
                        }
                    {
                        ((*(*p).stream.output).write).expect("non-null function pointer")(
                            (*p).stream.output,
                            ((*p).low >> 24 as i32) as u8,
                        );
                        (*p).range <<= 8 as i32;
                        (*p).low <<= 8 as i32;
                    }
                    update2(p);
                    return;
                }
                sum_0 = (sum_0 as u32).wrapping_add(
                    (*s_1).freq as u32
                        & *(charMask.as_mut_ptr() as *mut u8).offset(cur as isize) as u32,
                ) as u32 as u32;
                s_1 = s_1.offset(1);
                s_1;
                i_0 = i_0.wrapping_sub(1);
                if !(i_0 != 0) {
                    break;
                }
            }
            let mut total: u32 = sum_0.wrapping_add(escFreq);
            (*see).summ = ((*see).summ as u32).wrapping_add(total) as u16;
            if total > (*p).range {
                total = (*p).range;
            }
            range_enc_encode(p, sum_0, total.wrapping_sub(sum_0), total);
            let mut s2_0: *const State = ((*p).base)
                .offset((*(*p).min_context).union4.stats as isize)
                as *mut u8 as *mut State;
            s_1 = s_1.offset(-1);
            s_1;
            *(charMask.as_mut_ptr() as *mut u8).offset((*s_1).symbol as isize) = 0 as i32 as u8;
            loop {
                let sym0_1: u32 = (*s2_0.offset(0 as i32 as isize)).symbol as u32;
                let sym1_1: u32 = (*s2_0.offset(1 as i32 as isize)).symbol as u32;
                s2_0 = s2_0.offset(2 as i32 as isize);
                *(charMask.as_mut_ptr() as *mut u8).offset(sym0_1 as isize) = 0 as i32 as u8;
                *(charMask.as_mut_ptr() as *mut u8).offset(sym1_1 as isize) = 0 as i32 as u8;
                if !(s2_0 < s_1 as *const State) {
                    break;
                }
            }
        }
    }
}
