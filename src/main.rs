use axum::http::Method;
use axum::Server;
use clap::Parser;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::args::Args;
use crate::documents::Documents;
use crate::news::News;
use crate::routes::route;
use crate::state::FoundationState;
use crate::text_blocks::TextBlocks;

mod args;
mod documents;
mod lang;
mod news;
mod routes;
mod state;
mod text_blocks;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let args = Args::parse();

  let subscriber = FmtSubscriber::builder()
    .with_max_level(Level::INFO)
    .compact()
    .finish();

  tracing::subscriber::set_global_default(subscriber)?;

  info!(concat!(
    "Booting ",
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    "..."
  ));

  let state = FoundationState {
    news: News::load(&args.content_directory.join("news")).await?,
    text_blocks: TextBlocks::load(&args.content_directory.join("text_blocks")).await?,
    documents: Documents::load(&args.content_directory.join("documents")).await?,
  };

  let cors = CorsLayer::new()
    .allow_methods([Method::GET, Method::POST])
    .allow_origin(Any);

  let router = route(
    &args.content_directory.join("news/assets"),
    &args.content_directory.join("documents/download"),
  )
    .layer(cors)
    .with_state(state);

  let server = Server::bind(&args.listen_addr).serve(router.into_make_service());

  info!("Listening on http://{}...", server.local_addr());

  if let Err(err) = server.await {
    error!("Error while serving api: {}", err);
  }

  Ok(())
}
