use eyre::Result;
use webby_global_common::id::AddonCompiledWidgetPublicId;
use local_common::{AddonCompiledId, AddonCompiledWidgetId, AddonId, AddonWidgetId};
use serde::Serialize;
use sqlx::{types::Json, FromRow, SqliteConnection};
use webby_storage::{widget::CompiledWidgetSettings, DisplayStore, PageStoreV0, CURRENT_STORE_VERSION};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct NewAddonCompiledWidget {
    pub addon_id: AddonId,
    pub widget_id: AddonWidgetId,
    pub compiled_id: AddonCompiledId,

    pub data: DisplayStore,
    pub script: Option<String>,
    pub hash: String,

    pub title: Option<String>,
    pub description: Option<String>,
    pub thumbnail: Option<String>,
    pub settings: CompiledWidgetSettings,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonCompiledWidget {
    pub pk: AddonCompiledWidgetId,
    pub id: AddonCompiledWidgetPublicId,

    pub addon_id: AddonId,
    pub widget_id: AddonWidgetId,
    pub compiled_id: AddonCompiledId,

    pub data: Json<DisplayStore>,
    pub script: Option<String>,
    pub version: i32,
    pub hash: String,

    pub title: Option<String>,
    pub description: Option<String>,
    pub thumbnail: Option<String>,
    pub settings: Json<CompiledWidgetSettings>,

    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl NewAddonCompiledWidget {
    pub async fn insert(self, db: &mut SqliteConnection) -> Result<AddonCompiledWidget> {
        let id = AddonCompiledWidgetPublicId::new();
        let now = OffsetDateTime::now_utc();
        let data = Json(self.data);
        let settings = Json(self.settings);

        let res = sqlx::query(
            "INSERT INTO addon_compiled_widget (id, addon_id, widget_id, compiled_id, data, script, version, title, description, thumbnail, settings, hash, created_at, updated_at)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $13)",
        )
        .bind(id)
        .bind(self.addon_id)
        .bind(self.widget_id)
        .bind(self.compiled_id)
        .bind(&data)
        .bind(&self.script)
        .bind(CURRENT_STORE_VERSION)
        .bind(&self.title)
        .bind(&self.description)
        .bind(&self.thumbnail)
        .bind(&settings)
        .bind(&self.hash)
        .bind(now)
        .execute(db)
        .await?;

        Ok(AddonCompiledWidget {
            pk: AddonCompiledWidgetId::from(res.last_insert_rowid() as i32),
            id,
            addon_id: self.addon_id,
            widget_id: self.widget_id,
            compiled_id: self.compiled_id,

            data,
            script: self.script,
            version: CURRENT_STORE_VERSION,

            title: self.title,
            description: self.description,
            thumbnail: self.thumbnail,
            settings,
            hash: self.hash,

            created_at: now,
            updated_at: now,
        })
    }
}

impl AddonCompiledWidget {
    pub async fn find_one_by_public_id(
        id: AddonCompiledWidgetPublicId,
        db: &mut SqliteConnection,
    ) -> Result<Option<Self>> {
        Ok(
            sqlx::query_as(
                "SELECT pk, id, addon_id, widget_id, compiled_id, data, script, version, title, description, thumbnail, settings, hash, created_at, updated_at FROM addon_compiled_widget WHERE id = $1",
            )
            .bind(id)
            .fetch_optional(db)
            .await?,
        )
    }

    pub async fn find_one_by_compiled_id_and_widget_id(
        compiled_id: AddonCompiledId,
        widget_id: AddonWidgetId,
        db: &mut SqliteConnection,
    ) -> Result<Option<Self>> {
        Ok(
            sqlx::query_as(
                "SELECT pk, id, addon_id, widget_id, compiled_id, data, script, version, title, description, thumbnail, settings, hash, created_at, updated_at FROM addon_compiled_widget WHERE compiled_id = $1 AND widget_id = $2",
            )
            .bind(compiled_id)
            .bind(widget_id)
            .fetch_optional(db)
            .await?,
        )
    }
}

impl<'a, R: sqlx::Row> FromRow<'a, R> for AddonCompiledWidget
where
    &'a ::std::primitive::str: ::sqlx::ColumnIndex<R>,
    String: ::sqlx::decode::Decode<'a, R::Database>,
    String: ::sqlx::types::Type<R::Database>,
    AddonCompiledId: ::sqlx::decode::Decode<'a, R::Database>,
    AddonCompiledId: ::sqlx::types::Type<R::Database>,
    AddonCompiledWidgetPublicId: ::sqlx::decode::Decode<'a, R::Database>,
    AddonCompiledWidgetPublicId: ::sqlx::types::Type<R::Database>,
    AddonId: ::sqlx::decode::Decode<'a, R::Database>,
    AddonId: ::sqlx::types::Type<R::Database>,
    AddonWidgetId: ::sqlx::decode::Decode<'a, R::Database>,
    AddonWidgetId: ::sqlx::types::Type<R::Database>,
    Json<DisplayStore>: ::sqlx::decode::Decode<'a, R::Database>,
    Json<DisplayStore>: ::sqlx::types::Type<R::Database>,
    Json<PageStoreV0>: ::sqlx::decode::Decode<'a, R::Database>,
    Json<PageStoreV0>: ::sqlx::types::Type<R::Database>,
    Json<CompiledWidgetSettings>: ::sqlx::decode::Decode<'a, R::Database>,
    Json<CompiledWidgetSettings>: ::sqlx::types::Type<R::Database>,
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
        let widget_id = row.try_get("widget_id")?;
        let compiled_id = row.try_get("compiled_id")?;
        let script = row.try_get("script")?;
        let hash = row.try_get("hash")?;
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
            widget_id,
            data,
            script,
            version,
            title,
            description,
            thumbnail,
            settings,
            created_at,
            updated_at,
            compiled_id,
            hash,
        })
    }
}
