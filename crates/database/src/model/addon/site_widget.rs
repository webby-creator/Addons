use eyre::Result;
use webby_global_common::id::AddonWidgetPublicId;
use local_common::{AddonId, AddonWidgetId};
use serde::Serialize;
use sqlx::{FromRow, SqliteConnection};

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct WidgetModel {
    pub addon_id: AddonId,

    pub widget_id: AddonWidgetId,
    pub public_id: AddonWidgetPublicId,
}

impl WidgetModel {
    pub async fn insert(&self, db: &mut SqliteConnection) -> Result<()> {
        sqlx::query("INSERT INTO ref_widget (addon_id, widget_id, public_id) VALUES ($1, $2, $3)")
            .bind(self.addon_id)
            .bind(self.widget_id)
            .bind(self.public_id)
            .execute(db)
            .await?;

        Ok(())
    }

    pub async fn delete(id: AddonWidgetId, db: &mut SqliteConnection) -> Result<u64> {
        let res = sqlx::query("DELETE FROM ref_widget WHERE widget_id = $1")
            .bind(id)
            .execute(db)
            .await?;

        Ok(res.rows_affected())
    }

    pub async fn count(id: AddonId, db: &mut SqliteConnection) -> Result<i32> {
        Ok(
            sqlx::query_scalar("SELECT COUNT(*) FROM ref_widget WHERE addon_id = $1")
                .bind(id)
                .fetch_one(db)
                .await?,
        )
    }

    pub async fn find_one_by_id(
        id: AddonWidgetId,
        db: &mut SqliteConnection,
    ) -> Result<Option<Self>> {
        Ok(sqlx::query_as(
            "SELECT addon_id, widget_id, public_id FROM ref_widget WHERE widget_id = $1",
        )
        .bind(id)
        .fetch_optional(db)
        .await?)
    }

    pub async fn find_by_addon_id(id: AddonId, db: &mut SqliteConnection) -> Result<Vec<Self>> {
        Ok(sqlx::query_as(
            "SELECT addon_id, widget_id, public_id FROM ref_widget WHERE addon_id = $1",
        )
        .bind(id)
        .fetch_all(db)
        .await?)
    }
}
