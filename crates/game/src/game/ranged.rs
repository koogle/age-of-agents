//! Clear shots and firing positions for explicitly ordered archers.
use super::occupancy::{Claim, Occupancy};
use super::*;
use crate::navigation::offset;

const ARCHER_RANGE: i32 = 6;

impl GameWorld {
    pub(super) fn archer_positions(
        &self,
        target: CellCoordinate,
        occupancy: &Occupancy,
    ) -> Vec<CellCoordinate> {
        (-ARCHER_RANGE..=ARCHER_RANGE)
            .flat_map(|dy| {
                (-ARCHER_RANGE..=ARCHER_RANGE)
                    .filter_map(move |dx| offset(target, dx, dy, self.columns(), self.rows()))
            })
            .filter(|&cell| {
                !occupancy.is_static(cell) && self.archer_shot_clear(cell, target, occupancy)
            })
            .collect()
    }

    pub(super) fn archer_shot_clear(
        &self,
        from: CellCoordinate,
        to: CellCoordinate,
        occupancy: &Occupancy,
    ) -> bool {
        if from == to || from.center().distance(to.center()) > f64::from(ARCHER_RANGE) {
            return false;
        }
        // Supercover visits both cells at a diagonal corner, so arrows cannot
        // squeeze through touching blockers. Bodies do not obstruct arrows.
        let clear = |cell: CellCoordinate| {
            cell == from
                || cell == to
                || (self.terrain[usize::from(cell.row) * usize::from(self.columns())
                    + usize::from(cell.column)]
                .biome
                    != TerrainBiome::Mountain
                    && !matches!(occupancy.claim(cell), Some(Claim::Building(_)))
                    && !matches!(occupancy.claim(cell), Some(Claim::Resource(i)) if self.resources[i].kind != ResourceKind::Water))
        };
        let dx = i32::from(to.column) - i32::from(from.column);
        let dy = i32::from(to.row) - i32::from(from.row);
        let (nx, ny) = (dx.abs(), dy.abs());
        let (mut ix, mut iy) = (0, 0);
        let mut cell = from;
        while ix < nx || iy < ny {
            let crossing = (1 + 2 * ix) * ny - (1 + 2 * iy) * nx;
            if crossing == 0 {
                let x = offset(cell, dx.signum(), 0, self.columns(), self.rows()).unwrap();
                let y = offset(cell, 0, dy.signum(), self.columns(), self.rows()).unwrap();
                if !clear(x) || !clear(y) {
                    return false;
                }
            }
            let step_x = if crossing <= 0 {
                ix += 1;
                dx.signum()
            } else {
                0
            };
            let step_y = if crossing >= 0 {
                iy += 1;
                dy.signum()
            } else {
                0
            };
            cell = offset(cell, step_x, step_y, self.columns(), self.rows()).unwrap();
            if !clear(cell) {
                return false;
            }
        }
        true
    }
}
