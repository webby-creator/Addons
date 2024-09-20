use addon_common::{JsonResponse, WrappingResponse};
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use database::{AddonDashboardPage, AddonModel};
use local_common::DashboardPageInfo;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::Result;

pub fn routes() -> Router<SqlitePool> {
    Router::new()
        .route("/:addon/widget", post(create_widget))
        // .route("/:addon/widget/:widget", get(get_))
        .route("/:addon", get(get_addon_overview))
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
    let dash_pages = AddonDashboardPage::find_by_id(addon.id, &mut *db.acquire().await?).await?;

    Ok(Json(WrappingResponse::okay(serde_json::json!({
        "widgets": [],
        "sitePages": [],
        "dashboardPages": dash_pages.into_iter().map(|p| p.into()).collect::<Vec<DashboardPageInfo>>(),
        "dataGUIs": [],
        "schemas": []
    }))))
}

async fn create_widget(
    Path(guid): Path<Uuid>,
    State(db): State<SqlitePool>,
) -> Result<JsonResponse<serde_json::Value>> {
    let mut acq = db.acquire().await?;

    let Some(addon) = AddonModel::find_one_by_guid(guid, &mut *acq).await? else {
        return Err(eyre::eyre!("Addon not found"))?;
    };

    //
    let dash_pages = AddonDashboardPage::find_by_id(addon.id, &mut *db.acquire().await?).await?;

    Ok(Json(WrappingResponse::okay(serde_json::json!({
        "widgets": [],
        "sitePages": [],
        "dashboardPages": dash_pages.into_iter().map(|p| p.into()).collect::<Vec<DashboardPageInfo>>(),
        "dataGUIs": [],
        "schemas": []
    }))))
}
