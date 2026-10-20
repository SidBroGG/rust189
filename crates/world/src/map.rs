//! Minecraft 1.8.9 MapData representation, MapColor palette, and RGBA conversion.

/// Vanilla Minecraft 1.8.9 MapColor base RGB values (36 palette colors).
pub const MAP_BASE_COLORS: [u32; 36] = [
    0,        // 0: Air / Transparent
    8368696,  // 1: Grass
    16247203, // 2: Sand
    13092807, // 3: Cloth / Wool
    16711680, // 4: TNT
    10526975, // 5: Ice
    10987431, // 6: Iron
    31744,    // 7: Foliage
    16777215, // 8: Snow
    10791096, // 9: Clay
    9923917,  // 10: Dirt
    7368816,  // 11: Stone
    4210943,  // 12: Water
    9402184,  // 13: Wood
    16776437, // 14: Quartz
    14188339, // 15: Adobe / Clay (hardened)
    11685080, // 16: Magenta
    6724056,  // 17: Light Blue
    15066419, // 18: Yellow
    8375321,  // 19: Lime
    15892389, // 20: Pink
    5000268,  // 21: Gray
    10066329, // 22: Silver / Light Gray
    5013401,  // 23: Cyan
    8339378,  // 24: Purple
    3361970,  // 25: Blue
    6704179,  // 26: Brown
    6717235,  // 27: Green
    10040115, // 28: Red
    1644825,  // 29: Black
    16445005, // 30: Gold
    6085589,  // 31: Diamond
    4882687,  // 32: Lapis
    55610,    // 33: Emerald
    8476209,  // 34: Obsidian
    7340544,  // 35: Netherrack
];

/// Shade multipliers for brightness levels 0..3 (MCP MapColor.getMapColor).
pub const MAP_SHADE_MULTIPLIERS: [u32; 4] = [180, 220, 255, 135];

/// Represents a marker/icon on the map (e.g. player arrow, spawn marker).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapDecoration {
    pub icon_type: u8,
    pub direction: u8,
    pub x: i8,
    pub z: i8,
}

/// Dynamic 128x128 map data for a single filled map.
#[derive(Debug, Clone)]
pub struct MapData {
    pub map_id: i32,
    pub scale: u8,
    pub colors: Box<[u8; 16384]>,
    pub decorations: Vec<MapDecoration>,
    pub dirty: bool,
}

impl MapData {
    /// Creates a new empty map data container with all pixels set to 0.
    pub fn new(map_id: i32) -> Self {
        Self {
            map_id,
            scale: 0,
            colors: Box::new([0u8; 16384]),
            decorations: Vec::new(),
            dirty: true,
        }
    }

    /// Updates a rectangular region of the 128x128 map grid from packet data.
    pub fn update_data(&mut self, min_x: usize, min_y: usize, max_x: usize, max_y: usize, data: &[u8]) {
        let mut src_idx = 0;
        for y in 0..max_y {
            for x in 0..max_x {
                let target_x = min_x + x;
                let target_y = min_y + y;
                if target_x < 128 && target_y < 128 && src_idx < data.len() {
                    self.colors[target_x + target_y * 128] = data[src_idx];
                    src_idx += 1;
                }
            }
        }
        self.dirty = true;
    }

    /// Converts the 16384 color indices into an RGBA (128x128x4 = 65536 bytes) buffer.
    pub fn to_rgba(&self) -> Vec<u8> {
        let mut rgba = vec![0u8; 128 * 128 * 4];
        for i in 0..16384 {
            let val = self.colors[i];
            let color_idx = (val >> 2) as usize;
            let shade_idx = (val & 3) as usize;

            let out_idx = i * 4;
            if color_idx == 0 || color_idx >= MAP_BASE_COLORS.len() {
                // Subtle transparent checkerboard matching vanilla MCP MapItemRenderer
                let alpha = ((i + i / 128) & 1) * 8 + 16;
                rgba[out_idx] = 0;
                rgba[out_idx + 1] = 0;
                rgba[out_idx + 2] = 0;
                rgba[out_idx + 3] = alpha as u8;
            } else {
                let base = MAP_BASE_COLORS[color_idx];
                let mult = MAP_SHADE_MULTIPLIERS[shade_idx];

                let r = (((base >> 16) & 255) * mult / 255) as u8;
                let g = (((base >> 8) & 255) * mult / 255) as u8;
                let b = ((base & 255) * mult / 255) as u8;

                rgba[out_idx] = r;
                rgba[out_idx + 1] = g;
                rgba[out_idx + 2] = b;
                rgba[out_idx + 3] = 255;
            }
        }
        rgba
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_data_update_and_rgba_conversion() {
        let mut map = MapData::new(5);
        assert_eq!(map.map_id, 5);
        assert!(map.dirty);

        // Update a 2x2 area at (0, 0)
        // Value 4 = color_idx 1 (grass), shade 0 (mult 180)
        // Base grass = 8368696 (0x7FB238: R=127, G=178, B=56)
        // R = 127 * 180 / 255 = 89
        // G = 178 * 180 / 255 = 125
        // B = 56 * 180 / 255 = 39
        map.update_data(0, 0, 2, 2, &[4, 4, 4, 4]);
        assert_eq!(map.colors[0], 4);
        assert_eq!(map.colors[1], 4);
        assert_eq!(map.colors[128], 4);
        assert_eq!(map.colors[129], 4);

        let rgba = map.to_rgba();
        assert_eq!(rgba.len(), 65536);
        assert_eq!(rgba[0], 89);
        assert_eq!(rgba[1], 125);
        assert_eq!(rgba[2], 39);
        assert_eq!(rgba[3], 255);

        // Check index 2 (unmapped) -> transparent
        assert_eq!(rgba[8], 0);
        assert_eq!(rgba[9], 0);
        assert_eq!(rgba[10], 0);
        assert!(rgba[11] == 16 || rgba[11] == 24);
    }
}
