use common::{DeveloperId, MediaId};
use eyre::Result;
use sqlx::{FromRow, SqliteConnection};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug)]
pub struct NewMediaUploadModel {
    pub uploader_id: DeveloperId,
    pub member_uuid: Uuid,

    pub file_name: String,
    pub file_size: i64,
    pub file_type: String,

    pub media_width: Option<i32>,
    pub media_height: Option<i32>,
    pub media_duration: Option<i32>,

    pub has_thumbnail: bool,

    pub store_path: String,

    pub hash: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct MediaUploadModel {
    pub id: MediaId,

    pub uploader_id: DeveloperId,
    pub member_uuid: Uuid,

    pub file_name: String,
    pub file_size: i64,
    pub file_type: String,

    pub media_width: Option<i32>,
    pub media_height: Option<i32>,
    pub media_duration: Option<i32>,

    pub has_thumbnail: bool,

    pub store_path: String,

    pub hash: Option<String>,

    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub deleted_at: Option<OffsetDateTime>,
}

impl NewMediaUploadModel {
    pub fn pending(uploader_id: DeveloperId, member_uuid: Uuid, store_path: String) -> Self {
        Self {
            uploader_id,
            member_uuid,
            file_name: String::new(),
            file_size: 0,
            file_type: String::new(),
            media_width: None,
            media_height: None,
            media_duration: None,
            has_thumbnail: false,
            store_path,
            hash: None,
        }
    }

    pub async fn insert(mut self, db: &mut SqliteConnection) -> Result<MediaUploadModel> {
        let now = OffsetDateTime::now_utc();

        self.file_name.truncate(64);
        self.file_type.truncate(32);

        let res = sqlx::query(
            r#"
                INSERT INTO media_upload (uploader_id, member_uuid, file_name, file_size, file_type, media_width, media_height, media_duration, has_thumbnail, store_path, hash, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $12)"#,
        )
        .bind(self.uploader_id)
        .bind(self.member_uuid)
        .bind(&self.file_name)
        .bind(self.file_size)
        .bind(&self.file_type)
        .bind(self.media_width)
        .bind(self.media_height)
        .bind(self.media_duration)
        .bind(self.has_thumbnail)
        .bind(&self.store_path)
        .bind(&self.hash)
        .bind(now)
        .execute(db)
        .await?;

        Ok(self.into_model(MediaId::from(res.last_insert_rowid()), now))
    }

    fn into_model(self, id: MediaId, now: OffsetDateTime) -> MediaUploadModel {
        MediaUploadModel {
            id,
            uploader_id: self.uploader_id,
            member_uuid: self.member_uuid,
            file_name: self.file_name,
            file_size: self.file_size,
            file_type: self.file_type,
            media_width: self.media_width,
            media_height: self.media_height,
            media_duration: self.media_duration,
            has_thumbnail: self.has_thumbnail,
            store_path: self.store_path,
            hash: self.hash,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }
}

impl MediaUploadModel {
    pub async fn update(&mut self, db: &mut SqliteConnection) -> Result<u64> {
        self.updated_at = OffsetDateTime::now_utc();

        let res = sqlx::query(
            "UPDATE media_upload SET file_name = $2, file_size = $3, file_type = $4, media_width = $5, media_height = $6, media_duration = $7, has_thumbnail = $8, hash = $9 WHERE id = $1",
        )
        .bind(self.id)
        .bind(&self.file_name)
        .bind(&self.file_size)
        .bind(&self.file_type)
        .bind(&self.media_width)
        .bind(&self.media_height)
        .bind(&self.media_duration)
        .bind(&self.has_thumbnail)
        .bind(&self.hash)
        .execute(db)
        .await?;

        Ok(res.rows_affected())
    }

    pub async fn find_one_by_id(id: MediaId, db: &mut SqliteConnection) -> Result<Option<Self>> {
        Ok(sqlx::query_as(
            r#"SELECT id, uploader_id, member_uuid, file_name, file_size, file_type, media_width, media_height, media_duration, has_thumbnail, store_path, hash, created_at, updated_at, deleted_at
                FROM media_upload WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(db)
        .await?)
    }

    pub async fn find_one_by_public_id(
        id: &str,
        db: &mut SqliteConnection,
    ) -> Result<Option<Self>> {
        Ok(sqlx::query_as(
            r#"SELECT id, uploader_id, member_uuid, file_name, file_size, file_type, media_width, media_height, media_duration, has_thumbnail, store_path, hash, created_at, updated_at, deleted_at
                FROM media_upload WHERE store_path = $1"#,
        )
        .bind(id)
        .fetch_optional(db)
        .await?)
    }

    pub async fn find_by_ids(ids: Vec<MediaId>, db: &mut SqliteConnection) -> Result<Vec<Self>> {
        // TODO: Better way?
        Ok(sqlx::query_as(
            &format!(
                r#"SELECT id, uploader_id, member_uuid, file_name, file_size, file_type, media_width, media_height, media_duration, has_thumbnail, store_path, hash, created_at, updated_at, deleted_at
                FROM media_upload WHERE id IN ({})"#,
                ids.into_iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",")
            ),
        )
        .fetch_all(db)
        .await?)
    }
}
