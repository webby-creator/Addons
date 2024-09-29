use api::WebsitePageSettings;
use common::ObjectId;
use eyre::Result;
use local_common::{AddonId, AddonTemplatePageId};
use serde::Serialize;
use sqlx::{types::Json, FromRow, SqliteConnection};
use time::OffsetDateTime;
use uuid::Uuid;

pub struct NewAddonTemplatePageModel {
    pub addon_id: AddonId,

    pub public_id: Uuid,

    pub path: String,
    pub display_name: String,

    pub object_ids: Json<Vec<ObjectId>>,

    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl NewAddonTemplatePageModel {
    pub fn new(
        addon_id: AddonId,
        path: String,
        display_name: String,
        object_ids: Vec<ObjectId>,
    ) -> Self {
        Self {
            addon_id,
            public_id: Uuid::now_v7(),
            path,
            display_name,
            object_ids: Json(object_ids),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
        }
    }

    pub fn into_member(self, id: AddonTemplatePageId) -> AddonTemplatePageModel {
        AddonTemplatePageModel {
            id,
            addon_id: self.addon_id,
            public_id: self.public_id,
            path: self.path,
            display_name: self.display_name,
            object_ids: self.object_ids,
            settings: Json(WebsitePageSettings::default()),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct AddonTemplatePageModel {
    pub id: AddonTemplatePageId,
    pub addon_id: AddonId,

    pub public_id: Uuid,

    pub path: String,
    pub display_name: String,

    pub object_ids: Json<Vec<ObjectId>>,

    #[sqlx(default)]
    pub settings: Json<WebsitePageSettings>,

    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl NewAddonTemplatePageModel {
    pub async fn insert(self, db: &mut SqliteConnection) -> Result<AddonTemplatePageModel> {
        let res = sqlx::query(
            "INSERT INTO template_page (addon_id, public_id, path, display_name, object_ids, settings, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(self.addon_id)
        .bind(&self.public_id)
        .bind(&self.path)
        .bind(&self.display_name)
        .bind(&self.object_ids)
        .bind(Json(WebsitePageSettings::default()))
        .bind(self.created_at)
        .bind(self.updated_at)
        .execute(db)
        .await?;

        Ok(self.into_member(AddonTemplatePageId::from(res.last_insert_rowid())))
    }
}

impl AddonTemplatePageModel {
    pub async fn update(&mut self, db: &mut SqliteConnection) -> Result<u64> {
        self.updated_at = OffsetDateTime::now_utc();

        let res = sqlx::query(
            r#"UPDATE template_page SET
                public_id = $2,
                path = $3,
                display_name = $4,
                settings = $5,
                updated_at = $6,
                object_ids = $7
            WHERE id = $1"#,
        )
        .bind(self.id)
        .bind(&self.public_id)
        .bind(&self.path)
        .bind(&self.display_name)
        .bind(&self.settings)
        .bind(self.updated_at)
        .bind(&self.object_ids)
        .execute(db)
        .await?;

        Ok(res.rows_affected())
    }

    pub async fn delete(id: AddonId, db: &mut SqliteConnection) -> Result<u64> {
        let res = sqlx::query("DELETE FROM template_page WHERE id = $1")
            .bind(id)
            .execute(db)
            .await?;

        Ok(res.rows_affected())
    }

    pub async fn get_all_page_ids(id: AddonId, db: &mut SqliteConnection) -> Result<Vec<ObjectId>> {
        let object_ids_vec: Vec<(Json<Vec<String>>,)> =
            sqlx::query_as("SELECT object_ids FROM template_page WHERE addon_id = $1")
                .bind(id)
                .fetch_all(db)
                .await?;

        Ok(object_ids_vec
            .into_iter()
            .flat_map(|x| x.0 .0.into_iter().map(|v| ObjectId::from_specific(v)))
            .collect())
    }

    pub async fn count_by_addon_id(addon_id: AddonId, db: &mut SqliteConnection) -> Result<i32> {
        Ok(
            sqlx::query_scalar("SELECT COUNT(*) FROM template_page where addon_id = $1")
                .bind(addon_id)
                .fetch_one(db)
                .await?,
        )
    }

    pub async fn find_by_public_id(
        public_id: Uuid,
        db: &mut SqliteConnection,
    ) -> Result<Option<Self>> {
        Ok(sqlx::query_as(
            "SELECT id, addon_id, public_id, path, display_name, object_ids, settings, created_at, updated_at FROM template_page WHERE public_id = $1",
        )
        .bind(public_id)
        .fetch_optional(db)
        .await?)
    }

    pub async fn find_by_addon_id(
        addon_id: AddonId,
        db: &mut SqliteConnection,
    ) -> Result<Vec<Self>> {
        Ok(sqlx::query_as(
            "SELECT id, addon_id, public_id, path, display_name, object_ids, settings, created_at, updated_at FROM template_page WHERE addon_id = $1",
        )
        .bind(addon_id)
        .fetch_all(db)
        .await?)
    }

    pub async fn find_by_id(
        id: AddonTemplatePageId,
        db: &mut SqliteConnection,
    ) -> Result<Option<Self>> {
        Ok(sqlx::query_as(
            "SELECT id, addon_id, public_id, path, display_name, object_ids, settings, created_at, updated_at FROM template_page WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(db)
        .await?)
    }
}
