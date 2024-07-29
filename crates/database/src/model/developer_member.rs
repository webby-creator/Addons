use common::DeveloperId;
use eyre::Result;
use serde::Serialize;
use sqlx::{FromRow, SqliteConnection};
use uuid::Uuid;

#[derive(FromRow, Serialize)]
pub struct DeveloperMemberModel {
    pub developer_id: DeveloperId,
    pub member_guid: Uuid,
}

impl DeveloperMemberModel {
    pub async fn insert(&self, db: &mut SqliteConnection) -> Result<()> {
        sqlx::query("INSERT INTO developer_member (developer_id, member_guid) VALUES ($1, $2)")
            .bind(&self.developer_id)
            .bind(&self.member_guid)
            .execute(db)
            .await?;

        Ok(())
    }

    pub async fn find_by_member_guid(guid: Uuid, db: &mut SqliteConnection) -> Result<Vec<Self>> {
        Ok(sqlx::query_as(
            "SELECT developer_id, member_guid FROM developer_member WHERE member_guid = $1",
        )
        .bind(guid)
        .fetch_all(db)
        .await?)
    }

    pub async fn find_by_developer_id(
        id: DeveloperId,
        db: &mut SqliteConnection,
    ) -> Result<Vec<Self>> {
        Ok(sqlx::query_as(
            "SELECT developer_id, member_guid FROM developer_member WHERE developer_id = $1",
        )
        .bind(id)
        .fetch_all(db)
        .await?)
    }

    pub async fn delete(self, db: &mut SqliteConnection) -> Result<u64> {
        let res =
            sqlx::query("DELETE FROM developer_member WHERE developer_id = $1, member_guid = $2")
                .bind(self.developer_id)
                .bind(self.member_guid)
                .execute(db)
                .await?;

        Ok(res.rows_affected())
    }
}
