/// Instances of addons used on websites
use local_common::{AddonId, AddonInstanceId, WebsiteId};
use sqlx::{FromRow, Result, SqliteConnection};
use time::OffsetDateTime;
use uuid::Uuid;

pub struct NewAddonInstanceModel {
    pub addon_id: AddonId,

    pub website_id: WebsiteId,
    pub website_uuid: Uuid,
}

#[derive(Debug, FromRow)]
pub struct AddonInstanceModel {
    pub id: AddonInstanceId,
    pub public_id: Uuid,

    pub addon_id: AddonId,

    pub website_id: WebsiteId,
    pub website_uuid: Uuid,

    pub is_setup: bool,

    // TODO: Should I store some sort of settings here? Not related to addon, but the instance itself.
    pub delete_reason: Option<String>,

    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub deleted_at: Option<OffsetDateTime>,
}

impl NewAddonInstanceModel {
    pub async fn insert(self, db: &mut SqliteConnection) -> Result<AddonInstanceModel> {
        let public_id = Uuid::now_v7();
        let now = OffsetDateTime::now_utc();

        let resp = sqlx::query(
            "INSERT INTO addon_instance (public_id, addon_id, website_id, website_uuid, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $5)",
        )
            .bind(public_id)
            .bind(self.addon_id)
            .bind(self.website_id)
            .bind(self.website_uuid)
            .bind(now)
            .execute(db)
            .await?;

        Ok(AddonInstanceModel {
            id: AddonInstanceId::from(resp.last_insert_rowid()),
            public_id,
            addon_id: self.addon_id,
            website_id: self.website_id,
            website_uuid: self.website_uuid,
            is_setup: false,
            delete_reason: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        })
    }
}

impl AddonInstanceModel {
    pub async fn update(&mut self, db: &mut SqliteConnection) -> Result<u64> {
        self.updated_at = OffsetDateTime::now_utc();

        let res =
            sqlx::query("UPDATE addon_instance SET is_setup = $2, updated_at = $3 WHERE id = $1")
                .bind(self.id)
                .bind(self.is_setup)
                .bind(self.updated_at)
                .execute(db)
                .await?;

        Ok(res.rows_affected())
    }

    pub async fn delete(self, db: &mut SqliteConnection) -> Result<u64> {
        Self::delete_by_id(self.id, db).await
    }

    //

    pub async fn find_by_uuid(uuid: Uuid, db: &mut SqliteConnection) -> Result<Option<Self>> {
        Ok(sqlx::query_as(
            "SELECT id, public_id, addon_id, website_id, website_uuid, is_setup, delete_reason, created_at, updated_at, deleted_at FROM addon_instance WHERE public_id = $1",
        )
        .bind(uuid)
        .fetch_optional(db)
        .await?)
    }

    pub async fn find_by_addon_website_id(
        addon_id: AddonId,
        website_id: Uuid,
        db: &mut SqliteConnection,
    ) -> Result<Option<Self>> {
        Ok(sqlx::query_as(
            "SELECT id, public_id, addon_id, website_id, website_uuid, is_setup, delete_reason, created_at, updated_at, deleted_at FROM addon_instance WHERE addon_id = $1 AND website_uuid = $2",
        )
        .bind(addon_id)
        .bind(website_id)
        .fetch_optional(db)
        .await?)
    }

    pub async fn find_by_website_uuid(uuid: Uuid, db: &mut SqliteConnection) -> Result<Vec<Self>> {
        Ok(sqlx::query_as(
            "SELECT id, public_id, addon_id, website_id, website_uuid, is_setup, delete_reason, created_at, updated_at, deleted_at FROM addon_instance WHERE website_uuid = $1",
        )
        .bind(uuid)
        .fetch_all(db)
        .await?)
    }

    pub async fn delete_by_id(id: AddonInstanceId, db: &mut SqliteConnection) -> Result<u64> {
        let res = sqlx::query("DELETE FROM addon_instance WHERE id = $1")
            .bind(id)
            .execute(db)
            .await?;

        Ok(res.rows_affected())
    }
}
