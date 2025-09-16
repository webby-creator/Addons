use webby_api::ListResponse;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use database::{
    AddonCompiledModel, AddonCompiledWidget, AddonInstanceModel, AddonModel, AddonWidgetContent,
    NewAddonInstanceModel, WidgetModel,
};
use eyre::ContextCompat;
use local_common::{MemberModel, WebsiteId, WebsiteModel};
use serde::Deserialize;
use sqlx::SqlitePool;
use uuid::Uuid;
use webby_addon_common::{
    InstallResponse, JsonListResponse, JsonResponse, WebsiteUuid, WrappingResponse,
};
use webby_global_common::id::{AddonWidgetPublicId, WebsitePublicId};
use webby_storage::{widget::CompiledWidgetSettings, DisplayStore};

use crate::Result;

use super::CLIENT;

pub fn routes() -> Router<SqlitePool> {
    Router::new()
        .route("/duplicate", post(duplicate_website_addons))
        .route("/editor/addons", get(get_editor_widget_list))
        .route(
            "/editor/object/:object_id/data",
            get(get_editor_widget_data),
        )
        .route("/widget/:widget_id", get(get_website_addon_widget))
        .route("/widgets", get(get_website_addon_widgets))
}

#[derive(Deserialize)]
struct DuplicateWebsiteJson {
    pub new_website_id: WebsiteId,
    pub new_website_uuid: Uuid,

    // TODO: Both of these are said Models'
    member: MemberModel,
    new_website: WebsiteModel,
}

async fn duplicate_website_addons(
    Path(old_website): Path<Uuid>,
    State(db): State<SqlitePool>,
    Json(DuplicateWebsiteJson {
        new_website_id,
        new_website_uuid,
        member,
        new_website,
    }): Json<DuplicateWebsiteJson>,
) -> Result<JsonResponse<&'static str>> {
    let mut acq = db.acquire().await?;

    let instances = AddonInstanceModel::find_by_website_uuid(old_website, &mut acq).await?;

    for inst in instances {
        // TODO: Duplicate of install above

        let addon = AddonModel::find_one_by_id(inst.addon_id, &mut acq)
            .await?
            .context("Addon not found")?;

        if let Some(url) = addon.action_url {
            // 1. Insert Website Addon
            let mut inst = NewAddonInstanceModel {
                addon_id: inst.addon_id,
                website_id: new_website_id,
                website_uuid: new_website_uuid,
                // TODO: Set to version we're duplicating
                version: String::from("latest"),
            }
            .insert(&mut acq)
            .await?;

            // 2. Send install request
            let resp = CLIENT
                .post(format!("{url}/registration"))
                .json(&serde_json::json!({
                    "instanceId": inst.public_id,

                    "ownerId": member.id,
                    "websiteId": new_website_uuid,

                    // TODO: Use Permissions
                    "member": member,
                    "website": new_website,
                }))
                .send()
                .await?;

            // TODO: Create Addon Template Pages & Widget info in main program

            if resp.status().is_success() {
                // 3. Get Response - Can have multiple resolutions.
                //  - Could want to redirect the user to finish on another site.
                //  - Could be finished now
                //  - Could be step 1 and require multiple setup requests & permission steps.
                let resp: WrappingResponse<InstallResponse> = resp.json().await?;

                match resp {
                    WrappingResponse::Resp(InstallResponse::Complete) => {
                        inst.is_setup = true;
                        inst.update(&mut acq).await?;
                    }

                    WrappingResponse::Resp(InstallResponse::Redirect(_url)) => {
                        // TODO
                    }

                    WrappingResponse::Error(e) => return Ok(Json(WrappingResponse::Error(e))),
                }
            } else {
                return Err(eyre::eyre!("{}", resp.text().await?))?;
            }
        }
    }

    Ok(Json(WrappingResponse::okay("ok")))
}

async fn get_editor_widget_list(
    Path(website_uuid): Path<WebsiteUuid>,
    State(db): State<SqlitePool>,
) -> Result<JsonResponse<Vec<serde_json::Value>>> {
    let mut acq = db.acquire().await?;

    let mut items = Vec::new();

    for instance in AddonInstanceModel::find_by_website_uuid(*website_uuid, &mut acq).await? {
        let addon = AddonModel::find_one_by_id(instance.addon_id, &mut acq)
            .await?
            .context("Addon not found")?;

        let widget_refs = WidgetModel::find_by_addon_id(instance.addon_id, &mut acq).await?;

        items.push(serde_json::json!({
            "instance": instance.public_id,
            "guid": addon.guid,
            "name": addon.name,
            "widgets": widget_refs.into_iter().map(|w| w.public_id).collect::<Vec<_>>(),
        }));
    }

    Ok(Json(WrappingResponse::okay(items)))
}

async fn get_editor_widget_data(
    Path(website_uuid): Path<WebsiteUuid>,
    State(db): State<SqlitePool>,
) -> Result<JsonResponse<serde_json::Value>> {
    let acq = db.acquire().await?;

    // let using_addons = AddonInstanceModel::find_by_website_id(website_uuid).await?;

    Ok(Json(WrappingResponse::okay(serde_json::json!([
        {
            "addon_id": "Use Instance UUID",
            // "widgets"
            // "pages"
        }
    ]))))
}

// TODO: Move
#[derive(serde::Serialize)]
pub struct CompiledAddonWidgetInfo {
    pub data: DisplayStore,
    pub script: Option<String>,
    pub settings: CompiledWidgetSettings,
}

/// Gets' the Published Widget data
async fn get_website_addon_widget(
    State(db): State<SqlitePool>,

    Path((website_id, widget_id)): Path<(WebsitePublicId, AddonWidgetPublicId)>,
) -> Result<JsonResponse<Option<CompiledAddonWidgetInfo>>> {
    let mut acq = db.acquire().await?;

    let widget = AddonWidgetContent::find_one_by_public_id(widget_id, &mut acq)
        .await?
        .context("Widget not found")?;

    let active = AddonInstanceModel::find_by_website_uuid(*website_id, &mut acq).await?;

    for instance in active {
        let addon = AddonModel::find_one_by_id(instance.addon_id, &mut acq)
            .await?
            .unwrap();

        // TODO: A Temporary fix
        if instance.version.is_empty() {
            warn!("Missing Instance Version for Website Addon {}", addon.guid);
            continue;
        }

        // Ensure we're looking at the specific addon
        if widget.addon_id != addon.id {
            continue;
        }

        let addon_compiled = AddonCompiledModel::find_one_by_addon_uuid_and_version(
            addon.id,
            &instance.version,
            &mut acq,
        )
        .await?
        .context("Addon not found")?;

        let mut widget_comp = AddonCompiledWidget::find_one_by_compiled_id_and_widget_id(
            addon_compiled.pk,
            widget.pk,
            &mut acq,
        )
        .await?
        .context("Compiled Widget not found")?;

        for panel in &mut widget_comp.settings.0.panels {
            if let Some(script) = panel.script.as_mut() {
                *script = webby_scripting::swc::compile(script.clone())?;
            }
        }

        return Ok(Json(WrappingResponse::okay(Some(
            CompiledAddonWidgetInfo {
                data: widget_comp.data.0,
                script: widget_comp
                    .script
                    .map(webby_scripting::swc::compile)
                    .transpose()?,
                settings: widget_comp.settings.0,
            },
        ))));
    }

    Ok(Json(WrappingResponse::okay(None)))
}

async fn get_website_addon_widgets(
    State(db): State<SqlitePool>,

    Path(website_id): Path<WebsitePublicId>,
) -> Result<JsonListResponse<serde_json::Value>> {
    let mut acq = db.acquire().await?;

    let mut items = Vec::new();

    for instance in AddonInstanceModel::find_by_website_uuid(*website_id, &mut acq).await? {
        let addon = AddonModel::find_one_by_id(instance.addon_id, &mut acq)
            .await?
            .context("Addon not found")?;

        let widget_refs = WidgetModel::find_by_addon_id(instance.addon_id, &mut acq).await?;

        let mut widget_info = Vec::new();

        for model in widget_refs {
            let found =
                AddonWidgetContent::find_one_by_public_id_no_data(model.public_id, &mut acq)
                    .await?
                    .context("Missing Widget")?;

            widget_info.push(found);
        }

        items.push(serde_json::json!({
            "instance": instance.public_id,
            "guid": addon.guid,
            "name": addon.name,
            "widgets": widget_info,
        }));
    }

    Ok(Json(WrappingResponse::okay(ListResponse::all(items))))
}
