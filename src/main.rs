use tracing::info;
use tracing_subscriber;

fn main() {
    // initial global logger.
    tracing_subscriber::fmt::init();

    info!("mgds role chooser");
}
