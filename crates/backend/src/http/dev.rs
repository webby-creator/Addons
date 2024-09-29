use addon_common::{JsonResponse, WrappingResponse};
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use database::{
    AddonDashboardPage, AddonModel, AddonTemplatePageContentModel, AddonTemplatePageModel,
    NewAddonTemplatePageModel,
};
use local_common::DashboardPageInfo;
use serde::Deserialize;
use sqlx::{Connection, SqlitePool};
use storage::DisplayStore;
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

    //
    let dash_pages = AddonDashboardPage::find_by_id(addon.id, &mut *acq).await?;
    // let widgets = WidgetModel::find_by_addon_id(addon.id, &mut *acq).await?;
    let template_pages = AddonTemplatePageModel::find_by_addon_id(addon.id, &mut *acq).await?;

    Ok(Json(WrappingResponse::okay(serde_json::json!({
        // "widgets": widgets,
        "sitePages": template_pages,
        "dashboardPages": dash_pages.into_iter().map(|p| p.into()).collect::<Vec<DashboardPageInfo>>(),
        "dataGUIs": [],
        "schemas": []
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
