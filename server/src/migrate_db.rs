use anyhow::{Context, bail};
use log::{error, info};
use tokio_postgres::{Client, NoTls, Transaction, error::SqlState};

pub async fn migrate_db_and_check_lock(connection_string: &str) -> anyhow::Result<Client> {
    let (mut client, connection) = tokio_postgres::connect(connection_string, NoTls).await?;
    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("connection error: {}", e);
        }
    });

    // check and acquire lock 10
    let lock_acquired: bool = client
        .query("SELECT pg_try_advisory_lock(10)", &[])
        .await?
        .get(0)
        .context("no row 0 for lock acquisition result")?
        .get(0);
    if !lock_acquired {
        bail!(
            "Database advisory lock 10 acquired. There is likely already running server, which is not supported."
        );
    }

    async fn make_transaction_to_version<'a>(
        transaction: &Transaction<'a>,
        new_version: u64,
    ) -> anyhow::Result<u64> {
        Ok(transaction
            .execute(
                "UPDATE globals SET value = $1 WHERE name = 'version'",
                &[&format!("{}", new_version)],
            )
            .await?)
    }

    let mut version: u64 = match client
        .query("SELECT value FROM globals WHERE name='version'", &[])
        .await
    {
        Ok(r) => {
            if let Some(version_row) = r.first() {
                let version_value_text: String = version_row.try_get(0)?;
                version_value_text
                    .parse()
                    .context("parsing version globals text")?
            } else {
                bail!("no \"version\" globals in db");
            }
        }
        Err(e) => {
            if e.code() == Some(&SqlState::UNDEFINED_TABLE) {
                let transaction = client.transaction().await?;
                transaction
                    .execute(
                        "CREATE TABLE globals (name TEXT PRIMARY KEY, value TEXT)",
                        &[],
                    )
                    .await?;
                transaction
                    .execute(
                        "INSERT INTO globals (name, value) VALUES ('version', '1')",
                        &[],
                    )
                    .await?;
                transaction.commit().await?;
                1
            } else {
                bail!(e);
            }
        }
    };

    info!("Pre-migration db version is {}", version);

    if version == 1 {
        let transaction = client.transaction().await?;
        transaction
            .execute("CREATE TABLE users (id UUID PRIMARY KEY, data JSONB)", &[])
            .await?;
        make_transaction_to_version(&transaction, 2).await?;
        transaction.commit().await?;
        version = 2;
    }

    if version == 2 {
        let transaction = client.transaction().await?;
        transaction
            .execute(
                "CREATE TABLE characters (id UUID PRIMARY KEY, data JSONB)",
                &[],
            )
            .await?;
        make_transaction_to_version(&transaction, 3).await?;
        transaction.commit().await?;
        version = 3;
    }

    const EXPECTED_FINAL_VERSION: u64 = 3;
    if version != EXPECTED_FINAL_VERSION {
        bail!(
            "Expected post-migration version to be {}",
            EXPECTED_FINAL_VERSION
        );
    }

    info!("Post-migration db version is {}", version);
    Ok(client)
}
