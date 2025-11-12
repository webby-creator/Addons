use eyre::Result;
use local_common::{AddonId, AddonWidgetId, WebsiteId};
use serde::Serialize;
use sqlx::{types::Json, FromRow, SqliteConnection};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewWebsiteWidgetSettingsModel {
    pub website_id: WebsiteId,
    pub addon_id: AddonId,
    pub addon_widget_id: AddonWidgetId,
    pub object_id: Option<Uuid>,
    pub settings: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct WebsiteWidgetSettingsModel {
    pub pk: i32,
    pub website_id: WebsiteId,
    pub addon_id: AddonId,
    pub addon_widget_id: AddonWidgetId,
    pub object_id: Option<Uuid>,
    pub settings: Json<serde_json::Value>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl NewWebsiteWidgetSettingsModel {
    pub async fn insert(self, db: &mut SqliteConnection) -> Result<WebsiteWidgetSettingsModel> {
        let now = OffsetDateTime::now_utc();
        let settings_json = Json(self.settings);

        let res = sqlx::query(
            r#"INSERT INTO addon_widget_settings (
    website_id,
    addon_id,
    addon_widget_id,
    object_id,
    settings,
    created_at,
    updated_at
)
VALUES ($1, $2, $3, $4, $5, $6, $6)"#,
        )
        .bind(self.website_id)
        .bind(self.addon_id)
        .bind(self.addon_widget_id)
        .bind(self.object_id)
        .bind(&settings_json)
        .bind(now)
        .execute(db)
        .await?;

        Ok(WebsiteWidgetSettingsModel {
            pk: res.last_insert_rowid() as i32,
            website_id: self.website_id,
            addon_id: self.addon_id,
            addon_widget_id: self.addon_widget_id,
            object_id: self.object_id,
            settings: Json(settings_json.0.unwrap_or_else(|| serde_json::json!({}))),
            created_at: now,
            updated_at: now,
        })
    }
}

impl WebsiteWidgetSettingsModel {
    pub async fn update(&mut self, db: &mut SqliteConnection) -> Result<u64> {
        self.updated_at = OffsetDateTime::now_utc();

        let res = sqlx::query(
            r#"UPDATE addon_widget_settings SET
    addon_id = $2,
    addon_widget_id = $3,
    object_id = $4,
    settings = $5,
    updated_at = $6
WHERE pk = $1 AND website_id = $7"#,
        )
        .bind(self.pk)
        .bind(self.addon_id)
        .bind(self.addon_widget_id)
        .bind(self.object_id.clone())
        .bind(&self.settings)
        .bind(self.updated_at)
        .bind(self.website_id)
        .execute(db)
        .await?;

        Ok(res.rows_affected())
    }

    pub async fn find_one_by_pk(
        pk: i32,
        website_id: WebsiteId,
        db: &mut SqliteConnection,
    ) -> Result<Option<Self>> {
        Ok(sqlx::query_as(
            r#"SELECT
    pk,
    website_id,
    addon_id,
    addon_widget_id,
    object_id,
    settings,
    created_at,
    updated_at
FROM addon_widget_settings
WHERE pk = $1 AND website_id = $2"#,
        )
        .bind(pk)
        .bind(website_id)
        .fetch_optional(db)
        .await?)
    }

    pub async fn find_all_by_website_id(
        website_id: WebsiteId,
        db: &mut SqliteConnection,
    ) -> Result<Vec<Self>> {
        Ok(sqlx::query_as(
            r#"SELECT
    pk,
    website_id,
    addon_id,
    addon_widget_id,
    object_id,
    settings,
    created_at,
    updated_at
FROM addon_widget_settings
WHERE website_id = $1"#,
        )
        .bind(website_id)
        .fetch_all(db)
        .await?)
    }

    pub async fn find_one_by_website_id_and_object_id(
        website_id: WebsiteId,
        widget_id: AddonWidgetId,
        object_id: Option<Uuid>,
        db: &mut SqliteConnection,
    ) -> Result<Option<Self>> {
        Ok(sqlx::query_as(
            r#"SELECT
    pk,
    website_id,
    addon_id,
    addon_widget_id,
    object_id,
    settings,
    created_at,
    updated_at
FROM addon_widget_settings
WHERE website_id = $1 AND addon_widget_id = $2 AND object_id = $3"#,
        )
        .bind(website_id)
        .bind(widget_id)
        .bind(object_id)
        .fetch_optional(db)
        .await?)
    }
}
