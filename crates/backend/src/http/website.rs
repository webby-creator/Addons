use addon_common::{InstallResponse, JsonResponse, WebsiteUuid, WrappingResponse};
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use database::{AddonInstanceModel, AddonModel, NewAddonInstanceModel, WidgetModel};
use eyre::ContextCompat;
use local_common::{MemberModel, WebsiteId, WebsiteModel};
use serde::Deserialize;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::Result;

use super::CLIENT;

pub fn routes() -> Router<SqlitePool> {
    Router::new()
        .route("/:website_id/duplicate", post(duplicate_website_addons))
        .route("/:website_id/editor/addons", get(get_editor_widget_list))
        .route(
            "/:website_id/editor/object/:object_id/data",
            get(get_editor_widget_data),
        )
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

    let instances = AddonInstanceModel::find_by_website_uuid(old_website, &mut *acq).await?;

    for inst in instances {
        // TODO: Duplicate of install above

        let addon = AddonModel::find_one_by_id(inst.addon_id, &mut *acq)
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
            .insert(&mut *acq)
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
                        inst.update(&mut *acq).await?;
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

    for instance in AddonInstanceModel::find_by_website_uuid(*website_uuid, &mut *acq).await? {
        let addon = AddonModel::find_one_by_id(instance.addon_id, &mut *acq)
            .await?
            .context("Addon not found")?;

        let widget_refs = WidgetModel::find_by_addon_id(instance.addon_id, &mut *acq).await?;

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
    let mut acq = db.acquire().await?;

    // let using_addons = AddonInstanceModel::find_by_website_id(website_uuid).await?;

    Ok(Json(WrappingResponse::okay(serde_json::json!([
        {
            "addon_id": "Use Instance UUID",
            // "widgets"
            // "pages"
        }
    ]))))
}
