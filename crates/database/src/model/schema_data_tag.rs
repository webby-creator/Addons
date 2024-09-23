use eyre::Result;
use local_common::{SchemaDataTagId, SchemaId};
use sqlx::{FromRow, SqliteConnection};

#[derive(FromRow)]
pub struct SchemaDataTagModel {
    pub id: SchemaDataTagId,
    pub schema_id: SchemaId,
    pub row_id: String,

    pub name: String,
    pub color: String,
}

impl SchemaDataTagModel {
    pub async fn insert(
        schema_id: SchemaId,
        row_id: String,
        name: String,
        color: String,
        db: &mut SqliteConnection,
    ) -> Result<Self> {
        let resp = sqlx::query(
            "INSERT INTO schema_data_tag (schema_id, row_id, name, name_lower, color) VALUES ($1, $2, $3, $4, $5)",
        )
            .bind(schema_id)
            .bind(&row_id)
            .bind(&name)
            .bind(name.to_lowercase())
            .bind(&color)
            .execute(&mut *db)
            .await;

        match resp {
            Ok(resp) => Ok(Self {
                id: SchemaDataTagId::from(resp.last_insert_rowid()),
                schema_id,
                row_id,
                name,
                color,
            }),
            Err(e) => {
                if e.as_database_error()
                    .map(|v| v.is_unique_violation())
                    .unwrap_or_default()
                {
                    if let Some(found) = Self::find_one(schema_id, &row_id, &name, db).await? {
                        return Ok(found);
                    } else {
                        eyre::bail!("Unable to find Row which exists! Schema: {schema_id}, Row: {row_id}, Name: {name}");
                    }
                }

                return Err(e)?;
            }
        }
    }

    pub async fn delete(id: SchemaDataTagId, db: &mut SqliteConnection) -> Result<u64> {
        let res = sqlx::query("DELETE FROM schema_data_tag WHERE id = $1")
            .bind(id)
            .execute(db)
            .await?;

        Ok(res.rows_affected())
    }

    pub async fn find_one(
        schema_id: SchemaId,
        row_id: &str,
        name: &str,
        db: &mut SqliteConnection,
    ) -> Result<Option<Self>> {
        Ok(sqlx::query_as(
            "SELECT id, schema_id, row_id, name, color FROM schema_data_tag WHERE schema_id = $1 AND row_id = $2 AND name_lower = $3",
        )
        .bind(schema_id)
        .bind(row_id)
        .bind(name.to_lowercase())
        .fetch_optional(db)
        .await?)
    }

    pub async fn get_all(schema_id: SchemaId, db: &mut SqliteConnection) -> Result<Vec<Self>> {
        Ok(sqlx::query_as(
            "SELECT id, schema_id, row_id, name, color FROM schema_data_tag WHERE schema_id = $1",
        )
        .bind(schema_id)
        .fetch_all(db)
        .await?)
    }

    pub async fn count(schema_id: SchemaId, db: &mut SqliteConnection) -> Result<i64> {
        Ok(
            sqlx::query_scalar("SELECT COUNT(*) FROM schema_data_tag WHERE schema_id = $1")
                .bind(schema_id)
                .fetch_one(db)
                .await?,
        )
    }
}
