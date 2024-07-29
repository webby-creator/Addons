use common::{DeveloperId, MediaId};
use eyre::Result;
use serde::Serialize;
use sqlx::{FromRow, SqliteConnection};
use time::OffsetDateTime;
use uuid::Uuid;

pub struct NewDeveloperModel {
    pub name: String,
    pub description: String,
    pub icon: Option<MediaId>,
}

#[derive(FromRow, Serialize)]
pub struct DeveloperModel {
    pub id: DeveloperId,

    pub guid: Uuid,

    pub name: String,
    pub description: String,
    pub icon: Option<MediaId>,

    pub addon_count: i32,
    pub delete_reason: Option<String>,

    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub deleted_at: Option<OffsetDateTime>,
}

impl NewDeveloperModel {
    pub async fn insert(self, db: &mut SqliteConnection) -> Result<DeveloperModel> {
        let now = OffsetDateTime::now_utc();
        let guid = Uuid::now_v7();

        let resp = sqlx::query(
            "INSERT INTO developer (guid, name, description, icon, addon_count, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $6)",
        )
        .bind(&guid)
        .bind(&self.name)
        .bind(&self.description)
        .bind(&self.icon)
        .bind(0)
        .bind(now)
        .execute(db)
        .await?;

        Ok(DeveloperModel {
            id: DeveloperId::from(resp.last_insert_rowid() as i32),
            guid,
            name: self.name,
            description: self.description,
            icon: self.icon,
            addon_count: 0,
            delete_reason: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        })
    }
}

impl DeveloperModel {
    pub async fn find_one_by_guid(guid: Uuid, db: &mut SqliteConnection) -> Result<Option<Self>> {
        Ok(sqlx::query_as(
            "SELECT id, guid, name, description, icon, addon_count, delete_reason, created_at, updated_at, deleted_at FROM developer WHERE guid = $1"
        )
        .bind(guid)
        .fetch_optional(db)
        .await?)
    }

    pub async fn delete(id: DeveloperId, reason: String, db: &mut SqliteConnection) -> Result<u64> {
        let res =
            sqlx::query("UPDATE developer SET deleted_at = $2, delete_reason = $3 WHERE id = $1")
                .bind(id)
                .bind(OffsetDateTime::now_utc())
                .bind(reason)
                .execute(db)
                .await?;

        Ok(res.rows_affected())
    }
}
