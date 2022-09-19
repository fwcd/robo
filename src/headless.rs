use tokio::sync::mpsc;

use crate::{server::MainThreadMessage, controller::Controller};

fn run_main_msg_loop(mut rx: mpsc::Receiver<MainThreadMessage>) {
    let mut controller = Controller::new();
    while let Some(msg) = rx.blocking_recv() {
        match msg {
            MainThreadMessage::Perform(action) => controller.perform(action),
            MainThreadMessage::DidExit => break,
            _ => {},
        }
    }
}

pub fn bootstrap(rx: mpsc::Receiver<MainThreadMessage>) {
    // In headless mode we run a custom 'event loop' that handles messages from the server.
    run_main_msg_loop(rx);
}
