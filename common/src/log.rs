use anyhow::Error;
use tracing::{span, subscriber::SetGlobalDefaultError, warn, Level};
use tracing_futures::WithSubscriber;
use uuid::Uuid;
pub fn init_logger() {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_env_filter("scanner=debug,api=debug,blockchain=debug,common=debug,models=debug,other_project=off")
        .finish();
    if let Err(info) = tracing::subscriber::set_global_default(subscriber) {
        if info
            .to_string()
            .contains("a global default trace dispatcher has already been set")
        {
            warn!("a global default trace dispatcher has already been set");
        } else {
            panic!("{}", info.to_string());
        }
    }
}

pub fn generate_trace_id() -> String {
    let trace_id = Uuid::new_v4();
    trace_id.to_string()
}
