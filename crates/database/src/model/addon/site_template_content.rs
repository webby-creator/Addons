use eyre::Result;
use local_common::AddonTemplatePageId;
use sqlx::{FromRow, SqliteConnection};
use webby_storage::{DisplayStore, PageStoreV0, CURRENT_STORE_VERSION};
use time::OffsetDateTime;

use crate::Blob;

#[derive(Debug, Clone)]
pub struct AddonTemplatePageContentModel {
    pub template_page_id: AddonTemplatePageId,

    pub content: Blob<DisplayStore>,
    pub version: i32,

    pub updated_at: OffsetDateTime,
}

impl AddonTemplatePageContentModel {
    pub fn new(template_page_id: AddonTemplatePageId, content: DisplayStore) -> Self {
        Self {
            template_page_id,
            content: Blob(content),
            version: CURRENT_STORE_VERSION,
            updated_at: OffsetDateTime::now_utc(),
        }
    }

    pub async fn insert(&self, db: &mut SqliteConnection) -> Result<()> {
        sqlx::query(
            "INSERT INTO template_page_content (template_page_id, content, version, updated_at) VALUES ($1, $2, $3, $4)",
        )
        .bind(self.template_page_id)
        .bind(&self.content)
        .bind(self.version)
        .bind(self.updated_at)
        .execute(db)
        .await?;

        Ok(())
    }

    pub async fn update(&mut self, db: &mut SqliteConnection) -> Result<u64> {
        self.updated_at = OffsetDateTime::now_utc();

        let res = sqlx::query(
            "UPDATE template_page_content SET content = $2, version = $3, updated_at = $4 WHERE template_page_id = $1",
        )
        .bind(self.template_page_id)
        .bind(&self.content)
        .bind(self.version)
        .bind(self.updated_at)
        .execute(db)
        .await?;

        Ok(res.rows_affected())
    }

    pub async fn delete(id: AddonTemplatePageId, db: &mut SqliteConnection) -> Result<u64> {
        let res = sqlx::query("DELETE FROM template_page_content WHERE template_page_id = $1")
            .bind(id)
            .execute(db)
            .await?;

        Ok(res.rows_affected())
    }

    pub async fn get_all(db: &mut SqliteConnection) -> Result<Vec<Self>> {
        Ok(sqlx::query_as(
            "SELECT template_page_id, content, version, updated_at FROM template_page_content",
        )
        .fetch_all(db)
        .await?)
    }

    pub async fn count(db: &mut SqliteConnection) -> Result<i32> {
        Ok(
            sqlx::query_scalar("SELECT COUNT(*) FROM template_page_content")
                .fetch_one(db)
                .await?,
        )
    }

    pub async fn find_one_by_page_id(
        id: AddonTemplatePageId,
        db: &mut SqliteConnection,
    ) -> Result<Option<Self>> {
        Ok(
            sqlx::query_as(
                "SELECT template_page_id, content, version, updated_at FROM template_page_content WHERE template_page_id = $1",
            )
            .bind(id)
            .fetch_optional(db)
            .await?,
        )
    }
}

impl<'a, R: sqlx::Row> FromRow<'a, R> for AddonTemplatePageContentModel
where
    &'a ::std::primitive::str: ::sqlx::ColumnIndex<R>,
    AddonTemplatePageId: ::sqlx::decode::Decode<'a, R::Database>,
    AddonTemplatePageId: ::sqlx::types::Type<R::Database>,
    Blob<DisplayStore>: ::sqlx::decode::Decode<'a, R::Database>,
    Blob<DisplayStore>: ::sqlx::types::Type<R::Database>,
    Blob<PageStoreV0>: ::sqlx::decode::Decode<'a, R::Database>,
    Blob<PageStoreV0>: ::sqlx::types::Type<R::Database>,
    i32: ::sqlx::decode::Decode<'a, R::Database>,
    i32: ::sqlx::types::Type<R::Database>,
    OffsetDateTime: ::sqlx::decode::Decode<'a, R::Database>,
    OffsetDateTime: ::sqlx::types::Type<R::Database>,
{
    fn from_row(row: &'a R) -> std::result::Result<Self, sqlx::Error> {
        let template_page_id = row.try_get("template_page_id")?;
        let version = row.try_get("version")?;
        let content = match version {
            0 => Blob(row.try_get::<Blob<PageStoreV0>, _>("content")?.0.upgrade()),
            1 => row.try_get::<Blob<DisplayStore>, _>("content")?,
            _ => unimplemented!(),
        };
        let updated_at = row.try_get("updated_at")?;

        Ok(Self {
            template_page_id,
            content,
            version,
            updated_at,
        })
    }
}
