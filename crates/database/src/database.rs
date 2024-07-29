use eyre::Result;
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};

const DATABASE_PATH: &str = "./app/addons.db";

// TODO: After Postgres conversion, implement utilize sqlx "ipnetwork" feature

pub async fn init() -> Result<(bool, SqlitePool)> {
    let does_db_exist = Sqlite::database_exists(DATABASE_PATH)
        .await
        .unwrap_or(false);

    if !does_db_exist {
        debug!("Creating database {DATABASE_PATH}");

        Sqlite::create_database(DATABASE_PATH).await?;
    } else {
        debug!("Database already exists");
    }

    let pool = SqlitePool::connect(DATABASE_PATH).await?;

    match sqlx::migrate!("./migrations").run(&pool).await {
        Ok(_) => debug!("Migration success"),
        Err(error) => panic!("Migration Error: {error}"),
    }

    Ok((!does_db_exist, pool))
}
