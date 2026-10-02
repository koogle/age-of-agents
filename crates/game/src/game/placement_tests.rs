use super::tests::{cell, stand_beside};
use super::*;

#[test]
fn houses_can_share_edges_on_both_axes_but_never_overlap() {
    let mut world = fixture::fixture();
    world.stockpile.wood = 100.0;
    let first = cell(20, 24);
    let (columns, rows) = BuildingKind::House.size();
    for origin in [
        first,
        cell(first.column + columns, first.row),
        cell(first.column, first.row + rows),
    ] {
        let footprint = Footprint {
            origin,
            columns,
            rows,
        };
        stand_beside(&mut world, 0, footprint);
        world
            .apply_command(Command::Build {
                unit_id: "villager-1".into(),
                origin,
                kind: BuildingKind::House,
            })
            .unwrap();
        for _ in 0..600 {
            world.tick(0.1);
        }
        assert!(world.buildings.last().unwrap().is_complete());
        world.validate().unwrap();
    }
    assert_eq!(world.stockpile.wood, 55.0);
    let before = world.clone();
    assert_eq!(
        world.apply_command(Command::Build {
            unit_id: "villager-1".into(),
            origin: cell(first.column + columns - 1, first.row),
            kind: BuildingKind::House,
        }),
        Err(CommandError::InvalidBuildSite)
    );
    assert_eq!(world, before);
}
