pub(crate) mod ppmd7;
pub(crate) mod ppmd8;

const PPMD_INT_BITS: u32 = 7;
const PPMD_PERIOD_BITS: u32 = 7;
const PPMD_BIN_SCALE: u32 = 1 << (PPMD_INT_BITS + PPMD_PERIOD_BITS);

const fn ppmd_get_mean_spec(summ: u32, shift: u32, round: u32) -> u32 {
    (summ + (1 << (shift - round))) >> shift
}

const fn ppmd_get_mean(summ: u32) -> u32 {
    ppmd_get_mean_spec(summ, PPMD_PERIOD_BITS, 2)
}

const fn ppmd_update_prob_1(prob: u32) -> u32 {
    prob - ppmd_get_mean(prob)
}

const PPMD_N1: u32 = 4;
const PPMD_N2: u32 = 4;
const PPMD_N3: u32 = 4;
const PPMD_N4: u32 = (128 + 3 - PPMD_N1 - 2 * PPMD_N2 - 3 * PPMD_N3) / 4;
const PPMD_NUM_INDEXES: u32 = PPMD_N1 + PPMD_N2 + PPMD_N3 + PPMD_N4;
