//! Apply world updates and command feedback without replaying missed history.
use crate::{App, feedback, friendly};

impl App {
    pub(super) fn poll_world(&mut self, dt: f64, now: f64) {
        let reset_playback = self.source.poll(dt, &mut self.incoming);
        if reset_playback {
            self.feedback = feedback::Feedback::default();
        }
        while let Some(snapshot) = self.incoming.pop_front() {
            if !reset_playback {
                self.feedback
                    .observe(self.view.snapshot.as_ref(), &snapshot, self.clock);
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
                self.toast = Some((friendly(&error), now + 3.0));
            }
        }
        if self.toast.as_ref().is_some_and(|(_, until)| now > *until) {
            self.toast = None;
        }
    }
}
