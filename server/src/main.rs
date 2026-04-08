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
use clap::{Parser, Subcommand};
mod migrate_db;
use deadpool_postgres::{
    Manager, ManagerConfig, Pool,
    tokio_postgres::{self, NoTls},
};
use futures_util::FutureExt;
use log::debug;

mod announcements;
mod authentification;
mod character;
mod character_data_storage;
mod error;
mod session;

pub use error::BladeApiError;
use tokio_postgres::Config;

use crate::{
    character_data_storage::{CharacterFullDataLocalConfig, CharacterFullDataLocalStorage},
    session::SessionStore,
};

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
    /// Run database migrations
    Migrate {
        /// Database connection string
        #[arg(short, long)]
        connection_string: String,
    },
}

pub struct ServerGlobal {
    db_pool: Arc<Pool>,
    session_store: SessionStore,
    character_storage: CharacterFullDataLocalStorage,
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
            let _main_lock_client = migrate_db::migrate_db_and_check_lock(connection_string)
                .await
                .context("connecting to db/running migration")?;

            let db_mgr = Manager::from_config(
                connection_string
                    .parse::<Config>()
                    .expect("invalid connection string"),
                NoTls,
                ManagerConfig {
                    recycling_method: deadpool_postgres::RecyclingMethod::Fast,
                },
            );
            let db_pool = Arc::new(
                Pool::builder(db_mgr)
                    .max_size(16)
                    .build()
                    .context("building db connection pool")?,
            );

            let character_storage =
                CharacterFullDataLocalStorage::new(CharacterFullDataLocalConfig {
                    write_delay: 1, // 1 second for debug purpose, 30s should be a safer bet.
                    db_pool: db_pool.clone(),
                });

            let server_global = Arc::new(ServerGlobal {
                db_pool,
                session_store: SessionStore::new(Duration::from_hours(24)),
                character_storage,
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
        Commands::Migrate { connection_string } => {
            let _ = migrate_db::migrate_db_and_check_lock(connection_string)
                .await
                .context("connecting to db/running migration")?;
            println!("Migration performed, database connection checked");
        }
    }

    Ok(())
}
