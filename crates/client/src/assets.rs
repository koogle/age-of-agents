//! Generated art under `assets/`, read from disk natively and fetched in the
//! browser, then decoded once at startup.
use std::collections::HashMap;

use crate::terrain::GROUND_LAYERS;

/// Every file the client needs before the first frame.
pub fn manifest() -> Vec<String> {
    let mut files: Vec<String> = GROUND_LAYERS
        .iter()
        .map(|name| format!("terrain/{name}.webp"))
        .collect();
    for sheet in ["villager", "villager_woman", "villager_elder", "resources"] {
        files.push(format!("sprites/{sheet}.png"));
    }
    files.push("sprites/towncenter.png".into());
    files.push("sprites/villager.json".into());
    files.push("sprites/resources.json".into());
    files.push("sprites/towncenter.json".into());
    files.extend(crate::hud::files());
    files
}

#[derive(Clone)]
pub struct Rgba {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl Rgba {
    pub fn decode(bytes: &[u8]) -> Self {
        let image = image::load_from_memory(bytes)
            .expect("decode image")
            .to_rgba8();
        Self {
            width: image.width(),
            height: image.height(),
            pixels: image.into_raw(),
        }
    }

    pub fn resized(&self, width: u32, height: u32) -> Self {
        if (width, height) == (self.width, self.height) {
            return self.clone();
        }
        let source = image::RgbaImage::from_raw(self.width, self.height, self.pixels.clone())
            .expect("image");
        let out = image::imageops::resize(
            &source,
            width,
            height,
            image::imageops::FilterType::Triangle,
        );
        Self {
            width,
            height,
            pixels: out.into_raw(),
        }
    }

    /// Box-filtered half-size level for a mip chain.
    pub fn half(&self) -> Self {
        let (width, height) = ((self.width / 2).max(1), (self.height / 2).max(1));
        let mut pixels = vec![0u8; (width * height * 4) as usize];
        for y in 0..height {
            for x in 0..width {
                for c in 0..4 {
                    let mut sum = 0u32;
                    for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                        let sx = (x * 2 + dx).min(self.width - 1);
                        let sy = (y * 2 + dy).min(self.height - 1);
                        sum += self.pixels[((sy * self.width + sx) * 4 + c) as usize] as u32;
                    }
                    pixels[((y * width + x) * 4 + c) as usize] = (sum / 4) as u8;
                }
            }
        }
        Self {
            width,
            height,
            pixels,
        }
    }
}

pub struct Assets {
    files: HashMap<String, Vec<u8>>,
}

impl Assets {
    pub fn bytes(&self, path: &str) -> &[u8] {
        self.files
            .get(path)
            .unwrap_or_else(|| panic!("missing asset {path}"))
    }

    pub fn image(&self, path: &str) -> Rgba {
        Rgba::decode(self.bytes(path))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn load() -> Self {
        let root = std::env::var("AGE_OF_AGENTS_ASSETS")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| {
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets")
            });
        let files = manifest()
            .into_iter()
            .map(|path| {
                let bytes = std::fs::read(root.join(&path))
                    .unwrap_or_else(|error| panic!("read {path}: {error}"));
                (path, bytes)
            })
            .collect();
        Self { files }
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn load() -> Self {
        use wasm_bindgen::JsCast;
        use wasm_bindgen_futures::JsFuture;
        let window = web_sys::window().expect("window");
        let mut files = HashMap::new();
        for path in manifest() {
            let response: web_sys::Response =
                JsFuture::from(window.fetch_with_str(&format!("/assets/{path}")))
                    .await
                    .expect("fetch")
                    .dyn_into()
                    .expect("response");
            let buffer = JsFuture::from(response.array_buffer().expect("body"))
                .await
                .expect("bytes");
            files.insert(path, js_sys::Uint8Array::new(&buffer).to_vec());
        }
        Self { files }
    }
}
