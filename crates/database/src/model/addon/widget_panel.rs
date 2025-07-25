use api::WidgetPanelSettings;
use eyre::Result;
use global_common::id::{AddonWidgetPanelPublicId, AddonWidgetPublicId};
use local_common::{AddonId, AddonWidgetId, AddonWidgetPanelId};
use serde::Serialize;
use sqlx::{types::Json, FromRow, SqliteConnection};
use storage::{WidgetPanelContent, CURRENT_PANEL_VERSION};
use time::OffsetDateTime;

use crate::Binary;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewAddonWidgetPanelContentModel {
    pub addon_id: AddonId,
    pub addon_widget_id: AddonWidgetId,

    pub data: WidgetPanelContent,

    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AddonWidgetPanelContentModel {
    pub pk: AddonWidgetPanelId,
    pub id: AddonWidgetPanelPublicId,

    pub addon_id: AddonId,
    pub addon_widget_id: AddonWidgetId,

    pub data: Binary<WidgetPanelContent>,
    pub version: i32,

    pub title: Option<String>,
    pub settings: Json<WidgetPanelSettings>,

    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AddonWidgetPanelNoDataModel {
    pub pk: AddonWidgetPanelId,
    pub id: AddonWidgetPanelPublicId,

    pub addon_id: AddonId,
    pub addon_widget_id: AddonWidgetId,

    pub version: i32,

    pub title: Option<String>,
    pub settings: Json<WidgetPanelSettings>,

    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl NewAddonWidgetPanelContentModel {
    pub async fn insert(self, db: &mut SqliteConnection) -> Result<AddonWidgetPanelContentModel> {
        let id = AddonWidgetPanelPublicId::new();
        let now = OffsetDateTime::now_utc();
        let data = Binary(self.data);
        let settings = Json(WidgetPanelSettings::default());

        let res = sqlx::query(
            "INSERT INTO addon_widget_panel (id, addon_id, addon_widget_id, data, version, title, created_at, updated_at, settings) VALUES ($1, $2, $3, $4, $5, $6, $7, $7, $8)",
        )
        .bind(id)
        .bind(self.addon_id)
        .bind(self.addon_widget_id)
        .bind(&data)
        .bind(CURRENT_PANEL_VERSION)
        .bind(&self.title)
        .bind(now)
        .bind(&settings)
        .execute(db)
        .await?;

        Ok(AddonWidgetPanelContentModel {
            pk: AddonWidgetPanelId::from(res.last_insert_rowid() as i32),
            id,
            addon_id: self.addon_id,
            addon_widget_id: self.addon_widget_id,

            data,
            version: CURRENT_PANEL_VERSION,

            title: self.title,
            settings,

            created_at: now,
            updated_at: now,
        })
    }
}

impl AddonWidgetPanelContentModel {
    pub async fn update(&mut self, db: &mut SqliteConnection) -> Result<u64> {
        self.updated_at = OffsetDateTime::now_utc();

        let res = sqlx::query(
            r#"UPDATE addon_widget_panel SET
    data = $2,
    version = $3,
    title = $4,
    updated_at = $5,
    settings = $6
WHERE id = $1"#,
        )
        .bind(self.id)
        .bind(&self.data)
        .bind(self.version)
        .bind(&self.title)
        .bind(self.updated_at)
        .bind(&self.settings)
        .execute(db)
        .await?;

        Ok(res.rows_affected())
    }

    pub async fn delete(id: AddonWidgetPanelPublicId, db: &mut SqliteConnection) -> Result<u64> {
        let res = sqlx::query("DELETE FROM addon_widget_panel WHERE id = $1")
            .bind(id)
            .execute(db)
            .await?;

        Ok(res.rows_affected())
    }

    pub async fn get_all_no_data(
        widget_id: AddonWidgetPublicId,
        db: &mut SqliteConnection,
    ) -> Result<Vec<AddonWidgetPanelNoDataModel>> {
        Ok(sqlx::query_as(
            "SELECT
    addon_widget_panel.*
FROM
    addon_widget_panel
JOIN
    addon_widget_content ON addon_widget_content.id = $1
WHERE
    addon_widget_panel.addon_widget_id = addon_widget_content.pk",
        )
        .bind(widget_id)
        .fetch_all(db)
        .await?)
    }

    pub async fn count(db: &mut SqliteConnection) -> Result<i32> {
        Ok(
            sqlx::query_scalar("SELECT COUNT(*) FROM addon_widget_panel")
                .fetch_one(db)
                .await?,
        )
    }

    pub async fn find_by_addon_id(uuid: AddonId, db: &mut SqliteConnection) -> Result<Vec<Self>> {
        Ok(
            sqlx::query_as(
                "SELECT pk, id, addon_id, addon_widget_id, data, version, title, settings, created_at, updated_at FROM addon_widget_panel WHERE addon_id = $1",
            )
            .bind(uuid)
            .fetch_all(db)
            .await?,
        )
    }

    pub async fn find_one_by_public_id(
        id: AddonWidgetPanelPublicId,
        db: &mut SqliteConnection,
    ) -> Result<Option<Self>> {
        Ok(
            sqlx::query_as(
                "SELECT pk, id, addon_id, addon_widget_id, data, version, title, settings, created_at, updated_at FROM addon_widget_panel WHERE id = $1",
            )
            .bind(id)
            .fetch_optional(db)
            .await?,
        )
    }

    pub async fn find_one_by_public_id_no_data(
        id: AddonWidgetPanelPublicId,
        db: &mut SqliteConnection,
    ) -> Result<Option<AddonWidgetPanelNoDataModel>> {
        Ok(sqlx::query_as(
            "SELECT pk, id, addon_id, addon_widget_id, version, title, settings, created_at, updated_at FROM addon_widget_panel WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(db)
        .await?)
    }
}
