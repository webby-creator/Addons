/// Instances of addons used on websites
use common::{AddonInstanceId, WebsiteId};
use sqlx::{FromRow, Result, SqliteConnection};
use time::OffsetDateTime;
use uuid::Uuid;

pub struct NewAddonInstance {
    pub website_id: WebsiteId,
    pub website_uuid: Uuid,
}

#[derive(FromRow)]
pub struct AddonInstance {
    pub id: AddonInstanceId,
    pub public_id: Uuid,

    pub website_id: WebsiteId,
    pub website_uuid: Uuid,

    pub is_setup: bool,

    // TODO: Should I store some sort of settings here? Not related to addon, but the instance itself.
    pub delete_reason: Option<String>,

    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub deleted_at: Option<OffsetDateTime>,
}

impl NewAddonInstance {
    pub async fn insert(self, db: &mut SqliteConnection) -> Result<AddonInstance> {
        let public_id = Uuid::now_v7();
        let now = OffsetDateTime::now_utc();

        let resp = sqlx::query(
            "INSERT INTO addon_instance (public_id, website_id, website_uuid, created_at, updated_at) VALUES ($1, $2, $3, $4, $4)",
        )
            .bind(public_id)
            .bind(self.website_id)
            .bind(self.website_uuid)
            .bind(now)
            .execute(db)
            .await?;

        Ok(AddonInstance {
            id: AddonInstanceId::from(resp.last_insert_rowid()),
            public_id,
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

impl AddonInstance {
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

    pub async fn find_by_uuid(uuid: Uuid, db: &mut SqliteConnection) -> Result<Option<Self>> {
        Ok(sqlx::query_as(
            "SELECT id, public_id, website_id, website_uuid, delete_reason, created_at, updated_at, deleted_at FROM addon_instance WHERE public_id = $1",
        )
        .bind(uuid)
        .fetch_optional(db)
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
