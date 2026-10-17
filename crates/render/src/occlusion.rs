use crate::vertex::BlockFace;
use world::ChunkSection;
#[cfg(test)]
use world::BlockState;

/// 6x6 visibility bitmask (36 bits) indicating whether light/sight can pass
/// between each pair of faces of a 16x16x16 chunk section.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionVisibility {
    pub mask: u64,
}

impl Default for SectionVisibility {
    fn default() -> Self {
        Self::all_visible()
    }
}

impl SectionVisibility {
    pub const ALL_VISIBLE_MASK: u64 = (1 << 36) - 1;

    #[inline]
    pub const fn all_visible() -> Self {
        Self {
            mask: Self::ALL_VISIBLE_MASK,
        }
    }

    #[inline]
    pub const fn none_visible() -> Self {
        Self { mask: 0 }
    }

    #[inline]
    pub fn set_visible(&mut self, from: BlockFace, to: BlockFace, visible: bool) {
        let bit = (from as usize) * 6 + (to as usize);
        if visible {
            self.mask |= 1 << bit;
        } else {
            self.mask &= !(1 << bit);
        }
    }

    #[inline]
    pub fn is_visible(&self, from: BlockFace, to: BlockFace) -> bool {
        let bit = (from as usize) * 6 + (to as usize);
        (self.mask & (1 << bit)) != 0
    }
}

/// Precomputed boundary voxel indices on the 6 outer faces of a 16x16x16 section.
/// (16^3 - 14^3 = 4096 - 2744 = 1352 boundary voxels).
const fn generate_boundary_indices() -> [u16; 1352] {
    let mut indices = [0u16; 1352];
    let mut count = 0;
    let mut y = 0;
    while y < 16 {
        let mut z = 0;
        while z < 16 {
            let mut x = 0;
            while x < 16 {
                if x == 0 || x == 15 || y == 0 || y == 15 || z == 0 || z == 15 {
                    indices[count] = (x | (z << 4) | (y << 8)) as u16;
                    count += 1;
                }
                x += 1;
            }
            z += 1;
        }
        y += 1;
    }
    indices
}

pub static BOUNDARY_INDICES: [u16; 1352] = generate_boundary_indices();

/// Fast flood-fill visibility graph analyzer for 16x16x16 chunk sections.
/// Based on MCP VisGraph, optimized for Rust with zero heap allocations during flood-fill.
pub struct VisGraph;

impl VisGraph {
    /// Computes the 6x6 face visibility connectivity matrix for a chunk section.
    pub fn compute_visibility(section: &ChunkSection) -> SectionVisibility {
        let mut opaque_bits = [0u64; 64];
        let opaque_count = section.fill_opaque_bits(&mut opaque_bits);

        // Fast path 1: 0 opaque blocks (empty air or fully non-opaque foliage/glass)
        if opaque_count == 0 {
            return SectionVisibility::all_visible();
        }

        // Fast path 2: completely solid chunk section (all 4096 blocks opaque)
        if opaque_count == 4096 {
            return SectionVisibility::none_visible();
        }

        let mut visited = [0u64; 64];
        let mut visibility = SectionVisibility::none_visible();

        // Fixed-size BFS queue on the stack (4096 entries = 8 KB)
        let mut queue = [0u16; 4096];

        for &start_idx in BOUNDARY_INDICES.iter() {
            let word = (start_idx as usize) >> 6;
            let bit = 1u64 << ((start_idx as usize) & 63);

            // Skip if opaque or already visited
            if (opaque_bits[word] & bit) != 0 || (visited[word] & bit) != 0 {
                continue;
            }

            // Start BFS component from start_idx
            visited[word] |= bit;
            queue[0] = start_idx;
            let mut head = 0usize;
            let mut tail = 1usize;
            let mut reached_faces_mask = 0u8;

            while head < tail {
                let curr = queue[head];
                head += 1;

                let cx = (curr & 15) as usize;
                let cz = ((curr >> 4) & 15) as usize;
                let cy = ((curr >> 8) & 15) as usize;

                // Test which boundary faces this connected component reaches
                if cy == 15 {
                    reached_faces_mask |= 1 << (BlockFace::Up as u8);
                }
                if cy == 0 {
                    reached_faces_mask |= 1 << (BlockFace::Down as u8);
                }
                if cz == 0 {
                    reached_faces_mask |= 1 << (BlockFace::North as u8);
                }
                if cz == 15 {
                    reached_faces_mask |= 1 << (BlockFace::South as u8);
                }
                if cx == 0 {
                    reached_faces_mask |= 1 << (BlockFace::West as u8);
                }
                if cx == 15 {
                    reached_faces_mask |= 1 << (BlockFace::East as u8);
                }

                // Check 6 voxel neighbors
                // Up (+Y)
                if cy < 15 {
                    let n = curr + 256;
                    let nw = (n as usize) >> 6;
                    let nb = 1u64 << ((n as usize) & 63);
                    if (opaque_bits[nw] & nb) == 0 && (visited[nw] & nb) == 0 {
                        visited[nw] |= nb;
                        queue[tail] = n;
                        tail += 1;
                    }
                }
                // Down (-Y)
                if cy > 0 {
                    let n = curr - 256;
                    let nw = (n as usize) >> 6;
                    let nb = 1u64 << ((n as usize) & 63);
                    if (opaque_bits[nw] & nb) == 0 && (visited[nw] & nb) == 0 {
                        visited[nw] |= nb;
                        queue[tail] = n;
                        tail += 1;
                    }
                }
                // North (-Z)
                if cz > 0 {
                    let n = curr - 16;
                    let nw = (n as usize) >> 6;
                    let nb = 1u64 << ((n as usize) & 63);
                    if (opaque_bits[nw] & nb) == 0 && (visited[nw] & nb) == 0 {
                        visited[nw] |= nb;
                        queue[tail] = n;
                        tail += 1;
                    }
                }
                // South (+Z)
                if cz < 15 {
                    let n = curr + 16;
                    let nw = (n as usize) >> 6;
                    let nb = 1u64 << ((n as usize) & 63);
                    if (opaque_bits[nw] & nb) == 0 && (visited[nw] & nb) == 0 {
                        visited[nw] |= nb;
                        queue[tail] = n;
                        tail += 1;
                    }
                }
                // West (-X)
                if cx > 0 {
                    let n = curr - 1;
                    let nw = (n as usize) >> 6;
                    let nb = 1u64 << ((n as usize) & 63);
                    if (opaque_bits[nw] & nb) == 0 && (visited[nw] & nb) == 0 {
                        visited[nw] |= nb;
                        queue[tail] = n;
                        tail += 1;
                    }
                }
                // East (+X)
                if cx < 15 {
                    let n = curr + 1;
                    let nw = (n as usize) >> 6;
                    let nb = 1u64 << ((n as usize) & 63);
                    if (opaque_bits[nw] & nb) == 0 && (visited[nw] & nb) == 0 {
                        visited[nw] |= nb;
                        queue[tail] = n;
                        tail += 1;
                    }
                }
            }

            // All faces reached by this air passage can see each other
            for f1 in BlockFace::ALL {
                if (reached_faces_mask & (1 << (f1 as u8))) != 0 {
                    for f2 in BlockFace::ALL {
                        if (reached_faces_mask & (1 << (f2 as u8))) != 0 {
                            visibility.set_visible(f1, f2, true);
                        }
                    }
                }
            }
        }

        visibility
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vis_graph_empty_air_all_visible() {
        let section = ChunkSection::new(true);
        let vis = VisGraph::compute_visibility(&section);
        assert_eq!(vis, SectionVisibility::all_visible());
        for f1 in BlockFace::ALL {
            for f2 in BlockFace::ALL {
                assert!(vis.is_visible(f1, f2));
            }
        }
    }

    #[test]
    fn test_vis_graph_solid_stone_none_visible() {
        let mut section = ChunkSection::new(true);
        let stone = BlockState::new(1, 0);
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    section.set_block(x, y, z, stone);
                }
            }
        }
        let vis = VisGraph::compute_visibility(&section);
        assert_eq!(vis, SectionVisibility::none_visible());
        for f1 in BlockFace::ALL {
            for f2 in BlockFace::ALL {
                assert!(!vis.is_visible(f1, f2));
            }
        }
    }

    #[test]
    fn test_vis_graph_wall_blocks_visibility() {
        let mut section = ChunkSection::new(true);
        let stone = BlockState::new(1, 0);
        // Build a solid vertical wall at x = 8 spanning full Y (0..16) and Z (0..16)
        for y in 0..16 {
            for z in 0..16 {
                section.set_block(8, y, z, stone);
            }
        }

        let vis = VisGraph::compute_visibility(&section);
        // West face (x=0) CANNOT see East face (x=15) through the wall!
        assert!(!vis.is_visible(BlockFace::West, BlockFace::East));
        assert!(!vis.is_visible(BlockFace::East, BlockFace::West));

        // But West face CAN see North, South, Up, Down on its own side of the wall!
        assert!(vis.is_visible(BlockFace::West, BlockFace::North));
        assert!(vis.is_visible(BlockFace::West, BlockFace::South));
        assert!(vis.is_visible(BlockFace::West, BlockFace::Up));
        assert!(vis.is_visible(BlockFace::West, BlockFace::Down));
    }
}
