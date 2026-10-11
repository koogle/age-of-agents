//! Apply world updates and command feedback without replaying missed history.
use crate::{App, feedback};

impl App {
    pub(super) fn command_feedback(&mut self, text: &str) {
        if self.selection.units.is_empty() {
            self.toast = Some((text.into(), crate::now_seconds() + 3.0));
        } else if let Some(snapshot) = &self.view.snapshot {
            self.feedback
                .message(snapshot, &self.selection.units, text, self.clock);
        }
    }

    pub(super) fn poll_world(&mut self, dt: f64, now: f64) {
        let reset_playback = self.source.poll(dt, &mut self.incoming);
        if reset_playback {
            self.feedback = feedback::Feedback::default();
        }
        while let Some(snapshot) = self.incoming.pop_front() {
            if !reset_playback {
                self.feedback
                    .observe(self.view.snapshot.as_ref(), &snapshot, self.clock);
                if snapshot.raid_landed
                    && self.view.snapshot.as_ref().is_some_and(|s| !s.raid_landed)
                {
                    self.toast = Some((
                        "Barbarian raiders have landed on the second island".into(),
                        now + 8.0,
                    ));
                }
            }
            // The hosted server resets after a delay, so the view may have
            // framed the old island meanwhile: look again at the new one.
            if self.view.sync(snapshot) {
                self.selection.units.clear();
                self.selection.building = None;
                self.framed = false;
            }
            if !self.framed {
                self.frame_town_center();
            }
        }
        if reset_playback {
            self.view.reset_playback();
        }
        for result in self.source.take_results() {
            if let Err(error) = result {
                let text = error.message;
                if error.units.is_empty() {
                    self.toast = Some((text, now + 3.0));
                } else if let Some(snapshot) = &self.view.snapshot {
                    self.feedback
                        .message(snapshot, &error.units, &text, self.clock);
                }
            }
        }
        if self.toast.as_ref().is_some_and(|(_, until)| now > *until) {
            self.toast = None;
        }
    }
}
