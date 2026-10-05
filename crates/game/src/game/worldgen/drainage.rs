//! One depression-corrected relief field drives both visible terrain and runoff.
use super::*;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub(super) struct Drainage {
    pub downstream: Vec<usize>,
    pub accumulation: Vec<u32>,
}

/// Priority-flood depressions to their spill height, with a tiny positive slope
/// across flats. Every land cell then has a strictly lower route to water.
/// Heights are positive f32s, so their bit patterns have the same ordering.
pub(super) fn drain(land: &[bool], elevation: &mut [f32]) -> Drainage {
    let mut seen: Vec<bool> = land.iter().map(|&l| !l).collect();
    let mut heap = BinaryHeap::new();
    for i in (0..land.len()).filter(|&i| !land[i]) {
        if neighbours4(i).any(|n| land[n]) {
            heap.push(Reverse((0_u32, i)));
        }
    }
    while let Some(Reverse((bits, cell))) = heap.pop() {
        let level = f32::from_bits(bits);
        for next in neighbours4(cell) {
            if !seen[next] {
                seen[next] = true;
                elevation[next] = elevation[next].max(level + 0.0001);
                heap.push(Reverse((elevation[next].to_bits(), next)));
            }
        }
    }
    let mut downstream = vec![usize::MAX; land.len()];
    let mut order: Vec<usize> = (0..land.len()).filter(|&i| land[i]).collect();
    for &i in &order {
        downstream[i] = neighbours4(i)
            .filter(|&n| elevation[n] < elevation[i])
            .min_by(|&a, &b| elevation[a].total_cmp(&elevation[b]).then(a.cmp(&b)))
            .expect("filled relief drains to water");
    }
    order.sort_by(|&a, &b| elevation[b].total_cmp(&elevation[a]).then(a.cmp(&b)));
    let mut accumulation: Vec<u32> = land.iter().map(|&l| u32::from(l)).collect();
    for &i in &order {
        accumulation[downstream[i]] += accumulation[i];
    }
    Drainage {
        downstream,
        accumulation,
    }
}

impl Drainage {
    pub(super) fn rivers(&self, land: &[bool], rank: &[f64]) -> (Vec<bool>, Vec<bool>) {
        let mut river = vec![false; land.len()];
        let mut ford = vec![false; land.len()];
        // Headwaters need an upstream catchment, not an arbitrary random peak.
        let mut sources: Vec<usize> = (0..land.len())
            .filter(|&i| {
                land[i] && (0.55..MOUNTAIN_RANK).contains(&rank[i]) && self.accumulation[i] >= 8
            })
            .collect();
        sources.sort_by(|&a, &b| {
            self.accumulation[b]
                .cmp(&self.accumulation[a])
                .then(a.cmp(&b))
        });
        let mut rivers = 0;
        for source in sources {
            if rivers == RIVERS {
                break;
            }
            if river
                .iter()
                .enumerate()
                .any(|(i, &r)| r && near(i, source, 8))
            {
                continue;
            }
            let mut path = Vec::new();
            let mut cell = source;
            while land[cell] && !river[cell] {
                path.push(cell);
                cell = self.downstream[cell];
            }
            if path.len() < MIN_RIVER_LENGTH {
                continue;
            }
            let mut since_ford = FORD_SPACING / 2;
            for (step, &cell) in path.iter().enumerate() {
                river[cell] = true;
                since_ford += 1;
                let straight = step >= 3
                    && step + 3 < path.len()
                    && cell + cell == path[step - 1] + path[step + 1];
                if straight && since_ford >= FORD_SPACING {
                    ford[cell] = true;
                    since_ford = 0;
                }
            }
            rivers += 1;
        }
        (river, ford)
    }
}

fn near(a: usize, b: usize, cells: usize) -> bool {
    (a % COLUMNS).abs_diff(b % COLUMNS) <= cells && (a / COLUMNS).abs_diff(b / COLUMNS) <= cells
}
