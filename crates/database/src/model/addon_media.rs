use common::{AddonId, AddonMediaId, MediaId};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use sqlx::{
    encode::IsNull,
    error::BoxDynError,
    sqlite::{SqliteRow, SqliteTypeInfo},
    Decode, Encode, FromRow, Result, Row, Sqlite, SqliteConnection, Type,
};
use time::OffsetDateTime;

#[derive(Debug, Clone, Copy, serde::Serialize, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum AddonMediaType {
    Upload = 0,
    Embed = 1,
}

pub enum NewAddonMediaModel {
    Upload {
        addon_id: AddonId,
        upload_id: MediaId,
    },
    Embed {
        addon_id: AddonId,
        embed_url: String,
    },
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct AddonMediaModel {
    pub id: AddonMediaId,

    pub addon_id: AddonId,
    pub type_of: AddonMediaType,

    pub upload_id: Option<MediaId>,
    pub embed_url: Option<String>,

    pub idx: i32,

    pub created_at: OffsetDateTime,
    pub deleted_at: Option<OffsetDateTime>,
}

impl NewAddonMediaModel {
    pub async fn insert(self, db: &mut SqliteConnection) -> Result<AddonMediaModel> {
        let now = OffsetDateTime::now_utc();

        let type_of = match &self {
            NewAddonMediaModel::Upload { .. } => AddonMediaType::Upload,
            NewAddonMediaModel::Embed { .. } => AddonMediaType::Embed,
        };

        let (addon_id, upload_id, embed_url) = match self {
            NewAddonMediaModel::Upload {
                addon_id,
                upload_id,
            } => (addon_id, Some(upload_id), None),
            NewAddonMediaModel::Embed {
                addon_id,
                embed_url,
            } => (addon_id, None, Some(embed_url)),
        };

        let res = sqlx::query(
            r#"
                INSERT INTO addon_media (addon_id, type_of, upload_id, embed_url, created_at)
                VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(addon_id)
        .bind(type_of)
        .bind(&upload_id)
        .bind(&embed_url)
        .bind(now)
        .execute(db)
        .await?;

        Ok(AddonMediaModel {
            id: AddonMediaId::from(res.last_insert_rowid()),
            addon_id,
            type_of,
            upload_id,
            embed_url,
            idx: -1,
            created_at: now,
            deleted_at: None,
        })
    }
}

impl AddonMediaModel {
    pub async fn find_by_addon(addon_id: AddonId, db: &mut SqliteConnection) -> Result<Vec<Self>> {
        Ok(sqlx::query_as(
            "SELECT id, addon_id, type_of, upload_id, embed_url, idx, created_at, deleted_at FROM media_upload WHERE addon_id = $1",
        )
        .bind(addon_id)
        .fetch_all(db)
        .await?)
    }
}

impl FromRow<'_, SqliteRow> for AddonMediaType {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Self::try_from(row.try_get::<i32, _>(0)? as u8).unwrap())
    }
}

impl Encode<'_, Sqlite> for AddonMediaType {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> std::result::Result<IsNull, BoxDynError> {
        Encode::<Sqlite>::encode_by_ref(&(*self as u8 as i32), buf)
    }
}

impl Decode<'_, Sqlite> for AddonMediaType {
    fn decode(
        value: <Sqlite as sqlx::Database>::ValueRef<'_>,
    ) -> std::result::Result<Self, BoxDynError> {
        Ok(Self::try_from(
            <i32 as Decode<Sqlite>>::decode(value)? as u8
        )?)
    }
}

impl Type<Sqlite> for AddonMediaType {
    fn type_info() -> SqliteTypeInfo {
        <i32 as Type<Sqlite>>::type_info()
    }
}
