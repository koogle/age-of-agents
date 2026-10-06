//! Shared building upgrade wording; layout owns its header placement.
use super::*;
use aoa_game::{Building, Stockpile};

fn payoff(kind: BuildingKind) -> String {
    let optional = kind.technologies().iter().find(|t| t.requires_masonry());
    optional.map_or_else(
        || "Brick finish".into(),
        |&tech| format!("Unlocks {} research", tech_info(tech).1),
    )
}

pub(super) fn command(
    building: &Building,
    snapshot: &WorldSnapshot,
    stock: &Stockpile,
    queue_full: bool,
) -> Command {
    let kind = building.kind;
    let upgraded = building.masonry;
    let pending = building.upgrade_pending();
    Command {
        icon: "command_upgrade",
        label: if upgraded {
            "Masonry complete"
        } else {
            "Upgrade to masonry"
        }
        .into(),
        detail: if upgraded {
            format!(
                "Brick upgrade complete · {}",
                payoff(kind).replace("Unlocks", "Offers")
            )
        } else if pending {
            "Upgrade queued or in progress".into()
        } else if !snapshot.masonry_upgrades_available {
            "Discover clay to unlock masonry upgrades".into()
        } else if queue_full {
            "Queue is full".into()
        } else if !stock.affords(kind.upgrade_cost()) {
            format!(
                "Bring {} to this island · {}",
                cost_text(kind.upgrade_cost()),
                payoff(kind)
            )
        } else {
            format!(
                "{} from this island · {} seconds · {}",
                cost_text(kind.upgrade_cost()),
                aoa_game::BUILDING_UPGRADE_SECONDS,
                payoff(kind)
            )
        },
        enabled: !upgraded
            && !pending
            && !queue_full
            && snapshot.masonry_upgrades_available
            && stock.affords(kind.upgrade_cost()),
        action: Action::UpgradeBuilding,
    }
}
