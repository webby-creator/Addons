use webby_addon_common::{JsonResponse, WrappingResponse};
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use database::{
    AddonCompiledModel, AddonDashboardPage, AddonModel, AddonTemplatePageContentModel,
    AddonTemplatePageModel, AddonWidgetContent, NewAddonTemplatePageModel, SchemaModel,
};
use local_common::DashboardPageInfo;
use serde::Deserialize;
use sqlx::{Connection, SqlitePool};
use webby_storage::DisplayStore;
use uuid::Uuid;

use crate::Result;

pub fn routes() -> Router<SqlitePool> {
    Router::new()
        .route("/:addon", get(get_addon_overview))
        .route("/:addon/item", post(create_addon_item))
}

async fn get_addon_overview(
    Path(guid): Path<Uuid>,
    State(db): State<SqlitePool>,
) -> Result<JsonResponse<serde_json::Value>> {
    let mut acq = db.acquire().await?;

    let Some(addon) = AddonModel::find_one_by_guid(guid, &mut *acq).await? else {
        return Err(eyre::eyre!("Addon not found"))?;
    };

    let widgets = AddonWidgetContent::get_all_no_data(addon.id, &mut acq).await?;
    let published = AddonCompiledModel::get_all(addon.id, 0, 10, &mut acq).await?;
    let dash_pages = AddonDashboardPage::find_by_id(addon.id, &mut acq).await?;
    let template_pages = AddonTemplatePageModel::find_by_addon_id(addon.id, &mut acq).await?;

    let schemas = SchemaModel::find_by_addon_id(addon.id, &mut acq)
        .await?
        .into_iter()
        .map(|schema| webby_api::PublicSchema {
            schema_id: schema.name,
            namespace: Some(format!("@{}", addon.name_id)),
            primary_field: schema.primary_field,
            display_name: schema.display_name,
            permissions: schema.permissions.0,
            version: schema.version as f32,
            allowed_operations: schema.allowed_operations.0,
            is_single: false,
            fields: schema.fields.0,
            ttl: schema.ttl,
            default_sort: schema.default_sort,
            views: schema.views.0,
            created_at: schema.created_at,
            updated_at: schema.updated_at,
            deleted_at: schema.deleted_at,
        })
        .collect::<Vec<_>>();

    Ok(Json(WrappingResponse::okay(serde_json::json!({
        "widgets": widgets,
        "published": published,
        "sitePages": template_pages,
        "dashboardPages": dash_pages.into_iter().map(|p| p.into()).collect::<Vec<DashboardPageInfo>>(),
        "dataGUIs": [],
        "schemas": schemas
    }))))
}

#[derive(Deserialize)]
pub struct AddonItemJson {
    pub item: String,
}

pub async fn create_addon_item(
    Path(addon_id): Path<Uuid>,
    State(db): State<SqlitePool>,
    Json(AddonItemJson { item }): Json<AddonItemJson>,
) -> Result<JsonResponse<&'static str>> {
    let mut acq = db.acquire().await?;

    let Some(addon) = AddonModel::find_one_by_guid(addon_id, &mut *acq).await? else {
        return Err(eyre::eyre!("Addon not found"))?;
    };

    if item == "templatePage" {
        let page = DisplayStore::empty_template();
        let rand_num = rand::random::<u8>();

        let count = AddonTemplatePageModel::count_by_addon_id(addon.id, &mut *acq).await?;

        acq.transaction(|txn| {
            Box::pin(async move {
                let page_model = NewAddonTemplatePageModel::new(
                    addon.id,
                    format!("/template{count}{rand_num}"),
                    format!("Template {count}{rand_num}"),
                    page.get_object_ids().into_iter().map(|v| v.id).collect(),
                )
                .insert(txn)
                .await?;

                AddonTemplatePageContentModel::new(page_model.id, page)
                    .insert(txn)
                    .await?;

                Result::<_, crate::Error>::Ok(())
            })
        })
        .await?;
    }

    Ok(Json(WrappingResponse::okay("ok")))
}
