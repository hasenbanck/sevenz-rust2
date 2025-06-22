#![allow(
    dead_code,
    mutable_transmutes,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_assignments,
    unused_mut
)]

extern "C" {
    fn Ppmd8_Update1(p: *mut CPpmd8);
    fn Ppmd8_Update1_0(p: *mut CPpmd8);
    fn Ppmd8_Update2(p: *mut CPpmd8);
    fn Ppmd8_MakeEscFreq(
        p: *mut CPpmd8,
        numMasked: std::ffi::c_uint,
        scale: *mut UInt32,
    ) -> *mut CPpmd_See;
    fn Ppmd8_UpdateModel(p: *mut CPpmd8);
}

pub type size_t = std::ffi::c_ulong;
pub type Byte = std::ffi::c_uchar;
pub type UInt16 = std::ffi::c_ushort;
pub type Int32 = std::ffi::c_int;
pub type UInt32 = std::ffi::c_uint;
pub type BoolInt = std::ffi::c_int;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct IByteIn_ {
    pub Read: Option<unsafe extern "C" fn(IByteInPtr) -> Byte>,
}

pub type IByteInPtr = *const IByteIn_;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct IByteOut_ {
    pub Write: Option<unsafe extern "C" fn(IByteOutPtr, Byte) -> ()>,
}

pub type IByteOutPtr = *const IByteOut_;

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct CPpmd_See {
    pub Summ: UInt16,
    pub Shift: Byte,
    pub Count: Byte,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct CPpmd_State {
    pub Symbol: Byte,
    pub Freq: Byte,
    pub Successor_0: UInt16,
    pub Successor_1: UInt16,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct CPpmd_State2_ {
    pub Symbol: Byte,
    pub Freq: Byte,
}
pub type CPpmd_State2 = CPpmd_State2_;
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct CPpmd_State4_ {
    pub Successor_0: UInt16,
    pub Successor_1: UInt16,
}

pub type CPpmd_State4 = CPpmd_State4_;
pub type CPpmd_State_Ref = UInt32;
pub type CPpmd_Void_Ref = UInt32;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CPpmd8_Context_ {
    pub NumStats: Byte,
    pub Flags: Byte,
    pub Union2: C2RustUnnamed_0,
    pub Union4: C2RustUnnamed,
    pub Suffix: CPpmd8_Context_Ref,
}

pub type CPpmd8_Context_Ref = UInt32;

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed {
    pub Stats: CPpmd_State_Ref,
    pub State4: CPpmd_State4,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_0 {
    pub SummFreq: UInt16,
    pub State2: CPpmd_State2,
}

pub type CPpmd8_Context = CPpmd8_Context_;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CPpmd8 {
    pub MinContext: *mut CPpmd8_Context,
    pub MaxContext: *mut CPpmd8_Context,
    pub FoundState: *mut CPpmd_State,
    pub OrderFall: std::ffi::c_uint,
    pub InitEsc: std::ffi::c_uint,
    pub PrevSuccess: std::ffi::c_uint,
    pub MaxOrder: std::ffi::c_uint,
    pub RestoreMethod: std::ffi::c_uint,
    pub RunLength: Int32,
    pub InitRL: Int32,
    pub Size: UInt32,
    pub GlueCount: UInt32,
    pub AlignOffset: UInt32,
    pub Base: *mut Byte,
    pub LoUnit: *mut Byte,
    pub HiUnit: *mut Byte,
    pub Text: *mut Byte,
    pub UnitsStart: *mut Byte,
    pub Range: UInt32,
    pub Code: UInt32,
    pub Low: UInt32,
    pub Stream: C2RustUnnamed_1,
    pub Indx2Units: [Byte; 40],
    pub Units2Indx: [Byte; 128],
    pub FreeList: [CPpmd_Void_Ref; 38],
    pub Stamps: [UInt32; 38],
    pub NS2BSIndx: [Byte; 256],
    pub NS2Indx: [Byte; 260],
    pub ExpEscape: [Byte; 16],
    pub DummySee: CPpmd_See,
    pub See: [[CPpmd_See; 32]; 24],
    pub BinSumm: [[UInt16; 64]; 25],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_1 {
    pub In: IByteInPtr,
    pub Out: IByteOutPtr,
}

pub fn Ppmd8_Init_RangeDec(mut p: *mut CPpmd8) -> BoolInt {
    let mut i: std::ffi::c_uint = 0;
    (*p).Code = 0 as std::ffi::c_int as UInt32;
    (*p).Range = 0xffffffff as std::ffi::c_uint;
    (*p).Low = 0 as std::ffi::c_int as UInt32;
    i = 0 as std::ffi::c_int as std::ffi::c_uint;
    while i < 4 as std::ffi::c_int as std::ffi::c_uint {
        (*p).Code = (*p).Code << 8 as std::ffi::c_int
            | ((*(*p).Stream.In).Read).expect("non-null function pointer")((*p).Stream.In)
                as UInt32;
        i = i.wrapping_add(1);
        i;
    }
    ((*p).Code < 0xffffffff as std::ffi::c_uint) as std::ffi::c_int
}

#[inline(always)]
fn Ppmd8_RD_Decode(mut p: *mut CPpmd8, mut start: UInt32, mut size: UInt32) {
    start = start * (*p).Range;
    (*p).Low = ((*p).Low).wrapping_add(start);
    (*p).Code = ((*p).Code).wrapping_sub(start);
    (*p).Range = (*p).Range * size;
}

pub fn Ppmd8_DecodeSymbol(mut p: *mut CPpmd8) -> std::ffi::c_int {
    let mut charMask: [size_t; 32] = [0; 32];
    if (*(*p).MinContext).NumStats as std::ffi::c_int != 0 as std::ffi::c_int {
        let mut s: *mut CPpmd_State = ((*p).Base).offset((*(*p).MinContext).Union4.Stats as isize)
            as *mut std::ffi::c_void as *mut CPpmd_State;
        let mut i: std::ffi::c_uint = 0;
        let mut count: UInt32 = 0;
        let mut hiCnt: UInt32 = 0;
        let mut summFreq: UInt32 = (*(*p).MinContext).Union2.SummFreq as UInt32;
        if summFreq > (*p).Range {
            summFreq = (*p).Range;
        }
        (*p).Range = (*p).Range / summFreq;
        count = (*p).Code / (*p).Range;
        hiCnt = count;
        count = count.wrapping_sub((*s).Freq as UInt32);
        if (count as Int32) < 0 as std::ffi::c_int {
            let mut sym: Byte = 0;
            Ppmd8_RD_Decode(p, 0 as std::ffi::c_int as UInt32, (*s).Freq as UInt32);
            while (*p).Low ^ ((*p).Low).wrapping_add((*p).Range)
                < (1 as std::ffi::c_int as UInt32) << 24 as std::ffi::c_int
                || (*p).Range < (1 as std::ffi::c_int as UInt32) << 15 as std::ffi::c_int && {
                    (*p).Range = (0 as std::ffi::c_int as UInt32).wrapping_sub((*p).Low)
                        & ((1 as std::ffi::c_int as UInt32) << 15 as std::ffi::c_int)
                            .wrapping_sub(1 as std::ffi::c_int as UInt32);
                    1 as std::ffi::c_int != 0
                }
            {
                (*p).Code = (*p).Code << 8 as std::ffi::c_int
                    | ((*(*p).Stream.In).Read).expect("non-null function pointer")((*p).Stream.In)
                        as UInt32;
                (*p).Range <<= 8 as std::ffi::c_int;
                (*p).Low <<= 8 as std::ffi::c_int;
            }
            (*p).FoundState = s;
            sym = (*s).Symbol;
            Ppmd8_Update1_0(p);
            return sym as std::ffi::c_int;
        }
        (*p).PrevSuccess = 0 as std::ffi::c_int as std::ffi::c_uint;
        i = (*(*p).MinContext).NumStats as std::ffi::c_uint;
        loop {
            s = s.offset(1);
            count = count.wrapping_sub((*s).Freq as UInt32);
            if (count as Int32) < 0 as std::ffi::c_int {
                let mut sym_0: Byte = 0;
                Ppmd8_RD_Decode(
                    p,
                    hiCnt.wrapping_sub(count).wrapping_sub((*s).Freq as UInt32),
                    (*s).Freq as UInt32,
                );
                while (*p).Low ^ ((*p).Low).wrapping_add((*p).Range)
                    < (1 as std::ffi::c_int as UInt32) << 24 as std::ffi::c_int
                    || (*p).Range < (1 as std::ffi::c_int as UInt32) << 15 as std::ffi::c_int && {
                        (*p).Range = (0 as std::ffi::c_int as UInt32).wrapping_sub((*p).Low)
                            & ((1 as std::ffi::c_int as UInt32) << 15 as std::ffi::c_int)
                                .wrapping_sub(1 as std::ffi::c_int as UInt32);
                        1 as std::ffi::c_int != 0
                    }
                {
                    (*p).Code = (*p).Code << 8 as std::ffi::c_int
                        | ((*(*p).Stream.In).Read).expect("non-null function pointer")(
                            (*p).Stream.In,
                        ) as UInt32;
                    (*p).Range <<= 8 as std::ffi::c_int;
                    (*p).Low <<= 8 as std::ffi::c_int;
                }
                (*p).FoundState = s;
                sym_0 = (*s).Symbol;
                Ppmd8_Update1(p);
                return sym_0 as std::ffi::c_int;
            }
            i = i.wrapping_sub(1);
            if !(i != 0) {
                break;
            }
        }
        if hiCnt >= summFreq {
            return -(2 as std::ffi::c_int);
        }
        hiCnt = hiCnt.wrapping_sub(count);
        Ppmd8_RD_Decode(p, hiCnt, summFreq.wrapping_sub(hiCnt));
        let mut z: size_t = 0;
        z = 0 as std::ffi::c_int as size_t;
        while z
            < (256 as std::ffi::c_int as std::ffi::c_ulong)
                .wrapping_div(::core::mem::size_of::<size_t>() as std::ffi::c_ulong)
        {
            charMask[z.wrapping_add(0 as std::ffi::c_int as size_t) as usize] =
                !(0 as std::ffi::c_int as size_t);
            charMask[z.wrapping_add(1 as std::ffi::c_int as size_t) as usize] =
                charMask[z.wrapping_add(0 as std::ffi::c_int as size_t) as usize];
            charMask[z.wrapping_add(2 as std::ffi::c_int as size_t) as usize] =
                charMask[z.wrapping_add(1 as std::ffi::c_int as size_t) as usize];
            charMask[z.wrapping_add(3 as std::ffi::c_int as size_t) as usize] =
                charMask[z.wrapping_add(2 as std::ffi::c_int as size_t) as usize];
            charMask[z.wrapping_add(4 as std::ffi::c_int as size_t) as usize] =
                charMask[z.wrapping_add(3 as std::ffi::c_int as size_t) as usize];
            charMask[z.wrapping_add(5 as std::ffi::c_int as size_t) as usize] =
                charMask[z.wrapping_add(4 as std::ffi::c_int as size_t) as usize];
            charMask[z.wrapping_add(6 as std::ffi::c_int as size_t) as usize] =
                charMask[z.wrapping_add(5 as std::ffi::c_int as size_t) as usize];
            charMask[z.wrapping_add(7 as std::ffi::c_int as size_t) as usize] =
                charMask[z.wrapping_add(6 as std::ffi::c_int as size_t) as usize];
            z = z.wrapping_add(8 as std::ffi::c_int as size_t);
        }
        let mut s2: *mut CPpmd_State = ((*p).Base).offset((*(*p).MinContext).Union4.Stats as isize)
            as *mut std::ffi::c_void as *mut CPpmd_State;
        *(charMask.as_mut_ptr() as *mut Byte).offset((*s).Symbol as isize) =
            0 as std::ffi::c_int as Byte;
        loop {
            let sym0: std::ffi::c_uint =
                (*s2.offset(0 as std::ffi::c_int as isize)).Symbol as std::ffi::c_uint;
            let sym1: std::ffi::c_uint =
                (*s2.offset(1 as std::ffi::c_int as isize)).Symbol as std::ffi::c_uint;
            s2 = s2.offset(2 as std::ffi::c_int as isize);
            *(charMask.as_mut_ptr() as *mut Byte).offset(sym0 as isize) =
                0 as std::ffi::c_int as Byte;
            *(charMask.as_mut_ptr() as *mut Byte).offset(sym1 as isize) =
                0 as std::ffi::c_int as Byte;
            if !(s2 < s) {
                break;
            }
        }
    } else {
        let mut s_0: *mut CPpmd_State =
            &mut (*(*p).MinContext).Union2 as *mut C2RustUnnamed_0 as *mut CPpmd_State;
        let mut prob: *mut UInt16 = &mut *(*((*p).BinSumm).as_mut_ptr().offset(
            *((*p).NS2Indx).as_mut_ptr().offset(
                ((*(&mut (*(*p).MinContext).Union2 as *mut C2RustUnnamed_0 as *mut CPpmd_State))
                    .Freq as size_t)
                    .wrapping_sub(1 as std::ffi::c_int as size_t) as isize,
            ) as isize,
        ))
        .as_mut_ptr()
        .offset(
            ((*p).PrevSuccess)
                .wrapping_add(
                    ((*p).RunLength >> 26 as std::ffi::c_int & 0x20 as std::ffi::c_int)
                        as std::ffi::c_uint,
                )
                .wrapping_add(
                    *((*p).NS2BSIndx).as_mut_ptr().offset(
                        (*(((*p).Base).offset((*(*p).MinContext).Suffix as isize)
                            as *mut std::ffi::c_void
                            as *mut CPpmd8_Context))
                            .NumStats as isize,
                    ) as std::ffi::c_uint,
                )
                .wrapping_add((*(*p).MinContext).Flags as std::ffi::c_int as std::ffi::c_uint)
                as isize,
        ) as *mut UInt16;
        let mut pr: UInt32 = *prob as UInt32;
        let mut size0: UInt32 = ((*p).Range >> 14 as std::ffi::c_int) * pr;
        pr = pr.wrapping_sub(
            pr.wrapping_add(
                ((1 as std::ffi::c_int) << 7 as std::ffi::c_int - 2 as std::ffi::c_int) as UInt32,
            ) >> 7 as std::ffi::c_int,
        );
        if (*p).Code < size0 {
            let mut sym_1: Byte = 0;
            *prob = pr.wrapping_add(((1 as std::ffi::c_int) << 7 as std::ffi::c_int) as UInt32)
                as UInt16;
            (*p).Range = size0;
            while (*p).Low ^ ((*p).Low).wrapping_add((*p).Range)
                < (1 as std::ffi::c_int as UInt32) << 24 as std::ffi::c_int
                || (*p).Range < (1 as std::ffi::c_int as UInt32) << 15 as std::ffi::c_int && {
                    (*p).Range = (0 as std::ffi::c_int as UInt32).wrapping_sub((*p).Low)
                        & ((1 as std::ffi::c_int as UInt32) << 15 as std::ffi::c_int)
                            .wrapping_sub(1 as std::ffi::c_int as UInt32);
                    1 as std::ffi::c_int != 0
                }
            {
                (*p).Code = (*p).Code << 8 as std::ffi::c_int
                    | ((*(*p).Stream.In).Read).expect("non-null function pointer")((*p).Stream.In)
                        as UInt32;
                (*p).Range <<= 8 as std::ffi::c_int;
                (*p).Low <<= 8 as std::ffi::c_int;
            }
            let mut freq: std::ffi::c_uint = (*s_0).Freq as std::ffi::c_uint;
            let mut c: *mut CPpmd8_Context = ((*p).Base).offset(
                ((*s_0).Successor_0 as UInt32
                    | ((*s_0).Successor_1 as UInt32) << 16 as std::ffi::c_int)
                    as isize,
            ) as *mut std::ffi::c_void
                as *mut CPpmd8_Context;
            sym_1 = (*s_0).Symbol;
            (*p).FoundState = s_0;
            (*p).PrevSuccess = 1 as std::ffi::c_int as std::ffi::c_uint;
            (*p).RunLength += 1;
            (*p).RunLength;
            (*s_0).Freq = freq.wrapping_add(
                (freq < 196 as std::ffi::c_int as std::ffi::c_uint) as std::ffi::c_int
                    as std::ffi::c_uint,
            ) as Byte;
            if (*p).OrderFall == 0 as std::ffi::c_int as std::ffi::c_uint
                && c as *const Byte >= (*p).UnitsStart as *const Byte
            {
                (*p).MinContext = c;
                (*p).MaxContext = (*p).MinContext;
            } else {
                Ppmd8_UpdateModel(p);
            }
            return sym_1 as std::ffi::c_int;
        }
        *prob = pr as UInt16;
        (*p).InitEsc = (*p).ExpEscape[(pr >> 10 as std::ffi::c_int) as usize] as std::ffi::c_uint;
        (*p).Low = ((*p).Low).wrapping_add(size0);
        (*p).Code = ((*p).Code).wrapping_sub(size0);
        (*p).Range = ((*p).Range
            & !(((1 as std::ffi::c_int) << 7 as std::ffi::c_int + 7 as std::ffi::c_int) as UInt32)
                .wrapping_sub(1 as std::ffi::c_int as UInt32))
        .wrapping_sub(size0);
        let mut z_0: size_t = 0;
        z_0 = 0 as std::ffi::c_int as size_t;
        while z_0
            < (256 as std::ffi::c_int as std::ffi::c_ulong)
                .wrapping_div(::core::mem::size_of::<size_t>() as std::ffi::c_ulong)
        {
            charMask[z_0.wrapping_add(0 as std::ffi::c_int as size_t) as usize] =
                !(0 as std::ffi::c_int as size_t);
            charMask[z_0.wrapping_add(1 as std::ffi::c_int as size_t) as usize] =
                charMask[z_0.wrapping_add(0 as std::ffi::c_int as size_t) as usize];
            charMask[z_0.wrapping_add(2 as std::ffi::c_int as size_t) as usize] =
                charMask[z_0.wrapping_add(1 as std::ffi::c_int as size_t) as usize];
            charMask[z_0.wrapping_add(3 as std::ffi::c_int as size_t) as usize] =
                charMask[z_0.wrapping_add(2 as std::ffi::c_int as size_t) as usize];
            charMask[z_0.wrapping_add(4 as std::ffi::c_int as size_t) as usize] =
                charMask[z_0.wrapping_add(3 as std::ffi::c_int as size_t) as usize];
            charMask[z_0.wrapping_add(5 as std::ffi::c_int as size_t) as usize] =
                charMask[z_0.wrapping_add(4 as std::ffi::c_int as size_t) as usize];
            charMask[z_0.wrapping_add(6 as std::ffi::c_int as size_t) as usize] =
                charMask[z_0.wrapping_add(5 as std::ffi::c_int as size_t) as usize];
            charMask[z_0.wrapping_add(7 as std::ffi::c_int as size_t) as usize] =
                charMask[z_0.wrapping_add(6 as std::ffi::c_int as size_t) as usize];
            z_0 = z_0.wrapping_add(8 as std::ffi::c_int as size_t);
        }
        *(charMask.as_mut_ptr() as *mut Byte).offset(
            (*(&mut (*(*p).MinContext).Union2 as *mut C2RustUnnamed_0 as *mut CPpmd_State)).Symbol
                as isize,
        ) = 0 as std::ffi::c_int as Byte;
        (*p).PrevSuccess = 0 as std::ffi::c_int as std::ffi::c_uint;
    }
    loop {
        let mut s_1: *mut CPpmd_State = 0 as *mut CPpmd_State;
        let mut s2_0: *mut CPpmd_State = 0 as *mut CPpmd_State;
        let mut freqSum: UInt32 = 0;
        let mut count_0: UInt32 = 0;
        let mut hiCnt_0: UInt32 = 0;
        let mut freqSum2: UInt32 = 0;
        let mut see: *mut CPpmd_See = 0 as *mut CPpmd_See;
        let mut mc: *mut CPpmd8_Context = 0 as *mut CPpmd8_Context;
        let mut numMasked: std::ffi::c_uint = 0;
        while (*p).Low ^ ((*p).Low).wrapping_add((*p).Range)
            < (1 as std::ffi::c_int as UInt32) << 24 as std::ffi::c_int
            || (*p).Range < (1 as std::ffi::c_int as UInt32) << 15 as std::ffi::c_int && {
                (*p).Range = (0 as std::ffi::c_int as UInt32).wrapping_sub((*p).Low)
                    & ((1 as std::ffi::c_int as UInt32) << 15 as std::ffi::c_int)
                        .wrapping_sub(1 as std::ffi::c_int as UInt32);
                1 as std::ffi::c_int != 0
            }
        {
            (*p).Code = (*p).Code << 8 as std::ffi::c_int
                | ((*(*p).Stream.In).Read).expect("non-null function pointer")((*p).Stream.In)
                    as UInt32;
            (*p).Range <<= 8 as std::ffi::c_int;
            (*p).Low <<= 8 as std::ffi::c_int;
        }
        mc = (*p).MinContext;
        numMasked = (*mc).NumStats as std::ffi::c_uint;
        loop {
            (*p).OrderFall = ((*p).OrderFall).wrapping_add(1);
            (*p).OrderFall;
            if (*mc).Suffix == 0 {
                return -(1 as std::ffi::c_int);
            }
            mc = ((*p).Base).offset((*mc).Suffix as isize) as *mut std::ffi::c_void
                as *mut CPpmd8_Context;
            if !((*mc).NumStats as std::ffi::c_uint == numMasked) {
                break;
            }
        }
        s_1 = ((*p).Base).offset((*mc).Union4.Stats as isize) as *mut std::ffi::c_void
            as *mut CPpmd_State;
        let mut num: std::ffi::c_uint = ((*mc).NumStats as std::ffi::c_uint)
            .wrapping_add(1 as std::ffi::c_int as std::ffi::c_uint);
        let mut num2: std::ffi::c_uint = num.wrapping_div(2 as std::ffi::c_int as std::ffi::c_uint);
        num &= 1 as std::ffi::c_int as std::ffi::c_uint;
        hiCnt_0 = (*s_1).Freq as UInt32
            & *(charMask.as_mut_ptr() as *mut Byte).offset((*s_1).Symbol as isize) as UInt32
            & (0 as std::ffi::c_int as UInt32).wrapping_sub(num);
        s_1 = s_1.offset(num as isize);
        (*p).MinContext = mc;
        loop {
            let sym0_0: std::ffi::c_uint =
                (*s_1.offset(0 as std::ffi::c_int as isize)).Symbol as std::ffi::c_uint;
            let sym1_0: std::ffi::c_uint =
                (*s_1.offset(1 as std::ffi::c_int as isize)).Symbol as std::ffi::c_uint;
            s_1 = s_1.offset(2 as std::ffi::c_int as isize);
            hiCnt_0 = hiCnt_0.wrapping_add(
                (*s_1.offset(-(2 as std::ffi::c_int) as isize)).Freq as UInt32
                    & *(charMask.as_mut_ptr() as *mut Byte).offset(sym0_0 as isize) as UInt32,
            );
            hiCnt_0 = hiCnt_0.wrapping_add(
                (*s_1.offset(-(1 as std::ffi::c_int) as isize)).Freq as UInt32
                    & *(charMask.as_mut_ptr() as *mut Byte).offset(sym1_0 as isize) as UInt32,
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
            let mut sym_2: Byte = 0;
            s_1 = ((*p).Base).offset((*(*p).MinContext).Union4.Stats as isize)
                as *mut std::ffi::c_void as *mut CPpmd_State;
            hiCnt_0 = count_0;
            loop {
                count_0 = count_0.wrapping_sub(
                    (*s_1).Freq as UInt32
                        & *(charMask.as_mut_ptr() as *mut Byte).offset((*s_1).Symbol as isize)
                            as UInt32,
                );
                s_1 = s_1.offset(1);
                s_1;
                if (count_0 as Int32) < 0 as std::ffi::c_int {
                    break;
                }
            }
            s_1 = s_1.offset(-1);
            s_1;
            Ppmd8_RD_Decode(
                p,
                hiCnt_0
                    .wrapping_sub(count_0)
                    .wrapping_sub((*s_1).Freq as UInt32),
                (*s_1).Freq as UInt32,
            );
            while (*p).Low ^ ((*p).Low).wrapping_add((*p).Range)
                < (1 as std::ffi::c_int as UInt32) << 24 as std::ffi::c_int
                || (*p).Range < (1 as std::ffi::c_int as UInt32) << 15 as std::ffi::c_int && {
                    (*p).Range = (0 as std::ffi::c_int as UInt32).wrapping_sub((*p).Low)
                        & ((1 as std::ffi::c_int as UInt32) << 15 as std::ffi::c_int)
                            .wrapping_sub(1 as std::ffi::c_int as UInt32);
                    1 as std::ffi::c_int != 0
                }
            {
                (*p).Code = (*p).Code << 8 as std::ffi::c_int
                    | ((*(*p).Stream.In).Read).expect("non-null function pointer")((*p).Stream.In)
                        as UInt32;
                (*p).Range <<= 8 as std::ffi::c_int;
                (*p).Low <<= 8 as std::ffi::c_int;
            }
            if ((*see).Shift as std::ffi::c_int) < 7 as std::ffi::c_int && {
                (*see).Count = ((*see).Count).wrapping_sub(1);
                (*see).Count as std::ffi::c_int == 0 as std::ffi::c_int
            } {
                (*see).Summ = (((*see).Summ as std::ffi::c_int) << 1 as std::ffi::c_int) as UInt16;
                let fresh0 = (*see).Shift;
                (*see).Shift = ((*see).Shift).wrapping_add(1);
                (*see).Count = ((3 as std::ffi::c_int) << fresh0 as std::ffi::c_int) as Byte;
            }
            (*p).FoundState = s_1;
            sym_2 = (*s_1).Symbol;
            Ppmd8_Update2(p);
            return sym_2 as std::ffi::c_int;
        }
        if count_0 >= freqSum2 {
            return -(2 as std::ffi::c_int);
        }
        Ppmd8_RD_Decode(p, hiCnt_0, freqSum2.wrapping_sub(hiCnt_0));
        (*see).Summ = ((*see).Summ as UInt32).wrapping_add(freqSum) as UInt16;
        s_1 = ((*p).Base).offset((*(*p).MinContext).Union4.Stats as isize) as *mut std::ffi::c_void
            as *mut CPpmd_State;
        s2_0 = s_1
            .offset((*(*p).MinContext).NumStats as std::ffi::c_int as isize)
            .offset(1 as std::ffi::c_int as isize);
        loop {
            *(charMask.as_mut_ptr() as *mut Byte).offset((*s_1).Symbol as isize) =
                0 as std::ffi::c_int as Byte;
            s_1 = s_1.offset(1);
            s_1;
            if !(s_1 != s2_0) {
                break;
            }
        }
    }
}
