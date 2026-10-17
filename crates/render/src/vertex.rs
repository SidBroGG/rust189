use bytemuck::{Pod, Zeroable};

/// Compressed 16-byte bit-packed vertex for voxel terrain rendering.
///
/// Reduces VRAM consumption and memory bandwidth by 66.7% (from 48 bytes to 16 bytes).
/// - `data0`: local_pos (X: 10 bits, Y: 10 bits, Z: 10 bits, scaled by 32, range 0..512), ao (2 bits).
/// - `data1`: atlas UV coordinates (U: 16 bits, V: 16 bits, normalized 0..65535).
/// - `data2`: color tint (R: 8 bits, G: 8 bits, B: 8 bits), light (block_light: 4 bits, sky_light: 4 bits).
/// - `data3`: greedy meshing local UV (lu: 8 bits, lv: 8 bits), is_greedy flag (1 bit), normal_index (3 bits), reserved (12 bits).
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Pod, Zeroable)]
pub struct BlockVertex {
    pub data0: u32,
    pub data1: u32,
    pub data2: u32,
    pub data3: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnpackedBlockVertex {
    pub local_pos: [f32; 3],
    pub uv: [f32; 2],
    pub light: [f32; 2],
    pub ao: u8,
    pub color: [f32; 3],
    pub normal_index: u32,
    pub greedy_uv: (u8, u8),
    pub is_greedy: bool,
}

impl BlockVertex {
    /// Packs terrain vertex attributes into a 16-byte compressed struct.
    pub fn pack(
        local_pos: [f32; 3],
        uv: [f32; 2],
        light: [f32; 2],
        ao: u8,
        color: [f32; 3],
        normal_index: u32,
        greedy_uv: (u8, u8),
        is_greedy: bool,
    ) -> Self {
        let px = ((local_pos[0].clamp(0.0, 16.0) * 32.0).round() as u32) & 1023;
        let py = ((local_pos[1].clamp(0.0, 16.0) * 32.0).round() as u32) & 1023;
        let pz = ((local_pos[2].clamp(0.0, 16.0) * 32.0).round() as u32) & 1023;
        let ao_val = (ao as u32) & 3;
        let data0 = px | (py << 10) | (pz << 20) | (ao_val << 30);

        let u_raw = ((uv[0].clamp(0.0, 1.0) * 65535.0).round() as u32) & 65535;
        let v_raw = ((uv[1].clamp(0.0, 1.0) * 65535.0).round() as u32) & 65535;
        let data1 = u_raw | (v_raw << 16);

        let cr = ((color[0].clamp(0.0, 1.0) * 255.0).round() as u32) & 255;
        let cg = ((color[1].clamp(0.0, 1.0) * 255.0).round() as u32) & 255;
        let cb = ((color[2].clamp(0.0, 1.0) * 255.0).round() as u32) & 255;
        let bl = ((light[0].clamp(0.0, 1.0) * 15.0).round() as u32) & 15;
        let sl = ((light[1].clamp(0.0, 1.0) * 15.0).round() as u32) & 15;
        let data2 = cr | (cg << 8) | (cb << 16) | (bl << 24) | (sl << 28);

        let lu = (greedy_uv.0 as u32) & 255;
        let lv = (greedy_uv.1 as u32) & 255;
        let greedy_flag = if is_greedy { 1u32 } else { 0u32 };
        let norm = (normal_index & 7) as u32;
        let data3 = lu | (lv << 8) | (greedy_flag << 16) | (norm << 17);

        Self {
            data0,
            data1,
            data2,
            data3,
        }
    }

    /// Unpacks the compressed vertex into floating point attributes for inspection and testing.
    pub fn unpack(&self) -> UnpackedBlockVertex {
        let px = (self.data0 & 1023) as f32 / 32.0;
        let py = ((self.data0 >> 10) & 1023) as f32 / 32.0;
        let pz = ((self.data0 >> 20) & 1023) as f32 / 32.0;
        let ao = ((self.data0 >> 30) & 3) as u8;

        let u = (self.data1 & 65535) as f32 / 65535.0;
        let v = ((self.data1 >> 16) & 65535) as f32 / 65535.0;

        let cr = (self.data2 & 255) as f32 / 255.0;
        let cg = ((self.data2 >> 8) & 255) as f32 / 255.0;
        let cb = ((self.data2 >> 16) & 255) as f32 / 255.0;
        let bl = ((self.data2 >> 24) & 15) as f32 / 15.0;
        let sl = ((self.data2 >> 28) & 15) as f32 / 15.0;

        let lu = (self.data3 & 255) as u8;
        let lv = ((self.data3 >> 8) & 255) as u8;
        let is_greedy = ((self.data3 >> 16) & 1) != 0;
        let normal_index = (self.data3 >> 17) & 7;

        UnpackedBlockVertex {
            local_pos: [px, py, pz],
            uv: [u, v],
            light: [bl, sl],
            ao,
            color: [cr, cg, cb],
            normal_index,
            greedy_uv: (lu, lv),
            is_greedy,
        }
    }

    /// Returns the vertex buffer layout descriptor for `wgpu`.
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 1] = [
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Uint32x4,
            },
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<BlockVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

/// Chunk section instance data passed in vertex buffer slot 1 with `step_mode: Instance`.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct SectionInstance {
    pub chunk_pos: [f32; 3],
    pub _pad: f32,
}

impl SectionInstance {
    pub fn new(chunk_x: i32, section_y: usize, chunk_z: i32) -> Self {
        Self {
            chunk_pos: [
                (chunk_x * 16) as f32,
                (section_y * 16) as f32,
                (chunk_z * 16) as f32,
            ],
            _pad: 0.0,
        }
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 1] = [
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x3,
            },
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SectionInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &ATTRIBUTES,
        }
    }
}

/// Normal directions for voxel faces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockFace {
    Up = 0,    // +Y
    Down = 1,  // -Y
    North = 2, // -Z
    South = 3, // +Z
    West = 4,  // -X
    East = 5,  // +X
}

impl BlockFace {
    pub const ALL: [Self; 6] = [
        Self::Up,
        Self::Down,
        Self::North,
        Self::South,
        Self::West,
        Self::East,
    ];

    #[inline]
    pub const fn index(self) -> u32 {
        self as u32
    }

    #[inline]
    pub const fn normal(self) -> [f32; 3] {
        match self {
            Self::Up => [0.0, 1.0, 0.0],
            Self::Down => [0.0, -1.0, 0.0],
            Self::North => [0.0, 0.0, -1.0],
            Self::South => [0.0, 0.0, 1.0],
            Self::West => [-1.0, 0.0, 0.0],
            Self::East => [1.0, 0.0, 0.0],
        }
    }

    #[inline]
    pub const fn offset(self) -> (i32, i32, i32) {
        match self {
            Self::Up => (0, 1, 0),
            Self::Down => (0, -1, 0),
            Self::North => (0, 0, -1),
            Self::South => (0, 0, 1),
            Self::West => (-1, 0, 0),
            Self::East => (1, 0, 0),
        }
    }

    #[inline]
    pub const fn opposite(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::North => Self::South,
            Self::South => Self::North,
            Self::West => Self::East,
            Self::East => Self::West,
        }
    }

    #[inline]
    pub const fn tangents(self) -> ((i32, i32, i32), (i32, i32, i32), (i32, i32, i32), (i32, i32, i32)) {
        match self {
            Self::Up => ((-1, 0, 0), (1, 0, 0), (0, 0, -1), (0, 0, 1)),
            Self::Down => ((-1, 0, 0), (1, 0, 0), (0, 0, 1), (0, 0, -1)),
            Self::North => ((1, 0, 0), (-1, 0, 0), (0, -1, 0), (0, 1, 0)),
            Self::South => ((-1, 0, 0), (1, 0, 0), (0, -1, 0), (0, 1, 0)),
            Self::West => ((0, 0, -1), (0, 0, 1), (0, -1, 0), (0, 1, 0)),
            Self::East => ((0, 0, 1), (0, 0, -1), (0, -1, 0), (0, 1, 0)),
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "down" | "bottom" | "-y" => Some(Self::Down),
            "up" | "top" | "+y" => Some(Self::Up),
            "north" | "-z" => Some(Self::North),
            "south" | "+z" => Some(Self::South),
            "west" | "-x" => Some(Self::West),
            "east" | "+x" => Some(Self::East),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_size_is_16_bytes() {
        assert_eq!(std::mem::size_of::<BlockVertex>(), 16);
    }

    #[test]
    fn test_vertex_pack_unpack_roundtrip() {
        let local_pos = [12.5, 4.0, 15.0];
        let uv = [0.25, 0.75];
        let light = [0.8, 1.0];
        let ao = 2u8;
        let color = [0.4, 0.9, 0.2];
        let normal_index = 3u32;
        let greedy_uv = (4u8, 2u8);
        let is_greedy = true;

        let packed = BlockVertex::pack(
            local_pos,
            uv,
            light,
            ao,
            color,
            normal_index,
            greedy_uv,
            is_greedy,
        );

        let unpacked = packed.unpack();

        // Check positions match within 1/32th block precision
        assert!((unpacked.local_pos[0] - local_pos[0]).abs() < (1.0 / 32.0));
        assert!((unpacked.local_pos[1] - local_pos[1]).abs() < (1.0 / 32.0));
        assert!((unpacked.local_pos[2] - local_pos[2]).abs() < (1.0 / 32.0));

        // Check UV matches within 16-bit precision
        assert!((unpacked.uv[0] - uv[0]).abs() < 1e-4);
        assert!((unpacked.uv[1] - uv[1]).abs() < 1e-4);

        // Check normal & AO match exactly
        assert_eq!(unpacked.normal_index, normal_index);
        assert_eq!(unpacked.ao, ao);

        // Check greedy params match exactly
        assert_eq!(unpacked.greedy_uv, greedy_uv);
        assert_eq!(unpacked.is_greedy, is_greedy);
    }
}
