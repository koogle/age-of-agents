//! Randomized-but-deterministic play that checks the world's invariants after
//! every command and every tick.

use std::collections::BTreeMap;

use super::tests::cell;
use super::*;

/// A tiny deterministic generator so failures replay exactly.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }

    fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len() as u64) as usize]
    }
}

fn random_command(world: &GameWorld, rng: &mut Lcg) -> Command {
    let unit_ids: Vec<_> = world.units.iter().map(|unit| unit.id.clone()).collect();
    let building_ids: Vec<_> = world.buildings.iter().map(|b| b.id.clone()).collect();
    let resource_ids: Vec<_> = world.resources.iter().map(|r| r.id.clone()).collect();
    let any_cell = |rng: &mut Lcg| {
        cell(
            rng.below(u64::from(WORLD_COLUMNS) + 2) as u16,
            rng.below(u64::from(WORLD_ROWS) + 2) as u16,
        )
    };
    match rng.below(8) {
        0 | 1 => Command::Move {
            unit_id: rng.pick(&unit_ids).clone(),
            to: any_cell(rng),
        },
        2 => {
            let count = 1 + rng.below(unit_ids.len() as u64) as usize;
            Command::GroupMove {
                unit_ids: (0..count).map(|_| rng.pick(&unit_ids).clone()).collect(),
                to: any_cell(rng),
            }
        }
        3 | 4 => Command::Gather {
            unit_id: rng.pick(&unit_ids).clone(),
            resource_id: rng.pick(&resource_ids).clone(),
        },
        5 => Command::Build {
            unit_id: rng.pick(&unit_ids).clone(),
            origin: any_cell(rng),
        },
        6 => Command::Construct {
            unit_id: rng.pick(&unit_ids).clone(),
            building_id: rng.pick(&building_ids).clone(),
        },
        _ => Command::Produce {
            building_id: rng.pick(&building_ids).clone(),
            product: ProductKind::Villager,
        },
    }
}

/// Raw material in nodes, in villagers' arms, and in the stockpile.
fn material(world: &GameWorld) -> BTreeMap<ResourceKind, f64> {
    let mut totals = BTreeMap::new();
    for resource in &world.resources {
        *totals.entry(resource.kind).or_insert(0.0) += resource.amount;
    }
    for cargo in world.units.iter().filter_map(|unit| unit.cargo.as_ref()) {
        *totals.entry(cargo.kind).or_insert(0.0) += cargo.amount;
    }
    for (name, amount) in world.stockpile.entries() {
        let kind: ResourceKind = serde_json::from_value(name.into()).unwrap();
        *totals.entry(kind).or_insert(0.0) += amount;
    }
    totals
}

/// Returns how many builds and gathers were accepted, so the test can prove it
/// exercised the interesting paths rather than rejecting everything.
fn play(seed: u64, steps: usize) -> (usize, usize) {
    let mut rng = Lcg(seed);
    let mut world = GameWorld::default();
    world.stockpile.wood = 400.0;
    world.stockpile.food = 400.0;
    let mut accepted_builds = 0;
    let mut accepted_gathers = 0;
    for _ in 0..steps {
        if rng.below(3) == 0 {
            let command = random_command(&world, &mut rng);
            let before = world.clone();
            let is_build = matches!(command, Command::Build { .. });
            let is_gather = matches!(command, Command::Gather { .. });
            match world.apply_command(command.clone()) {
                Ok(()) => {
                    accepted_builds += usize::from(is_build);
                    accepted_gathers += usize::from(is_gather);
                }
                Err(error) => assert_eq!(world, before, "{command:?} -> {error} mutated"),
            }
            world.validate().unwrap();
        }
        let before = material(&world);
        world.tick(0.1 + rng.below(4) as f64 * 0.1);
        world.validate().unwrap();
        for (kind, amount) in material(&world) {
            assert!(
                (amount - before[&kind]).abs() < 1e-6,
                "seed {seed}: {kind:?} changed from {} to {amount} during a tick",
                before[&kind]
            );
        }
        assert_eq!(world.buildings.len(), 1 + accepted_builds, "seed {seed}");
    }
    assert_eq!(world.tick as usize, steps);
    (accepted_builds, accepted_gathers)
}

#[test]
fn random_play_preserves_every_invariant() {
    let (mut builds, mut gathers) = (0, 0);
    for seed in 1..=8 {
        let (b, g) = play(seed, 1_200);
        builds += b;
        gathers += g;
    }
    assert!(
        builds >= 8 && gathers >= 8,
        "builds={builds} gathers={gathers}"
    );
}

#[test]
fn validation_rejects_overlapping_or_inconsistent_worlds() {
    let valid = GameWorld::default();
    type Corruption = (&'static str, fn(&mut GameWorld));
    let corrupt: [Corruption; 9] = [
        ("units share a cell", |w| w.units[1].cell = w.units[0].cell),
        ("unit inside a building", |w| w.units[0].cell = cell(14, 9)),
        ("unit inside a resource", |w| {
            w.units[0].cell = w.resources[0].cell;
        }),
        ("step is not adjacent", |w| {
            w.units[0].step = Some(Step {
                to: cell(0, 0),
                progress: 0.5,
            });
        }),
        ("step into another unit", |w| {
            w.units[0].step = Some(Step {
                to: w.units[1].cell,
                progress: 0.5,
            });
        }),
        ("duplicate reservation", |w| {
            w.units[0].action = UnitAction::Move { to: cell(3, 3) };
            w.units[1].action = UnitAction::Move { to: cell(3, 3) };
        }),
        ("overlapping buildings", |w| {
            w.buildings.push(town_center("overlap", cell(15, 10), None));
        }),
        ("negative stockpile", |w| w.stockpile.food = -1.0),
        ("building a finished building", |w| {
            w.units[0].action = UnitAction::Build {
                building_id: "base-1".into(),
            };
        }),
    ];
    for (name, corrupt) in corrupt {
        let mut world = valid.clone();
        corrupt(&mut world);
        assert!(world.validate().is_err(), "{name} was accepted");
    }
}
