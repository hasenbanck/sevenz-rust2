use super::*;

pub unsafe fn init_range_dec(p: *mut Ppmd8) -> i32 {
    unsafe {
        let mut i: u32 = 0;
        (*p).code = 0 as i32 as u32;
        (*p).range = 0xFFFFFFFF as u32;
        (*p).low = 0 as i32 as u32;
        i = 0 as i32 as u32;
        while i < 4 as i32 as u32 {
            (*p).code = (*p).code << 8 as i32
                | ((*(*p).stream.input).read).expect("non-null function pointer")((*p).stream.input)
                    as u32;
            i = i.wrapping_add(1);
        }
        ((*p).code < 0xFFFFFFFF as u32) as i32
    }
}

#[inline(always)]
unsafe fn rd_decode(p: *mut Ppmd8, mut start: u32, size: u32) {
    unsafe {
        start = start * (*p).range;
        (*p).low = ((*p).low).wrapping_add(start);
        (*p).code = ((*p).code).wrapping_sub(start);
        (*p).range = (*p).range * size;
    }
}

pub unsafe fn decode_symbol(p: *mut Ppmd8) -> i32 {
    unsafe {
        let mut charMask: [usize; 32] = [0; 32];
        if (*(*p).min_context).num_stats as i32 != 0 as i32 {
            let mut s: *mut State = ((*p).base).offset((*(*p).min_context).union4.stats as isize)
                as *mut u8 as *mut State;
            let mut i: u32 = 0;
            let mut count: u32 = 0;
            let mut hiCnt: u32 = 0;
            let mut summFreq: u32 = (*(*p).min_context).union2.summ_freq as u32;
            if summFreq > (*p).range {
                summFreq = (*p).range;
            }
            (*p).range = (*p).range / summFreq;
            count = (*p).code / (*p).range;
            hiCnt = count;
            count = count.wrapping_sub((*s).freq as u32);
            if (count as i32) < 0 as i32 {
                let mut sym: u8 = 0;
                rd_decode(p, 0 as i32 as u32, (*s).freq as u32);
                while (*p).low ^ ((*p).low).wrapping_add((*p).range)
                    < (1 as i32 as u32) << 24 as i32
                    || (*p).range < (1 as i32 as u32) << 15 as i32 && {
                        (*p).range = (0 as i32 as u32).wrapping_sub((*p).low)
                            & ((1 as i32 as u32) << 15 as i32).wrapping_sub(1 as i32 as u32);
                        1 as i32 != 0
                    }
                {
                    (*p).code = (*p).code << 8 as i32
                        | ((*(*p).stream.input).read).expect("non-null function pointer")(
                            (*p).stream.input,
                        ) as u32;
                    (*p).range <<= 8 as i32;
                    (*p).low <<= 8 as i32;
                }
                (*p).found_state = s;
                sym = (*s).symbol;
                update1_0(p);
                return sym as i32;
            }
            (*p).prev_success = 0 as i32 as u32;
            i = (*(*p).min_context).num_stats as u32;
            loop {
                s = s.offset(1);
                count = count.wrapping_sub((*s).freq as u32);
                if (count as i32) < 0 as i32 {
                    let mut sym_0: u8 = 0;
                    rd_decode(
                        p,
                        hiCnt.wrapping_sub(count).wrapping_sub((*s).freq as u32),
                        (*s).freq as u32,
                    );
                    while (*p).low ^ ((*p).low).wrapping_add((*p).range)
                        < (1 as i32 as u32) << 24 as i32
                        || (*p).range < (1 as i32 as u32) << 15 as i32 && {
                            (*p).range = (0 as i32 as u32).wrapping_sub((*p).low)
                                & ((1 as i32 as u32) << 15 as i32).wrapping_sub(1 as i32 as u32);
                            1 as i32 != 0
                        }
                    {
                        (*p).code = (*p).code << 8 as i32
                            | ((*(*p).stream.input).read).expect("non-null function pointer")(
                                (*p).stream.input,
                            ) as u32;
                        (*p).range <<= 8 as i32;
                        (*p).low <<= 8 as i32;
                    }
                    (*p).found_state = s;
                    sym_0 = (*s).symbol;
                    update1(p);
                    return sym_0 as i32;
                }
                i = i.wrapping_sub(1);
                if !(i != 0) {
                    break;
                }
            }
            if hiCnt >= summFreq {
                return -(2 as i32);
            }
            hiCnt = hiCnt.wrapping_sub(count);
            rd_decode(p, hiCnt, summFreq.wrapping_sub(hiCnt));
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
            let s_0: *mut State = &mut (*(*p).min_context).union2 as *mut Union2 as *mut State;
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
            let mut pr: u32 = *prob as u32;
            let size0: u32 = ((*p).range >> 14 as i32) * pr;
            pr = pr.wrapping_sub(
                pr.wrapping_add(((1 as i32) << 7 as i32 - 2 as i32) as u32) >> 7 as i32,
            );
            if (*p).code < size0 {
                let mut sym_1: u8 = 0;
                *prob = pr.wrapping_add(((1 as i32) << 7 as i32) as u32) as u16;
                (*p).range = size0;
                while (*p).low ^ ((*p).low).wrapping_add((*p).range)
                    < (1 as i32 as u32) << 24 as i32
                    || (*p).range < (1 as i32 as u32) << 15 as i32 && {
                        (*p).range = (0 as i32 as u32).wrapping_sub((*p).low)
                            & ((1 as i32 as u32) << 15 as i32).wrapping_sub(1 as i32 as u32);
                        1 as i32 != 0
                    }
                {
                    (*p).code = (*p).code << 8 as i32
                        | ((*(*p).stream.input).read).expect("non-null function pointer")(
                            (*p).stream.input,
                        ) as u32;
                    (*p).range <<= 8 as i32;
                    (*p).low <<= 8 as i32;
                }
                let freq: u32 = (*s_0).freq as u32;
                let c: *mut Context = ((*p).base).offset(
                    ((*s_0).successor_0 as u32 | ((*s_0).successor_1 as u32) << 16 as i32) as isize,
                ) as *mut u8 as *mut Context;
                sym_1 = (*s_0).symbol;
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
                return sym_1 as i32;
            }
            *prob = pr as u16;
            (*p).init_esc = (*p).exp_escape[(pr >> 10 as i32) as usize] as u32;
            (*p).low = ((*p).low).wrapping_add(size0);
            (*p).code = ((*p).code).wrapping_sub(size0);
            (*p).range = ((*p).range
                & !(((1 as i32) << 7 as i32 + 7 as i32) as u32).wrapping_sub(1 as i32 as u32))
            .wrapping_sub(size0);
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
            *(charMask.as_mut_ptr() as *mut u8).offset(
                (*(&mut (*(*p).min_context).union2 as *mut Union2 as *mut State)).symbol as isize,
            ) = 0 as i32 as u8;
            (*p).prev_success = 0 as i32 as u32;
        }
        loop {
            let mut s_1: *mut State = 0 as *mut State;
            let mut s2_0: *mut State = 0 as *mut State;
            let mut freqSum: u32 = 0;
            let mut count_0: u32 = 0;
            let mut hiCnt_0: u32 = 0;
            let mut freqSum2: u32 = 0;
            let mut see: *mut See = 0 as *mut See;
            let mut mc: *mut Context = 0 as *mut Context;
            let mut numMasked: u32 = 0;
            while (*p).low ^ ((*p).low).wrapping_add((*p).range) < (1 as i32 as u32) << 24 as i32
                || (*p).range < (1 as i32 as u32) << 15 as i32 && {
                    (*p).range = (0 as i32 as u32).wrapping_sub((*p).low)
                        & ((1 as i32 as u32) << 15 as i32).wrapping_sub(1 as i32 as u32);
                    1 as i32 != 0
                }
            {
                (*p).code = (*p).code << 8 as i32
                    | ((*(*p).stream.input).read).expect("non-null function pointer")(
                        (*p).stream.input,
                    ) as u32;
                (*p).range <<= 8 as i32;
                (*p).low <<= 8 as i32;
            }
            mc = (*p).min_context;
            numMasked = (*mc).num_stats as u32;
            loop {
                (*p).order_fall = ((*p).order_fall).wrapping_add(1);
                (*p).order_fall;
                if (*mc).suffix == 0 {
                    return -(1 as i32);
                }
                mc = ((*p).base).offset((*mc).suffix as isize) as *mut u8 as *mut Context;
                if !((*mc).num_stats as u32 == numMasked) {
                    break;
                }
            }
            s_1 = ((*p).base).offset((*mc).union4.stats as isize) as *mut u8 as *mut State;
            let mut num: u32 = ((*mc).num_stats as u32).wrapping_add(1 as i32 as u32);
            let mut num2: u32 = num.wrapping_div(2 as i32 as u32);
            num &= 1 as i32 as u32;
            hiCnt_0 = (*s_1).freq as u32
                & *(charMask.as_mut_ptr() as *mut u8).offset((*s_1).symbol as isize) as u32
                & (0 as i32 as u32).wrapping_sub(num);
            s_1 = s_1.offset(num as isize);
            (*p).min_context = mc;
            loop {
                let sym0_0: u32 = (*s_1.offset(0 as i32 as isize)).symbol as u32;
                let sym1_0: u32 = (*s_1.offset(1 as i32 as isize)).symbol as u32;
                s_1 = s_1.offset(2 as i32 as isize);
                hiCnt_0 = hiCnt_0.wrapping_add(
                    (*s_1.offset(-(2 as i32) as isize)).freq as u32
                        & *(charMask.as_mut_ptr() as *mut u8).offset(sym0_0 as isize) as u32,
                );
                hiCnt_0 = hiCnt_0.wrapping_add(
                    (*s_1.offset(-(1 as i32) as isize)).freq as u32
                        & *(charMask.as_mut_ptr() as *mut u8).offset(sym1_0 as isize) as u32,
                );
                num2 = num2.wrapping_sub(1);
                if !(num2 != 0) {
                    break;
                }
            }
            see = make_esc_freq(p, numMasked, &mut freqSum);
            freqSum = freqSum.wrapping_add(hiCnt_0);
            freqSum2 = freqSum;
            if freqSum2 > (*p).range {
                freqSum2 = (*p).range;
            }
            (*p).range = (*p).range / freqSum2;
            count_0 = (*p).code / (*p).range;
            if count_0 < hiCnt_0 {
                let mut sym_2: u8 = 0;
                s_1 = ((*p).base).offset((*(*p).min_context).union4.stats as isize) as *mut u8
                    as *mut State;
                hiCnt_0 = count_0;
                loop {
                    count_0 = count_0.wrapping_sub(
                        (*s_1).freq as u32
                            & *(charMask.as_mut_ptr() as *mut u8).offset((*s_1).symbol as isize)
                                as u32,
                    );
                    s_1 = s_1.offset(1);
                    if (count_0 as i32) < 0 as i32 {
                        break;
                    }
                }
                s_1 = s_1.offset(-1);
                rd_decode(
                    p,
                    hiCnt_0
                        .wrapping_sub(count_0)
                        .wrapping_sub((*s_1).freq as u32),
                    (*s_1).freq as u32,
                );
                while (*p).low ^ ((*p).low).wrapping_add((*p).range)
                    < (1 as i32 as u32) << 24 as i32
                    || (*p).range < (1 as i32 as u32) << 15 as i32 && {
                        (*p).range = (0 as i32 as u32).wrapping_sub((*p).low)
                            & ((1 as i32 as u32) << 15 as i32).wrapping_sub(1 as i32 as u32);
                        1 as i32 != 0
                    }
                {
                    (*p).code = (*p).code << 8 as i32
                        | ((*(*p).stream.input).read).expect("non-null function pointer")(
                            (*p).stream.input,
                        ) as u32;
                    (*p).range <<= 8 as i32;
                    (*p).low <<= 8 as i32;
                }
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
                sym_2 = (*s_1).symbol;
                update2(p);
                return sym_2 as i32;
            }
            if count_0 >= freqSum2 {
                return -(2 as i32);
            }
            rd_decode(p, hiCnt_0, freqSum2.wrapping_sub(hiCnt_0));
            (*see).summ = ((*see).summ as u32).wrapping_add(freqSum) as u16;
            s_1 = ((*p).base).offset((*(*p).min_context).union4.stats as isize) as *mut u8
                as *mut State;
            s2_0 = s_1
                .offset((*(*p).min_context).num_stats as i32 as isize)
                .offset(1 as i32 as isize);
            loop {
                *(charMask.as_mut_ptr() as *mut u8).offset((*s_1).symbol as isize) = 0 as i32 as u8;
                s_1 = s_1.offset(1);
                s_1;
                if !(s_1 != s2_0) {
                    break;
                }
            }
        }
    }
}
