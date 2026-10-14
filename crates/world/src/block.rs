use serde::{Deserialize, Serialize};

/// Represents a 16-bit block state combining a 12-bit block ID and 4-bit metadata.
/// Matching Protocol 47 raw format: `(id << 4) | (metadata & 0x0F)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct BlockState(pub u16);

impl BlockState {
    pub const AIR: Self = Self(0);

    #[inline]
    pub const fn new(id: u16, meta: u8) -> Self {
        Self(((id & 0x0FFF) << 4) | ((meta as u16) & 0x0F))
    }

    #[inline]
    pub const fn from_raw(raw: u16) -> Self {
        Self(raw)
    }

    #[inline]
    pub const fn id(self) -> u16 {
        self.0 >> 4
    }

    #[inline]
    pub const fn meta(self) -> u8 {
        (self.0 & 0x0F) as u8
    }

    #[inline]
    pub const fn raw(self) -> u16 {
        self.0
    }

    #[inline]
    pub const fn is_air(self) -> bool {
        self.id() == 0
    }

    #[inline]
    pub const fn is_opaque(self) -> bool {
        let id = self.id();
        !matches!(
            id,
            0 | 6 | 8 | 9 | 10 | 11 | 18 | 20 | 26 | 27 | 28 | 30 | 31 | 32 | 37 | 38 | 39 | 40
                | 44 | 50 | 51 | 52 | 53 | 54 | 55 | 59 | 60 | 63 | 64 | 65 | 66 | 67 | 68 | 69
                | 70 | 71 | 72 | 75 | 76 | 77 | 78 | 79 | 81 | 83 | 85 | 92 | 93 | 94 | 95 | 96
                | 101 | 102 | 104 | 105 | 106 | 107 | 108 | 109 | 111 | 113 | 114 | 115 | 116 | 117
                | 118 | 119 | 120 | 126 | 127 | 128 | 130 | 131 | 132 | 134 | 135 | 136 | 139 | 140
                | 141 | 142 | 143 | 144 | 145 | 146 | 147 | 148 | 149 | 150 | 151 | 154 | 156 | 157
                | 160 | 161 | 163 | 164 | 165 | 166 | 167 | 171 | 175 | 176 | 177 | 178 | 180 | 182
                | 183 | 184 | 185 | 186 | 187 | 188 | 189 | 190 | 191 | 192 | 193 | 194 | 195 | 196
                | 197
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_state_packing() {
        let state = BlockState::new(1, 2); // Stone with variant 2 (Granite)
        assert_eq!(state.id(), 1);
        assert_eq!(state.meta(), 2);
        assert_eq!(state.raw(), (1 << 4) | 2);

        let state2 = BlockState::from_raw((123 << 4) | 15);
        assert_eq!(state2.id(), 123);
        assert_eq!(state2.meta(), 15);
    }
}
