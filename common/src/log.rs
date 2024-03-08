use tracing::{span, Level};
use tracing_futures::WithSubscriber;
use uuid::Uuid;
pub fn init_logger() {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_env_filter("api=debug,blockchain=debug,common=debug,models=debug,other_project=off")
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

pub fn generate_trace_id() -> String {
    let trace_id = Uuid::new_v4();
    trace_id.to_string()
}
