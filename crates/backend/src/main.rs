#[macro_use]
extern crate tracing;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod error;
mod http;

pub use error::{Error, Result};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "addons_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let (_is_new, pool) = database::init().await?;

    http::serve(pool).await?;

    Ok(())
}
