use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use actix_files::Files;
use actix_web::{
    App, HttpServer,
    dev::Service,
    http::header::{HeaderName, HeaderValue},
    main,
    web::Data,
};
use anyhow::{Context, Result};
use bb8::Pool;
use clap::{Parser, Subcommand};
use diesel_async::{AsyncPgConnection, pooled_connection::AsyncDieselConnectionManager};
use futures_util::FutureExt;
use log::debug;

mod announcements;
mod authentification;
mod character;
mod error;
mod events;
mod json_db;
pub mod models;
pub mod schema;
mod session;
mod util;
mod wallet;

pub use error::BladeApiError;

use crate::session::SessionStore;

#[derive(Parser)]
#[command(name = "blade")]
#[command(about = "Blade server", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the server
    Run {
        /// Database connection string
        #[arg(short, long)]
        connection_string: String,
        #[arg(long)]
        host: String,
        #[arg(long)]
        port: u16,
        #[arg(long)]
        static_data: PathBuf,
    },
}

type DbPool = Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;

pub struct ServerGlobal {
    db_pool: DbPool,
    session_store: SessionStore,
}

#[main]
async fn main() -> Result<()> {
    env_logger::init();
    debug!("logger initialised");

    let cli = Cli::parse();

    match &cli.command {
        Commands::Run {
            connection_string,
            host,
            port,
            static_data,
        } => {
            let db_pool = Pool::builder()
                .build(AsyncDieselConnectionManager::<AsyncPgConnection>::new(
                    connection_string,
                ))
                .await
                .unwrap();

            let server_global = Arc::new(ServerGlobal {
                db_pool,
                session_store: SessionStore::new(Duration::from_hours(24)),
            });

            let static_data_clone = static_data.clone();

            HttpServer::new(move || {
                App::new()
                    .app_data(Data::new(server_global.clone()))
                    .wrap_fn(|req, srv| {
                        let start_timestamp = SystemTime::now();
                        let is_from_blades_api =
                            req.uri().path().starts_with("/blades.bgs.services/");
                        srv.call(req).map(move |res| match res {
                            Ok(mut res) => {
                                if is_from_blades_api {
                                    res.headers_mut().insert(
                                        HeaderName::from_static("server-request-timestamp"),
                                        HeaderValue::from_str(&format!(
                                            "{}",
                                            start_timestamp
                                                .duration_since(UNIX_EPOCH)
                                                .map(|x| x.as_millis())
                                                .unwrap_or(0)
                                        ))
                                        .unwrap(),
                                    );
                                    res.headers_mut().insert(
                                        HeaderName::from_static("server-timestamp"),
                                        HeaderValue::from_str(&format!(
                                            "{}",
                                            SystemTime::now()
                                                .duration_since(UNIX_EPOCH)
                                                .map(|x| x.as_millis())
                                                .unwrap_or(0)
                                        ))
                                        .unwrap(),
                                    );
                                }
                                Ok(res)
                            }
                            Err(err) => Err(err),
                        })
                    })
                    .service(announcements::check_status)
                    .service(session::sync)
                    .service(authentification::anon_log_in)
                    .service(character::list_characters)
                    .service(character::create_characters)
                    .service(character::get_character)
                    .service(wallet::get_wallet)
                    .service(events::list_events)
                    .service(
                        Files::new(
                            "/bundles.blades.bgs.services/",
                            static_data_clone.join("bundles.blades.bgs.services"),
                        )
                        .show_files_listing(),
                    )
            })
            .bind((host.as_str(), *port))
            .context("binding server")?
            .run()
            .await
            .context("running the server")?;
        }
    }

    Ok(())
}
