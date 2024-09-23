use eyre::Result;
use local_common::global::{SchemaFieldMap, SchemaView, SchematicPermissions};
use local_common::{AddonId, SchemaId};
use serde::{Deserialize, Serialize};
use sqlx::{types::Json, FromRow, SqliteConnection};
use time::OffsetDateTime;

pub struct NewSchemaModel {
    pub name: String,

    pub addon_id: AddonId,

    pub primary_field: String,
    pub display_name: String,

    pub permissions: SchematicPermissions,

    pub version: f64,

    pub allowed_operations: Vec<String>,

    pub ttl: Option<i32>,
    pub default_sort: Option<String>,
    pub views: Vec<SchemaView>,

    /// addon/local
    pub store: String,

    pub fields: SchemaFieldMap,
}

impl NewSchemaModel {
    pub fn into_form(
        self,
        id: SchemaId,
        fields: SchemaFieldMap,
        created_at: OffsetDateTime,
    ) -> SchemaModel {
        SchemaModel {
            id,

            name: self.name,
            addon_id: self.addon_id,

            primary_field: self.primary_field,
            display_name: self.display_name,

            permissions: Json(self.permissions),

            version: self.version,

            allowed_operations: Json(self.allowed_operations),

            fields: Json(fields),

            ttl: self.ttl,
            default_sort: self.default_sort,
            views: Json(self.views),
            store: self.store,

            created_at,
            updated_at: created_at,
            deleted_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SchemaModel {
    pub id: SchemaId,

    pub name: String,

    pub addon_id: AddonId,

    pub primary_field: String,
    pub display_name: String,

    pub permissions: Json<SchematicPermissions>,

    pub version: f64,

    pub allowed_operations: Json<Vec<String>>,

    pub ttl: Option<i32>,
    pub default_sort: Option<String>,
    pub views: Json<Vec<SchemaView>>,

    /// addon/local
    pub store: String,

    pub fields: Json<SchemaFieldMap>,

    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub deleted_at: Option<OffsetDateTime>,
}

impl NewSchemaModel {
    pub async fn insert(
        self,
        fields: SchemaFieldMap,
        db: &mut SqliteConnection,
    ) -> Result<SchemaModel> {
        let now = OffsetDateTime::now_utc();

        let res = sqlx::query(
            r#"INSERT INTO schema (name, addon_id, primary_field, display_name, permissions, version, allowed_operations, ttl, default_sort, views, fields, store, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $13)"#,
        )
        .bind(&self.name)
        .bind(&self.addon_id)
        .bind(&self.primary_field)
        .bind(&self.display_name)
        .bind(Json(&self.permissions))
        .bind(&self.version)
        .bind(Json(&self.allowed_operations))
        .bind(&self.ttl)
        .bind(&self.default_sort)
        .bind(Json(&self.views))
        .bind(Json(&self.fields))
        .bind(&self.store)
        .bind(now)
        .execute(db)
        .await?;

        Ok(self.into_form(SchemaId::from(res.last_insert_rowid() as i32), fields, now))
    }
}

impl SchemaModel {
    pub async fn update(&mut self, db: &mut SqliteConnection) -> Result<u64> {
        self.updated_at = OffsetDateTime::now_utc();

        let res = sqlx::query(
            r#"UPDATE schema SET
                name = $2,
                addon_id = $3,
                fields = $4,
                primary_field = $5,
                display_name = $6,
                permissions = $7,
                version = $8,
                allowed_operations = $9,
                ttl = $10,
                default_sort = $11,
                views = $12,
                store = $13,
                updated_at = $14
            WHERE id = $1"#,
        )
        .bind(self.id)
        .bind(&self.name)
        .bind(&self.addon_id)
        .bind(&self.fields)
        .bind(&self.primary_field)
        .bind(&self.display_name)
        .bind(&self.permissions)
        .bind(&self.version)
        .bind(&self.allowed_operations)
        .bind(&self.ttl)
        .bind(&self.default_sort)
        .bind(&self.views)
        .bind(&self.store)
        .bind(self.updated_at)
        .execute(db)
        .await?;

        Ok(res.rows_affected())
    }

    pub async fn delete(id: SchemaId, db: &mut SqliteConnection) -> Result<u64> {
        let res = sqlx::query("UPDATE schema SET deleted_at = $2 WHERE id = $1")
            .bind(id)
            .bind(OffsetDateTime::now_utc())
            .execute(db)
            .await?;

        Ok(res.rows_affected())
    }

    pub async fn get_all(addon_id: AddonId, db: &mut SqliteConnection) -> Result<Vec<Self>> {
        Ok(sqlx::query_as(
            "SELECT id, name, addon_id, primary_field, display_name, permissions, version, allowed_operations, ttl, default_sort, views, store, fields, created_at, updated_at, deleted_at FROM schema WHERE addon_id = $1",
        )
        .bind(addon_id)
        .fetch_all(db)
        .await?)
    }

    pub async fn count(addon_id: AddonId, db: &mut SqliteConnection) -> Result<i32> {
        Ok(
            sqlx::query_scalar("SELECT COUNT(*) FROM schema WHERE addon_id = $1")
                .bind(addon_id)
                .fetch_one(db)
                .await?,
        )
    }

    pub async fn find_one_by_id(id: SchemaId, db: &mut SqliteConnection) -> Result<Option<Self>> {
        Ok(sqlx::query_as(
            "SELECT id, name, addon_id, primary_field, display_name, permissions, version, allowed_operations, ttl, default_sort, views, store, fields, created_at, updated_at, deleted_at FROM schema WHERE schema.id = $1",
        )
        .bind(id)
        .fetch_optional(db)
        .await?)
    }

    pub async fn find_one_by_public_id(
        addon_id: AddonId,
        name: &str,
        db: &mut SqliteConnection,
    ) -> Result<Option<Self>> {
        Ok(sqlx::query_as(
            "SELECT id, name, addon_id, primary_field, display_name, permissions, version, allowed_operations, ttl, default_sort, views, store, fields, created_at, updated_at, deleted_at FROM schema WHERE addon_id = $1 AND schema.name = $2",
        )
        .bind(addon_id)
        .bind(name)
        .fetch_optional(db)
        .await?)
    }

    pub async fn find_by_addon_id(
        addon_id: AddonId,
        db: &mut SqliteConnection,
    ) -> Result<Vec<Self>> {
        Ok(sqlx::query_as(
            "SELECT id, name, addon_id, primary_field, display_name, permissions, version, allowed_operations, ttl, default_sort, views, store, fields, created_at, updated_at, deleted_at FROM schema WHERE addon_id = $1",
        )
        .bind(addon_id)
        .fetch_all(db)
        .await?)
    }
}
