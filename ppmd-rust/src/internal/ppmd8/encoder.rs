#![allow(
    dead_code,
    mutable_transmutes,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_assignments,
    unused_mut
)]

use super::*;

pub unsafe fn Ppmd8_Flush_RangeEnc(mut p: *mut CPpmd8) {
    unsafe {
        let mut i: u32 = 0;
        i = 0 as i32 as u32;
        while i < 4 as i32 as u32 {
            ((*(*p).Stream.Out).Write).expect("non-null function pointer")(
                (*p).Stream.Out,
                ((*p).Low >> 24 as i32) as u8,
            );
            i = i.wrapping_add(1);
            i;
            (*p).Low <<= 8 as i32;
        }
    }
}

unsafe fn Ppmd8_RangeEnc_Encode(mut p: *mut CPpmd8, mut start: u32, mut size: u32, mut total: u32) {
    unsafe {
        (*p).Range = (*p).Range / total;
        (*p).Low = ((*p).Low).wrapping_add(start * (*p).Range);
        (*p).Range = (*p).Range * size;
    }
}

pub unsafe fn Ppmd8_EncodeSymbol(mut p: *mut CPpmd8, mut symbol: i32) {
    unsafe {
        let mut charMask: [usize; 32] = [0; 32];
        if (*(*p).MinContext).NumStats as i32 != 0 as i32 {
            let mut s: *mut CPpmd_State = ((*p).Base)
                .offset((*(*p).MinContext).Union4.Stats as isize)
                as *mut u8 as *mut CPpmd_State;
            let mut sum: u32 = 0;
            let mut i: u32 = 0;
            let mut summFreq: u32 = (*(*p).MinContext).Union2.SummFreq as u32;
            if summFreq > (*p).Range {
                summFreq = (*p).Range;
            }
            if (*s).Symbol as i32 == symbol {
                Ppmd8_RangeEnc_Encode(p, 0 as i32 as u32, (*s).Freq as u32, summFreq);
                while (*p).Low ^ ((*p).Low).wrapping_add((*p).Range)
                    < (1 as i32 as u32) << 24 as i32
                    || (*p).Range < (1 as i32 as u32) << 15 as i32 && {
                        (*p).Range = (0 as i32 as u32).wrapping_sub((*p).Low)
                            & ((1 as i32 as u32) << 15 as i32).wrapping_sub(1 as i32 as u32);
                        1 as i32 != 0
                    }
                {
                    ((*(*p).Stream.Out).Write).expect("non-null function pointer")(
                        (*p).Stream.Out,
                        ((*p).Low >> 24 as i32) as u8,
                    );
                    (*p).Range <<= 8 as i32;
                    (*p).Low <<= 8 as i32;
                }
                (*p).FoundState = s;
                Ppmd8_Update1_0(p);
                return;
            }
            (*p).PrevSuccess = 0 as i32 as u32;
            sum = (*s).Freq as u32;
            i = (*(*p).MinContext).NumStats as u32;
            loop {
                s = s.offset(1);
                if (*s).Symbol as i32 == symbol {
                    Ppmd8_RangeEnc_Encode(p, sum, (*s).Freq as u32, summFreq);
                    while (*p).Low ^ ((*p).Low).wrapping_add((*p).Range)
                        < (1 as i32 as u32) << 24 as i32
                        || (*p).Range < (1 as i32 as u32) << 15 as i32 && {
                            (*p).Range = (0 as i32 as u32).wrapping_sub((*p).Low)
                                & ((1 as i32 as u32) << 15 as i32).wrapping_sub(1 as i32 as u32);
                            1 as i32 != 0
                        }
                    {
                        ((*(*p).Stream.Out).Write).expect("non-null function pointer")(
                            (*p).Stream.Out,
                            ((*p).Low >> 24 as i32) as u8,
                        );
                        (*p).Range <<= 8 as i32;
                        (*p).Low <<= 8 as i32;
                    }
                    (*p).FoundState = s;
                    Ppmd8_Update1(p);
                    return;
                }
                sum = sum.wrapping_add((*s).Freq as u32);
                i = i.wrapping_sub(1);
                if !(i != 0) {
                    break;
                }
            }
            Ppmd8_RangeEnc_Encode(p, sum, summFreq.wrapping_sub(sum), summFreq);
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
            let mut s2: *mut CPpmd_State = ((*p).Base)
                .offset((*(*p).MinContext).Union4.Stats as isize)
                as *mut u8 as *mut CPpmd_State;
            *(charMask.as_mut_ptr() as *mut u8).offset((*s).Symbol as isize) = 0 as i32 as u8;
            loop {
                let sym0: u32 = (*s2.offset(0 as i32 as isize)).Symbol as u32;
                let sym1: u32 = (*s2.offset(1 as i32 as isize)).Symbol as u32;
                s2 = s2.offset(2 as i32 as isize);
                *(charMask.as_mut_ptr() as *mut u8).offset(sym0 as isize) = 0 as i32 as u8;
                *(charMask.as_mut_ptr() as *mut u8).offset(sym1 as isize) = 0 as i32 as u8;
                if !(s2 < s) {
                    break;
                }
            }
        } else {
            let mut prob: *mut u16 = &mut *(*((*p).BinSumm).as_mut_ptr().offset(
                *((*p).NS2Indx).as_mut_ptr().offset(
                    ((*(&mut (*(*p).MinContext).Union2 as *mut Union2 as *mut CPpmd_State)).Freq
                        as usize)
                        .wrapping_sub(1 as i32 as usize) as isize,
                ) as isize,
            ))
            .as_mut_ptr()
            .offset(
                ((*p).PrevSuccess)
                    .wrapping_add(((*p).RunLength >> 26 as i32 & 0x20 as i32) as u32)
                    .wrapping_add(
                        *((*p).NS2BSIndx).as_mut_ptr().offset(
                            (*(((*p).Base).offset((*(*p).MinContext).Suffix as isize) as *mut u8
                                as *mut CPpmd8_Context))
                                .NumStats as isize,
                        ) as u32,
                    )
                    .wrapping_add((*(*p).MinContext).Flags as i32 as u32) as isize,
            ) as *mut u16;
            let mut s_0: *mut CPpmd_State =
                &mut (*(*p).MinContext).Union2 as *mut Union2 as *mut CPpmd_State;
            let mut pr: u32 = *prob as u32;
            let bound: u32 = ((*p).Range >> 14 as i32) * pr;
            pr = pr.wrapping_sub(
                pr.wrapping_add(((1 as i32) << 7 as i32 - 2 as i32) as u32) >> 7 as i32,
            );
            if (*s_0).Symbol as i32 == symbol {
                *prob = pr.wrapping_add(((1 as i32) << 7 as i32) as u32) as u16;
                (*p).Range = bound;
                while (*p).Low ^ ((*p).Low).wrapping_add((*p).Range)
                    < (1 as i32 as u32) << 24 as i32
                    || (*p).Range < (1 as i32 as u32) << 15 as i32 && {
                        (*p).Range = (0 as i32 as u32).wrapping_sub((*p).Low)
                            & ((1 as i32 as u32) << 15 as i32).wrapping_sub(1 as i32 as u32);
                        1 as i32 != 0
                    }
                {
                    ((*(*p).Stream.Out).Write).expect("non-null function pointer")(
                        (*p).Stream.Out,
                        ((*p).Low >> 24 as i32) as u8,
                    );
                    (*p).Range <<= 8 as i32;
                    (*p).Low <<= 8 as i32;
                }
                let freq: u32 = (*s_0).Freq as u32;
                let mut c: *mut CPpmd8_Context = ((*p).Base).offset(
                    ((*s_0).Successor_0 as u32 | ((*s_0).Successor_1 as u32) << 16 as i32) as isize,
                ) as *mut u8
                    as *mut CPpmd8_Context;
                (*p).FoundState = s_0;
                (*p).PrevSuccess = 1 as i32 as u32;
                (*p).RunLength += 1;
                (*p).RunLength;
                (*s_0).Freq = freq.wrapping_add((freq < 196 as i32 as u32) as i32 as u32) as u8;
                if (*p).OrderFall == 0 as i32 as u32
                    && c as *const u8 >= (*p).UnitsStart as *const u8
                {
                    (*p).MinContext = c;
                    (*p).MaxContext = (*p).MinContext;
                } else {
                    Ppmd8_UpdateModel(p);
                }
                return;
            }
            *prob = pr as u16;
            (*p).InitEsc = (*p).ExpEscape[(pr >> 10 as i32) as usize] as u32;
            (*p).Low = ((*p).Low).wrapping_add(bound);
            (*p).Range = ((*p).Range
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
            *(charMask.as_mut_ptr() as *mut u8).offset((*s_0).Symbol as isize) = 0 as i32 as u8;
            (*p).PrevSuccess = 0 as i32 as u32;
        }
        loop {
            let mut see: *mut CPpmd_See = 0 as *mut CPpmd_See;
            let mut s_1: *mut CPpmd_State = 0 as *mut CPpmd_State;
            let mut sum_0: u32 = 0;
            let mut escFreq: u32 = 0;
            let mut mc: *mut CPpmd8_Context = 0 as *mut CPpmd8_Context;
            let mut i_0: u32 = 0;
            let mut numMasked: u32 = 0;
            while (*p).Low ^ ((*p).Low).wrapping_add((*p).Range) < (1 as i32 as u32) << 24 as i32
                || (*p).Range < (1 as i32 as u32) << 15 as i32 && {
                    (*p).Range = (0 as i32 as u32).wrapping_sub((*p).Low)
                        & ((1 as i32 as u32) << 15 as i32).wrapping_sub(1 as i32 as u32);
                    1 as i32 != 0
                }
            {
                ((*(*p).Stream.Out).Write).expect("non-null function pointer")(
                    (*p).Stream.Out,
                    ((*p).Low >> 24 as i32) as u8,
                );
                (*p).Range <<= 8 as i32;
                (*p).Low <<= 8 as i32;
            }
            mc = (*p).MinContext;
            numMasked = (*mc).NumStats as u32;
            loop {
                (*p).OrderFall = ((*p).OrderFall).wrapping_add(1);
                (*p).OrderFall;
                if (*mc).Suffix == 0 {
                    return;
                }
                mc = ((*p).Base).offset((*mc).Suffix as isize) as *mut u8 as *mut CPpmd8_Context;
                if !((*mc).NumStats as u32 == numMasked) {
                    break;
                }
            }
            (*p).MinContext = mc;
            see = Ppmd8_MakeEscFreq(p, numMasked, &mut escFreq);
            s_1 = ((*p).Base).offset((*(*p).MinContext).Union4.Stats as isize) as *mut u8
                as *mut CPpmd_State;
            sum_0 = 0 as i32 as u32;
            i_0 = ((*(*p).MinContext).NumStats as u32).wrapping_add(1 as i32 as u32);
            loop {
                let cur: u32 = (*s_1).Symbol as u32;
                if cur as i32 == symbol {
                    let low: u32 = sum_0;
                    let freq_0: u32 = (*s_1).Freq as u32;
                    let mut num2: u32 = 0;
                    if ((*see).Shift as i32) < 7 as i32 && {
                        (*see).Count = ((*see).Count).wrapping_sub(1);
                        (*see).Count as i32 == 0 as i32
                    } {
                        (*see).Summ = (((*see).Summ as i32) << 1 as i32) as u16;
                        let fresh0 = (*see).Shift;
                        (*see).Shift = ((*see).Shift).wrapping_add(1);
                        (*see).Count = ((3 as i32) << fresh0 as i32) as u8;
                    }
                    (*p).FoundState = s_1;
                    sum_0 = sum_0.wrapping_add(escFreq);
                    num2 = i_0.wrapping_div(2 as i32 as u32);
                    i_0 &= 1 as i32 as u32;
                    sum_0 = sum_0.wrapping_add(freq_0 & (0 as i32 as u32).wrapping_sub(i_0));
                    if num2 != 0 as i32 as u32 {
                        s_1 = s_1.offset(i_0 as isize);
                        loop {
                            let sym0_0: u32 = (*s_1.offset(0 as i32 as isize)).Symbol as u32;
                            let sym1_0: u32 = (*s_1.offset(1 as i32 as isize)).Symbol as u32;
                            s_1 = s_1.offset(2 as i32 as isize);
                            sum_0 = (sum_0 as u32).wrapping_add(
                                (*s_1.offset(-(2 as i32) as isize)).Freq as u32
                                    & *(charMask.as_mut_ptr() as *mut u8).offset(sym0_0 as isize)
                                        as u32,
                            ) as u32 as u32;
                            sum_0 = (sum_0 as u32).wrapping_add(
                                (*s_1.offset(-(1 as i32) as isize)).Freq as u32
                                    & *(charMask.as_mut_ptr() as *mut u8).offset(sym1_0 as isize)
                                        as u32,
                            ) as u32 as u32;
                            num2 = num2.wrapping_sub(1);
                            if !(num2 != 0) {
                                break;
                            }
                        }
                    }
                    if sum_0 > (*p).Range {
                        sum_0 = (*p).Range;
                    }
                    Ppmd8_RangeEnc_Encode(p, low, freq_0, sum_0);
                    while (*p).Low ^ ((*p).Low).wrapping_add((*p).Range)
                        < (1 as i32 as u32) << 24 as i32
                        || (*p).Range < (1 as i32 as u32) << 15 as i32 && {
                            (*p).Range = (0 as i32 as u32).wrapping_sub((*p).Low)
                                & ((1 as i32 as u32) << 15 as i32).wrapping_sub(1 as i32 as u32);
                            1 as i32 != 0
                        }
                    {
                        ((*(*p).Stream.Out).Write).expect("non-null function pointer")(
                            (*p).Stream.Out,
                            ((*p).Low >> 24 as i32) as u8,
                        );
                        (*p).Range <<= 8 as i32;
                        (*p).Low <<= 8 as i32;
                    }
                    Ppmd8_Update2(p);
                    return;
                }
                sum_0 = (sum_0 as u32).wrapping_add(
                    (*s_1).Freq as u32
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
            (*see).Summ = ((*see).Summ as u32).wrapping_add(total) as u16;
            if total > (*p).Range {
                total = (*p).Range;
            }
            Ppmd8_RangeEnc_Encode(p, sum_0, total.wrapping_sub(sum_0), total);
            let mut s2_0: *const CPpmd_State = ((*p).Base)
                .offset((*(*p).MinContext).Union4.Stats as isize)
                as *mut u8 as *mut CPpmd_State;
            s_1 = s_1.offset(-1);
            s_1;
            *(charMask.as_mut_ptr() as *mut u8).offset((*s_1).Symbol as isize) = 0 as i32 as u8;
            loop {
                let sym0_1: u32 = (*s2_0.offset(0 as i32 as isize)).Symbol as u32;
                let sym1_1: u32 = (*s2_0.offset(1 as i32 as isize)).Symbol as u32;
                s2_0 = s2_0.offset(2 as i32 as isize);
                *(charMask.as_mut_ptr() as *mut u8).offset(sym0_1 as isize) = 0 as i32 as u8;
                *(charMask.as_mut_ptr() as *mut u8).offset(sym1_1 as isize) = 0 as i32 as u8;
                if !(s2_0 < s_1 as *const CPpmd_State) {
                    break;
                }
            }
        }
    }
}
