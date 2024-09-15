use common::{AddonId, DashboardPageInfo};
use sqlx::{FromRow, Result, SqliteConnection};

#[derive(FromRow)]
pub struct AddonDashboardPage {
    pub addon_id: AddonId,

    #[sqlx(rename = "type")]
    pub type_of: String,
    pub name: String,
    pub path: String,

    pub is_sidebar_visible: bool,
}

impl AddonDashboardPage {
    pub async fn insert(&self, db: &mut SqliteConnection) -> Result<()> {
        sqlx::query(
            "INSERT INTO addon_dashboard_page (addon_id, type, name, path, is_sidebar_visible) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(self.addon_id)
        .bind(&self.type_of)
        .bind(&self.name)
        .bind(&self.path)
        .bind(&self.is_sidebar_visible)
        .execute(db)
        .await?;

        Ok(())
    }

    pub async fn find_by_id(id: AddonId, db: &mut SqliteConnection) -> Result<Vec<Self>> {
        Ok(sqlx::query_as(
            "SELECT addon_id, type, name, path, is_sidebar_visible FROM addon_dashboard_page WHERE addon_id = $1",
        )
        .bind(id)
        .fetch_all(db)
        .await?)
    }

    pub async fn delete_by_id(id: AddonId, db: &mut SqliteConnection) -> Result<u64> {
        let res = sqlx::query("DELETE FROM addon_dashboard_page WHERE addon_id = $1")
            .bind(id)
            .execute(db)
            .await?;

        Ok(res.rows_affected())
    }
}

impl Into<DashboardPageInfo> for AddonDashboardPage {
    fn into(self) -> DashboardPageInfo {
        DashboardPageInfo {
            type_of: self.type_of,
            name: self.name,
            path: self.path,
        }
    }
}
