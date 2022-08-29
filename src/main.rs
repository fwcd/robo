use druid::{AppLauncher, WindowDesc, widget::Label};

fn bootstrap_tracing() {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Could not set up tracing subscriber");
}

fn run_gui() {
    let state = ();
    let window = WindowDesc::new(Label::new("Hello world!"))
        .title("Robo")
        .window_size((640., 480.));
    AppLauncher::with_window(window)
        .launch(state)
        .expect("Could not launch GUI");
}

#[tokio::main]
async fn main() {
    bootstrap_tracing();
    run_gui();
}
