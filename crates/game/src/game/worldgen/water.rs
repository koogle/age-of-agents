//! Renewable collection points on dry riverbanks. Keep existing routes open.
use super::*;

pub(super) fn place(
    terrain: &[TerrainCell],
    ford: &[bool],
    resources: &mut Vec<ResourceNode>,
    town_center: CellCoordinate,
    start: CellCoordinate,
) {
    let (columns, rows) = BuildingKind::TownCenter.size();
    let base = Footprint {
        origin: town_center,
        columns,
        rows,
    };
    let mut blocked: Vec<bool> = terrain.iter().map(|c| !c.biome.is_walkable()).collect();
    for cell in base.cells().chain(resources.iter().map(|r| r.cell)) {
        blocked[at(usize::from(cell.column), usize::from(cell.row))] = true;
    }
    let start = at(usize::from(start.column), usize::from(start.row));
    let mut reached = distance_from(std::iter::once(start), |i| !blocked[i]);
    let mut candidates: Vec<usize> = (0..terrain.len())
        .filter(|&i| {
            !blocked[i]
                && !ford[i]
                && coordinate(i).center().distance(base.center())
                    > STARTING_BASE_RESOURCE_CLEARANCE + 2.0
                && neighbours4(i).any(|n| terrain[n].biome == TerrainBiome::River)
                && !neighbours8(i).any(|n| ford[n])
        })
        .collect();
    candidates.sort_by(|&a, &b| reached[a].cmp(&reached[b]).then(a.cmp(&b)));
    let mut sources: Vec<usize> = Vec::new();
    for i in candidates {
        if sources.len() == 8 {
            break;
        }
        if reached[i] == u32::MAX
            || sources
                .iter()
                .any(|&n| coordinate(n).center().distance(coordinate(i).center()) < 10.0)
        {
            continue;
        }
        blocked[i] = true;
        let next = distance_from(std::iter::once(start), |n| !blocked[n]);
        // A permanent node may claim its own cell, but cannot cut off any other
        // reachable land (including approaches to existing resources).
        if reached
            .iter()
            .zip(&next)
            .enumerate()
            .any(|(n, (&before, &after))| n != i && before != u32::MAX && after == u32::MAX)
            || resources.iter().any(|r| {
                !neighbours4(at(usize::from(r.cell.column), usize::from(r.cell.row)))
                    .any(|n| next[n] != u32::MAX)
            })
        {
            blocked[i] = false;
            continue;
        }
        reached = next;
        sources.push(i);
        resources.push(ResourceNode {
            id: format!("water-{}", sources.len()),
            kind: ResourceKind::Water,
            cell: coordinate(i),
            amount: 120.0,
            capacity: 120.0,
            field: None,
        });
    }
}
