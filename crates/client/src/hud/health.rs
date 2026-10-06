use super::*;

impl Hud {
    /// A compact, non-interactive bar in physical screen pixels.
    pub fn health_bar(&mut self, top: Vec2, fraction: f32, scale: f32) {
        let fraction = fraction.clamp(0.0, 1.0);
        let width = 24.0 * scale;
        let height = 3.0 * scale;
        let x = top.x - width * 0.5;
        let y = top.y - 6.0 * scale;
        self.shape(
            [
                x - scale,
                y - scale,
                width + 2.0 * scale,
                height + 2.0 * scale,
            ],
            [0.12, 0.15, 0.10, 0.9],
            1.0,
            scale,
        );
        let color = if fraction <= 0.25 {
            [0.88, 0.20, 0.15, 1.0]
        } else if fraction <= 0.5 {
            [0.95, 0.55, 0.12, 1.0]
        } else {
            [0.27, 0.75, 0.30, 1.0]
        };
        if fraction > 0.0 {
            self.shape([x, y, width * fraction, height], color, 1.0, scale);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_fill_shrinks_changes_color_and_scales_for_phone() {
        for scale in [1.0, 2.0] {
            let mut hud = Hud::new();
            for (fraction, expected) in [
                (1.0, [0.27, 0.75, 0.30, 1.0]),
                (0.5, [0.95, 0.55, 0.12, 1.0]),
                (0.25, [0.88, 0.20, 0.15, 1.0]),
            ] {
                hud.quads.clear();
                hud.health_bar(Vec2::new(100.0, 100.0), fraction, scale);
                assert_eq!(hud.quads.len(), 2);
                assert_eq!(hud.quads[1].rect[2], 24.0 * fraction * scale);
                assert_eq!(hud.quads[1].rect[3], 3.0 * scale);
                assert_eq!(hud.quads[1].color, expected);
                assert!(hud.regions.is_empty());
            }
            hud.quads.clear();
            hud.health_bar(Vec2::ZERO, 0.0, scale);
            assert_eq!(hud.quads.len(), 1);
        }
    }
}
