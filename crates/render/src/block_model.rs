use crate::atlas::TextureRegion;
use crate::mesher::BlockWorldAccessor;
use crate::vertex::BlockFace;
use hashbrown::HashMap;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use world::BlockState;

fn default_true() -> bool {
    true
}

fn default_neg1() -> i32 {
    -1
}

/// JSON deserialization structure for a blockstate JSON file.
#[derive(Debug, Deserialize, Clone)]
pub struct BlockstateJson {
    pub variants: Option<HashMap<String, VariantEntry>>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum VariantEntry {
    Single(ModelVariantJson),
    Multiple(Vec<ModelVariantJson>),
}

impl VariantEntry {
    pub fn first(&self) -> &ModelVariantJson {
        match self {
            Self::Single(m) => m,
            Self::Multiple(vec) => &vec[0],
        }
    }
}

/// A specific model variant with rotations and UV lock.
#[derive(Debug, Deserialize, Clone)]
pub struct ModelVariantJson {
    pub model: String,
    #[serde(default)]
    pub x: u32,
    #[serde(default)]
    pub y: u32,
    #[serde(default)]
    pub uvlock: bool,
    #[serde(default)]
    pub weight: Option<u32>,
}

/// Raw JSON block model definition.
#[derive(Debug, Deserialize, Clone)]
pub struct RawModelJson {
    pub parent: Option<String>,
    pub ambientocclusion: Option<bool>,
    pub textures: Option<HashMap<String, String>>,
    pub elements: Option<Vec<RawElementJson>>,
}

/// A cuboid element within a raw block model.
#[derive(Debug, Deserialize, Clone)]
pub struct RawElementJson {
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub rotation: Option<RawRotationJson>,
    #[serde(default = "default_true")]
    pub shade: bool,
    pub faces: HashMap<String, RawFaceJson>,
}

/// Rotation specification for an individual model element.
#[derive(Debug, Deserialize, Clone)]
pub struct RawRotationJson {
    pub origin: [f32; 3],
    pub axis: String,
    pub angle: f32,
    #[serde(default)]
    pub rescale: bool,
}

/// A face of an element in a raw model.
#[derive(Debug, Deserialize, Clone)]
pub struct RawFaceJson {
    pub uv: Option<[f32; 4]>,
    pub texture: String,
    pub cullface: Option<String>,
    #[serde(default)]
    pub rotation: u32,
    #[serde(default = "default_neg1")]
    pub tintindex: i32,
}

/// A pre-baked, GPU-ready quad.
#[derive(Debug, Clone, Copy)]
pub struct BakedQuad {
    /// 4 vertex positions relative to block origin [0.0..1.0].
    pub positions: [[f32; 3]; 4],
    /// 4 UV coordinates in texture atlas space.
    pub uvs: [[f32; 2]; 4],
    /// Direction for face culling. If present and neighbor block is opaque, this quad is culled.
    pub cull_face: Option<BlockFace>,
    /// Normal face direction for lighting and tangent AO calculations.
    pub face: BlockFace,
    /// Tint index: -1 if untinted, >= 0 if tinted by foliage/biome color.
    pub tint_index: i32,
    /// Whether ambient occlusion and directional shading should be computed.
    pub shade: bool,
}

/// Complete baked model for a block state.
#[derive(Debug, Clone, Default)]
pub struct BakedModel {
    pub quads: Vec<BakedQuad>,
    pub ambient_occlusion: bool,
    pub is_translucent: bool,
}

impl BakedModel {
    /// Returns true if this model represents a standard 1x1x1 solid cube with 6 cullable faces.
    pub fn is_standard_cube(&self) -> bool {
        if self.quads.len() != 6 || self.is_translucent {
            return false;
        }
        for q in &self.quads {
            if q.cull_face != Some(q.face) {
                return false;
            }
            for p in &q.positions {
                if p[0] < -0.01 || p[0] > 1.01 || p[1] < -0.01 || p[1] > 1.01 || p[2] < -0.01 || p[2] > 1.01 {
                    return false;
                }
            }
        }
        true
    }

    /// Finds the baked quad corresponding to a specific face direction.
    pub fn get_face_quad(&self, face: BlockFace) -> Option<&BakedQuad> {
        self.quads.iter().find(|q| q.face == face)
    }
}

/// Manages loading of raw models, parent resolution, and quad baking.
pub struct ModelBakery {
    raw_models: HashMap<String, RawModelJson>,
}

impl ModelBakery {
    pub fn new() -> Self {
        Self {
            raw_models: HashMap::new(),
        }
    }

    /// Normalizes a model reference string (e.g., "block/stone", "minecraft:block/stone" -> "stone").
    pub fn normalize_model_name(name: &str) -> String {
        let trimmed = name.strip_prefix("minecraft:").unwrap_or(name);
        let trimmed = trimmed.strip_prefix("block/").unwrap_or(trimmed);
        trimmed.to_string()
    }

    /// Normalizes a texture reference string (e.g., "blocks/stone", "minecraft:blocks/stone" -> "stone").
    pub fn normalize_texture_name(name: &str) -> String {
        let trimmed = name.strip_prefix("minecraft:").unwrap_or(name);
        let trimmed = trimmed.strip_prefix("blocks/").unwrap_or(trimmed);
        trimmed.to_string()
    }

    /// Loads all JSON models from the models directory.
    pub fn load_models_dir(&mut self, models_dir: &Path) {
        if !models_dir.exists() {
            return;
        }

        if let Ok(entries) = fs::read_dir(models_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    let stem = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string();
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(raw) = serde_json::from_str::<RawModelJson>(&content) {
                            self.raw_models.insert(stem, raw);
                        }
                    }
                }
            }
        }
    }

    /// Resolves the list of elements for a model by following the parent chain.
    fn resolve_elements(&self, model_name: &str, depth: usize) -> Option<Vec<RawElementJson>> {
        if depth > 10 {
            return None;
        }
        let norm = Self::normalize_model_name(model_name);
        let raw = self.raw_models.get(&norm)?;

        if let Some(elements) = &raw.elements {
            Some(elements.clone())
        } else if let Some(parent) = &raw.parent {
            self.resolve_elements(parent, depth + 1)
        } else {
            None
        }
    }

    /// Resolves ambient occlusion flag by following the parent chain.
    fn resolve_ambient_occlusion(&self, model_name: &str, depth: usize) -> bool {
        if depth > 10 {
            return true;
        }
        let norm = Self::normalize_model_name(model_name);
        if let Some(raw) = self.raw_models.get(&norm) {
            if let Some(ao) = raw.ambientocclusion {
                return ao;
            }
            if let Some(parent) = &raw.parent {
                return self.resolve_ambient_occlusion(parent, depth + 1);
            }
        }
        true
    }

    /// Resolves a texture variable (e.g. "#down", "#all") through the model hierarchy.
    fn resolve_texture(
        &self,
        texture_ref: &str,
        origin_model: &str,
        current_model: &str,
        depth: usize,
    ) -> Option<String> {
        if depth > 12 {
            return None;
        }

        if !texture_ref.starts_with('#') {
            return Some(texture_ref.to_string());
        }

        let var_name = &texture_ref[1..];
        let norm_curr = Self::normalize_model_name(current_model);
        let norm_origin = Self::normalize_model_name(origin_model);

        // 1. Look in origin model textures
        if let Some(raw_origin) = self.raw_models.get(&norm_origin) {
            if let Some(textures) = &raw_origin.textures {
                if let Some(val) = textures.get(var_name) {
                    if val.starts_with('#') {
                        return self.resolve_texture(val, origin_model, current_model, depth + 1);
                    } else {
                        return Some(val.clone());
                    }
                }
            }
        }

        // 2. Look in current model textures
        if let Some(raw_curr) = self.raw_models.get(&norm_curr) {
            if let Some(textures) = &raw_curr.textures {
                if let Some(val) = textures.get(var_name) {
                    if val.starts_with('#') {
                        return self.resolve_texture(val, origin_model, current_model, depth + 1);
                    } else {
                        return Some(val.clone());
                    }
                }
            }
            // 3. Follow parent in current model
            if let Some(parent) = &raw_curr.parent {
                return self.resolve_texture(texture_ref, origin_model, parent, depth + 1);
            }
        }

        None
    }

    /// Bakes a single model variant with rotations into a BakedModel.
    pub fn bake_variant(
        &self,
        variant: &ModelVariantJson,
        regions: &HashMap<String, TextureRegion>,
        fallback: TextureRegion,
        is_translucent: bool,
    ) -> Option<BakedModel> {
        let norm_name = Self::normalize_model_name(&variant.model);
        let elements = self.resolve_elements(&norm_name, 0)?;
        let ambient_occlusion = self.resolve_ambient_occlusion(&norm_name, 0);

        let mut baked_quads = Vec::new();

        let rad_x = ((variant.x % 360) as f32).to_radians();
        let rad_y = ((variant.y % 360) as f32).to_radians();

        for element in elements {
            let from = element.from;
            let to = element.to;

            // Element coordinate bounds in 0.0..1.0
            let x0 = from[0] / 16.0;
            let y0 = from[1] / 16.0;
            let z0 = from[2] / 16.0;
            let x1 = to[0] / 16.0;
            let y1 = to[1] / 16.0;
            let z1 = to[2] / 16.0;

            for (face_name, face_data) in element.faces {
                let canonical_face = match BlockFace::from_name(&face_name) {
                    Some(f) => f,
                    None => continue,
                };

                // 4 canonical face vertices in counter-clockwise winding
                let mut v = match canonical_face {
                    BlockFace::Up => [
                        [x0, y1, z1],
                        [x1, y1, z1],
                        [x1, y1, z0],
                        [x0, y1, z0],
                    ],
                    BlockFace::Down => [
                        [x0, y0, z0],
                        [x1, y0, z0],
                        [x1, y0, z1],
                        [x0, y0, z1],
                    ],
                    BlockFace::North => [
                        [x1, y0, z0],
                        [x0, y0, z0],
                        [x0, y1, z0],
                        [x1, y1, z0],
                    ],
                    BlockFace::South => [
                        [x0, y0, z1],
                        [x1, y0, z1],
                        [x1, y1, z1],
                        [x0, y1, z1],
                    ],
                    BlockFace::West => [
                        [x0, y0, z0],
                        [x0, y0, z1],
                        [x0, y1, z1],
                        [x0, y1, z0],
                    ],
                    BlockFace::East => [
                        [x1, y0, z1],
                        [x1, y0, z0],
                        [x1, y1, z0],
                        [x1, y1, z1],
                    ],
                };

                // 1. Element rotation around origin along axis
                if let Some(rot) = &element.rotation {
                    let origin = [rot.origin[0] / 16.0, rot.origin[1] / 16.0, rot.origin[2] / 16.0];
                    let rad = rot.angle.to_radians();
                    let cos_a = rad.cos();
                    let sin_a = rad.sin();
                    let scale = if rot.rescale {
                        let abs_deg = rot.angle.abs();
                        if (abs_deg - 22.5).abs() < 1e-3 {
                            1.0 / (22.5f32.to_radians().cos())
                        } else if (abs_deg - 45.0).abs() < 1e-3 {
                            1.0 / (45.0f32.to_radians().cos())
                        } else {
                            1.0 / cos_a.abs().max(1e-4)
                        }
                    } else {
                        1.0
                    };

                    for p in &mut v {
                        let mut px = p[0] - origin[0];
                        let mut py = p[1] - origin[1];
                        let mut pz = p[2] - origin[2];

                        match rot.axis.as_str() {
                            "x" => {
                                let ny = py * cos_a - pz * sin_a;
                                let nz = py * sin_a + pz * cos_a;
                                py = ny * scale;
                                pz = nz * scale;
                            }
                            "y" => {
                                let nx = px * cos_a + pz * sin_a;
                                let nz = -px * sin_a + pz * cos_a;
                                px = nx * scale;
                                pz = nz * scale;
                            }
                            "z" => {
                                let nx = px * cos_a - py * sin_a;
                                let ny = px * sin_a + py * cos_a;
                                px = nx * scale;
                                py = ny * scale;
                            }
                            _ => {}
                        }

                        p[0] = px + origin[0];
                        p[1] = py + origin[1];
                        p[2] = pz + origin[2];
                    }
                }

                // 2. Model variant rotation (x, y) around block center [0.5, 0.5, 0.5]
                if variant.x != 0 || variant.y != 0 {
                    let cos_x = rad_x.cos();
                    let sin_x = rad_x.sin();
                    let cos_y = rad_y.cos();
                    let sin_y = rad_y.sin();

                    for p in &mut v {
                        let px = p[0] - 0.5;
                        let py = p[1] - 0.5;
                        let pz = p[2] - 0.5;

                        // X rotation: (NORTH [0, -1] -> DOWN [-1, 0])
                        let y_x = py * cos_x + pz * sin_x;
                        let z_x = -py * sin_x + pz * cos_x;

                        // Y rotation: (NORTH [0, -1] -> EAST [1, 0])
                        let x_y = px * cos_y - z_x * sin_y;
                        let z_y = px * sin_y + z_x * cos_y;

                        p[0] = x_y + 0.5;
                        p[1] = y_x + 0.5;
                        p[2] = z_y + 0.5;
                    }
                }

                // Normal direction calculation after rotations
                let mut normal = canonical_face.normal();
                if variant.x != 0 || variant.y != 0 {
                    let cos_x = rad_x.cos();
                    let sin_x = rad_x.sin();
                    let cos_y = rad_y.cos();
                    let sin_y = rad_y.sin();

                    let y_x = normal[1] * cos_x + normal[2] * sin_x;
                    let z_x = -normal[1] * sin_x + normal[2] * cos_x;
                    let x_y = normal[0] * cos_y - z_x * sin_y;
                    let z_y = normal[0] * sin_y + z_x * cos_y;
                    normal = [x_y, y_x, z_y];
                }

                // Match rotated normal to closest canonical BlockFace
                let rotated_face = BlockFace::ALL
                    .iter()
                    .copied()
                    .max_by(|a, b| {
                        let dot_a = a.normal()[0] * normal[0]
                            + a.normal()[1] * normal[1]
                            + a.normal()[2] * normal[2];
                        let dot_b = b.normal()[0] * normal[0]
                            + b.normal()[1] * normal[1]
                            + b.normal()[2] * normal[2];
                        dot_a.partial_cmp(&dot_b).unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .unwrap_or(canonical_face);

                // Cullface rotation
                let rotated_cullface = face_data.cullface.as_deref().and_then(|cf_str| {
                    let cf = BlockFace::from_name(cf_str)?;
                    if variant.x == 0 && variant.y == 0 {
                        return Some(cf);
                    }
                    let mut cf_norm = cf.normal();
                    let cos_x = rad_x.cos();
                    let sin_x = rad_x.sin();
                    let cos_y = rad_y.cos();
                    let sin_y = rad_y.sin();

                    let y_x = cf_norm[1] * cos_x + cf_norm[2] * sin_x;
                    let z_x = -cf_norm[1] * sin_x + cf_norm[2] * cos_x;
                    let x_y = cf_norm[0] * cos_y - z_x * sin_y;
                    let z_y = cf_norm[0] * sin_y + z_x * cos_y;
                    cf_norm = [x_y, y_x, z_y];

                    BlockFace::ALL.iter().copied().max_by(|a, b| {
                        let dot_a = a.normal()[0] * cf_norm[0]
                            + a.normal()[1] * cf_norm[1]
                            + a.normal()[2] * cf_norm[2];
                        let dot_b = b.normal()[0] * cf_norm[0]
                            + b.normal()[1] * cf_norm[1]
                            + b.normal()[2] * cf_norm[2];
                        dot_a.partial_cmp(&dot_b).unwrap_or(std::cmp::Ordering::Equal)
                    })
                });

                // Resolve texture region from atlas
                let resolved_tex_str = self
                    .resolve_texture(&face_data.texture, &norm_name, &norm_name, 0)
                    .unwrap_or_else(|| face_data.texture.clone());
                let norm_tex_name = Self::normalize_texture_name(&resolved_tex_str);
                let region = regions.get(&norm_tex_name).copied().unwrap_or(fallback);

                // UV calculations
                let raw_uv = face_data.uv.unwrap_or_else(|| match canonical_face {
                    BlockFace::Down | BlockFace::Up => [from[0], from[2], to[0], to[2]],
                    BlockFace::North | BlockFace::South => {
                        [from[0], 16.0 - to[1], to[0], 16.0 - from[1]]
                    }
                    BlockFace::West | BlockFace::East => {
                        [from[2], 16.0 - to[1], to[2], 16.0 - from[1]]
                    }
                });

                let uvs = if variant.uvlock {
                    // Lock UV projection to world direction
                    let mut locked = [[0.0f32; 2]; 4];
                    for i in 0..4 {
                        let pos = v[i];
                        let (u, v_coord) = match rotated_face {
                            BlockFace::Up => (pos[0] * 16.0, pos[2] * 16.0),
                            BlockFace::Down => (pos[0] * 16.0, (1.0 - pos[2]) * 16.0),
                            BlockFace::North => ((1.0 - pos[0]) * 16.0, (1.0 - pos[1]) * 16.0),
                            BlockFace::South => (pos[0] * 16.0, (1.0 - pos[1]) * 16.0),
                            BlockFace::West => (pos[2] * 16.0, (1.0 - pos[1]) * 16.0),
                            BlockFace::East => ((1.0 - pos[2]) * 16.0, (1.0 - pos[1]) * 16.0),
                        };
                        locked[i] = [u, v_coord];
                    }
                    locked
                } else {
                    let mut base = match canonical_face {
                        BlockFace::Up | BlockFace::South => [
                            [raw_uv[0], raw_uv[3]],
                            [raw_uv[2], raw_uv[3]],
                            [raw_uv[2], raw_uv[1]],
                            [raw_uv[0], raw_uv[1]],
                        ],
                        BlockFace::Down => [
                            [raw_uv[0], raw_uv[1]],
                            [raw_uv[2], raw_uv[1]],
                            [raw_uv[2], raw_uv[3]],
                            [raw_uv[0], raw_uv[3]],
                        ],
                        BlockFace::North => [
                            [raw_uv[2], raw_uv[3]],
                            [raw_uv[0], raw_uv[3]],
                            [raw_uv[0], raw_uv[1]],
                            [raw_uv[2], raw_uv[1]],
                        ],
                        BlockFace::West => [
                            [raw_uv[0], raw_uv[3]],
                            [raw_uv[2], raw_uv[3]],
                            [raw_uv[2], raw_uv[1]],
                            [raw_uv[0], raw_uv[1]],
                        ],
                        BlockFace::East => [
                            [raw_uv[0], raw_uv[3]],
                            [raw_uv[2], raw_uv[3]],
                            [raw_uv[2], raw_uv[1]],
                            [raw_uv[0], raw_uv[1]],
                        ],
                    };

                    // Face UV rotation (0, 90, 180, 270)
                    let rot_steps = (face_data.rotation / 90) % 4;
                    base.rotate_left(rot_steps as usize);
                    base
                };

                // Map 0..16 UV to atlas region coordinates
                let du = region.u_max - region.u_min;
                let dv = region.v_max - region.v_min;
                let mut atlas_uvs = [[0.0f32; 2]; 4];
                for i in 0..4 {
                    atlas_uvs[i] = [
                        region.u_min + (uvs[i][0].clamp(0.0, 16.0) / 16.0) * du,
                        region.v_min + (uvs[i][1].clamp(0.0, 16.0) / 16.0) * dv,
                    ];
                }

                baked_quads.push(BakedQuad {
                    positions: v,
                    uvs: atlas_uvs,
                    cull_face: rotated_cullface,
                    face: rotated_face,
                    tint_index: face_data.tintindex,
                    shade: element.shade,
                });
            }
        }

        Some(BakedModel {
            quads: baked_quads,
            ambient_occlusion,
            is_translucent,
        })
    }
}

/// Global registry of all baked block models for high-performance chunk meshing.
pub struct BakedModelRegistry {
    /// Direct array lookup: `(block_id << 4) | meta` -> `BakedModel`
    pub static_models: Vec<Option<BakedModel>>,
    /// Pre-baked fence connection models: `block_id` -> 16 connection variants
    pub fence_models: HashMap<u16, [BakedModel; 16]>,
    /// Pre-baked pane / iron bar connection models: `block_id` -> 16 connection variants
    pub pane_models: HashMap<u16, [BakedModel; 16]>,
    /// Pre-baked wall connection models: `block_id` -> 16 connection variants
    pub wall_models: HashMap<u16, [BakedModel; 16]>,
    /// Pre-baked stairs models: `block_id` -> 40 precomputed variants (facing * 10 + half * 5 + shape)
    pub stairs_models: HashMap<u16, [Option<BakedModel>; 40]>,
}

impl BakedModelRegistry {
    /// Builds the entire model registry from assets directory and texture atlas regions.
    pub fn build(
        assets_dir: &Path,
        regions: &HashMap<String, TextureRegion>,
        fallback: TextureRegion,
    ) -> Self {
        let mut bakery = ModelBakery::new();
        bakery.load_models_dir(&assets_dir.join("models/block"));

        let blockstates_dir = assets_dir.join("blockstates");
        let mut blockstates = HashMap::new();
        if blockstates_dir.exists() {
            if let Ok(entries) = fs::read_dir(&blockstates_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("json") {
                        let stem = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("")
                            .to_string();
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Ok(bs) = serde_json::from_str::<BlockstateJson>(&content) {
                                blockstates.insert(stem, bs);
                            }
                        }
                    }
                }
            }
        }

        let mut static_models = vec![None; 4096];
        let mut fence_models = HashMap::new();
        let mut pane_models = HashMap::new();
        let mut wall_models = HashMap::new();
        let mut stairs_models = HashMap::new();

        let colors = [
            "white", "orange", "magenta", "light_blue", "yellow", "lime", "pink", "gray",
            "silver", "cyan", "purple", "blue", "brown", "green", "red", "black",
        ];

        // 1. Bake Static Models for all 256 block IDs and 16 metadata states
        for id in 1u16..=255 {
            let is_translucent = Self::is_translucent(id);

            for meta in 0u8..16 {
                let state_key = (id << 4) | (meta as u16);

                let mapping = Self::map_block_state(id, meta, &colors);

                if let Some((bs_name, variant_query)) = mapping {
                    if let Some(bs) = blockstates.get(&bs_name) {
                        if let Some(variants) = &bs.variants {
                            let variant_entry = variants
                                .get(&variant_query)
                                .or_else(|| variants.get("normal"))
                                .or_else(|| variants.values().next());

                            if let Some(ve) = variant_entry {
                                if let Some(baked) = bakery.bake_variant(
                                    ve.first(),
                                    regions,
                                    fallback,
                                    is_translucent,
                                ) {
                                    static_models[state_key as usize] = Some(baked);
                                    continue;
                                }
                            }
                        }
                    }
                }

                // Fallback cube model if not mapped or missing in assets
                if static_models[state_key as usize].is_none() && id != 0 {
                    let region = regions
                        .get(&Self::get_default_texture_name(id, meta))
                        .copied()
                        .unwrap_or(fallback);
                    static_models[state_key as usize] =
                        Some(Self::create_unit_cube_model(region, is_translucent));
                }
            }
        }

        // 2. Pre-bake 16 connection variants for Fences
        let fence_ids = [85, 113, 188, 189, 190, 191, 192];
        for fid in fence_ids {
            let bs_name = match fid {
                113 => "nether_brick_fence",
                188 => "spruce_fence",
                189 => "birch_fence",
                190 => "jungle_fence",
                191 => "dark_oak_fence",
                192 => "acacia_fence",
                _ => "fence",
            };

            let variants_16 = Self::bake_16_connections(
                &bakery,
                blockstates.get(bs_name),
                regions,
                fallback,
                false,
            );
            fence_models.insert(fid, variants_16);
        }

        // 3. Pre-bake 16 connection variants for Panes and Iron Bars
        let mut pane_entries = vec![
            (101u16, "iron_bars".to_string(), false),
            (102u16, "glass_pane".to_string(), true),
        ];
        for (c_idx, c_name) in colors.iter().enumerate() {
            let key = (160u16 << 4) | (c_idx as u16);
            let bs_name = format!("{}_stained_glass_pane", c_name);
            pane_entries.push((key, bs_name, true));
        }
        for (key, bs_name, is_translucent) in pane_entries {
            let variants_16 = Self::bake_16_connections(
                &bakery,
                blockstates.get(&bs_name),
                regions,
                fallback,
                is_translucent,
            );
            pane_models.insert(key, variants_16);
        }

        // 4. Pre-bake 16 connection variants for Walls
        let variants_16 = Self::bake_16_connections(
            &bakery,
            blockstates.get("cobblestone_wall"),
            regions,
            fallback,
            false,
        );
        wall_models.insert(139, variants_16);

        // 5. Pre-bake Stairs models: 4 facings * 2 halves * 5 shapes = 40 variants
        let stair_ids = [53, 67, 108, 109, 114, 128, 134, 135, 136, 156, 163, 164, 180];
        for sid in stair_ids {
            let bs_name = match sid {
                53 => "oak_stairs",
                67 => "stone_stairs",
                108 => "brick_stairs",
                109 => "stone_brick_stairs",
                114 => "nether_brick_stairs",
                128 => "sandstone_stairs",
                134 => "spruce_stairs",
                135 => "birch_stairs",
                136 => "jungle_stairs",
                156 => "quartz_stairs",
                163 => "acacia_stairs",
                164 => "dark_oak_stairs",
                180 => "red_sandstone_stairs",
                _ => "oak_stairs",
            };

            let mut variants_40: [Option<BakedModel>; 40] = std::array::from_fn(|_| None);
            if let Some(bs) = blockstates.get(bs_name) {
                if let Some(variants) = &bs.variants {
                    let facings = ["east", "west", "south", "north"];
                    let halves = ["bottom", "top"];
                    let shapes = [
                        "straight",
                        "inner_left",
                        "inner_right",
                        "outer_left",
                        "outer_right",
                    ];

                    for (f_idx, f_str) in facings.iter().enumerate() {
                        for (h_idx, h_str) in halves.iter().enumerate() {
                            for (s_idx, s_str) in shapes.iter().enumerate() {
                                let key_str =
                                    format!("facing={},half={},shape={}", f_str, h_str, s_str);
                                if let Some(ve) = variants.get(&key_str) {
                                    if let Some(baked) =
                                        bakery.bake_variant(ve.first(), regions, fallback, false)
                                    {
                                        let idx = (f_idx * 10) + (h_idx * 5) + s_idx;
                                        variants_40[idx] = Some(baked);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            stairs_models.insert(sid, variants_40);
        }

        Self {
            static_models,
            fence_models,
            pane_models,
            wall_models,
            stairs_models,
        }
    }

    /// Bakes the 16 connection variants (east, north, south, west combinations).
    fn bake_16_connections(
        bakery: &ModelBakery,
        blockstate: Option<&BlockstateJson>,
        regions: &HashMap<String, TextureRegion>,
        fallback: TextureRegion,
        is_translucent: bool,
    ) -> [BakedModel; 16] {
        let mut array: [BakedModel; 16] = std::array::from_fn(|_| BakedModel::default());

        if let Some(bs) = blockstate {
            if let Some(variants) = &bs.variants {
                for bits in 0u8..16 {
                    let north = (bits & 8) != 0;
                    let south = (bits & 4) != 0;
                    let west = (bits & 2) != 0;
                    let east = (bits & 1) != 0;

                    let key_str = format!(
                        "east={},north={},south={},west={}",
                        east, north, south, west
                    );
                    let key_str_up = format!(
                        "east={},north={},south={},up=false,west={}",
                        east, north, south, west
                    );

                    let ve = variants
                        .get(&key_str)
                        .or_else(|| variants.get(&key_str_up))
                        .or_else(|| variants.values().next());

                    if let Some(ve) = ve {
                        if let Some(baked) =
                            bakery.bake_variant(ve.first(), regions, fallback, is_translucent)
                        {
                            array[bits as usize] = baked;
                        }
                    }
                }
                return array;
            }
        }

        // Fallback simple model if blockstate missing
        for bits in 0..16 {
            array[bits] = Self::create_unit_cube_model(fallback, is_translucent);
        }
        array
    }

    /// Fast O(1) query for the baked model of a block at (wx, wy, wz).
    #[inline]
    pub fn get_model<W: BlockWorldAccessor>(
        &self,
        block: BlockState,
        wx: i32,
        wy: i32,
        wz: i32,
        world: &W,
    ) -> Option<&BakedModel> {
        let id = block.id();
        if id == 0 {
            return None;
        }

        // 1. Fences
        if matches!(id, 85 | 113 | 188..=192) {
            if let Some(variants) = self.fence_models.get(&id) {
                let can_connect = |nx: i32, ny: i32, nz: i32| -> bool {
                    let b = world.block_at(nx, ny, nz);
                    b.is_opaque() || matches!(b.id(), 85 | 107 | 113 | 183..=192)
                };
                let n = can_connect(wx, wy, wz - 1) as u8;
                let s = can_connect(wx, wy, wz + 1) as u8;
                let w = can_connect(wx - 1, wy, wz) as u8;
                let e = can_connect(wx + 1, wy, wz) as u8;
                let bits = (n << 3) | (s << 2) | (w << 1) | e;
                return Some(&variants[bits as usize]);
            }
        }

        // 2. Panes & Iron Bars
        if matches!(id, 101 | 102 | 160) {
            let lookup_key = if id == 160 {
                (160u16 << 4) | (block.meta() as u16 & 15)
            } else {
                id
            };
            if let Some(variants) = self.pane_models.get(&lookup_key) {
                let can_connect = |nx: i32, ny: i32, nz: i32| -> bool {
                    let b = world.block_at(nx, ny, nz);
                    b.is_opaque() || matches!(b.id(), 20 | 95 | 101 | 102 | 160)
                };
                let n = can_connect(wx, wy, wz - 1) as u8;
                let s = can_connect(wx, wy, wz + 1) as u8;
                let w = can_connect(wx - 1, wy, wz) as u8;
                let e = can_connect(wx + 1, wy, wz) as u8;
                let bits = (n << 3) | (s << 2) | (w << 1) | e;
                return Some(&variants[bits as usize]);
            }
        }

        // 3. Walls
        if id == 139 {
            if let Some(variants) = self.wall_models.get(&139) {
                let can_connect = |nx: i32, ny: i32, nz: i32| -> bool {
                    let b = world.block_at(nx, ny, nz);
                    b.is_opaque() || b.id() == 139 || b.id() == 107 || matches!(b.id(), 183..=187)
                };
                let n = can_connect(wx, wy, wz - 1) as u8;
                let s = can_connect(wx, wy, wz + 1) as u8;
                let w = can_connect(wx - 1, wy, wz) as u8;
                let e = can_connect(wx + 1, wy, wz) as u8;
                let bits = (n << 3) | (s << 2) | (w << 1) | e;
                return Some(&variants[bits as usize]);
            }
        }

        // 4. Stairs
        if matches!(id, 53 | 67 | 108 | 109 | 114 | 128 | 134 | 135 | 136 | 156 | 163 | 164 | 180) {
            if let Some(variants) = self.stairs_models.get(&id) {
                let meta = block.meta();
                let facing_idx = (meta & 3) as usize;
                let half_idx = if (meta & 4) != 0 { 1 } else { 0 };
                let shape_idx = Self::compute_stairs_shape(wx, wy, wz, block, world) as usize;
                let idx = (facing_idx * 10) + (half_idx * 5) + shape_idx;
                if let Some(m) = &variants[idx] {
                    return Some(m);
                }
            }
        }

        // 5. All static blocks (0..4095)
        let raw = block.raw() as usize;
        if raw < self.static_models.len() {
            self.static_models[raw].as_ref()
        } else {
            None
        }
    }

    /// Computes stairs shape index (0: straight, 1: inner_left, 2: inner_right, 3: outer_left, 4: outer_right).
    #[inline]
    fn compute_stairs_shape<W: BlockWorldAccessor>(
        wx: i32,
        wy: i32,
        wz: i32,
        block: BlockState,
        world: &W,
    ) -> u8 {
        let meta = block.meta();
        let facing = meta & 3;
        let is_top = (meta & 4) != 0;

        let is_same_stairs = |nx: i32, ny: i32, nz: i32| -> Option<u8> {
            let b = world.block_at(nx, ny, nz);
            if matches!(b.id(), 53 | 67 | 108 | 109 | 114 | 128 | 134 | 135 | 136 | 156 | 163 | 164 | 180) {
                let m = b.meta();
                if ((m & 4) != 0) == is_top {
                    return Some(m & 3);
                }
            }
            None
        };

        match facing {
            0 => {
                // East (+X)
                if let Some(f) = is_same_stairs(wx + 1, wy, wz) {
                    if f == 3 && is_same_stairs(wx, wy, wz + 1) != Some(facing) {
                        return if is_top { 4 } else { 3 }; // outer_right / outer_left
                    }
                    if f == 2 && is_same_stairs(wx, wy, wz - 1) != Some(facing) {
                        return if is_top { 3 } else { 4 };
                    }
                }
                if let Some(f) = is_same_stairs(wx - 1, wy, wz) {
                    if f == 3 && is_same_stairs(wx, wy, wz - 1) != Some(facing) {
                        return if is_top { 2 } else { 1 }; // inner_right / inner_left
                    }
                    if f == 2 && is_same_stairs(wx, wy, wz + 1) != Some(facing) {
                        return if is_top { 1 } else { 2 };
                    }
                }
            }
            1 => {
                // West (-X)
                if let Some(f) = is_same_stairs(wx - 1, wy, wz) {
                    if f == 3 && is_same_stairs(wx, wy, wz + 1) != Some(facing) {
                        return if is_top { 3 } else { 4 };
                    }
                    if f == 2 && is_same_stairs(wx, wy, wz - 1) != Some(facing) {
                        return if is_top { 4 } else { 3 };
                    }
                }
                if let Some(f) = is_same_stairs(wx + 1, wy, wz) {
                    if f == 3 && is_same_stairs(wx, wy, wz - 1) != Some(facing) {
                        return if is_top { 1 } else { 2 };
                    }
                    if f == 2 && is_same_stairs(wx, wy, wz + 1) != Some(facing) {
                        return if is_top { 2 } else { 1 };
                    }
                }
            }
            2 => {
                // South (+Z)
                if let Some(f) = is_same_stairs(wx, wy, wz + 1) {
                    if f == 1 && is_same_stairs(wx + 1, wy, wz) != Some(facing) {
                        return if is_top { 3 } else { 4 };
                    }
                    if f == 0 && is_same_stairs(wx - 1, wy, wz) != Some(facing) {
                        return if is_top { 4 } else { 3 };
                    }
                }
                if let Some(f) = is_same_stairs(wx, wy, wz - 1) {
                    if f == 1 && is_same_stairs(wx - 1, wy, wz) != Some(facing) {
                        return if is_top { 2 } else { 1 };
                    }
                    if f == 0 && is_same_stairs(wx + 1, wy, wz) != Some(facing) {
                        return if is_top { 1 } else { 2 };
                    }
                }
            }
            _ => {
                // North (-Z)
                if let Some(f) = is_same_stairs(wx, wy, wz - 1) {
                    if f == 1 && is_same_stairs(wx + 1, wy, wz) != Some(facing) {
                        return if is_top { 4 } else { 3 };
                    }
                    if f == 0 && is_same_stairs(wx - 1, wy, wz) != Some(facing) {
                        return if is_top { 3 } else { 4 };
                    }
                }
                if let Some(f) = is_same_stairs(wx, wy, wz + 1) {
                    if f == 1 && is_same_stairs(wx - 1, wy, wz) != Some(facing) {
                        return if is_top { 1 } else { 2 };
                    }
                    if f == 0 && is_same_stairs(wx + 1, wy, wz) != Some(facing) {
                        return if is_top { 2 } else { 1 };
                    }
                }
            }
        }

        0 // straight
    }

    /// Determines if a block is rendered in the translucent pass.
    #[inline]
    pub fn is_translucent(id: u16) -> bool {
        matches!(id, 8 | 9 | 20 | 79 | 90 | 95 | 102 | 160 | 165)
    }

    /// Returns foliage/biome color tint for a block.
    #[inline]
    pub fn get_block_tint(block: BlockState) -> [f32; 3] {
        match block.id() {
            2 | 31 | 111 | 175 => [0.49, 0.74, 0.33], // Grass / foliage tint
            18 | 161 => [0.36, 0.62, 0.22],          // Leaves foliage tint
            8 | 9 => [0.24, 0.46, 0.88],             // Water blue tint
            _ => [1.0, 1.0, 1.0],
        }
    }

    /// Maps block ID and metadata to a (blockstate_file, variant_query) pair.
    fn map_block_state(id: u16, meta: u8, colors: &[&str; 16]) -> Option<(String, String)> {
        let mapping = match id {
            1 => match meta {
                1 => ("granite".into(), "normal".into()),
                2 => ("smooth_granite".into(), "normal".into()),
                3 => ("diorite".into(), "normal".into()),
                4 => ("smooth_diorite".into(), "normal".into()),
                5 => ("andesite".into(), "normal".into()),
                6 => ("smooth_andesite".into(), "normal".into()),
                _ => ("stone".into(), "normal".into()),
            },
            2 => ("grass".into(), "snowy=false".into()),
            3 => match meta {
                1 => ("coarse_dirt".into(), "normal".into()),
                2 => ("podzol".into(), "snowy=false".into()),
                _ => ("dirt".into(), "variant=dirt,snowy=false".into()),
            },
            4 => ("cobblestone".into(), "normal".into()),
            5 => {
                let woods = ["oak", "spruce", "birch", "jungle", "acacia", "dark_oak"];
                let w = woods[(meta as usize & 7).min(5)];
                (format!("{}_planks", w), "normal".into())
            }
            6 => {
                let woods = ["oak", "spruce", "birch", "jungle", "acacia", "dark_oak"];
                let w = woods[(meta as usize & 7).min(5)];
                (format!("{}_sapling", w), "stage=0".into())
            }
            7 => ("bedrock".into(), "normal".into()),
            12 => {
                if meta == 1 {
                    ("red_sand".into(), "normal".into())
                } else {
                    ("sand".into(), "normal".into())
                }
            }
            13 => ("gravel".into(), "normal".into()),
            14 => ("gold_ore".into(), "normal".into()),
            15 => ("iron_ore".into(), "normal".into()),
            16 => ("coal_ore".into(), "normal".into()),
            17 => {
                let woods = ["oak", "spruce", "birch", "jungle"];
                let w = woods[meta as usize & 3];
                let axis = match (meta >> 2) & 3 {
                    1 => "axis=x",
                    2 => "axis=z",
                    3 => "axis=none",
                    _ => "axis=y",
                };
                (format!("{}_log", w), axis.into())
            }
            18 => {
                let woods = ["oak", "spruce", "birch", "jungle"];
                let w = woods[meta as usize & 3];
                (format!("{}_leaves", w), "normal".into())
            }
            19 => (
                "sponge".into(),
                if meta == 1 { "wet=true" } else { "wet=false" }.into(),
            ),
            20 => ("glass".into(), "normal".into()),
            21 => ("lapis_ore".into(), "normal".into()),
            22 => ("lapis_block".into(), "normal".into()),
            24 => match meta {
                1 => ("chiseled_sandstone".into(), "normal".into()),
                2 => ("smooth_sandstone".into(), "normal".into()),
                _ => ("sandstone".into(), "type=sandstone".into()),
            },
            30 => ("web".into(), "normal".into()),
            31 => match meta {
                0 => ("dead_bush".into(), "normal".into()),
                2 => ("fern".into(), "normal".into()),
                _ => ("tall_grass".into(), "normal".into()),
            },
            32 => ("dead_bush".into(), "normal".into()),
            35 => {
                let c = colors[meta as usize & 15];
                (format!("{}_wool", c), "normal".into())
            }
            37 => ("dandelion".into(), "normal".into()),
            38 => {
                let fls = [
                    "poppy",
                    "blue_orchid",
                    "allium",
                    "houstonia",
                    "red_tulip",
                    "orange_tulip",
                    "white_tulip",
                    "pink_tulip",
                    "oxeye_daisy",
                ];
                let f = fls[(meta as usize).min(8)];
                (f.into(), "normal".into())
            }
            39 => ("brown_mushroom".into(), "normal".into()),
            40 => ("red_mushroom".into(), "normal".into()),
            41 => ("gold_block".into(), "normal".into()),
            42 => ("iron_block".into(), "normal".into()),
            43 => {
                let slabs = [
                    "stone", "sandstone", "wood_old", "cobblestone", "brick", "stone_brick",
                    "nether_brick", "quartz",
                ];
                let s = slabs[meta as usize & 7];
                (format!("{}_double_slab", s), "normal".into())
            }
            44 => {
                let slabs = [
                    "stone", "sandstone", "wood_old", "cobblestone", "brick", "stone_brick",
                    "nether_brick", "quartz",
                ];
                let s = slabs[meta as usize & 7];
                let half = if (meta & 8) != 0 {
                    "half=top"
                } else {
                    "half=bottom"
                };
                (format!("{}_slab", s), half.into())
            }
            45 => ("brick_block".into(), "normal".into()),
            46 => ("tnt".into(), "normal".into()),
            47 => ("bookshelf".into(), "normal".into()),
            48 => ("mossy_cobblestone".into(), "normal".into()),
            49 => ("obsidian".into(), "normal".into()),
            50 => {
                let facing = match meta {
                    1 => "facing=east",
                    2 => "facing=west",
                    3 => "facing=south",
                    4 => "facing=north",
                    _ => "facing=up",
                };
                ("torch".into(), facing.into())
            }
            56 => ("diamond_ore".into(), "normal".into()),
            57 => ("diamond_block".into(), "normal".into()),
            58 => ("crafting_table".into(), "normal".into()),
            65 => {
                let facing = match meta {
                    2 => "facing=north",
                    3 => "facing=south",
                    4 => "facing=west",
                    _ => "facing=east",
                };
                ("ladder".into(), facing.into())
            }
            73 => ("redstone_ore".into(), "normal".into()),
            74 => ("lit_redstone_ore".into(), "normal".into()),
            78 => {
                let layers = (meta & 7) + 1;
                ("snow_layer".into(), format!("layers={}", layers))
            }
            79 => ("ice".into(), "normal".into()),
            80 => ("snow".into(), "normal".into()),
            81 => ("cactus".into(), "normal".into()),
            82 => ("clay".into(), "normal".into()),
            83 => ("reeds".into(), "normal".into()),
            87 => ("netherrack".into(), "normal".into()),
            88 => ("soul_sand".into(), "normal".into()),
            89 => ("glowstone".into(), "normal".into()),
            95 => {
                let c = colors[meta as usize & 15];
                (format!("{}_stained_glass", c), "normal".into())
            }
            96 => {
                let half = if (meta & 8) != 0 { "top" } else { "bottom" };
                let open = if (meta & 4) != 0 { "true" } else { "false" };
                let facing = match meta & 3 {
                    0 => "north",
                    1 => "south",
                    2 => "west",
                    _ => "east",
                };
                (
                    "trapdoor".into(),
                    format!("facing={},half={},open={}", facing, half, open),
                )
            }
            98 => match meta {
                1 => ("mossy_stonebrick".into(), "normal".into()),
                2 => ("cracked_stonebrick".into(), "normal".into()),
                3 => ("chiseled_stonebrick".into(), "normal".into()),
                _ => ("stonebrick".into(), "variant=stonebrick".into()),
            },
            110 => ("mycelium".into(), "snowy=false".into()),
            111 => ("waterlily".into(), "normal".into()),
            112 => ("nether_brick".into(), "normal".into()),
            121 => ("end_stone".into(), "normal".into()),
            123 => ("redstone_lamp".into(), "normal".into()),
            124 => ("lit_redstone_lamp".into(), "normal".into()),
            125 => {
                let woods = ["oak", "spruce", "birch", "jungle", "acacia", "dark_oak"];
                let w = woods[(meta as usize & 7).min(5)];
                (format!("{}_double_slab", w), "normal".into())
            }
            126 => {
                let woods = ["oak", "spruce", "birch", "jungle", "acacia", "dark_oak"];
                let w = woods[(meta as usize & 7).min(5)];
                let half = if (meta & 8) != 0 {
                    "half=top"
                } else {
                    "half=bottom"
                };
                (format!("{}_slab", w), half.into())
            }
            129 => ("emerald_ore".into(), "normal".into()),
            133 => ("emerald_block".into(), "normal".into()),
            152 => ("redstone_block".into(), "normal".into()),
            153 => ("quartz_ore".into(), "normal".into()),
            155 => match meta {
                1 => ("chiseled_quartz_block".into(), "normal".into()),
                2..=4 => ("quartz_column".into(), "axis=y".into()),
                _ => ("quartz_block".into(), "normal".into()),
            },
            159 => {
                let c = colors[meta as usize & 15];
                (format!("{}_stained_hardened_clay", c), "normal".into())
            }
            161 => {
                let w = if (meta & 1) == 0 { "acacia" } else { "dark_oak" };
                (format!("{}_leaves", w), "normal".into())
            }
            162 => {
                let w = if (meta & 1) == 0 { "acacia" } else { "dark_oak" };
                let axis = match (meta >> 2) & 3 {
                    1 => "axis=x",
                    2 => "axis=z",
                    3 => "axis=none",
                    _ => "axis=y",
                };
                (format!("{}_log", w), axis.into())
            }
            168 => match meta {
                1 => ("prismarine_bricks".into(), "normal".into()),
                2 => ("dark_prismarine".into(), "normal".into()),
                _ => ("prismarine".into(), "type=prismarine".into()),
            },
            169 => ("sea_lantern".into(), "normal".into()),
            170 => ("hay_block".into(), "axis=y".into()),
            171 => {
                let c = colors[meta as usize & 15];
                (format!("{}_carpet", c), "normal".into())
            }
            172 => ("hardened_clay".into(), "normal".into()),
            173 => ("coal_block".into(), "normal".into()),
            174 => ("packed_ice".into(), "normal".into()),
            179 => match meta {
                1 => ("chiseled_red_sandstone".into(), "normal".into()),
                2 => ("smooth_red_sandstone".into(), "normal".into()),
                _ => ("red_sandstone".into(), "type=red_sandstone".into()),
            },
            181 => ("red_sandstone_double_slab".into(), "normal".into()),
            182 => {
                let half = if (meta & 8) != 0 {
                    "half=top"
                } else {
                    "half=bottom"
                };
                ("red_sandstone_slab".into(), half.into())
            }
            _ => return None,
        };
        Some(mapping)
    }

    /// Default texture stem name for a block if unmapped.
    fn get_default_texture_name(id: u16, meta: u8) -> String {
        match id {
            1 => match meta {
                1 => "stone_granite",
                2 => "stone_granite_smooth",
                3 => "stone_diorite",
                4 => "stone_diorite_smooth",
                5 => "stone_andesite",
                6 => "stone_andesite_smooth",
                _ => "stone",
            },
            2 => "grass_top",
            3 => match meta {
                1 => "coarse_dirt",
                2 => "dirt_podzol_top",
                _ => "dirt",
            },
            4 => "cobblestone",
            5 => "planks_oak",
            7 => "bedrock",
            8 | 9 => "water_still",
            10 | 11 => "lava_still",
            12 => "sand",
            13 => "gravel",
            14 => "gold_ore",
            15 => "iron_ore",
            16 => "coal_ore",
            17 => "log_oak",
            18 => "leaves_oak",
            20 => "glass",
            45 => "brick",
            49 => "obsidian",
            56 => "diamond_ore",
            57 => "diamond_block",
            _ => "stone",
        }
        .to_string()
    }

    /// Creates a fallback 6-face unit cube model.
    pub fn create_unit_cube_model(region: TextureRegion, is_translucent: bool) -> BakedModel {
        let mut quads = Vec::with_capacity(6);
        for face in BlockFace::ALL {
            let (v, uvs) = match face {
                BlockFace::Up => (
                    [
                        [0.0, 1.0, 1.0],
                        [1.0, 1.0, 1.0],
                        [1.0, 1.0, 0.0],
                        [0.0, 1.0, 0.0],
                    ],
                    [
                        [region.u_min, region.v_max],
                        [region.u_max, region.v_max],
                        [region.u_max, region.v_min],
                        [region.u_min, region.v_min],
                    ],
                ),
                BlockFace::Down => (
                    [
                        [0.0, 0.0, 0.0],
                        [1.0, 0.0, 0.0],
                        [1.0, 0.0, 1.0],
                        [0.0, 0.0, 1.0],
                    ],
                    [
                        [region.u_min, region.v_min],
                        [region.u_max, region.v_min],
                        [region.u_max, region.v_max],
                        [region.u_min, region.v_max],
                    ],
                ),
                BlockFace::North => (
                    [
                        [1.0, 0.0, 0.0],
                        [0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0],
                        [1.0, 1.0, 0.0],
                    ],
                    [
                        [region.u_max, region.v_max],
                        [region.u_min, region.v_max],
                        [region.u_min, region.v_min],
                        [region.u_max, region.v_min],
                    ],
                ),
                BlockFace::South => (
                    [
                        [0.0, 0.0, 1.0],
                        [1.0, 0.0, 1.0],
                        [1.0, 1.0, 1.0],
                        [0.0, 1.0, 1.0],
                    ],
                    [
                        [region.u_min, region.v_max],
                        [region.u_max, region.v_max],
                        [region.u_max, region.v_min],
                        [region.u_min, region.v_min],
                    ],
                ),
                BlockFace::West => (
                    [
                        [0.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0],
                        [0.0, 1.0, 1.0],
                        [0.0, 1.0, 0.0],
                    ],
                    [
                        [region.u_min, region.v_max],
                        [region.u_max, region.v_max],
                        [region.u_max, region.v_min],
                        [region.u_min, region.v_min],
                    ],
                ),
                BlockFace::East => (
                    [
                        [1.0, 0.0, 1.0],
                        [1.0, 0.0, 0.0],
                        [1.0, 1.0, 0.0],
                        [1.0, 1.0, 1.0],
                    ],
                    [
                        [region.u_min, region.v_max],
                        [region.u_max, region.v_max],
                        [region.u_max, region.v_min],
                        [region.u_min, region.v_min],
                    ],
                ),
            };

            quads.push(BakedQuad {
                positions: v,
                uvs,
                cull_face: Some(face),
                face,
                tint_index: -1,
                shade: true,
            });
        }

        BakedModel {
            quads,
            ambient_occlusion: true,
            is_translucent,
        }
    }
}
