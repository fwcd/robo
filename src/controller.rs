use enigo::{Enigo, KeyboardControllable};

use crate::protocol::Action;

pub struct Controller {
    enigo: Enigo,
}

impl Controller {
    pub fn new() -> Self {
        Self { enigo: Enigo::new() }
    }

    pub fn perform(&mut self, action: Action) {
        match action {
            Action::KeySequence { text } => {
                self.enigo.key_sequence(&text);
            },
        }
    }
}

// Is this safe? At least on macOS there doesn't seem to be an automatic impl for Send.
// We run the (Druid) GUI on the main thread, which seems to run its own blocking event
// loop (outside of Tokio). Scheduling keyboard/mouse events should probably be done there
// too (disgarding for a moment that all of this is highly OS-dependent anyway).
unsafe impl Send for Controller {}
