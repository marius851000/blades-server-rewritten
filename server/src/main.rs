use std::{path::PathBuf, sync::Arc, time::Duration};

use actix_files::Files;
use actix_web::{App, HttpServer, main, web::Data};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
mod migrate_db;
use deadpool_postgres::{
    Manager, ManagerConfig, Pool,
    tokio_postgres::{self, NoTls},
};
use log::debug;

mod announcements;
mod authentification;
mod error;
mod session;
pub use error::BladeApiError;
use tokio_postgres::Config;

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
    /// Run database migrations
    Migrate {
        /// Database connection string
        #[arg(short, long)]
        connection_string: String,
    },
}

pub struct ServerGlobal {
    db_pool: Pool,
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
            let main_lock_client = migrate_db::migrate_db_and_check_lock(connection_string)
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
            let db_pool = Pool::builder(db_mgr)
                .max_size(16)
                .build()
                .context("building db connection pool")?;

            let server_global = Arc::new(ServerGlobal {
                db_pool,
                session_store: SessionStore::new(Duration::from_hours(24)),
            });

            let static_data_clone = static_data.clone();

            HttpServer::new(move || {
                App::new()
                    .app_data(Data::new(server_global.clone()))
                    .service(authentification::anon_log_in)
                    .service(announcements::check_status)
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
