// Requests Installer to give access to specific permissions
// E.g:
// - Read/Write Data Items
// - Manage Data Indexes
// - Manage Data (whole) Collection
// - Access & Manage External Database Connections
// | Access Official Apps (optional/required)

// Examples:
// member.info.read
// member.info.read.email
// member.info.read.name
// member.info.read.avatar

use local_common::{AddonId, AddonPermission};
use sqlx::{Result, SqliteConnection};

pub struct AddonPermissionModel {
    pub addon_id: AddonId,

    pub perm: AddonPermission,
}

impl AddonPermissionModel {
    // pub fn try_from(addon_id: AddonId, permission: String) {}

    pub async fn insert(&self, db: &mut SqliteConnection) -> Result<()> {
        sqlx::query(
            "INSERT INTO addon_permission (addon_id, scope, category, operation, info) VALUES ($1, $2, $3, $4, $5)",
        )
            .bind(self.addon_id)
            .bind(&self.perm.scope)
            .bind(&self.perm.category)
            .bind(&self.perm.operation)
            .bind(self.perm.info.as_ref())
            .execute(db)
            .await?;

        Ok(())
    }

    pub async fn find_by_addon_id(id: AddonId, db: &mut SqliteConnection) -> Result<Vec<Self>> {
        Ok(sqlx::query_as(
            "SELECT addon_id, scope, category, operation, info FROM addon_permission WHERE addon_id = $1",
        )
        .bind(id)
        .fetch_all(db)
        .await?)
    }

    pub async fn find_by_scope_addon_id(
        id: AddonId,
        scope: &str,
        db: &mut SqliteConnection,
    ) -> Result<Vec<Self>> {
        Ok(sqlx::query_as(
            "SELECT addon_id, scope, category, operation, info FROM addon_permission WHERE addon_id = $1 AND scope = $2",
        )
        .bind(id)
        .bind(scope)
        .fetch_all(db)
        .await?)
    }

    pub async fn delete_by_addon_id(id: AddonId, db: &mut SqliteConnection) -> Result<u64> {
        let res = sqlx::query("DELETE FROM addon_permission WHERE addon_id = $1")
            .bind(id)
            .execute(db)
            .await?;

        Ok(res.rows_affected())
    }
}

impl<'a, R: ::sqlx::Row> ::sqlx::FromRow<'a, R> for AddonPermissionModel
where
    &'a ::std::primitive::str: ::sqlx::ColumnIndex<R>,
    AddonId: ::sqlx::decode::Decode<'a, R::Database>,
    AddonId: ::sqlx::types::Type<R::Database>,
    String: ::sqlx::decode::Decode<'a, R::Database>,
    String: ::sqlx::types::Type<R::Database>,
    String: ::sqlx::decode::Decode<'a, R::Database>,
    String: ::sqlx::types::Type<R::Database>,
    Option<String>: ::sqlx::decode::Decode<'a, R::Database>,
    Option<String>: ::sqlx::types::Type<R::Database>,
    Option<String>: ::sqlx::decode::Decode<'a, R::Database>,
    Option<String>: ::sqlx::types::Type<R::Database>,
{
    fn from_row(row: &'a R) -> ::sqlx::Result<Self> {
        let addon_id: AddonId = row.try_get("addon_id")?;
        let scope: String = row.try_get("scope")?;
        let category: String = row.try_get("category")?;
        let operation: Option<String> = row.try_get("operation")?;
        let info: Option<String> = row.try_get("info")?;

        ::std::result::Result::Ok(AddonPermissionModel {
            addon_id,
            perm: AddonPermission {
                scope,
                category,
                operation,
                info,
            },
        })
    }
}
