//! Level the rendered ground beneath rectangular foundations. The simulation's
//! terrain, costs and claims are untouched; this is the same height field used
//! by terrain, sprites, the placement grid and picking.
use super::{CELL, Heights};

#[derive(Clone)]
pub(super) struct Plot {
    bounds: [f32; 4],
    height: f32,
    placed_height: f32,
}

fn root(parents: &[usize], mut index: usize) -> usize {
    while parents[index] != index {
        index = parents[index];
    }
    index
}

impl Heights {
    /// Bounds are [x, z, width, depth], in authoritative building order, with
    /// an optional preview last. Touching plots inherit the oldest foundation's
    /// level, keeping neighboring houses seated on one shared wall plane.
    pub fn set_plots(&mut self, bounds: Vec<[f32; 4]>, preview: bool) -> bool {
        if self.preview == preview
            && self
                .plots
                .iter()
                .map(|p| p.bounds)
                .eq(bounds.iter().copied())
        {
            return false;
        }
        let mut parents: Vec<_> = (0..bounds.len()).collect();
        let mut placed_parents = parents.clone();
        for (i, a) in bounds.iter().enumerate() {
            if preview && i == bounds.len() - 1 {
                placed_parents.clone_from(&parents);
            }
            for (j, b) in bounds[..i].iter().enumerate() {
                if a[0] <= b[0] + b[2]
                    && b[0] <= a[0] + a[2]
                    && a[1] <= b[1] + b[3]
                    && b[1] <= a[1] + a[3]
                {
                    let (a, b) = (root(&parents, i), root(&parents, j));
                    parents[a.max(b)] = a.min(b);
                }
            }
        }
        if !preview {
            placed_parents.clone_from(&parents);
        }
        self.preview = preview;
        self.plots = bounds
            .iter()
            .enumerate()
            .map(|(i, rect)| {
                let first = bounds[root(&parents, i)];
                let placed = bounds[root(&placed_parents, i)];
                Plot {
                    bounds: *rect,
                    placed_height: self
                        .natural_height(placed[0] + placed[2] * 0.5, placed[1] + placed[3] * 0.5),
                    height: self
                        .natural_height(first[0] + first[2] * 0.5, first[1] + first[3] * 0.5),
                }
            })
            .collect();
        true
    }

    pub(super) fn levelled_height(&self, x: f32, z: f32, natural: f32, placement: bool) -> f32 {
        let mut nearest = (CELL, natural);
        let count = self.plots.len() - usize::from(placement && self.preview);
        for plot in &self.plots[..count] {
            let height = if placement {
                plot.placed_height
            } else {
                plot.height
            };
            let [px, pz, width, depth] = plot.bounds;
            let distance = (px - x)
                .max(x - px - width)
                .max(pz - z)
                .max(z - pz - depth)
                .max(0.0);
            if distance == 0.0 {
                return height;
            }
            if distance < nearest.0 {
                nearest = (distance, height);
            }
        }
        // A short shoulder joins the foundation to the island without a crack.
        let t = nearest.0 / CELL;
        let t = t * t * (3.0 - 2.0 * t);
        nearest.1 + (natural - nearest.1) * t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connected_plots_and_preview_share_one_level_without_changing_distant_land() {
        let mut heights =
            Heights::from_cells((0..9600).map(|i| (Some(0.2 + (i % 120) as f32 * 0.008), None)));
        let original = heights.clone();
        let first = [10.0, 10.0, 1.5, 1.5];
        let across = [11.5, 10.0, 1.5, 1.5];
        let below = [10.0, 11.5, 1.5, 1.5];
        assert!(heights.set_plots(vec![first, across], false));
        let level = heights.at(10.0, 10.0);
        assert_eq!(level, original.at(10.75, 10.75));
        for x in [10.0, 10.5, 11.5, 12.5, 13.0] {
            for z in [10.0, 10.5, 11.5] {
                assert_eq!(heights.at(x, z), level);
            }
        }
        assert!(heights.set_plots(vec![first, across, below], true));
        assert_eq!(heights.at(10.0, 10.0), level);
        assert_eq!(heights.at(10.0, 13.0), level);
        assert_eq!(heights.placement_at(10.5, 12.5), original.at(10.5, 12.5));
        assert!(!heights.set_plots(vec![first, across, below], true));
        assert_eq!(heights.at(20.0, 18.0), original.at(20.0, 18.0));
        assert!(heights.set_plots(vec![first, across], false));
        assert_eq!(heights.at(10.5, 12.5), original.at(10.5, 12.5));
        heights.set_plots(vec![], false);
        assert_eq!(heights.at(10.0, 10.0), original.at(10.0, 10.0));
    }
}
