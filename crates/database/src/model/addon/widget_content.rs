use api::WidgetSettings;
use eyre::Result;
use global_common::id::AddonWidgetPublicId;
use local_common::{AddonId, AddonWidgetId};
use serde::Serialize;
use sqlx::{types::Json, FromRow, SqliteConnection};
use storage::{DisplayStore, PageStoreV0, CURRENT_STORE_VERSION};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct NewAddonWidgetContent {
    pub addon_id: AddonId,

    pub data: DisplayStore,

    pub title: Option<String>,
    pub description: Option<String>,
    pub thumbnail: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonWidgetContent {
    pub pk: AddonWidgetId,
    pub id: AddonWidgetPublicId,

    pub addon_id: AddonId,

    // NOTE: In the SQL table its' referred to as a Blob
    pub data: Json<DisplayStore>,
    pub version: i32,

    pub title: Option<String>,
    pub description: Option<String>,
    pub thumbnail: Option<String>,
    pub settings: Json<WidgetSettings>,

    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AddonWidgetNoDataModel {
    pub pk: AddonWidgetId,
    pub id: AddonWidgetPublicId,

    pub addon_id: AddonId,

    pub version: i32,

    pub title: Option<String>,
    pub description: Option<String>,
    pub thumbnail: Option<String>,
    pub settings: Json<WidgetSettings>,

    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl NewAddonWidgetContent {
    pub async fn insert(self, db: &mut SqliteConnection) -> Result<AddonWidgetContent> {
        let id = AddonWidgetPublicId::new();
        let now = OffsetDateTime::now_utc();
        let data = Json(self.data);

        let res = sqlx::query(
            "INSERT INTO addon_widget_content (id, addon_id, data, version, title, description, thumbnail, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(id)
        .bind(self.addon_id)
        .bind(&data)
        .bind(CURRENT_STORE_VERSION)
        .bind(&self.title)
        .bind(&self.description)
        .bind(&self.thumbnail)
        .bind(now)
        .bind(now)
        .execute(db)
        .await?;

        Ok(AddonWidgetContent {
            pk: AddonWidgetId::from(res.last_insert_rowid() as i32),
            id,
            addon_id: self.addon_id,

            data,
            version: CURRENT_STORE_VERSION,

            title: self.title,
            description: self.description,
            thumbnail: self.thumbnail,
            settings: Json(WidgetSettings::default()),

            created_at: now,
            updated_at: now,
        })
    }
}

impl AddonWidgetContent {
    pub async fn update(&mut self, db: &mut SqliteConnection) -> Result<u64> {
        self.updated_at = OffsetDateTime::now_utc();

        let res = sqlx::query(
            r#"UPDATE addon_widget_content SET
                data = $2,
                version = $3,
                title = $4,
                description = $5,
                thumbnail = $6,
                updated_at = $7,
                settings = $8
            WHERE id = $1"#,
        )
        .bind(self.id)
        .bind(&self.data)
        .bind(self.version)
        .bind(&self.title)
        .bind(&self.description)
        .bind(&self.thumbnail)
        .bind(self.updated_at)
        .bind(&self.settings)
        .execute(db)
        .await?;

        Ok(res.rows_affected())
    }

    pub async fn delete(id: AddonWidgetPublicId, db: &mut SqliteConnection) -> Result<u64> {
        let res = sqlx::query("DELETE FROM addon_widget_content WHERE id = $1")
            .bind(id)
            .execute(db)
            .await?;

        Ok(res.rows_affected())
    }

    pub async fn get_all_no_data(
        addon_id: AddonId,
        db: &mut SqliteConnection,
    ) -> Result<Vec<AddonWidgetNoDataModel>> {
        Ok(sqlx::query_as(
            "SELECT pk, id, addon_id, version, title, description, thumbnail, settings, created_at, updated_at FROM addon_widget_content WHERE addon_id = $1",
        )
        .bind(addon_id)
        .fetch_all(db)
        .await?)
    }

    pub async fn count(db: &mut SqliteConnection) -> Result<i32> {
        Ok(
            sqlx::query_scalar("SELECT COUNT(*) FROM addon_widget_content")
                .fetch_one(db)
                .await?,
        )
    }

    pub async fn find_by_addon_id(id: AddonId, db: &mut SqliteConnection) -> Result<Vec<Self>> {
        Ok(
            sqlx::query_as(
                "SELECT pk, id, addon_id, data, version, title, description, thumbnail, settings, created_at, updated_at FROM addon_widget_content WHERE addon_id = $1",
            )
            .bind(id)
            .fetch_all(db)
            .await?,
        )
    }

    pub async fn find_one_by_public_id(
        id: AddonWidgetPublicId,
        db: &mut SqliteConnection,
    ) -> Result<Option<Self>> {
        Ok(
            sqlx::query_as(
                "SELECT pk, id, addon_id, data, version, title, description, thumbnail, settings, created_at, updated_at FROM addon_widget_content WHERE id = $1",
            )
            .bind(id)
            .fetch_optional(db)
            .await?,
        )
    }

    pub async fn find_one_by_public_id_no_data(
        id: AddonWidgetPublicId,
        db: &mut SqliteConnection,
    ) -> Result<Option<AddonWidgetNoDataModel>> {
        Ok(sqlx::query_as(
            "SELECT pk, id, addon_id, version, title, description, thumbnail, settings, created_at, updated_at FROM addon_widget_content WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(db)
        .await?)
    }
}

impl<'a, R: sqlx::Row> FromRow<'a, R> for AddonWidgetContent
where
    &'a ::std::primitive::str: ::sqlx::ColumnIndex<R>,
    String: ::sqlx::decode::Decode<'a, R::Database>,
    String: ::sqlx::types::Type<R::Database>,
    AddonWidgetPublicId: ::sqlx::decode::Decode<'a, R::Database>,
    AddonWidgetPublicId: ::sqlx::types::Type<R::Database>,
    AddonId: ::sqlx::decode::Decode<'a, R::Database>,
    AddonId: ::sqlx::types::Type<R::Database>,
    Json<DisplayStore>: ::sqlx::decode::Decode<'a, R::Database>,
    Json<DisplayStore>: ::sqlx::types::Type<R::Database>,
    Json<PageStoreV0>: ::sqlx::decode::Decode<'a, R::Database>,
    Json<PageStoreV0>: ::sqlx::types::Type<R::Database>,
    Json<WidgetSettings>: ::sqlx::decode::Decode<'a, R::Database>,
    Json<WidgetSettings>: ::sqlx::types::Type<R::Database>,
    i32: ::sqlx::decode::Decode<'a, R::Database>,
    i32: ::sqlx::types::Type<R::Database>,
    i64: ::sqlx::decode::Decode<'a, R::Database>,
    i64: ::sqlx::types::Type<R::Database>,
    OffsetDateTime: ::sqlx::decode::Decode<'a, R::Database>,
    OffsetDateTime: ::sqlx::types::Type<R::Database>,
{
    fn from_row(row: &'a R) -> std::result::Result<Self, sqlx::Error> {
        let pk = row.try_get("pk")?;
        let id = row.try_get("id")?;
        let addon_id = row.try_get("addon_id")?;
        let version = row.try_get("version")?;
        let title = row.try_get("title")?;
        let description = row.try_get("description")?;
        let thumbnail = row.try_get("thumbnail")?;
        let settings = row.try_get("settings")?;
        let created_at = row.try_get("created_at")?;
        let updated_at = row.try_get("updated_at")?;

        let data = match version {
            0 => Json(row.try_get::<Json<PageStoreV0>, _>("data")?.0.upgrade()),
            1 => row.try_get::<Json<DisplayStore>, _>("data")?,
            _ => unimplemented!(),
        };

        Ok(Self {
            pk,
            id,
            addon_id,
            data,
            version,
            title,
            description,
            thumbnail,
            settings,
            created_at,
            updated_at,
        })
    }
}
