use common::{api::AddonPublic, AddonId, DeveloperId, MediaId};
use eyre::Result;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::Serialize;
use sqlx::{FromRow, SqliteConnection};
use time::OffsetDateTime;
use uuid::Uuid;

pub struct NewAddonModel {
    pub developer_id: DeveloperId,

    pub name: String,
    pub tag_line: String,
    pub description: String,
    pub icon: Option<MediaId>,
    pub version: String,
}

#[derive(FromRow, Serialize)]
pub struct AddonModel {
    pub id: AddonId,

    pub developer_id: DeveloperId,
    pub guid: Uuid,
    // TODO: Secret Key
    // TODO: App URL Redirect After Install (w/ auth code)
    // TODO: App URL Redirect After Authorization (w/ temp auth code)
    pub name: String,
    pub tag_line: String,
    pub description: String,
    pub icon: Option<MediaId>,
    pub version: String,

    pub is_visible: bool,
    pub is_accepted: bool,

    pub install_count: i32,

    pub delete_reason: Option<String>,

    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub deleted_at: Option<OffsetDateTime>,
}

impl NewAddonModel {
    pub async fn insert(self, db: &mut SqliteConnection) -> Result<AddonModel> {
        let now = OffsetDateTime::now_utc();
        let guid = Uuid::now_v7();

        let resp = sqlx::query(
            "INSERT INTO addon (developer_id, guid, name, tag_line, description, icon, version, is_visible, is_accepted, install_count, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8, $9, $10, $10)",
        )
        .bind(self.developer_id)
        .bind(&guid)
        .bind(&self.name)
        .bind(&self.tag_line)
        .bind(&self.description)
        .bind(&self.icon)
        .bind(&self.version)
        .bind(false)
        .bind(0)
        .bind(now)
        .execute(db)
        .await?;

        Ok(AddonModel {
            id: AddonId::from(resp.last_insert_rowid() as i32),
            developer_id: self.developer_id,
            guid,
            name: self.name,
            tag_line: self.tag_line,
            description: self.description,
            icon: self.icon,
            version: self.version,
            is_accepted: false,
            is_visible: false,
            install_count: 0,
            delete_reason: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        })
    }
}

impl AddonModel {
    pub async fn find_one_by_guid(guid: Uuid, db: &mut SqliteConnection) -> Result<Option<Self>> {
        Ok(sqlx::query_as(
            "SELECT id, developer_id, guid, name, tag_line, description, icon, version, is_visible, is_accepted, install_count, delete_reason, created_at, updated_at, deleted_at FROM addon WHERE guid = $1"
        )
        .bind(guid)
        .fetch_optional(db)
        .await?)
    }

    pub async fn find_all(db: &mut SqliteConnection) -> Result<Vec<Self>> {
        Ok(sqlx::query_as(
            "SELECT id, developer_id, guid, name, tag_line, description, icon, version, is_visible, is_accepted, install_count, delete_reason, created_at, updated_at, deleted_at FROM addon"
        )
        .fetch_all(db)
        .await?)
    }

    pub async fn delete(id: AddonId, reason: String, db: &mut SqliteConnection) -> Result<u64> {
        let res = sqlx::query("UPDATE addon SET deleted_at = $2, delete_reason = $3 WHERE id = $1")
            .bind(id)
            .bind(OffsetDateTime::now_utc())
            .bind(reason)
            .execute(db)
            .await?;

        Ok(res.rows_affected())
    }

    pub fn into_public(
        self,
        developer_uuid: Uuid,
        icon: Option<String>,
        gallery: Option<Vec<String>>,
    ) -> AddonPublic {
        AddonPublic {
            developer_uuid,
            guid: self.guid,
            name: self.name,
            tag_line: self.tag_line,
            description: self.description,
            icon,
            gallery,
            version: self.version,
            is_visible: self.is_visible,
            is_accepted: self.is_accepted,
            install_count: self.install_count,
            delete_reason: self.delete_reason,
            created_at: self.created_at,
            updated_at: self.updated_at,
            deleted_at: self.deleted_at,
        }
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum AddonType {
    /// Built using the in-house code editor
    Admin = 0,
    /// Built using the in-house code editor
    Native = 1,
    /// Built using your own backend that you host
    SelfHost = 2,
    /// Built using Node.js which is hosted on our servers
    NodeJS = 3,
}

impl FromRow<'_, ::sqlx::sqlite::SqliteRow> for AddonType {
    fn from_row(row: &::sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        use ::sqlx::Row;

        Ok(Self::try_from(row.try_get::<i32, _>(0)? as u8).unwrap())
    }
}

impl ::sqlx::Encode<'_, ::sqlx::sqlite::Sqlite> for AddonType {
    fn encode_by_ref(
        &self,
        buf: &mut <::sqlx::sqlite::Sqlite as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> std::result::Result<::sqlx::encode::IsNull, ::sqlx::error::BoxDynError> {
        ::sqlx::Encode::<::sqlx::sqlite::Sqlite>::encode_by_ref(&(*self as u8 as i32), buf)
    }
}

impl ::sqlx::Decode<'_, ::sqlx::sqlite::Sqlite> for AddonType {
    fn decode(
        value: <::sqlx::sqlite::Sqlite as sqlx::Database>::ValueRef<'_>,
    ) -> std::result::Result<Self, ::sqlx::error::BoxDynError> {
        Ok(Self::try_from(
            <i32 as ::sqlx::Decode<::sqlx::sqlite::Sqlite>>::decode(value)? as u8,
        )?)
    }
}

impl ::sqlx::Type<::sqlx::sqlite::Sqlite> for AddonType {
    fn type_info() -> ::sqlx::sqlite::SqliteTypeInfo {
        <i32 as ::sqlx::Type<::sqlx::sqlite::Sqlite>>::type_info()
    }
}
