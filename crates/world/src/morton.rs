pub const MORTON_SPREAD_LUT: [u16; 16] = [
    0,   // 0: 0000 -> 000_000_000_000
    1,   // 1: 0001 -> 000_000_000_001
    8,   // 2: 0010 -> 000_000_001_000
    9,   // 3: 0011 -> 000_000_001_001
    64,  // 4: 0100 -> 000_001_000_000
    65,  // 5: 0101 -> 000_001_000_001
    72,  // 6: 0110 -> 000_001_001_000
    73,  // 7: 0111 -> 000_001_001_001
    512, // 8: 1000 -> 001_000_000_000
    513, // 9: 1001 -> 001_000_000_001
    520, // 10: 1010 -> 001_000_001_000
    521, // 11: 1011 -> 001_000_001_001
    576, // 12: 1100 -> 001_001_000_000
    577, // 13: 1101 -> 001_001_000_001
    584, // 14: 1110 -> 001_001_001_000
    585, // 15: 1111 -> 001_001_001_001
];

/// Encodes 3D coordinates (0..15) into a 12-bit Z-order Morton code (0..4095).
#[inline(always)]
pub const fn encode_morton(x: usize, y: usize, z: usize) -> usize {
    let sx = MORTON_SPREAD_LUT[x & 15] as usize;
    let sy = (MORTON_SPREAD_LUT[y & 15] as usize) << 1;
    let sz = (MORTON_SPREAD_LUT[z & 15] as usize) << 2;
    sx | sy | sz
}

/// Decodes a 12-bit Z-order Morton code (0..4095) back into 3D coordinates (x, y, z).
#[inline(always)]
pub fn decode_morton(morton: usize) -> (usize, usize, usize) {
    let m = morton as u32;

    #[cfg(target_arch = "x86_64")]
    {
        if std::is_x86_feature_detected!("bmi2") {
            unsafe {
                use std::arch::x86_64::_pext_u32;
                let x = _pext_u32(m, 0x249) as usize;
                let y = _pext_u32(m, 0x492) as usize;
                let z = _pext_u32(m, 0x924) as usize;
                return (x, y, z);
            }
        }
    }

    let x = (m & 1)
        | (((m >> 3) & 1) << 1)
        | (((m >> 6) & 1) << 2)
        | (((m >> 9) & 1) << 3);
    let y = ((m >> 1) & 1)
        | (((m >> 4) & 1) << 1)
        | (((m >> 7) & 1) << 2)
        | (((m >> 10) & 1) << 3);
    let z = ((m >> 2) & 1)
        | (((m >> 5) & 1) << 1)
        | (((m >> 8) & 1) << 2)
        | (((m >> 11) & 1) << 3);

    (x as usize, y as usize, z as usize)
}

/// Compile-time generation of linear-to-Morton translation table.
const fn generate_linear_to_morton() -> [u16; 4096] {
    let mut table = [0u16; 4096];
    let mut i = 0;
    while i < 4096 {
        let y = (i >> 8) & 15;
        let z = (i >> 4) & 15;
        let x = i & 15;
        table[i] = encode_morton(x, y, z) as u16;
        i += 1;
    }
    table
}

/// Compile-time generation of Morton-to-linear translation table.
const fn generate_morton_to_linear() -> [u16; 4096] {
    let mut table = [0u16; 4096];
    let mut m = 0;
    while m < 4096 {
        let x = (m & 1)
            | (((m >> 3) & 1) << 1)
            | (((m >> 6) & 1) << 2)
            | (((m >> 9) & 1) << 3);
        let y = ((m >> 1) & 1)
            | (((m >> 4) & 1) << 1)
            | (((m >> 7) & 1) << 2)
            | (((m >> 10) & 1) << 3);
        let z = ((m >> 2) & 1)
            | (((m >> 5) & 1) << 1)
            | (((m >> 8) & 1) << 2)
            | (((m >> 11) & 1) << 3);

        let linear = (y << 8) | (z << 4) | x;
        table[m] = linear as u16;
        m += 1;
    }
    table
}

pub const LINEAR_TO_MORTON: [u16; 4096] = generate_linear_to_morton();
pub const MORTON_TO_LINEAR: [u16; 4096] = generate_morton_to_linear();

/// Converts a linear index (y << 8 | z << 4 | x) into a Morton code index.
#[inline(always)]
pub fn linear_to_morton(linear: usize) -> usize {
    LINEAR_TO_MORTON[linear & 4095] as usize
}

/// Converts a Morton code index back into a linear index (y << 8 | z << 4 | x).
#[inline(always)]
pub fn morton_to_linear(morton: usize) -> usize {
    MORTON_TO_LINEAR[morton & 4095] as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_morton_roundtrip_all() {
        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    let code = encode_morton(x, y, z);
                    assert!(code < 4096);
                    let (dx, dy, dz) = decode_morton(code);
                    assert_eq!((x, y, z), (dx, dy, dz));
                }
            }
        }
    }

    #[test]
    fn test_linear_morton_bijective() {
        let mut visited = [false; 4096];
        for linear in 0..4096 {
            let m = linear_to_morton(linear);
            assert!(m < 4096);
            assert!(!visited[m], "Duplicate Morton code {m}");
            visited[m] = true;

            let back = morton_to_linear(m);
            assert_eq!(back, linear);
        }
    }
}
