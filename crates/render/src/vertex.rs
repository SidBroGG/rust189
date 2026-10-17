use bytemuck::{Pod, Zeroable};

/// Represents a single vertex in the voxel terrain mesh.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct BlockVertex {
    /// 3D position in world coordinate space.
    pub position: [f32; 3],
    /// UV coordinates on the global texture atlas.
    pub uv: [f32; 2],
    /// Light levels: `[block_light, sky_light]` normalized to `[0.0, 1.0]`.
    pub light: [f32; 2],
    /// Ambient occlusion factor computed from neighboring voxels `[0.4, 1.0]`.
    pub ao: f32,
    /// Color tint multiplier (RGB) for biome coloration (e.g., grass and foliage).
    pub color: [f32; 3],
    /// Face direction index (0..5) used for directional shading and normal calculation.
    pub normal_index: u32,
}

impl BlockVertex {
    /// Returns the vertex buffer layout descriptor for `wgpu`.
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 6] = [
            // position
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            },
            // uv
            wgpu::VertexAttribute {
                offset: 12,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x2,
            },
            // light
            wgpu::VertexAttribute {
                offset: 20,
                shader_location: 2,
                format: wgpu::VertexFormat::Float32x2,
            },
            // ao
            wgpu::VertexAttribute {
                offset: 28,
                shader_location: 3,
                format: wgpu::VertexFormat::Float32,
            },
            // color
            wgpu::VertexAttribute {
                offset: 32,
                shader_location: 4,
                format: wgpu::VertexFormat::Float32x3,
            },
            // normal_index
            wgpu::VertexAttribute {
                offset: 44,
                shader_location: 5,
                format: wgpu::VertexFormat::Uint32,
            },
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<BlockVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
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
