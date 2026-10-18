use crate::block::BlockState;
use crate::morton::{encode_morton, morton_to_linear, LINEAR_TO_MORTON};

pub const SECTION_VOXELS: usize = 4096;
pub const WORDS_B4: usize = 256; // 4096 * 4 bits / 64 = 256 u64 words (2048 bytes)
pub const WORDS_B8: usize = 512; // 4096 * 8 bits / 64 = 512 u64 words (4096 bytes)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteKind {
    SingleValue,
    B4,
    B8,
    Direct,
}

/// Dynamic paletted container with adaptive bits per block (0, 4, 8, 16 bits).
/// Voxels are arranged in Morton (Z-order) space-filling curve for optimal L1/L2 cache locality.
#[derive(Debug, Clone, PartialEq)]
pub enum PalettedContainer {
    /// 0 bits per voxel: uniform section (e.g. completely empty air or solid stone).
    /// Zero heap allocation!
    SingleValue(BlockState),

    /// 4 bits per voxel (up to 16 unique BlockStates in palette).
    /// 256 u64 words = 2048 bytes on heap.
    B4 {
        palette: Vec<BlockState>,
        data: Box<[u64; WORDS_B4]>,
    },

    /// 8 bits per voxel (up to 256 unique BlockStates in palette).
    /// 512 u64 words = 4096 bytes on heap.
    B8 {
        palette: Vec<BlockState>,
        data: Box<[u64; WORDS_B8]>,
    },

    /// Direct / Global palette (16 bits per voxel, > 256 unique BlockStates).
    /// 4096 u16 = 8192 bytes on heap.
    Direct {
        data: Box<[u16; SECTION_VOXELS]>,
    },
}

#[inline(always)]
fn read_u16_le(slice: &[u8], idx: usize) -> u16 {
    let offset = idx * 2;
    (slice[offset] as u16) | ((slice[offset + 1] as u16) << 8)
}

impl Default for PalettedContainer {
    fn default() -> Self {
        Self::new_empty()
    }
}

thread_local! {
    static PALETTE_LUT: std::cell::RefCell<[u8; 65536]> = const { std::cell::RefCell::new([0xFF; 65536]) };
}

impl PalettedContainer {
    /// Creates a new empty section with SingleValue(AIR), allocating 0 heap bytes.
    pub fn new_empty() -> Self {
        Self::SingleValue(BlockState::AIR)
    }

    /// Creates a single-value section of uniform block state.
    pub fn new_single(state: BlockState) -> Self {
        Self::SingleValue(state)
    }

    /// Returns the active palette strategy.
    #[inline]
    pub fn kind(&self) -> PaletteKind {
        match self {
            Self::SingleValue(_) => PaletteKind::SingleValue,
            Self::B4 { .. } => PaletteKind::B4,
            Self::B8 { .. } => PaletteKind::B8,
            Self::Direct { .. } => PaletteKind::Direct,
        }
    }

    /// Returns the number of bits allocated per voxel (0, 4, 8, or 16).
    #[inline]
    pub fn bits_per_block(&self) -> usize {
        match self {
            Self::SingleValue(_) => 0,
            Self::B4 { .. } => 4,
            Self::B8 { .. } => 8,
            Self::Direct { .. } => 16,
        }
    }

    /// Returns heap memory usage in bytes.
    pub fn heap_memory_usage(&self) -> usize {
        match self {
            Self::SingleValue(_) => 0,
            Self::B4 { palette, .. } => {
                palette.capacity() * std::mem::size_of::<BlockState>() + WORDS_B4 * 8
            }
            Self::B8 { palette, .. } => {
                palette.capacity() * std::mem::size_of::<BlockState>() + WORDS_B8 * 8
            }
            Self::Direct { .. } => SECTION_VOXELS * std::mem::size_of::<u16>(),
        }
    }

    /// Returns slice of palette entries if indirect.
    pub fn palette_entries(&self) -> Option<&[BlockState]> {
        match self {
            Self::SingleValue(_) => None,
            Self::B4 { palette, .. } => Some(palette.as_slice()),
            Self::B8 { palette, .. } => Some(palette.as_slice()),
            Self::Direct { .. } => None,
        }
    }

    /// Gets block state by section 3D coordinates (x: 0..15, y: 0..15, z: 0..15).
    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockState {
        let m = encode_morton(x, y, z);
        self.get_morton(m)
    }

    /// Gets block state by 12-bit Morton code index (0..4095).
    #[inline]
    pub fn get_morton(&self, m: usize) -> BlockState {
        match self {
            Self::SingleValue(s) => *s,
            Self::B4 { palette, data } => {
                let word = data[m >> 4];
                let shift = (m & 15) * 4;
                let idx = ((word >> shift) & 0x0F) as usize;
                palette[idx]
            }
            Self::B8 { palette, data } => {
                let word = data[m >> 3];
                let shift = (m & 7) * 8;
                let idx = ((word >> shift) & 0xFF) as usize;
                palette[idx]
            }
            Self::Direct { data } => BlockState(data[m]),
        }
    }

    /// Gets block state by Protocol 47 linear index `(y << 8) | (z << 4) | x`.
    #[inline]
    pub fn get_linear(&self, linear: usize) -> BlockState {
        let m = LINEAR_TO_MORTON[linear & 4095] as usize;
        self.get_morton(m)
    }

    /// Sets block state at section 3D coordinates (x: 0..15, y: 0..15, z: 0..15).
    #[inline]
    pub fn set(&mut self, x: usize, y: usize, z: usize, state: BlockState) {
        let m = encode_morton(x, y, z);
        self.set_morton(m, state);
    }

    /// Sets block state by 12-bit Morton code index (0..4095).
    pub fn set_morton(&mut self, m: usize, state: BlockState) {
        match self {
            Self::SingleValue(curr) => {
                if *curr == state {
                    return;
                }
                let mut palette = Vec::with_capacity(16);
                palette.push(*curr);
                palette.push(state);

                let mut data = Box::new([0u64; WORDS_B4]);
                let word = m >> 4;
                let shift = (m & 15) * 4;
                data[word] |= 1u64 << shift;

                *self = Self::B4 { palette, data };
            }
            Self::B4 { palette, data } => {
                if let Some(pos) = palette.iter().position(|&s| s == state) {
                    let word = m >> 4;
                    let shift = (m & 15) * 4;
                    data[word] = (data[word] & !(0x0F << shift)) | ((pos as u64) << shift);
                    return;
                }

                if palette.len() < 16 {
                    let pos = palette.len();
                    palette.push(state);
                    let word = m >> 4;
                    let shift = (m & 15) * 4;
                    data[word] = (data[word] & !(0x0F << shift)) | ((pos as u64) << shift);
                    return;
                }

                // Promote B4 -> B8
                let mut new_palette = Vec::with_capacity(32);
                new_palette.extend_from_slice(palette);
                new_palette.push(state);

                let mut new_data = Box::new([0u64; WORDS_B8]);
                for i in 0..SECTION_VOXELS {
                    let w = data[i >> 4];
                    let s = (i & 15) * 4;
                    let val = (w >> s) & 0x0F;
                    let nw = i >> 3;
                    let ns = (i & 7) * 8;
                    new_data[nw] |= val << ns;
                }

                let nw = m >> 3;
                let ns = (m & 7) * 8;
                new_data[nw] = (new_data[nw] & !(0xFF << ns)) | (16u64 << ns);

                *self = Self::B8 {
                    palette: new_palette,
                    data: new_data,
                };
            }
            Self::B8 { palette, data } => {
                if let Some(pos) = palette.iter().position(|&s| s == state) {
                    let word = m >> 3;
                    let shift = (m & 7) * 8;
                    data[word] = (data[word] & !(0xFF << shift)) | ((pos as u64) << shift);
                    return;
                }

                if palette.len() < 256 {
                    let pos = palette.len();
                    palette.push(state);
                    let word = m >> 3;
                    let shift = (m & 7) * 8;
                    data[word] = (data[word] & !(0xFF << shift)) | ((pos as u64) << shift);
                    return;
                }

                // Promote B8 -> Direct
                let mut direct_data = Box::new([0u16; SECTION_VOXELS]);
                for i in 0..SECTION_VOXELS {
                    let w = data[i >> 3];
                    let s = (i & 7) * 8;
                    let p_idx = ((w >> s) & 0xFF) as usize;
                    direct_data[i] = palette[p_idx].0;
                }
                direct_data[m] = state.0;

                *self = Self::Direct { data: direct_data };
            }
            Self::Direct { data } => {
                data[m] = state.0;
            }
        }
    }

    /// Sets block state by Protocol 47 linear index `(y << 8) | (z << 4) | x`.
    #[inline]
    pub fn set_linear(&mut self, linear: usize, state: BlockState) {
        let m = LINEAR_TO_MORTON[linear & 4095] as usize;
        self.set_morton(m, state);
    }
    /// Loads block data from a raw 8192-byte Protocol 47 payload slice and computes non-air count in one pass.
    pub fn from_raw_slice_counted(slice: &[u8]) -> (Self, usize) {
        assert!(slice.len() >= SECTION_VOXELS * 2);

        let mut unique_states = Vec::with_capacity(16);
        let first = read_u16_le(slice, 0);
        unique_states.push(BlockState(first));
        let mut is_uniform = true;
        let mut non_air = if !BlockState(first).is_air() { 1 } else { 0 };

        for i in 1..SECTION_VOXELS {
            let raw = read_u16_le(slice, i);
            if !BlockState(raw).is_air() {
                non_air += 1;
            }
            if raw != first {
                is_uniform = false;
                if unique_states.len() <= 256 && !unique_states.iter().any(|s| s.0 == raw) {
                    unique_states.push(BlockState(raw));
                }
            }
        }

        if is_uniform {
            return (Self::SingleValue(BlockState(first)), non_air);
        }

        if unique_states.len() <= 16 {
            let palette = unique_states;
            let mut data = Box::new([0u64; WORDS_B4]);

            PALETTE_LUT.with(|lut_cell| {
                let mut lut = lut_cell.borrow_mut();
                for (idx, s) in palette.iter().enumerate() {
                    lut[s.0 as usize] = idx as u8;
                }
                for linear in 0..SECTION_VOXELS {
                    let raw = read_u16_le(slice, linear);
                    let m = LINEAR_TO_MORTON[linear] as usize;
                    let p_idx = lut[raw as usize] as usize;
                    let word = m >> 4;
                    let shift = (m & 15) * 4;
                    data[word] |= (p_idx as u64) << shift;
                }
                for s in &palette {
                    lut[s.0 as usize] = 0xFF;
                }
            });

            return (Self::B4 { palette, data }, non_air);
        }

        if unique_states.len() <= 256 {
            let palette = unique_states;
            let mut data = Box::new([0u64; WORDS_B8]);

            PALETTE_LUT.with(|lut_cell| {
                let mut lut = lut_cell.borrow_mut();
                for (idx, s) in palette.iter().enumerate() {
                    lut[s.0 as usize] = idx as u8;
                }
                for linear in 0..SECTION_VOXELS {
                    let raw = read_u16_le(slice, linear);
                    let m = LINEAR_TO_MORTON[linear] as usize;
                    let p_idx = lut[raw as usize] as usize;
                    let word = m >> 3;
                    let shift = (m & 7) * 8;
                    data[word] |= (p_idx as u64) << shift;
                }
                for s in &palette {
                    lut[s.0 as usize] = 0xFF;
                }
            });

            return (Self::B8 { palette, data }, non_air);
        }

        let mut data = Box::new([0u16; SECTION_VOXELS]);
        for linear in 0..SECTION_VOXELS {
            let raw = read_u16_le(slice, linear);
            let m = LINEAR_TO_MORTON[linear] as usize;
            data[m] = raw;
        }
        (Self::Direct { data }, non_air)
    }

    /// Loads block data from a raw 8192-byte Protocol 47 payload slice (linear order).
    #[inline]
    pub fn from_raw_slice(slice: &[u8]) -> Self {
        Self::from_raw_slice_counted(slice).0
    }

    /// Converts voxels back to a flat linear array matching Protocol 47 ordering.
    pub fn to_linear_array(&self) -> Box<[u16; SECTION_VOXELS]> {
        let mut out = Box::new([0u16; SECTION_VOXELS]);
        match self {
            Self::SingleValue(s) => {
                out.fill(s.0);
            }
            Self::B4 { palette, data } => {
                for linear in 0..SECTION_VOXELS {
                    let m = LINEAR_TO_MORTON[linear] as usize;
                    let word = data[m >> 4];
                    let shift = (m & 15) * 4;
                    let idx = ((word >> shift) & 0x0F) as usize;
                    out[linear] = palette[idx].0;
                }
            }
            Self::B8 { palette, data } => {
                for linear in 0..SECTION_VOXELS {
                    let m = LINEAR_TO_MORTON[linear] as usize;
                    let word = data[m >> 3];
                    let shift = (m & 7) * 8;
                    let idx = ((word >> shift) & 0xFF) as usize;
                    out[linear] = palette[idx].0;
                }
            }
            Self::Direct { data } => {
                for linear in 0..SECTION_VOXELS {
                    let m = LINEAR_TO_MORTON[linear] as usize;
                    out[linear] = data[m];
                }
            }
        }
        out
    }

    /// Writes linear 16-bit little endian voxel data into a buffer.
    pub fn write_linear_bytes(&self, buf: &mut Vec<u8>) {
        buf.reserve(SECTION_VOXELS * 2);
        match self {
            Self::SingleValue(s) => {
                let low = (s.0 & 0xFF) as u8;
                let high = (s.0 >> 8) as u8;
                for _ in 0..SECTION_VOXELS {
                    buf.push(low);
                    buf.push(high);
                }
            }
            Self::B4 { palette, data } => {
                for linear in 0..SECTION_VOXELS {
                    let m = LINEAR_TO_MORTON[linear] as usize;
                    let word = data[m >> 4];
                    let shift = (m & 15) * 4;
                    let idx = ((word >> shift) & 0x0F) as usize;
                    let raw = palette[idx].0;
                    buf.push((raw & 0xFF) as u8);
                    buf.push((raw >> 8) as u8);
                }
            }
            Self::B8 { palette, data } => {
                for linear in 0..SECTION_VOXELS {
                    let m = LINEAR_TO_MORTON[linear] as usize;
                    let word = data[m >> 3];
                    let shift = (m & 7) * 8;
                    let idx = ((word >> shift) & 0xFF) as usize;
                    let raw = palette[idx].0;
                    buf.push((raw & 0xFF) as u8);
                    buf.push((raw >> 8) as u8);
                }
            }
            Self::Direct { data } => {
                for linear in 0..SECTION_VOXELS {
                    let m = LINEAR_TO_MORTON[linear] as usize;
                    let raw = data[m];
                    buf.push((raw & 0xFF) as u8);
                    buf.push((raw >> 8) as u8);
                }
            }
        }
    }

    /// Recalculates number of non-air blocks in the container.
    pub fn count_non_air(&self) -> usize {
        match self {
            Self::SingleValue(s) => {
                if s.is_air() {
                    0
                } else {
                    SECTION_VOXELS
                }
            }
            Self::B4 { palette, data } => {
                let mut is_non_air = [false; 16];
                for (i, s) in palette.iter().enumerate() {
                    is_non_air[i] = !s.is_air();
                }
                let mut count = 0;
                for m in 0..SECTION_VOXELS {
                    let word = data[m >> 4];
                    let shift = (m & 15) * 4;
                    let p_idx = ((word >> shift) & 0x0F) as usize;
                    if is_non_air[p_idx] {
                        count += 1;
                    }
                }
                count
            }
            Self::B8 { palette, data } => {
                let mut is_non_air = [false; 256];
                for (i, s) in palette.iter().enumerate() {
                    is_non_air[i] = !s.is_air();
                }
                let mut count = 0;
                for m in 0..SECTION_VOXELS {
                    let word = data[m >> 3];
                    let shift = (m & 7) * 8;
                    let p_idx = ((word >> shift) & 0xFF) as usize;
                    if is_non_air[p_idx] {
                        count += 1;
                    }
                }
                count
            }
            Self::Direct { data } => data.iter().filter(|&&id| id != 0).count(),
        }
    }

    /// Fills 64-word bitmask of opaque blocks indexed by linear coordinates.
    /// Returns total number of opaque blocks in the section.
    pub fn fill_opaque_bits(&self, opaque_bits: &mut [u64; 64]) -> usize {
        match self {
            Self::SingleValue(s) => {
                if s.is_opaque() {
                    opaque_bits.fill(!0u64);
                    SECTION_VOXELS
                } else {
                    opaque_bits.fill(0);
                    0
                }
            }
            Self::B4 { palette, data } => {
                opaque_bits.fill(0);
                let mut is_opaque = [false; 16];
                for (i, s) in palette.iter().enumerate() {
                    is_opaque[i] = s.is_opaque();
                }
                let mut count = 0;
                for m in 0..SECTION_VOXELS {
                    let word = data[m >> 4];
                    let shift = (m & 15) * 4;
                    let p_idx = ((word >> shift) & 0x0F) as usize;
                    if is_opaque[p_idx] {
                        let linear = morton_to_linear(m);
                        opaque_bits[linear >> 6] |= 1u64 << (linear & 63);
                        count += 1;
                    }
                }
                count
            }
            Self::B8 { palette, data } => {
                opaque_bits.fill(0);
                let mut is_opaque = [false; 256];
                for (i, s) in palette.iter().enumerate() {
                    is_opaque[i] = s.is_opaque();
                }
                let mut count = 0;
                for m in 0..SECTION_VOXELS {
                    let word = data[m >> 3];
                    let shift = (m & 7) * 8;
                    let p_idx = ((word >> shift) & 0xFF) as usize;
                    if is_opaque[p_idx] {
                        let linear = morton_to_linear(m);
                        opaque_bits[linear >> 6] |= 1u64 << (linear & 63);
                        count += 1;
                    }
                }
                count
            }
            Self::Direct { data } => {
                opaque_bits.fill(0);
                let mut count = 0;
                for m in 0..SECTION_VOXELS {
                    let raw = data[m];
                    if raw != 0 && BlockState(raw).is_opaque() {
                        let linear = morton_to_linear(m);
                        opaque_bits[linear >> 6] |= 1u64 << (linear & 63);
                        count += 1;
                    }
                }
                count
            }
        }
    }

    /// Compacts container to the most minimal palette representation possible.
    /// Demotes Direct -> B8 -> B4 -> SingleValue, freeing unneeded heap memory.
    pub fn compact(&mut self) {
        if let Self::SingleValue(_) = self {
            return;
        }

        // Collect unique states
        let mut unique = Vec::with_capacity(16);
        for m in 0..SECTION_VOXELS {
            let state = self.get_morton(m);
            if !unique.contains(&state) {
                unique.push(state);
                if unique.len() > 256 {
                    return; // Stays Direct
                }
            }
        }

        if unique.len() == 1 {
            *self = Self::SingleValue(unique[0]);
            return;
        }

        if unique.len() <= 16 {
            let palette = unique;
            let mut data = Box::new([0u64; WORDS_B4]);
            for m in 0..SECTION_VOXELS {
                let state = self.get_morton(m);
                let p_idx = palette.iter().position(|&s| s == state).unwrap();
                let word = m >> 4;
                let shift = (m & 15) * 4;
                data[word] |= (p_idx as u64) << shift;
            }
            *self = Self::B4 { palette, data };
            return;
        }

        if unique.len() <= 256 {
            let palette = unique;
            let mut data = Box::new([0u64; WORDS_B8]);
            for m in 0..SECTION_VOXELS {
                let state = self.get_morton(m);
                let p_idx = palette.iter().position(|&s| s == state).unwrap();
                let word = m >> 3;
                let shift = (m & 7) * 8;
                data[word] |= (p_idx as u64) << shift;
            }
            *self = Self::B8 { palette, data };
        }
    }
}
