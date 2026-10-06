//! Where snapshots come from. `Local` runs the authoritative simulation
//! in-process (the default for `cargo run`, and `?local` in the browser);
//! `Remote` talks to the hosted Rust server over a WebSocket.
use std::collections::VecDeque;

use aoa_game::{Command, GameWorld, WorldSnapshot};

const TICK_SECONDS: f64 = 0.1;

#[cfg(any(target_arch = "wasm32", test))]
mod inbox;

pub enum Source {
    Local {
        world: Box<GameWorld>,
        accumulator: f64,
        fresh: bool,
        results: Vec<CommandResult>,
    },
    #[cfg(target_arch = "wasm32")]
    Remote(remote::Remote),
}

/// A rejection retains the addressed units even if selection changes before its reply.
#[derive(Debug, PartialEq)]
pub struct CommandFailure {
    pub message: String,
    pub units: Vec<String>,
}

impl From<&str> for CommandFailure {
    fn from(message: &str) -> Self {
        Self {
            message: message.into(),
            units: Vec::new(),
        }
    }
}

pub type CommandResult = Result<(), CommandFailure>;

fn command_units(command: &Command) -> Vec<String> {
    match command {
        Command::BuildRoad { unit_id, .. }
        | Command::Board { unit_id, .. }
        | Command::Move { unit_id, .. }
        | Command::Gather { unit_id, .. }
        | Command::PlantField { unit_id, .. }
        | Command::Cultivate { unit_id, .. }
        | Command::Build { unit_id, .. }
        | Command::Construct { unit_id, .. }
        | Command::Deposit { unit_id, .. }
        | Command::Stop { unit_id } => vec![unit_id.clone()],
        Command::GroupMove { unit_ids, .. } | Command::AttackAnimal { unit_ids, .. } => {
            unit_ids.clone()
        }
        _ => Vec::new(),
    }
}

impl Source {
    /// The simulation in-process, on the island grown from `seed`.
    pub fn local(seed: u64) -> Self {
        Source::Local {
            world: Box::new(GameWorld::generate(seed)),
            accumulator: 0.0,
            fresh: true,
            results: Vec::new(),
        }
    }

    /// Advances the clock and drains snapshots. True starts a fresh remote playback baseline.
    pub fn poll(&mut self, dt: f64, out: &mut VecDeque<WorldSnapshot>) -> bool {
        match self {
            Source::Local {
                world,
                accumulator,
                fresh,
                ..
            } => {
                if std::mem::take(fresh) {
                    out.push_back(world.snapshot());
                }
                *accumulator += dt.min(1.0);
                while *accumulator >= TICK_SECONDS {
                    *accumulator -= TICK_SECONDS;
                    world.tick(TICK_SECONDS);
                    out.push_back(world.snapshot());
                }
                false
            }
            #[cfg(target_arch = "wasm32")]
            Source::Remote(remote) => remote.drain(out),
        }
    }

    /// Local rendering uses the exact fixed-step accumulator, not a network clock.
    pub fn presentation_tick(&self) -> Option<f64> {
        match self {
            Self::Local {
                world, accumulator, ..
            } => Some(if world.simulation_speed == 0.0 {
                world.tick as f64
            } else {
                (world.tick as f64 - 1.0 + accumulator / TICK_SECONDS).max(0.0)
            }),
            #[cfg(target_arch = "wasm32")]
            Self::Remote(_) => None,
        }
    }

    /// Outcomes of commands sent since the last call.
    pub fn take_results(&mut self) -> Vec<CommandResult> {
        match self {
            Source::Local { results, .. } => std::mem::take(results),
            #[cfg(target_arch = "wasm32")]
            Source::Remote(remote) => std::mem::take(&mut remote.inbox.borrow_mut().results),
        }
    }

    /// Starts over on the specified island, or chooses a fresh seed when blank.
    pub fn reset(&mut self, seed: Option<u64>) {
        match self {
            Source::Local {
                world,
                accumulator,
                fresh,
                results,
            } => {
                let seed = seed.unwrap_or_else(|| {
                    (crate::now_seconds() * 1000.0) as u64 ^ world.seed.rotate_left(17)
                });
                **world = GameWorld::generate(seed);
                *accumulator = 0.0;
                results.clear();
                *fresh = true;
            }
            #[cfg(target_arch = "wasm32")]
            Source::Remote(remote) => remote.reset(seed),
        }
    }

    pub fn send(&mut self, command: Command) {
        match self {
            Source::Local {
                world,
                fresh,
                results,
                ..
            } => {
                let units = command_units(&command);
                results.push(
                    world
                        .apply_command(command)
                        .map_err(|error| CommandFailure {
                            message: error.to_string(),
                            units,
                        }),
                );
                *fresh = true;
            }
            #[cfg(target_arch = "wasm32")]
            Source::Remote(remote) => remote.send(&command),
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub mod remote {
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::rc::Rc;

    use aoa_game::{Command, WorldSnapshot};
    use serde::Deserialize;
    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;

    use super::inbox::Inbox;

    #[derive(Deserialize)]
    #[serde(tag = "type", rename_all = "snake_case")]
    enum ServerMessage {
        Snapshot {
            sequence: u64,
            world: Box<WorldSnapshot>,
        },
        CommandResult {
            request_id: String,
            ok: bool,
            #[serde(default)]
            error: Option<String>,
        },
    }

    pub struct Remote {
        url: String,
        socket: web_sys::WebSocket,
        pub(super) inbox: Rc<RefCell<Inbox>>,
        request: u64,
        /// When to try again after the connection drops (a deploy restarts
        /// the server), in page seconds.
        retry_at: Option<f64>,
        on_message: Closure<dyn FnMut(web_sys::MessageEvent)>,
    }

    impl Remote {
        pub fn connect() -> Self {
            let location = web_sys::window().expect("window").location();
            let scheme = if location.protocol().unwrap_or_default() == "https:" {
                "wss"
            } else {
                "ws"
            };
            let url = format!("{scheme}://{}/ws", location.host().unwrap_or_default());
            let socket = web_sys::WebSocket::new(&url).expect("websocket");
            let inbox = Rc::new(RefCell::new(Inbox::default()));
            let sink = inbox.clone();
            let on_message = Closure::<dyn FnMut(web_sys::MessageEvent)>::new(
                move |event: web_sys::MessageEvent| {
                    let Some(text) = event.data().as_string() else {
                        return;
                    };
                    let mut inbox = sink.borrow_mut();
                    match serde_json::from_str::<ServerMessage>(&text) {
                        Ok(ServerMessage::Snapshot { sequence, world }) => {
                            inbox.snapshot(sequence, *world);
                        }
                        Ok(ServerMessage::CommandResult {
                            request_id,
                            ok,
                            error,
                        }) => {
                            inbox.command_result(&request_id, ok, error);
                        }
                        Err(error) => log::warn!("bad server message: {error}"),
                    }
                },
            );
            socket.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
            Self {
                url,
                socket,
                inbox,
                request: 0,
                retry_at: None,
                on_message,
            }
        }

        pub fn drain(&mut self, out: &mut VecDeque<WorldSnapshot>) -> bool {
            self.keep_connected();
            self.inbox.borrow_mut().drain(out)
        }

        /// Reopens the socket after it closes. A restarted server numbers its
        /// snapshots from zero again, so the sequence check starts over too.
        fn keep_connected(&mut self) {
            if self.socket.ready_state() != web_sys::WebSocket::CLOSED {
                self.retry_at = None;
                return;
            }
            let now = crate::now_seconds();
            match self.retry_at {
                None => {
                    self.retry_at = Some(now + 1.0);
                    self.inbox
                        .borrow_mut()
                        .results
                        .push(Err("Reconnecting to the island…".into()));
                }
                Some(at) if now >= at => {
                    if let Ok(socket) = web_sys::WebSocket::new(&self.url) {
                        socket.set_onmessage(Some(self.on_message.as_ref().unchecked_ref()));
                        self.socket.set_onmessage(None);
                        self.socket = socket;
                        self.inbox.borrow_mut().reconnect();
                    }
                    self.retry_at = Some(now + 3.0);
                }
                Some(_) => {}
            }
        }

        /// Asks the server for a new island; the next snapshots carry it.
        pub fn reset(&mut self, seed: Option<u64>) {
            let Some(window) = web_sys::window() else {
                return;
            };
            let init = web_sys::RequestInit::new();
            init.set_method("POST");
            let url = seed.map_or_else(|| "/reset".into(), |seed| format!("/reset?seed={seed}"));
            let request = window.fetch_with_str_and_init(&url, &init);
            let inbox = self.inbox.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let result = wasm_bindgen_futures::JsFuture::from(request).await;
                let succeeded = result
                    .ok()
                    .and_then(|value| value.dyn_into::<web_sys::Response>().ok())
                    .is_some_and(|response| response.ok());
                if !succeeded {
                    inbox
                        .borrow_mut()
                        .results
                        .push(Err("Could not reset the game. Try again.".into()));
                }
            });
        }

        pub fn send(&mut self, command: &Command) {
            self.request += 1;
            let request_id = format!("r{}", self.request);
            let units = super::command_units(command);
            let message = serde_json::json!({ "type": "command", "request_id": request_id, "command": command });
            if self.socket.send_with_str(&message.to_string()).is_err() {
                self.inbox
                    .borrow_mut()
                    .results
                    .push(Err(super::CommandFailure {
                        message: "not connected".into(),
                        units,
                    }));
            } else {
                self.inbox.borrow_mut().pending.insert(request_id, units);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejected_orders_retain_the_addressed_unit() {
        let mut source = Source::local(aoa_game::DEFAULT_SEED);
        source.send(Command::Build {
            unit_id: "villager-1".into(),
            origin: aoa_game::CellCoordinate::new(u16::MAX, u16::MAX),
            kind: aoa_game::BuildingKind::House,
        });
        let failure = source.take_results().pop().unwrap().unwrap_err();
        assert_eq!(failure.units, ["villager-1"]);
        assert!(!failure.message.is_empty());
    }

    #[test]
    fn seeded_reset_restarts_the_world_and_discards_old_tick_and_command_state() {
        let mut source = Source::local(1);
        let mut out = VecDeque::new();
        source.poll(0.19, &mut out);
        source.send(Command::SetSimulationSpeed { multiplier: 7.0 });
        source.reset(Some(u64::MAX));
        assert!(source.take_results().is_empty());
        out.clear();
        source.poll(0.02, &mut out);
        assert_eq!(
            out.len(),
            1,
            "old fractional ticks must not advance the new world"
        );
        let expected = GameWorld::generate(u64::MAX).snapshot();
        assert_eq!(out.pop_front().unwrap(), expected);
        source.reset(Some(0));
        source.poll(0.0, &mut out);
        assert_eq!(out.pop_front().unwrap(), GameWorld::generate(0).snapshot());
    }

    #[test]
    fn the_local_source_ticks_ten_times_a_second_and_reports_rejections() {
        let mut source = Source::local(aoa_game::DEFAULT_SEED);
        let mut out = VecDeque::new();
        source.poll(0.0, &mut out);
        assert_eq!(out.len(), 1, "the first poll publishes the starting world");
        out.clear();
        source.poll(0.35, &mut out);
        assert_eq!(out.len(), 3);
        source.send(Command::SetSimulationSpeed { multiplier: 7.0 });
        assert!(source.take_results()[0].is_err());
        source.send(Command::SetSimulationSpeed { multiplier: 2.0 });
        assert_eq!(source.take_results(), vec![Ok(())]);
    }
}
