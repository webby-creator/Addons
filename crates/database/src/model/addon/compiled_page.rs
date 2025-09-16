use webby_api::{WebsitePageSettings, WebsitePageType};
use eyre::Result;
use webby_global_common::{id::AddonCompiledPagePublicId, Either};
use local_common::{AddonCompiledId, AddonCompiledPageId, AddonId};
use webby_scripting::json::VisslContent;
use serde::Serialize;
use sqlx::{types::Json, FromRow, SqliteConnection};
use webby_storage::{DisplayStore, PageStoreV0, CURRENT_STORE_VERSION};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewAddonCompiledPage {
    pub addon_id: AddonId,
    pub compiled_id: AddonCompiledId,

    pub hash: String,

    pub data: DisplayStore,
    pub script: Option<Either<String, VisslContent>>,
    pub settings: WebsitePageSettings,

    pub type_of: WebsitePageType,
    pub path: String,
    pub display_name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonCompiledPage {
    pub pk: AddonCompiledPageId,
    pub id: AddonCompiledPagePublicId,

    pub addon_id: AddonId,
    pub compiled_id: AddonCompiledId,

    pub hash: String,

    pub data: Json<DisplayStore>,
    pub script: Json<Option<Either<String, VisslContent>>>,
    pub settings: Json<WebsitePageSettings>,
    pub version: i32,

    pub type_of: WebsitePageType,
    pub path: String,
    pub display_name: String,
}

impl NewAddonCompiledPage {
    pub async fn insert(self, db: &mut SqliteConnection) -> Result<AddonCompiledPage> {
        let id = AddonCompiledPagePublicId::new();
        let data = Json(self.data);
        let script = Json(self.script);
        let settings = Json(self.settings);

        let res = sqlx::query(
            "INSERT INTO addon_compiled_page (id, addon_id, compiled_id, data, script, settings, version, type_of, path, display_name, hash)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
        )
        .bind(id)
        .bind(self.addon_id)
        .bind(self.compiled_id)
        .bind(&data)
        .bind(&script)
        .bind(&settings)
        .bind(CURRENT_STORE_VERSION)
        .bind(self.type_of)
        .bind(&self.path)
        .bind(&self.display_name)
        .bind(&self.hash)
        .execute(db)
        .await?;

        Ok(AddonCompiledPage {
            pk: AddonCompiledPageId::from(res.last_insert_rowid() as i32),
            id,
            addon_id: self.addon_id,
            compiled_id: self.compiled_id,

            data,
            script,
            settings,
            version: CURRENT_STORE_VERSION,

            type_of: self.type_of,
            path: self.path,
            display_name: self.display_name,
            hash: self.hash,
        })
    }
}

impl AddonCompiledPage {
    pub async fn find_one_by_public_id(
        id: AddonCompiledPagePublicId,
        db: &mut SqliteConnection,
    ) -> Result<Option<Self>> {
        Ok(
            sqlx::query_as(
                "SELECT pk, id, addon_id, compiled_id, data, script, settings, version, type_of, path, display_name, hash FROM addon_compiled_page WHERE id = $1",
            )
            .bind(id)
            .fetch_optional(db)
            .await?,
        )
    }

    pub async fn find_by_compiled_id(
        compiled_id: AddonCompiledId,
        db: &mut SqliteConnection,
    ) -> Result<Vec<Self>> {
        Ok(
            sqlx::query_as(
                "SELECT pk, id, addon_id, compiled_id, data, script, settings, version, type_of, path, display_name, hash FROM addon_compiled_page WHERE compiled_id = $1",
            )
            .bind(compiled_id)
            .fetch_all(db)
            .await?,
        )
    }
}

impl<'a, R: sqlx::Row> FromRow<'a, R> for AddonCompiledPage
where
    &'a ::std::primitive::str: ::sqlx::ColumnIndex<R>,
    String: ::sqlx::decode::Decode<'a, R::Database>,
    String: ::sqlx::types::Type<R::Database>,
    AddonCompiledId: ::sqlx::decode::Decode<'a, R::Database>,
    AddonCompiledId: ::sqlx::types::Type<R::Database>,
    AddonCompiledPagePublicId: ::sqlx::decode::Decode<'a, R::Database>,
    AddonCompiledPagePublicId: ::sqlx::types::Type<R::Database>,
    AddonId: ::sqlx::decode::Decode<'a, R::Database>,
    AddonId: ::sqlx::types::Type<R::Database>,
    Json<DisplayStore>: ::sqlx::decode::Decode<'a, R::Database>,
    Json<DisplayStore>: ::sqlx::types::Type<R::Database>,
    Json<PageStoreV0>: ::sqlx::decode::Decode<'a, R::Database>,
    Json<PageStoreV0>: ::sqlx::types::Type<R::Database>,
    i32: ::sqlx::decode::Decode<'a, R::Database>,
    i32: ::sqlx::types::Type<R::Database>,
    i64: ::sqlx::decode::Decode<'a, R::Database>,
    i64: ::sqlx::types::Type<R::Database>,
    Json<Option<Either<String, VisslContent>>>: ::sqlx::decode::Decode<'a, R::Database>,
    Json<Option<Either<String, VisslContent>>>: ::sqlx::types::Type<R::Database>,
    WebsitePageType: ::sqlx::decode::Decode<'a, R::Database>,
    WebsitePageType: ::sqlx::types::Type<R::Database>,
    Json<WebsitePageSettings>: ::sqlx::decode::Decode<'a, R::Database>,
    Json<WebsitePageSettings>: ::sqlx::types::Type<R::Database>,
{
    fn from_row(row: &'a R) -> std::result::Result<Self, sqlx::Error> {
        let pk = row.try_get("pk")?;
        let id = row.try_get("id")?;
        let addon_id = row.try_get("addon_id")?;
        let compiled_id = row.try_get("compiled_id")?;
        let script = row.try_get("script")?;
        let hash = row.try_get("hash")?;
        let version = row.try_get("version")?;
        let type_of = row.try_get("type_of")?;
        let path = row.try_get("path")?;
        let display_name = row.try_get("display_name")?;
        let settings = row.try_get("settings")?;

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
            script,
            version,
            type_of,
            path,
            display_name,
            compiled_id,
            hash,
            settings,
        })
    }
}
