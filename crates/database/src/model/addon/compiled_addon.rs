use eyre::Result;
use global_common::id::AddonCompiledPublicId;
use local_common::{AddonCompiledId, AddonId};
use serde::Serialize;
use sqlx::{
    database::{HasArguments, HasValueRef},
    encode::IsNull,
    error::BoxDynError,
    sqlite::SqliteTypeInfo,
    types::Json,
    Decode, Encode, FromRow, Sqlite, SqliteConnection, Type,
};
use storage::widget::CompiledAddonSettings;
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewAddonCompiledModel {
    pub addon_id: AddonId,

    pub settings: CompiledAddonSettings,

    pub type_of: AddonPublishType,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AddonCompiledModel {
    pub pk: AddonCompiledId,
    pub id: AddonCompiledPublicId,

    pub addon_id: AddonId,

    pub settings: Json<CompiledAddonSettings>,

    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub type_of: AddonPublishType,
    pub version: String,

    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub deleted_at: Option<OffsetDateTime>,
}

impl NewAddonCompiledModel {
    pub async fn insert(self, db: &mut SqliteConnection) -> Result<AddonCompiledModel> {
        let id = AddonCompiledPublicId::new();
        let now = OffsetDateTime::now_utc();
        let settings = Json(self.settings);

        let res = sqlx::query(
            "INSERT INTO addon_compiled (id, addon_id, settings, type, version, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $6)",
        )
        .bind(id)
        .bind(self.addon_id)
        .bind(&settings)
        .bind(self.type_of)
        .bind(&self.version)
        .bind(now)
        .execute(db)
        .await?;

        Ok(AddonCompiledModel {
            pk: AddonCompiledId::from(res.last_insert_rowid() as i32),
            id,

            addon_id: self.addon_id,
            settings,

            type_of: self.type_of,
            version: self.version,

            created_at: now,
            updated_at: now,
            deleted_at: None,
        })
    }
}

impl AddonCompiledModel {
    pub async fn find_one_by_public_id(
        id: AddonCompiledPublicId,
        db: &mut SqliteConnection,
    ) -> Result<Option<Self>> {
        Ok(
            sqlx::query_as(
                "SELECT pk, id, addon_id, settings, type, version, created_at, updated_at, deleted_at FROM addon_compiled WHERE id = $1",
            )
            .bind(id)
            .fetch_optional(db)
            .await?,
        )
    }

    pub async fn find_one_by_addon_uuid_and_version(
        uuid: AddonId,
        version: &str,
        db: &mut SqliteConnection,
    ) -> Result<Option<Self>> {
        Ok(
            sqlx::query_as(
                "SELECT pk, id, addon_id, settings, type, version, created_at, updated_at, deleted_at FROM addon_compiled WHERE addon_id = $1 AND version = $2",
            )
            .bind(uuid)
            .bind(version)
            .fetch_optional(db)
            .await?,
        )
    }

    pub async fn get_all(
        uuid: AddonId,
        offset: usize,
        limit: usize,
        db: &mut SqliteConnection,
    ) -> Result<Vec<Self>> {
        Ok(
            sqlx::query_as(
                "SELECT pk, id, addon_id, settings, type, version, created_at, updated_at, deleted_at FROM addon_compiled WHERE addon_id = $1 ORDER BY created_at DESC OFFSET $2 LIMIT $3",
            )
            .bind(uuid)
            .bind((offset * limit) as i64)
            .bind(limit as i64)
            .fetch_all(db)
            .await?,
        )
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AddonPublishType {
    Draft,
    Published,
}

impl Encode<'_, Sqlite> for AddonPublishType {
    fn encode_by_ref(&self, buf: &mut <Sqlite as HasArguments<'_>>::ArgumentBuffer) -> IsNull {
        Encode::<Sqlite>::encode_by_ref(
            &String::from(match self {
                Self::Draft => "draft",
                Self::Published => "publish",
            }),
            buf,
        )
    }
}

impl Decode<'_, Sqlite> for AddonPublishType {
    fn decode(value: <Sqlite as HasValueRef<'_>>::ValueRef) -> Result<Self, BoxDynError> {
        Ok(match <String as Decode<Sqlite>>::decode(value)?.as_str() {
            "draft" => Self::Draft,
            "publish" => Self::Published,
            _ => unreachable!(),
        })
    }
}

impl Type<Sqlite> for AddonPublishType {
    fn type_info() -> SqliteTypeInfo {
        <String as Type<Sqlite>>::type_info()
    }
}
