use std::sync::OnceLock;

use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

static LOGGING_INIT: OnceLock<()> = OnceLock::new();

pub fn init_logging() {
    LOGGING_INIT.get_or_init(|| {
        tracing_subscriber::registry()
            .with(
                EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| EnvFilter::new("apollo=info,apollo_desktop=info")),
            )
            .with(tracing_subscriber::fmt::layer().with_target(false))
            .init();
    });
}
