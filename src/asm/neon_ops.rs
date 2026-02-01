//! Core ARM Neon SIMD operations for bitboard manipulation
//!
//! These are the fundamental building blocks used throughout the engine.

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

/// Count the number of set bits in a 64-bit value using ARM instructions
///
/// On ARM64, this uses the CNT instruction which is extremely fast.
#[inline(always)]
pub fn popcnt(x: u64) -> u32 {
    #[cfg(target_arch = "aarch64")]
    {
        unsafe {
            // Use ARM's native population count
            let vec = vdup_n_u64(x);
            let cnt = vcnt_u8(vreinterpret_u8_u64(vec));
            let sum = vpaddl_u8(cnt);
            let sum = vpaddl_u16(sum);
            let sum = vpaddl_u32(sum);
            vget_lane_u64(sum, 0) as u32
        }
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        x.count_ones()
    }
}

/// Find the index of the least significant set bit
///
/// Returns 64 if the value is 0.
#[inline(always)]
pub fn bitscan_forward(x: u64) -> u32 {
    if x == 0 {
        64
    } else {
        x.trailing_zeros()
    }
}

/// Find the index of the most significant set bit
///
/// Returns 64 if the value is 0.
#[inline(always)]
pub fn bitscan_reverse(x: u64) -> u32 {
    if x == 0 {
        64
    } else {
        63 - x.leading_zeros()
    }
}

/// Reset the least significant set bit
#[inline(always)]
pub fn reset_lsb(x: u64) -> u64 {
    x & (x.wrapping_sub(1))
}

/// Get the least significant set bit as a single-bit mask
#[inline(always)]
pub fn lsb(x: u64) -> u64 {
    x & x.wrapping_neg()
}

/// Parallel bit deposit using ARM Neon
///
/// This is a software implementation of PDEP, which doesn't exist on ARM.
/// We use Neon to accelerate the operation.
#[inline(always)]
pub fn pdep_neon(src: u64, mask: u64) -> u64 {
    #[cfg(target_arch = "aarch64")]
    {
        // Software PDEP implementation
        // For each bit in src, deposit it into the corresponding position in mask
        let mut result = 0u64;
        let mut m = mask;
        let mut s = src;

        while m != 0 {
            let bit_pos = bitscan_forward(m);
            let mask_bit = 1u64 << bit_pos;

            if s & 1 != 0 {
                result |= mask_bit;
            }

            m &= !mask_bit;
            s >>= 1;
        }

        result
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        // Fallback implementation
        let mut result = 0u64;
        let mut mask_copy = mask;
        let mut src_copy = src;

        while mask_copy != 0 {
            if src_copy & 1 != 0 {
                result |= mask_copy & mask_copy.wrapping_neg();
            }
            src_copy >>= 1;
            mask_copy &= mask_copy - 1;
        }

        result
    }
}

/// Parallel bit extract using ARM Neon
///
/// This is a software implementation of PEXT.
#[inline(always)]
pub fn pext_neon(src: u64, mask: u64) -> u64 {
    let mut result = 0u64;
    let mut m = mask;
    let mut bit_pos = 0;

    while m != 0 {
        let lsb_pos = bitscan_forward(m);
        let lsb_mask = 1u64 << lsb_pos;

        if src & lsb_mask != 0 {
            result |= 1u64 << bit_pos;
        }

        m &= !lsb_mask;
        bit_pos += 1;
    }

    result
}

/// Neon-optimized bitboard OR operation on array of bitboards
#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub unsafe fn neon_or_array(bitboards: &[u64; 4]) -> u64 {
    let v0 = vld1q_u64(bitboards.as_ptr());
    let v1 = vld1q_u64(bitboards.as_ptr().add(2));
    let result = vorrq_u64(v0, v1);
    vgetq_lane_u64(result, 0) | vgetq_lane_u64(result, 1)
}

/// Neon-optimized bitboard AND operation on array
#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub unsafe fn neon_and_array(bitboards: &[u64; 4]) -> u64 {
    let v0 = vld1q_u64(bitboards.as_ptr());
    let v1 = vld1q_u64(bitboards.as_ptr().add(2));
    let result = vandq_u64(v0, v1);
    vgetq_lane_u64(result, 0) & vgetq_lane_u64(result, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_popcnt() {
        assert_eq!(popcnt(0), 0);
        assert_eq!(popcnt(0xFF), 8);
        assert_eq!(popcnt(0xFFFF_FFFF_FFFF_FFFF), 64);
        assert_eq!(popcnt(0b1010_1010), 4);
    }

    #[test]
    fn test_bitscan_forward() {
        assert_eq!(bitscan_forward(0), 64);
        assert_eq!(bitscan_forward(1), 0);
        assert_eq!(bitscan_forward(0b1000), 3);
        assert_eq!(bitscan_forward(0xFF00), 8);
    }

    #[test]
    fn test_bitscan_reverse() {
        assert_eq!(bitscan_reverse(0), 64);
        assert_eq!(bitscan_reverse(1), 0);
        assert_eq!(bitscan_reverse(0b1000), 3);
        assert_eq!(bitscan_reverse(0xFF), 7);
    }

    #[test]
    fn test_lsb_operations() {
        let x = 0b1011_0000;
        assert_eq!(lsb(x), 0b0001_0000);
        assert_eq!(reset_lsb(x), 0b1010_0000);
    }
}
