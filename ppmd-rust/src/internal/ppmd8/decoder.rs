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

pub unsafe fn Ppmd8_Init_RangeDec(mut p: *mut CPpmd8) -> i32 {
    unsafe {
        let mut i: u32 = 0;
        (*p).Code = 0 as i32 as u32;
        (*p).Range = 0xffffffff as u32;
        (*p).Low = 0 as i32 as u32;
        i = 0 as i32 as u32;
        while i < 4 as i32 as u32 {
            (*p).Code = (*p).Code << 8 as i32
                | ((*(*p).Stream.In).Read).expect("non-null function pointer")((*p).Stream.In)
                    as u32;
            i = i.wrapping_add(1);
            i;
        }
        ((*p).Code < 0xffffffff as u32) as i32
    }
}

#[inline(always)]
unsafe fn Ppmd8_RD_Decode(mut p: *mut CPpmd8, mut start: u32, mut size: u32) {
    unsafe {
        start = start * (*p).Range;
        (*p).Low = ((*p).Low).wrapping_add(start);
        (*p).Code = ((*p).Code).wrapping_sub(start);
        (*p).Range = (*p).Range * size;
    }
}

pub unsafe fn Ppmd8_DecodeSymbol(mut p: *mut CPpmd8) -> i32 {
    unsafe {
        let mut charMask: [usize; 32] = [0; 32];
        if (*(*p).MinContext).NumStats as i32 != 0 as i32 {
            let mut s: *mut CPpmd_State = ((*p).Base)
                .offset((*(*p).MinContext).Union4.Stats as isize)
                as *mut u8 as *mut CPpmd_State;
            let mut i: u32 = 0;
            let mut count: u32 = 0;
            let mut hiCnt: u32 = 0;
            let mut summFreq: u32 = (*(*p).MinContext).Union2.SummFreq as u32;
            if summFreq > (*p).Range {
                summFreq = (*p).Range;
            }
            (*p).Range = (*p).Range / summFreq;
            count = (*p).Code / (*p).Range;
            hiCnt = count;
            count = count.wrapping_sub((*s).Freq as u32);
            if (count as i32) < 0 as i32 {
                let mut sym: u8 = 0;
                Ppmd8_RD_Decode(p, 0 as i32 as u32, (*s).Freq as u32);
                while (*p).Low ^ ((*p).Low).wrapping_add((*p).Range)
                    < (1 as i32 as u32) << 24 as i32
                    || (*p).Range < (1 as i32 as u32) << 15 as i32 && {
                        (*p).Range = (0 as i32 as u32).wrapping_sub((*p).Low)
                            & ((1 as i32 as u32) << 15 as i32).wrapping_sub(1 as i32 as u32);
                        1 as i32 != 0
                    }
                {
                    (*p).Code = (*p).Code << 8 as i32
                        | ((*(*p).Stream.In).Read).expect("non-null function pointer")(
                            (*p).Stream.In,
                        ) as u32;
                    (*p).Range <<= 8 as i32;
                    (*p).Low <<= 8 as i32;
                }
                (*p).FoundState = s;
                sym = (*s).Symbol;
                Ppmd8_Update1_0(p);
                return sym as i32;
            }
            (*p).PrevSuccess = 0 as i32 as u32;
            i = (*(*p).MinContext).NumStats as u32;
            loop {
                s = s.offset(1);
                count = count.wrapping_sub((*s).Freq as u32);
                if (count as i32) < 0 as i32 {
                    let mut sym_0: u8 = 0;
                    Ppmd8_RD_Decode(
                        p,
                        hiCnt.wrapping_sub(count).wrapping_sub((*s).Freq as u32),
                        (*s).Freq as u32,
                    );
                    while (*p).Low ^ ((*p).Low).wrapping_add((*p).Range)
                        < (1 as i32 as u32) << 24 as i32
                        || (*p).Range < (1 as i32 as u32) << 15 as i32 && {
                            (*p).Range = (0 as i32 as u32).wrapping_sub((*p).Low)
                                & ((1 as i32 as u32) << 15 as i32).wrapping_sub(1 as i32 as u32);
                            1 as i32 != 0
                        }
                    {
                        (*p).Code = (*p).Code << 8 as i32
                            | ((*(*p).Stream.In).Read).expect("non-null function pointer")(
                                (*p).Stream.In,
                            ) as u32;
                        (*p).Range <<= 8 as i32;
                        (*p).Low <<= 8 as i32;
                    }
                    (*p).FoundState = s;
                    sym_0 = (*s).Symbol;
                    Ppmd8_Update1(p);
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
            Ppmd8_RD_Decode(p, hiCnt, summFreq.wrapping_sub(hiCnt));
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
            let mut s_0: *mut CPpmd_State =
                &mut (*(*p).MinContext).Union2 as *mut Union2 as *mut CPpmd_State;
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
            let mut pr: u32 = *prob as u32;
            let mut size0: u32 = ((*p).Range >> 14 as i32) * pr;
            pr = pr.wrapping_sub(
                pr.wrapping_add(((1 as i32) << 7 as i32 - 2 as i32) as u32) >> 7 as i32,
            );
            if (*p).Code < size0 {
                let mut sym_1: u8 = 0;
                *prob = pr.wrapping_add(((1 as i32) << 7 as i32) as u32) as u16;
                (*p).Range = size0;
                while (*p).Low ^ ((*p).Low).wrapping_add((*p).Range)
                    < (1 as i32 as u32) << 24 as i32
                    || (*p).Range < (1 as i32 as u32) << 15 as i32 && {
                        (*p).Range = (0 as i32 as u32).wrapping_sub((*p).Low)
                            & ((1 as i32 as u32) << 15 as i32).wrapping_sub(1 as i32 as u32);
                        1 as i32 != 0
                    }
                {
                    (*p).Code = (*p).Code << 8 as i32
                        | ((*(*p).Stream.In).Read).expect("non-null function pointer")(
                            (*p).Stream.In,
                        ) as u32;
                    (*p).Range <<= 8 as i32;
                    (*p).Low <<= 8 as i32;
                }
                let mut freq: u32 = (*s_0).Freq as u32;
                let mut c: *mut CPpmd8_Context = ((*p).Base).offset(
                    ((*s_0).Successor_0 as u32 | ((*s_0).Successor_1 as u32) << 16 as i32) as isize,
                ) as *mut u8
                    as *mut CPpmd8_Context;
                sym_1 = (*s_0).Symbol;
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
                return sym_1 as i32;
            }
            *prob = pr as u16;
            (*p).InitEsc = (*p).ExpEscape[(pr >> 10 as i32) as usize] as u32;
            (*p).Low = ((*p).Low).wrapping_add(size0);
            (*p).Code = ((*p).Code).wrapping_sub(size0);
            (*p).Range = ((*p).Range
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
                (*(&mut (*(*p).MinContext).Union2 as *mut Union2 as *mut CPpmd_State)).Symbol
                    as isize,
            ) = 0 as i32 as u8;
            (*p).PrevSuccess = 0 as i32 as u32;
        }
        loop {
            let mut s_1: *mut CPpmd_State = 0 as *mut CPpmd_State;
            let mut s2_0: *mut CPpmd_State = 0 as *mut CPpmd_State;
            let mut freqSum: u32 = 0;
            let mut count_0: u32 = 0;
            let mut hiCnt_0: u32 = 0;
            let mut freqSum2: u32 = 0;
            let mut see: *mut CPpmd_See = 0 as *mut CPpmd_See;
            let mut mc: *mut CPpmd8_Context = 0 as *mut CPpmd8_Context;
            let mut numMasked: u32 = 0;
            while (*p).Low ^ ((*p).Low).wrapping_add((*p).Range) < (1 as i32 as u32) << 24 as i32
                || (*p).Range < (1 as i32 as u32) << 15 as i32 && {
                    (*p).Range = (0 as i32 as u32).wrapping_sub((*p).Low)
                        & ((1 as i32 as u32) << 15 as i32).wrapping_sub(1 as i32 as u32);
                    1 as i32 != 0
                }
            {
                (*p).Code = (*p).Code << 8 as i32
                    | ((*(*p).Stream.In).Read).expect("non-null function pointer")((*p).Stream.In)
                        as u32;
                (*p).Range <<= 8 as i32;
                (*p).Low <<= 8 as i32;
            }
            mc = (*p).MinContext;
            numMasked = (*mc).NumStats as u32;
            loop {
                (*p).OrderFall = ((*p).OrderFall).wrapping_add(1);
                (*p).OrderFall;
                if (*mc).Suffix == 0 {
                    return -(1 as i32);
                }
                mc = ((*p).Base).offset((*mc).Suffix as isize) as *mut u8 as *mut CPpmd8_Context;
                if !((*mc).NumStats as u32 == numMasked) {
                    break;
                }
            }
            s_1 = ((*p).Base).offset((*mc).Union4.Stats as isize) as *mut u8 as *mut CPpmd_State;
            let mut num: u32 = ((*mc).NumStats as u32).wrapping_add(1 as i32 as u32);
            let mut num2: u32 = num.wrapping_div(2 as i32 as u32);
            num &= 1 as i32 as u32;
            hiCnt_0 = (*s_1).Freq as u32
                & *(charMask.as_mut_ptr() as *mut u8).offset((*s_1).Symbol as isize) as u32
                & (0 as i32 as u32).wrapping_sub(num);
            s_1 = s_1.offset(num as isize);
            (*p).MinContext = mc;
            loop {
                let sym0_0: u32 = (*s_1.offset(0 as i32 as isize)).Symbol as u32;
                let sym1_0: u32 = (*s_1.offset(1 as i32 as isize)).Symbol as u32;
                s_1 = s_1.offset(2 as i32 as isize);
                hiCnt_0 = hiCnt_0.wrapping_add(
                    (*s_1.offset(-(2 as i32) as isize)).Freq as u32
                        & *(charMask.as_mut_ptr() as *mut u8).offset(sym0_0 as isize) as u32,
                );
                hiCnt_0 = hiCnt_0.wrapping_add(
                    (*s_1.offset(-(1 as i32) as isize)).Freq as u32
                        & *(charMask.as_mut_ptr() as *mut u8).offset(sym1_0 as isize) as u32,
                );
                num2 = num2.wrapping_sub(1);
                if !(num2 != 0) {
                    break;
                }
            }
            see = Ppmd8_MakeEscFreq(p, numMasked, &mut freqSum);
            freqSum = freqSum.wrapping_add(hiCnt_0);
            freqSum2 = freqSum;
            if freqSum2 > (*p).Range {
                freqSum2 = (*p).Range;
            }
            (*p).Range = (*p).Range / freqSum2;
            count_0 = (*p).Code / (*p).Range;
            if count_0 < hiCnt_0 {
                let mut sym_2: u8 = 0;
                s_1 = ((*p).Base).offset((*(*p).MinContext).Union4.Stats as isize) as *mut u8
                    as *mut CPpmd_State;
                hiCnt_0 = count_0;
                loop {
                    count_0 = count_0.wrapping_sub(
                        (*s_1).Freq as u32
                            & *(charMask.as_mut_ptr() as *mut u8).offset((*s_1).Symbol as isize)
                                as u32,
                    );
                    s_1 = s_1.offset(1);
                    s_1;
                    if (count_0 as i32) < 0 as i32 {
                        break;
                    }
                }
                s_1 = s_1.offset(-1);
                s_1;
                Ppmd8_RD_Decode(
                    p,
                    hiCnt_0
                        .wrapping_sub(count_0)
                        .wrapping_sub((*s_1).Freq as u32),
                    (*s_1).Freq as u32,
                );
                while (*p).Low ^ ((*p).Low).wrapping_add((*p).Range)
                    < (1 as i32 as u32) << 24 as i32
                    || (*p).Range < (1 as i32 as u32) << 15 as i32 && {
                        (*p).Range = (0 as i32 as u32).wrapping_sub((*p).Low)
                            & ((1 as i32 as u32) << 15 as i32).wrapping_sub(1 as i32 as u32);
                        1 as i32 != 0
                    }
                {
                    (*p).Code = (*p).Code << 8 as i32
                        | ((*(*p).Stream.In).Read).expect("non-null function pointer")(
                            (*p).Stream.In,
                        ) as u32;
                    (*p).Range <<= 8 as i32;
                    (*p).Low <<= 8 as i32;
                }
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
                sym_2 = (*s_1).Symbol;
                Ppmd8_Update2(p);
                return sym_2 as i32;
            }
            if count_0 >= freqSum2 {
                return -(2 as i32);
            }
            Ppmd8_RD_Decode(p, hiCnt_0, freqSum2.wrapping_sub(hiCnt_0));
            (*see).Summ = ((*see).Summ as u32).wrapping_add(freqSum) as u16;
            s_1 = ((*p).Base).offset((*(*p).MinContext).Union4.Stats as isize) as *mut u8
                as *mut CPpmd_State;
            s2_0 = s_1
                .offset((*(*p).MinContext).NumStats as i32 as isize)
                .offset(1 as i32 as isize);
            loop {
                *(charMask.as_mut_ptr() as *mut u8).offset((*s_1).Symbol as isize) = 0 as i32 as u8;
                s_1 = s_1.offset(1);
                s_1;
                if !(s_1 != s2_0) {
                    break;
                }
            }
        }
    }
}
