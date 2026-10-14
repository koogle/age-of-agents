//! Generated ground shadows (assets/shadows/README.md): one authored mask per
//! building stage and per character facing, looked up from the frame a sprite draws.
use std::collections::HashMap;

use serde::Deserialize;

use crate::render::Sprite;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    size: [f32; 2],
    cell_offset: [f32; 2],
    sheet_sizes: HashMap<String, [f32; 2]>,
    shadows: HashMap<String, Slot>,
    frames: Vec<Frame>,
}

#[derive(Deserialize)]
struct Slot {
    rect: [f32; 4],
    channel: usize,
}

#[derive(Deserialize)]
struct Frame {
    sheet: usize,
    rect: [f32; 4],
    mirror: bool,
    shadow: String,
}

/// Sheet index, frame rect in pixels and uv mirroring.
type Key = (usize, [i32; 4], bool);

pub struct Shadows {
    cell_offset: [f32; 2],
    sheet_sizes: HashMap<usize, [f32; 2]>,
    /// Atlas uv and colour channel for each drawn frame.
    frames: HashMap<Key, ([f32; 4], usize)>,
}

impl Shadows {
    pub fn parse(json: &[u8]) -> Self {
        let manifest: Manifest = serde_json::from_slice(json).expect("shadows.json");
        let [w, h] = manifest.size;
        let frames = manifest
            .frames
            .iter()
            .map(|f| {
                let slot = &manifest.shadows[&f.shadow];
                let [x, y, sw, sh] = slot.rect;
                (
                    (f.sheet, f.rect.map(|v| v as i32), f.mirror),
                    ([x / w, y / h, (x + sw) / w, (y + sh) / h], slot.channel),
                )
            })
            .collect();
        Self {
            cell_offset: manifest.cell_offset,
            sheet_sizes: manifest
                .sheet_sizes
                .iter()
                .map(|(k, v)| (k.parse().expect("sheet index"), *v))
                .collect(),
            frames,
        }
    }

    /// The ground shadow for a drawn sprite, if its frame has one. The mask
    /// covers twice the sprite quad with the sprite cell at `cell_offset`.
    pub fn of(&self, sheet: usize, sprite: &Sprite) -> Option<Sprite> {
        if sprite.tint[3] < 1.0 {
            return None;
        }
        let [w, h] = *self.sheet_sizes.get(&sheet)?;
        let [u0, v0, u1, v1] = sprite.uv;
        let rect =
            [u0.min(u1) * w, v0 * h, (u1 - u0).abs() * w, (v1 - v0) * h].map(|v| v.round() as i32);
        let (uv, channel) = *self.frames.get(&(sheet, rect, u0 > u1))?;
        let mut channels = [0.0; 4];
        channels[channel] = 1.0;
        let [ox, oy] = self.cell_offset;
        Some(Sprite {
            anchor: sprite.anchor,
            size: sprite.size.map(|s| s * 2.0),
            pivot: [
                ox + sprite.pivot[0] * 0.5,
                1.0 - oy - 0.5 + sprite.pivot[1] * 0.5,
            ],
            uv,
            pull: 0.0,
            tint: channels,
            footprint: [0.0; 2],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const JSON: &[u8] = include_bytes!("../../../assets/sprites/shadows.json");

    /// Every mapped frame, drawn as a sprite would draw it, finds its mask.
    #[test]
    fn every_mapped_frame_resolves_from_its_sprite_uv() {
        let shadows = Shadows::parse(JSON);
        let manifest: Manifest = serde_json::from_slice(JSON).unwrap();
        assert!(!manifest.frames.is_empty());
        for frame in &manifest.frames {
            let [w, h] = manifest.sheet_sizes[&frame.sheet.to_string()];
            let [x, y, fw, fh] = frame.rect;
            let (u0, u1) = (x / w, (x + fw) / w);
            let uv = if frame.mirror {
                [u1, y / h, u0, (y + fh) / h]
            } else {
                [u0, y / h, u1, (y + fh) / h]
            };
            let sprite = Sprite {
                anchor: [0.0; 3],
                size: [1.0, 1.0],
                pivot: [0.5, 0.0],
                uv,
                pull: 0.0,
                tint: [1.0; 4],
                footprint: [0.0; 2],
            };
            let shadow = shadows
                .of(frame.sheet, &sprite)
                .expect("frame has a shadow");
            assert_eq!(shadow.size, [2.0, 2.0]);
            assert_eq!(shadow.tint.iter().sum::<f32>(), 1.0);
        }
    }
}
