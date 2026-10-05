//! Bounded remote snapshot delivery, independent of browser socket plumbing.
use super::{CommandResult, VecDeque, WorldSnapshot};

const MAX_PENDING_SNAPSHOTS: usize = 8;

#[derive(Default)]
pub(super) struct Inbox {
    snapshots: VecDeque<WorldSnapshot>,
    pub(super) results: Vec<CommandResult>,
    last_sequence: Option<u64>,
    reset_playback: bool,
}

impl Inbox {
    pub(super) fn reconnect(&mut self) {
        self.snapshots.clear();
        self.last_sequence = None;
        self.reset_playback = false;
    }

    pub(super) fn snapshot(&mut self, sequence: u64, world: WorldSnapshot) {
        if self.last_sequence.is_some_and(|last| sequence <= last) {
            return;
        }
        // Commands and paused broadcasts may repeat a tick; only its final
        // state matters for playback, and replies remain in `results`.
        if self
            .snapshots
            .back()
            .is_some_and(|last| last.tick == world.tick)
        {
            self.snapshots.pop_back();
        }
        // Initial connection, reconnection, or a suspended renderer must start
        // at the current world rather than play queued history.
        self.reset_playback |=
            self.last_sequence.is_none() || self.snapshots.len() >= MAX_PENDING_SNAPSHOTS;
        self.last_sequence = Some(sequence);
        if self.reset_playback {
            self.snapshots.clear();
        }
        self.snapshots.push_back(world);
    }

    pub(super) fn drain(&mut self, out: &mut VecDeque<WorldSnapshot>) -> bool {
        let reset = std::mem::take(&mut self.reset_playback);
        if reset {
            out.clear();
        }
        out.extend(self.snapshots.drain(..));
        reset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> WorldSnapshot {
        serde_json::from_str(include_str!(
            "../../../../docs/verification/presentation-fixture.json"
        ))
        .expect("browser replay fixture must match the snapshot schema")
    }

    #[test]
    fn reconnect_discards_old_delivery_and_accepts_a_restarted_sequence() {
        let world = fixture();
        let mut inbox = Inbox::default();
        let mut out = VecDeque::new();
        inbox.snapshot(100, world.clone());
        assert!(inbox.drain(&mut out));
        inbox.snapshot(101, world.clone());
        inbox.results.push(Err("pending command failed".into()));
        inbox.reconnect();
        assert_eq!(inbox.results, [Err("pending command failed".into())]);
        assert!(
            !inbox.drain(&mut VecDeque::new()),
            "wait for a fresh snapshot"
        );
        let mut paused = world;
        paused.tick += 2;
        paused.simulation_speed = 0.0;
        inbox.snapshot(0, paused.clone());
        inbox.snapshot(0, paused.clone());
        assert!(inbox.drain(&mut out));
        assert_eq!(out, VecDeque::from([paused]));
        assert!(!inbox.drain(&mut out));
    }

    #[test]
    fn repeated_paused_ticks_do_not_trigger_catch_up_or_grow_the_queue() {
        let mut world = fixture();
        world.simulation_speed = 0.0;
        let mut inbox = Inbox::default();
        let mut out = VecDeque::new();
        inbox.snapshot(1, world.clone());
        assert!(inbox.drain(&mut out));
        out.clear();
        for sequence in 2..100 {
            inbox.snapshot(sequence, world.clone());
        }
        assert!(!inbox.drain(&mut out));
        assert_eq!(out, VecDeque::from([world]));
    }

    #[test]
    fn short_batches_keep_history_but_suspended_rendering_keeps_only_latest() {
        let mut world = fixture();
        let mut inbox = Inbox::default();
        let mut out = VecDeque::new();
        inbox.snapshot(1, world.clone());
        assert!(inbox.drain(&mut out));
        out.clear();
        for tick in 2..=4 {
            world.tick = tick;
            inbox.snapshot(tick, world.clone());
        }
        assert!(!inbox.drain(&mut out));
        assert_eq!(out.iter().map(|s| s.tick).collect::<Vec<_>>(), [2, 3, 4]);
        for tick in 5..=100 {
            world.tick = tick;
            inbox.snapshot(tick, world.clone());
            assert!(inbox.snapshots.len() <= MAX_PENDING_SNAPSHOTS);
        }
        assert!(inbox.drain(&mut out));
        assert_eq!(out, VecDeque::from([world]));
    }
}
