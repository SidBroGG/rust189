use hashbrown::HashMap;
use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

/// Texture coordinate bounds within the stitched texture atlas.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextureRegion {
    pub u_min: f32,
    pub v_min: f32,
    pub u_max: f32,
    pub v_max: f32,
}

impl Default for TextureRegion {
    fn default() -> Self {
        Self {
            u_min: 0.0,
            v_min: 0.0,
            u_max: 1.0,
            v_max: 1.0,
        }
    }
}

/// Metadata for animated texture parsed from `.png.mcmeta`.
#[derive(Debug, Deserialize)]
struct AnimationMeta {
    animation: Option<AnimationData>,
}

#[derive(Debug, Deserialize)]
struct AnimationData {
    frametime: Option<u32>,
    #[allow(dead_code)]
    frames: Option<Vec<serde_json::Value>>,
}

/// Tracks runtime animation state of a texture within the atlas.
#[derive(Clone)]
pub struct AnimatedTexture {
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub frames: Vec<Vec<u8>>,
    pub frametime_ms: u64,
    pub current_frame: usize,
    pub elapsed_ms: u64,
}

impl AnimatedTexture {
    pub fn update(&mut self, dt_ms: u64) -> Option<&[u8]> {
        if self.frames.len() <= 1 {
            return None;
        }

        self.elapsed_ms += dt_ms;
        if self.elapsed_ms >= self.frametime_ms {
            self.elapsed_ms %= self.frametime_ms;
            self.current_frame = (self.current_frame + 1) % self.frames.len();
            Some(&self.frames[self.current_frame])
        } else {
            None
        }
    }
}

/// Holds the GPU texture atlas, texture regions, animated texture controllers, and baked block models.
pub struct TextureAtlas {
    pub width: u32,
    pub height: u32,
    pub regions: HashMap<String, TextureRegion>,
    pub animations: Vec<AnimatedTexture>,
    pub fallback_region: TextureRegion,
    pub image_data: Vec<u8>,
    pub models: std::sync::Arc<crate::block_model::BakedModelRegistry>,
}

impl TextureAtlas {
    /// Builds a stitched texture atlas from the provided directory of block textures.
    pub fn build(blocks_dir: &Path) -> Self {
        let mut loaded_textures: Vec<(String, DynamicImage, Option<PathBuf>)> = Vec::new();

        if blocks_dir.exists() {
            if let Ok(entries) = fs::read_dir(blocks_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("png") {
                        let stem = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("")
                            .to_string();
                        if let Ok(img) = image::open(&path) {
                            let meta_path = path.with_extension("png.mcmeta");
                            let meta_opt = if meta_path.exists() {
                                Some(meta_path)
                            } else {
                                None
                            };
                            loaded_textures.push((stem, img, meta_opt));
                        }
                    }
                }
            }
        }

        loaded_textures.sort_by(|a, b| a.0.cmp(&b.0));

        let fallback_img = Self::create_fallback_texture();
        let tile_size = 16u32;

        let total_tiles = (loaded_textures.len() + 1) as u32;
        let tiles_per_row = (total_tiles as f32).sqrt().ceil().max(16.0) as u32;
        let tiles_per_col = ((total_tiles + tiles_per_row - 1) / tiles_per_row).max(16);

        let atlas_w = (tiles_per_row * tile_size).next_power_of_two().max(512);
        let atlas_h = (tiles_per_col * tile_size).next_power_of_two().max(512);

        let mut atlas_image = RgbaImage::new(atlas_w, atlas_h);
        let mut regions = HashMap::new();
        let mut animations = Vec::new();

        // Slot 0: Fallback checkerboard
        let fallback_region = Self::place_tile(
            &mut atlas_image,
            &fallback_img,
            0,
            0,
            tile_size,
            atlas_w,
            atlas_h,
        );

        let mut current_slot = 1u32;

        for (name, img, meta_path) in loaded_textures {
            let (w, h) = img.dimensions();
            let is_animated = h > w || meta_path.is_some();

            let slot_x = (current_slot % tiles_per_row) * tile_size;
            let slot_y = (current_slot / tiles_per_row) * tile_size;

            if is_animated {
                let frame_height = w;
                let frame_count = (h / frame_height).max(1);
                let first_frame = img.crop_imm(0, 0, w, frame_height);

                let region = Self::place_tile(
                    &mut atlas_image,
                    &first_frame,
                    slot_x,
                    slot_y,
                    tile_size,
                    atlas_w,
                    atlas_h,
                );
                regions.insert(name.clone(), region);

                let mut frametime_ms = 100u64;
                if let Some(mp) = meta_path {
                    if let Ok(content) = fs::read_to_string(&mp) {
                        if let Ok(meta) = serde_json::from_str::<AnimationMeta>(&content) {
                            if let Some(anim) = meta.animation {
                                if let Some(ft) = anim.frametime {
                                    frametime_ms = (ft as u64 * 50).max(50);
                                }
                            }
                        }
                    }
                }

                let mut frames = Vec::new();
                for f in 0..frame_count {
                    let sub = img.crop_imm(0, f * frame_height, w, frame_height);
                    let resized = if w != tile_size || frame_height != tile_size {
                        sub.resize_exact(tile_size, tile_size, image::imageops::FilterType::Nearest)
                    } else {
                        sub
                    };
                    frames.push(resized.to_rgba8().into_raw());
                }

                animations.push(AnimatedTexture {
                    name,
                    x: slot_x,
                    y: slot_y,
                    width: tile_size,
                    height: tile_size,
                    frames,
                    frametime_ms,
                    current_frame: 0,
                    elapsed_ms: 0,
                });
            } else {
                let region = Self::place_tile(
                    &mut atlas_image,
                    &img,
                    slot_x,
                    slot_y,
                    tile_size,
                    atlas_w,
                    atlas_h,
                );
                regions.insert(name, region);
            }

            current_slot += 1;
        }

        info!(
            tiles = regions.len(),
            animated = animations.len(),
            width = atlas_w,
            height = atlas_h,
            "Built texture atlas"
        );

        let assets_dir = if blocks_dir.join("blockstates").exists() {
            blocks_dir.to_path_buf()
        } else if let Some(parent) = blocks_dir.parent().and_then(|p| p.parent()) {
            if parent.join("blockstates").exists() {
                parent.to_path_buf()
            } else if Path::new("assets/minecraft/blockstates").exists() {
                PathBuf::from("assets/minecraft")
            } else {
                PathBuf::from("../../assets/minecraft")
            }
        } else if Path::new("assets/minecraft/blockstates").exists() {
            PathBuf::from("assets/minecraft")
        } else {
            PathBuf::from("../../assets/minecraft")
        };

        let models = std::sync::Arc::new(crate::block_model::BakedModelRegistry::build(
            &assets_dir,
            &regions,
            fallback_region,
        ));

        Self {
            width: atlas_w,
            height: atlas_h,
            regions,
            animations,
            fallback_region,
            image_data: atlas_image.into_raw(),
            models,
        }
    }

    fn place_tile(
        atlas: &mut RgbaImage,
        tile: &DynamicImage,
        x: u32,
        y: u32,
        target_size: u32,
        atlas_w: u32,
        atlas_h: u32,
    ) -> TextureRegion {
        let resized = if tile.width() != target_size || tile.height() != target_size {
            tile.resize_exact(
                target_size,
                target_size,
                image::imageops::FilterType::Nearest,
            )
        } else {
            tile.clone()
        };

        let rgba = resized.to_rgba8();
        for py in 0..target_size {
            for px in 0..target_size {
                let pixel = rgba.get_pixel(px, py);
                atlas.put_pixel(x + px, y + py, *pixel);
            }
        }

        // Add 0.05px inset to prevent bleeding when sampled
        let u_min = (x as f32 + 0.05) / atlas_w as f32;
        let v_min = (y as f32 + 0.05) / atlas_h as f32;
        let u_max = ((x + target_size) as f32 - 0.05) / atlas_w as f32;
        let v_max = ((y + target_size) as f32 - 0.05) / atlas_h as f32;

        TextureRegion {
            u_min,
            v_min,
            u_max,
            v_max,
        }
    }

    fn create_fallback_texture() -> DynamicImage {
        let mut img = RgbaImage::new(16, 16);
        let magenta = Rgba([255, 0, 255, 255]);
        let black = Rgba([0, 0, 0, 255]);
        for y in 0..16 {
            for x in 0..16 {
                let color = if ((x / 8) + (y / 8)) % 2 == 0 {
                    magenta
                } else {
                    black
                };
                img.put_pixel(x, y, color);
            }
        }
        DynamicImage::ImageRgba8(img)
    }

    /// Returns the texture region for a named texture, falling back to the default checkerboard.
    #[inline]
    pub fn get_region(&self, name: &str) -> TextureRegion {
        self.regions.get(name).copied().unwrap_or(self.fallback_region)
    }
}
