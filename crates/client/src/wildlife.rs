//! Mouse and touch use the same animal inspection / group attack interaction.
use super::*;

impl App {
    pub(super) fn interact_with_animal(&mut self, id: &str, units: Vec<String>) {
        if !units.is_empty() {
            self.send(Command::AttackAnimal {
                unit_ids: units,
                animal_id: id.into(),
            });
        } else if let Some(animal) = self
            .view
            .snapshot
            .as_ref()
            .and_then(|s| s.animals.iter().find(|a| a.id == id))
        {
            self.toast = Some((
                format!(
                    "{} · {:.0}/{:.0} health · Select units, then tap to attack",
                    animal.kind.name(),
                    animal.health,
                    animal.kind.max_health()
                ),
                now_seconds() + 4.0,
            ));
        }
    }
}
