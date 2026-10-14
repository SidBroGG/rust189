use image::{ImageBuffer, Rgba, RgbaImage};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Helper to decode standard base64 strings without external dependencies.
pub fn decode_base64(input: &str) -> Option<Vec<u8>> {
    let mut table = [255u8; 256];
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    for (i, &b) in alphabet.iter().enumerate() {
        table[b as usize] = i as u8;
    }

    let input_bytes: Vec<u8> = input
        .bytes()
        .filter(|&b| b != b'=' && !b.is_ascii_whitespace())
        .collect();

    let mut output = Vec::with_capacity((input_bytes.len() * 3) / 4);
    let mut buffer = 0u32;
    let mut bits = 0;

    for &b in &input_bytes {
        let val = table[b as usize];
        if val == 255 {
            return None;
        }
        buffer = (buffer << 6) | (val as u32);
        bits += 6;

        if bits >= 8 {
            bits -= 8;
            output.push(((buffer >> bits) & 0xFF) as u8);
        }
    }

    Some(output)
}

fn copy_sub_rect(
    out: &mut RgbaImage,
    src: &RgbaImage,
    dx1: u32,
    dy1: u32,
    dx2: u32,
    dy2: u32,
    sx1: u32,
    sy1: u32,
    sx2: u32,
    sy2: u32,
) {
    let w = (sx2 as i32 - sx1 as i32).unsigned_abs();
    let h = (sy2 as i32 - sy1 as i32).unsigned_abs();
    let flip_x = dx1 > dx2;
    let min_dx = dx1.min(dx2);
    let min_dy = dy1.min(dy2);

    for y in 0..h {
        for x in 0..w {
            let src_x = sx1 + x;
            let src_y = sy1 + y;
            let dst_x = if flip_x { min_dx + (w - 1 - x) } else { min_dx + x };
            let dst_y = min_dy + y;
            if src_x < src.width() && src_y < src.height() && dst_x < out.width() && dst_y < out.height() {
                out.put_pixel(dst_x, dst_y, *src.get_pixel(src_x, src_y));
            }
        }
    }
}

/// Clears alpha for solid/opaque outer layers if skin author didn't use alpha channel.
pub fn fix_skin_transparency(img: &mut RgbaImage) {
    let check_and_clear = |target: &mut RgbaImage, x1: u32, y1: u32, x2: u32, y2: u32| {
        let mut has_trans = false;
        for y in y1..y2 {
            for x in x1..x2 {
                if target.get_pixel(x, y)[3] < 128 {
                    has_trans = true;
                    break;
                }
            }
            if has_trans {
                break;
            }
        }
        if !has_trans {
            for y in y1..y2 {
                for x in x1..x2 {
                    let mut p = *target.get_pixel(x, y);
                    p[3] = 0;
                    target.put_pixel(x, y, p);
                }
            }
        }
    };

    // Standard outer-layer regions from ImageBufferDownload
    check_and_clear(img, 32, 0, 64, 32);  // Headwear
    check_and_clear(img, 0, 32, 16, 48);  // Right Legwear
    check_and_clear(img, 16, 32, 40, 48); // Bodywear
    check_and_clear(img, 40, 32, 56, 48); // Right Armwear
    check_and_clear(img, 0, 48, 16, 64);  // Left Legwear
    check_and_clear(img, 48, 48, 64, 64); // Left Armwear
}

/// Converts a legacy 64x32 skin into modern 64x64 format by mirroring sub-rectangles.
pub fn normalize_skin_texture(img: &RgbaImage) -> RgbaImage {
    let mut out: RgbaImage = if img.width() == 64 && img.height() == 64 {
        img.clone()
    } else if img.width() == 64 && img.height() == 32 {
        let mut out: RgbaImage = ImageBuffer::new(64, 64);
        // Copy existing top half (64x32)
        for y in 0..32 {
            for x in 0..64 {
                out.put_pixel(x, y, *img.get_pixel(x, y));
            }
        }

        // Sub-rectangles ported directly from ImageBufferDownload
        copy_sub_rect(&mut out, img, 24, 48, 20, 52, 4, 16, 8, 20);
        copy_sub_rect(&mut out, img, 28, 48, 24, 52, 8, 16, 12, 20);
        copy_sub_rect(&mut out, img, 20, 52, 16, 64, 8, 20, 12, 32);
        copy_sub_rect(&mut out, img, 24, 52, 20, 64, 4, 20, 8, 32);
        copy_sub_rect(&mut out, img, 28, 52, 24, 64, 0, 20, 4, 32);
        copy_sub_rect(&mut out, img, 32, 52, 28, 64, 12, 20, 16, 32);
        copy_sub_rect(&mut out, img, 40, 48, 36, 52, 44, 16, 48, 20);
        copy_sub_rect(&mut out, img, 44, 48, 40, 52, 48, 16, 52, 20);
        copy_sub_rect(&mut out, img, 36, 52, 32, 64, 48, 20, 52, 32);
        copy_sub_rect(&mut out, img, 40, 52, 36, 64, 44, 20, 48, 32);
        copy_sub_rect(&mut out, img, 44, 52, 40, 64, 40, 20, 44, 32);
        copy_sub_rect(&mut out, img, 48, 52, 44, 64, 52, 20, 56, 32);

        out
    } else {
        ImageBuffer::from_pixel(64, 64, Rgba([200, 200, 200, 255]))
    };

    fix_skin_transparency(&mut out);
    out
}

/// Manages downloading, disk caching, and memory retrieval of player skin textures.
pub struct SkinManager {
    cache_dir: PathBuf,
    default_steve: RgbaImage,
    default_alex: RgbaImage,
    skins: HashMap<Uuid, RgbaImage>,
}

impl SkinManager {
    pub fn new(assets_root: &Path) -> Self {
        let cache_dir = assets_root.join("skins");
        let _ = std::fs::create_dir_all(&cache_dir);

        let steve_path = assets_root.join("minecraft/textures/entity/steve.png");
        let alex_path = assets_root.join("minecraft/textures/entity/alex.png");

        let default_steve = match image::open(&steve_path) {
            Ok(img) => normalize_skin_texture(&img.to_rgba8()),
            Err(_) => ImageBuffer::from_pixel(64, 64, Rgba([30, 80, 140, 255])),
        };

        let default_alex = match image::open(&alex_path) {
            Ok(img) => normalize_skin_texture(&img.to_rgba8()),
            Err(_) => ImageBuffer::from_pixel(64, 64, Rgba([40, 110, 50, 255])),
        };

        Self {
            cache_dir,
            default_steve,
            default_alex,
            skins: HashMap::new(),
        }
    }

    /// Determines if a UUID should use slim Alex model vs standard Steve model.
    pub fn is_slim(uuid: &Uuid) -> bool {
        let bytes = uuid.as_bytes();
        ((bytes[3] ^ bytes[7] ^ bytes[11] ^ bytes[15]) & 1) != 0
    }

    /// Gets the default skin (Steve or Alex) for a given UUID.
    pub fn default_skin_for(&self, uuid: &Uuid) -> &RgbaImage {
        if Self::is_slim(uuid) {
            &self.default_alex
        } else {
            &self.default_steve
        }
    }

    /// Gets or loads a skin from memory, disk cache, or returns the default fallback.
    pub fn get_skin(&mut self, uuid: &Uuid) -> &RgbaImage {
        if self.skins.contains_key(uuid) {
            return self.skins.get(uuid).unwrap();
        }

        // Try disk cache
        let cache_file = self.cache_dir.join(format!("{}.png", uuid));
        if cache_file.exists() {
            if let Ok(img) = image::open(&cache_file) {
                let norm = normalize_skin_texture(&img.to_rgba8());
                self.skins.insert(*uuid, norm);
                return self.skins.get(uuid).unwrap();
            }
        }

        // Fallback to default
        let default_img = self.default_skin_for(uuid).clone();
        self.skins.insert(*uuid, default_img);
        self.skins.get(uuid).unwrap()
    }

    /// Caches a raw downloaded RGBA skin to memory and disk.
    pub fn store_skin(&mut self, uuid: Uuid, raw_img: RgbaImage) {
        let norm = normalize_skin_texture(&raw_img);
        let cache_file = self.cache_dir.join(format!("{}.png", uuid));
        let _ = norm.save(&cache_file);
        self.skins.insert(uuid, norm);
    }

    /// Attempts to extract skin URL from a base64 encoded textures property.
    pub fn parse_skin_url(base64_prop: &str) -> Option<String> {
        let bytes = decode_base64(base64_prop)?;
        let json_str = String::from_utf8(bytes).ok()?;
        let v: serde_json::Value = serde_json::from_str(&json_str).ok()?;
        let url = v.get("textures")?.get("SKIN")?.get("url")?.as_str()?;
        Some(url.to_string())
    }

    /// Asynchronously downloads and caches a player's skin given their textures property.
    pub async fn fetch_skin_async(
        cache_dir: PathBuf,
        uuid: Uuid,
        skin_url: String,
    ) -> Option<RgbaImage> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .ok()?;

        let resp = client.get(&skin_url).send().await.ok()?;
        if !resp.status().is_success() {
            return None;
        }

        let bytes = resp.bytes().await.ok()?;
        let img = image::load_from_memory(&bytes).ok()?.to_rgba8();
        let norm = normalize_skin_texture(&img);

        let cache_file = cache_dir.join(format!("{}.png", uuid));
        let _ = norm.save(&cache_file);

        Some(norm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_decode() {
        let text = "Hello Antigravity!";
        // Base64 of "Hello Antigravity!" is "SGVsbG8gQW50aWdyYXZpdHkh"
        let encoded = "SGVsbG8gQW50aWdyYXZpdHkh";
        let decoded = decode_base64(encoded).expect("Decode failed");
        assert_eq!(String::from_utf8(decoded).unwrap(), text);
    }

    #[test]
    fn test_skin_normalization_64x64() {
        let img = ImageBuffer::from_pixel(64, 64, Rgba([10, 20, 30, 255]));
        let norm = normalize_skin_texture(&img);
        assert_eq!(norm.width(), 64);
        assert_eq!(norm.height(), 64);
    }

    #[test]
    fn test_skin_normalization_64x32() {
        let img = ImageBuffer::from_pixel(64, 32, Rgba([10, 20, 30, 255]));
        let norm = normalize_skin_texture(&img);
        assert_eq!(norm.width(), 64);
        assert_eq!(norm.height(), 64);
    }
}
