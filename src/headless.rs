use tokio::{sync::mpsc, runtime::Runtime};

use crate::{server::{MainThreadMessage, ServerContext, run_server}, controller::Controller};

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

pub fn bootstrap(
    host: &str,
    port: u16,
    rx: mpsc::Receiver<MainThreadMessage>,
    ctx: ServerContext,
    runtime: Runtime
) {
    // In headless mode we run a custom 'event loop' that handles messages from the server.

    let host = host.to_owned();

    runtime.spawn(async move {
        run_server(&host, port, ctx).await;
    });

    run_main_msg_loop(rx);
}
