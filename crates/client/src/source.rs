//! Where snapshots come from. `Local` runs the authoritative simulation
//! in-process (the default for `cargo run`, and `?local` in the browser);
//! `Remote` talks to the hosted Rust server over a WebSocket.
use std::collections::VecDeque;

use aoa_game::{Command, GameWorld, WorldSnapshot};

const TICK_SECONDS: f64 = 0.1;

pub enum Source {
    Local {
        world: Box<GameWorld>,
        accumulator: f64,
        fresh: bool,
    },
    #[cfg(target_arch = "wasm32")]
    Remote(remote::Remote),
}

/// Outcome of a command, for the toast line.
pub type CommandResult = Result<(), String>;

impl Source {
    pub fn local() -> Self {
        Source::Local {
            world: Box::default(),
            accumulator: 0.0,
            fresh: true,
        }
    }

    /// Advances the clock and returns every new snapshot.
    pub fn poll(
        &mut self,
        dt: f64,
        out: &mut VecDeque<WorldSnapshot>,
        results: &mut Vec<CommandResult>,
    ) {
        match self {
            Source::Local {
                world,
                accumulator,
                fresh,
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
            }
            #[cfg(target_arch = "wasm32")]
            Source::Remote(remote) => remote.drain(out, results),
        }
        let _ = results;
    }

    pub fn send(&mut self, command: Command, results: &mut Vec<CommandResult>) {
        match self {
            Source::Local { world, fresh, .. } => {
                results.push(
                    world
                        .apply_command(command)
                        .map_err(|error| error.to_string()),
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

    use super::CommandResult;

    #[derive(Deserialize)]
    #[serde(tag = "type", rename_all = "snake_case")]
    enum ServerMessage {
        Snapshot {
            sequence: u64,
            world: Box<WorldSnapshot>,
        },
        CommandResult {
            ok: bool,
            #[serde(default)]
            error: Option<String>,
        },
    }

    #[derive(Default)]
    struct Inbox {
        snapshots: VecDeque<WorldSnapshot>,
        results: Vec<CommandResult>,
        last_sequence: u64,
    }

    pub struct Remote {
        socket: web_sys::WebSocket,
        inbox: Rc<RefCell<Inbox>>,
        request: u64,
        _on_message: Closure<dyn FnMut(web_sys::MessageEvent)>,
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
                        Ok(ServerMessage::Snapshot { sequence, world })
                            if sequence > inbox.last_sequence =>
                        {
                            inbox.last_sequence = sequence;
                            inbox.snapshots.push_back(*world);
                        }
                        Ok(ServerMessage::Snapshot { .. }) => {}
                        Ok(ServerMessage::CommandResult { ok, error }) => {
                            inbox.results.push(if ok {
                                Ok(())
                            } else {
                                Err(error.unwrap_or_else(|| "rejected".into()))
                            });
                        }
                        Err(error) => log::warn!("bad server message: {error}"),
                    }
                },
            );
            socket.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
            Self {
                socket,
                inbox,
                request: 0,
                _on_message: on_message,
            }
        }

        pub fn drain(
            &mut self,
            out: &mut VecDeque<WorldSnapshot>,
            results: &mut Vec<CommandResult>,
        ) {
            let mut inbox = self.inbox.borrow_mut();
            out.extend(inbox.snapshots.drain(..));
            results.append(&mut inbox.results);
        }

        pub fn send(&mut self, command: &Command) {
            self.request += 1;
            let message = serde_json::json!({ "type": "command", "request_id": format!("r{}", self.request), "command": command });
            if self.socket.send_with_str(&message.to_string()).is_err() {
                self.inbox
                    .borrow_mut()
                    .results
                    .push(Err("not connected".into()));
            }
        }
    }
}
