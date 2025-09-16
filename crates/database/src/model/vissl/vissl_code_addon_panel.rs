use eyre::Result;
use webby_global_common::Either;
use local_common::{AddonId, AddonWidgetId, AddonWidgetPanelId, VisslAddonPanelCodeId};
use webby_scripting::json::VisslContent;
use serde::Serialize;
use sqlx::{types::Json, SqliteConnection};
use time::OffsetDateTime;

pub enum NewVisslCodeAddonPanelModel {
    Visual {
        addon_id: AddonId,
        widget_id: Option<AddonWidgetId>,
        widget_panel_id: Option<AddonWidgetPanelId>,

        visual_data: VisslContent,
    },
    Scripting {
        addon_id: AddonId,
        widget_id: Option<AddonWidgetId>,
        widget_panel_id: Option<AddonWidgetPanelId>,

        script_data: String,
    },
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum VisslCodeAddonPanelModel {
    Visual {
        pk: VisslAddonPanelCodeId,

        addon_id: AddonId,
        widget_id: Option<AddonWidgetId>,
        widget_panel_id: Option<AddonWidgetPanelId>,

        visual_data: Json<VisslContent>,

        created_at: OffsetDateTime,
        updated_at: OffsetDateTime,
    },
    Scripting {
        pk: VisslAddonPanelCodeId,

        addon_id: AddonId,
        widget_id: Option<AddonWidgetId>,
        widget_panel_id: Option<AddonWidgetPanelId>,

        script_data: String,

        created_at: OffsetDateTime,
        updated_at: OffsetDateTime,
    },
}

impl NewVisslCodeAddonPanelModel {
    pub async fn insert(self, db: &mut SqliteConnection) -> Result<VisslCodeAddonPanelModel> {
        let now = OffsetDateTime::now_utc();

        match self {
            Self::Visual {
                addon_id,
                widget_id,
                widget_panel_id,
                visual_data,
            } => {
                let visual_data = Json(visual_data);

                let resp = sqlx::query(
                    "INSERT INTO vissl_code_addon_panel (addon_id, widget_id, widget_panel_id, visual_data, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $5)",
                )
                .bind(addon_id)
                .bind(widget_id)
                .bind(widget_panel_id)
                .bind(&visual_data)
                .bind(now)
                .execute(db)
                .await?;

                Ok(VisslCodeAddonPanelModel::Visual {
                    pk: VisslAddonPanelCodeId::from(resp.last_insert_rowid() as i32),
                    addon_id,
                    widget_id,
                    widget_panel_id,
                    visual_data,
                    created_at: now,
                    updated_at: now,
                })
            }

            Self::Scripting {
                addon_id,
                widget_id,
                widget_panel_id,
                script_data,
            } => {
                let resp = sqlx::query(
                    "INSERT INTO vissl_code_addon_panel (addon_id, widget_id, widget_panel_id, script_data, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $5)",
                )
                .bind(addon_id)
                .bind(widget_id)
                .bind(widget_panel_id)
                .bind(&script_data)
                .bind(now)
                .execute(db)
                .await?;

                Ok(VisslCodeAddonPanelModel::Scripting {
                    pk: VisslAddonPanelCodeId::from(resp.last_insert_rowid() as i32),
                    addon_id,
                    widget_id,
                    widget_panel_id,
                    script_data,
                    created_at: now,
                    updated_at: now,
                })
            }
        }
    }
}

impl VisslCodeAddonPanelModel {
    pub fn pk(&self) -> VisslAddonPanelCodeId {
        match self {
            Self::Visual { pk, .. } => *pk,
            Self::Scripting { pk, .. } => *pk,
        }
    }

    pub fn take_data(self) -> Either<VisslContent, String> {
        match self {
            Self::Visual { visual_data, .. } => Either::Left(visual_data.0),
            Self::Scripting { script_data, .. } => Either::Right(script_data),
        }
    }

    pub async fn update(&mut self, db: &mut SqliteConnection) -> Result<u64> {
        match self {
            Self::Visual {
                pk,
                addon_id,
                widget_id,
                visual_data,
                updated_at,
                ..
            } => {
                *updated_at = OffsetDateTime::now_utc();

                let res = sqlx::query(
                    r#"UPDATE vissl_code_addon_panel SET
                        addon_id = $2,
                        widget_id = $3,
                        visual_data = $4,
                        updated_at = $5
                    WHERE pk = $1"#,
                )
                .bind(*pk)
                .bind(*addon_id)
                .bind(*widget_id)
                .bind(&*visual_data)
                .bind(*updated_at)
                .execute(db)
                .await?;

                Ok(res.rows_affected())
            }

            Self::Scripting {
                pk,
                addon_id,
                widget_id,
                script_data,
                updated_at,
                ..
            } => {
                *updated_at = OffsetDateTime::now_utc();

                let res = sqlx::query(
                    r#"UPDATE vissl_code_addon_panel SET
                        addon_id = $2,
                        widget_id = $3,
                        script_data = $4,
                        updated_at = $5
                    WHERE pk = $1"#,
                )
                .bind(*pk)
                .bind(*addon_id)
                .bind(*widget_id)
                .bind(&*script_data)
                .bind(*updated_at)
                .execute(db)
                .await?;

                Ok(res.rows_affected())
            }
        }
    }

    pub async fn delete_by_id(pk: VisslAddonPanelCodeId, db: &mut SqliteConnection) -> Result<u64> {
        let res = sqlx::query("DELETE FROM vissl_code_addon_panel WHERE pk = $1")
            .bind(pk)
            .execute(db)
            .await?;

        Ok(res.rows_affected())
    }

    pub async fn find_by_id(
        pk: VisslAddonPanelCodeId,
        db: &mut SqliteConnection,
    ) -> Result<Option<Self>> {
        Ok(
            sqlx::query_as("SELECT pk, addon_id, widget_id, widget_panel_id, visual_data, script_data, created_at, updated_at FROM vissl_code_addon_panel WHERE pk = $1")
                .bind(pk)
                .fetch_optional(db)
                .await?,
        )
    }

    pub async fn find_all_by_addon_id(
        widget_id: AddonWidgetId,
        db: &mut SqliteConnection,
    ) -> Result<Vec<Self>> {
        Ok(
            sqlx::query_as("SELECT pk, addon_id, widget_id, widget_panel_id, visual_data, script_data, created_at, updated_at FROM vissl_code_addon_panel WHERE widget_id = $1")
                .bind(widget_id)
                .fetch_all(db)
                .await?,
        )
    }

    pub async fn find_one_addon_widget(
        addon_id: AddonId,
        panel_id: Option<AddonWidgetPanelId>,
        db: &mut SqliteConnection,
    ) -> Result<Option<Self>> {
        Ok(
            sqlx::query_as("SELECT pk, addon_id, widget_id, widget_panel_id, visual_data, script_data, created_at, updated_at FROM vissl_code_addon_panel WHERE addon_id = $1 AND widget_panel_id = $2")
                .bind(addon_id)
                .bind(panel_id)
                .fetch_optional(db)
                .await?,
        )
    }
}

impl<'a, R: ::sqlx::Row> ::sqlx::FromRow<'a, R> for VisslCodeAddonPanelModel
where
    &'a ::std::primitive::str: ::sqlx::ColumnIndex<R>,
    VisslAddonPanelCodeId: ::sqlx::decode::Decode<'a, R::Database>,
    VisslAddonPanelCodeId: ::sqlx::types::Type<R::Database>,
    AddonId: ::sqlx::decode::Decode<'a, R::Database>,
    AddonId: ::sqlx::types::Type<R::Database>,
    Option<AddonWidgetId>: ::sqlx::decode::Decode<'a, R::Database>,
    Option<AddonWidgetId>: ::sqlx::types::Type<R::Database>,
    Option<AddonWidgetPanelId>: ::sqlx::decode::Decode<'a, R::Database>,
    Option<AddonWidgetPanelId>: ::sqlx::types::Type<R::Database>,
    Json<VisslContent>: ::sqlx::decode::Decode<'a, R::Database>,
    Json<VisslContent>: ::sqlx::types::Type<R::Database>,
    OffsetDateTime: ::sqlx::decode::Decode<'a, R::Database>,
    OffsetDateTime: ::sqlx::types::Type<R::Database>,
    OffsetDateTime: ::sqlx::decode::Decode<'a, R::Database>,
    OffsetDateTime: ::sqlx::types::Type<R::Database>,
    String: ::sqlx::decode::Decode<'a, R::Database>,
    String: ::sqlx::types::Type<R::Database>,
{
    fn from_row(row: &'a R) -> ::sqlx::Result<Self> {
        if let Ok(visual_data) = row.try_get("visual_data") {
            let pk = row.try_get("pk")?;
            let addon_id = row.try_get("addon_id")?;
            let widget_id = row.try_get("widget_id")?;
            let widget_panel_id = row.try_get("widget_panel_id")?;
            let created_at = row.try_get("created_at")?;
            let updated_at = row.try_get("updated_at")?;

            Ok(Self::Visual {
                pk,
                addon_id,
                widget_id,
                widget_panel_id,
                visual_data,
                created_at,
                updated_at,
            })
        } else {
            let pk = row.try_get("pk")?;
            let addon_id = row.try_get("addon_id")?;
            let widget_id = row.try_get("widget_id")?;
            let widget_panel_id = row.try_get("widget_panel_id")?;
            let created_at = row.try_get("created_at")?;
            let updated_at = row.try_get("updated_at")?;
            let script_data = row.try_get("script_data")?;

            Ok(Self::Scripting {
                pk,
                addon_id,
                widget_id,
                widget_panel_id,
                script_data,
                created_at,
                updated_at,
            })
        }
    }
}
