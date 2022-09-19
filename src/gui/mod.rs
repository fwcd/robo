mod state;
mod widget;

use std::sync::{Arc, Mutex};

use druid::{AppLauncher, WindowDesc, ExtEventSink};
use local_ip_address::local_ip;
use tokio::{runtime::Runtime, sync::mpsc};

use crate::{security::Security, server::{MainThreadMessage, ServerContext}, utils::UnsafeSync, controller::Controller};

use self::{state::{AppState, SecurityInfo}, widget::app_widget};

fn app_launcher() -> AppLauncher<AppState> {
    let window = WindowDesc::new(app_widget())
        .title("Robo")
        .window_size((640., 480.));

    AppLauncher::with_window(window)
}

async fn run_main_msg_loop(mut rx: mpsc::Receiver<MainThreadMessage>, event_sink: ExtEventSink) {
    // We use `UnsafeSync` since the compiler cannot verify that we indeed always call the controller
    // from the same (main) thread due to our use of idle callbacks.
    let controller = Arc::new(Mutex::new(UnsafeSync::new(Controller::new())));

    while let Some(msg) = rx.recv().await {
        if let MainThreadMessage::DidExit = msg {
            break;
        };
        let controller = controller.clone();
        event_sink.add_idle_callback(move |state: &mut AppState| {
            match msg {
                MainThreadMessage::Perform(action) => controller.lock().unwrap().perform(action),
                MainThreadMessage::DidConnect(client) => state.connected_clients.push_back(client),
                MainThreadMessage::DidDisconnect(client) => {
                    // TODO: Identify clients exactly, we currently rely on uniqueness of names
                    // (which currently is given since we name clients after their IP + port)
                    state.connected_clients.retain(|c| c.name != client.name);
                },
                _ => {},
            }
        });
    }
}

fn resolve_host(host: &str) -> String {
    if host == "0.0.0.0" {
        local_ip().expect("No local IP found").to_string()
    } else {
        host.to_owned()
    }
}

fn run(launcher: AppLauncher<AppState>, host: &str, port: u16, security_info: SecurityInfo) {
    let host = resolve_host(host);
    let state = AppState::new(host, port, security_info);
    
    launcher
        .launch(state)
        .expect("Could not launch app")
}

fn derive_security_info(security: &dyn Security) -> SecurityInfo {
    SecurityInfo::new(security.kind().to_owned(), security.key().unwrap_or_default())
}

pub fn bootstrap(
    ctx: ServerContext,
    rx: mpsc::Receiver<MainThreadMessage>,
    runtime: Runtime
) {
    // In GUI mode druid's event loop blocks the main thread

    let launcher = app_launcher();
    let security_info = derive_security_info(&*ctx.security);
    let event_sink = launcher.get_external_handle();

    runtime.spawn(async move {
        run_main_msg_loop(rx, event_sink).await;
    });

    run(launcher, &ctx.host, ctx.port, security_info);
}
