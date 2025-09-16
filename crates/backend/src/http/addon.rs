use webby_addon_common::{
    InstallResponse, MemberPartial, MemberUuid, RegisterNewJson, WebsitePartial, WebsiteUuid,
};
use webby_api::{ListResponse, WrappingResponse};
use axum::{
    extract::{self, Path, State},
    routing::{get, post},
    Json, Router,
};
use database::{
    AddonCompiledModel, AddonCompiledPage, AddonCompiledWidget, AddonDashboardPage,
    AddonInstanceModel, AddonModel, AddonPermissionModel, AddonTemplatePageContentModel,
    AddonTemplatePageModel, AddonWidgetContent, AddonWidgetNoDataModel,
    AddonWidgetPanelContentModel, AddonWidgetPanelNoDataModel, NewAddonCompiledModel,
    NewAddonCompiledPage, NewAddonCompiledWidget, NewAddonInstanceModel, NewAddonTemplatePageModel,
    NewAddonWidgetContent, NewAddonWidgetPanelContentModel, SchemaModel, VisslCodeAddonModel,
    VisslCodeAddonPanelModel, WidgetModel,
};
use eyre::ContextCompat;
use webby_global_common::{
    id::{AddonUuid, AddonWidgetPanelPublicId, AddonWidgetPublicId},
    response::AddonInstallResponse,
    Either,
};
use lazy_static::lazy_static;
use local_common::{DashboardPageInfo, MemberModel, WebsiteModel};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{Connection, SqliteConnection, SqlitePool};
use webby_storage::{
    widget::CompiledWidgetSettings, CompiledWidgetPanel, DisplayStore, WidgetPanelContent,
};
use time::format_description;
use uuid::Uuid;

use crate::{
    http::{query_active_addon_list, website::CompiledAddonWidgetInfo},
    Result,
};

use super::{JsonListResponse, JsonResponse};

// TODO: Currently we're leaking the ip addresses when its' unable to connect. We'll need to prevent that.
// TODO: Ping/pong websocket
// TODO: Move Addon Models to the addons db - we'll create two addon servers, one for updating and one for serving.

lazy_static! {
    static ref CLIENT: reqwest::Client = reqwest::Client::new();
}

pub fn routes() -> Router<SqlitePool> {
    Router::new()
        // TODO: Should I handle the addon-specific api here?
        // .route("/_api/:addon_id", any(proxy_addon_api))
        // .route("/_api/:addon_id/*O", any(proxy_addon_api))
        // Addon Specific
        .route("/install", post(website_addon_install))
        .route("/publish", post(publish_addon))
        .route("/item", get(get_addon_overview).post(create_addon_item))
        // TODO: Move to Widget
        .route(
            "/widget/:widget",
            get(get_widget_no_data).post(update_widget),
        )
        .route("/widget/:widget/compiled", get(get_widget_compiled))
        .route("/widget/:widget/data", get(get_widget_data))
        .route(
            "/widget/:widget/panels",
            get(get_widget_panel_list).post(create_website_panel),
        )
        .route(
            "/widget/:widget/panels/:panel",
            post(update_widget_panel_data),
        )
        .route(
            "/widget/:widget/panels/:panel/data",
            get(get_widget_panel_data),
        )
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishAddonJson {
    pub draft: bool,
    pub version: Option<String>,
}

pub async fn publish_addon(
    extract::State(db): extract::State<SqlitePool>,
    Path(addon_id): Path<AddonUuid>,

    Json(PublishAddonJson { draft, version }): Json<PublishAddonJson>,
) -> Result<JsonResponse<&'static str>> {
    let mut acq = db.acquire().await?;

    // TODO: When I implement importing local files, I'd need to figure out a way to cache it.
    // TODO: Get Addon Specific Settings from Addon Server

    let mut addon = AddonModel::find_one_by_guid(*addon_id, &mut acq)
        .await?
        .context("Addon not found")?;

    let widgets = AddonWidgetContent::find_by_addon_id(addon.id, &mut acq).await?;
    let panels = AddonWidgetPanelContentModel::find_by_addon_id(addon.id, &mut acq).await?;

    // TODO: Save Dashboard pages

    // Request Addon Pages

    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AddonPageWithDataItem {
        pub public_id: Uuid,

        pub path: String,
        pub display_name: String,

        pub settings: webby_api::WebsitePageSettings,

        pub content: DisplayStore,
        pub version: i32,
    }

    let addon_pages = {
        // TODO: Combine into a single query
        let list = AddonTemplatePageModel::find_by_addon_id(addon.id, &mut acq).await?;

        let mut items = Vec::new();

        for model in list {
            let Some(content) =
                AddonTemplatePageContentModel::find_one_by_page_id(model.id, &mut acq).await?
            else {
                panic!("Unable to find Page Content");
            };

            items.push(AddonPageWithDataItem {
                public_id: model.public_id,
                path: model.path,
                display_name: model.display_name,
                settings: model.settings.0,
                content: content.content.0,
                version: content.version,
            });
        }

        items
    };

    let type_of = if draft {
        database::AddonPublishType::Draft
    } else {
        database::AddonPublishType::Published
    };

    // TODO: Ensure semver r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$"
    // If draft: version is optional
    // If publish: version is required
    let version = if draft {
        let desc =
            format_description::parse("[year]-[month]-[day]-[hour]-[minute]-[second]").unwrap();

        version.unwrap_or_else(|| {
            format!(
                "draft-{}",
                time::OffsetDateTime::now_utc().format(&desc).unwrap()
            )
        })
    } else {
        version.context("Missing Version")?
    };

    acq.transaction(|trx| {
        Box::pin(async move {
            let compiled = NewAddonCompiledModel {
                addon_id: addon.id,
                settings: webby_storage::widget::CompiledAddonSettings {},
                type_of,
                version: version.clone(),
            }
            .insert(trx)
            .await?;

            for page in addon_pages {
                let mut sha = Sha256::new();

                sha.update(serde_json::to_vec(&page.content).unwrap());

                // if let Some(script) = script.as_deref() {
                //     sha.update(script);
                // }

                NewAddonCompiledPage {
                    addon_id: addon.id,
                    compiled_id: compiled.pk,
                    data: page.content,
                    // TODO: Handle scripts
                    script: None,
                    settings: page.settings,
                    hash: format!("{:X}", sha.finalize()),
                    type_of: webby_api::WebsitePageType::Basic,
                    path: page.path,
                    display_name: page.display_name,
                }
                .insert(trx)
                .await?;
            }

            for widget in widgets {
                // TODO: Do this outside of the transaction
                let script =
                    VisslCodeAddonModel::find_one_addon_widget(addon.id, Some(widget.pk), trx)
                        .await?;

                let script = script.map(|s| match s.take_data() {
                    Either::Left(v) => unimplemented!(),
                    // Either::Left(v) => webby_scripting::compile(
                    //     webby_scripting::ModuleInit {
                    //         id: String::new(),
                    //         name: String::new(),
                    //         content: VisslContent::default(),
                    //         inputs: Vec::new(),
                    //         outputs: Vec::new(),
                    //     },
                    //     String::from(""),
                    //     &HashMap::new()
                    // ),
                    Either::Right(v) => v,
                });

                let mut sha = Sha256::new();

                sha.update(serde_json::to_vec(&widget.data.0).unwrap());

                if let Some(script) = script.as_deref() {
                    sha.update(script);
                }

                // Acquire panels
                let mut found_panels = Vec::new();

                for panel in &panels {
                    if panel.addon_widget_id == widget.pk {
                        let script = VisslCodeAddonPanelModel::find_one_addon_widget(
                            addon.id,
                            Some(panel.pk),
                            trx,
                        )
                        .await?;

                        let script = script.map(|s| match s.take_data() {
                            Either::Left(v) => unimplemented!(),
                            // Either::Left(v) => webby_scripting::compile(
                            //     webby_scripting::ModuleInit {
                            //         id: String::new(),
                            //         name: String::new(),
                            //         content: VisslContent::default(),
                            //         inputs: Vec::new(),
                            //         outputs: Vec::new(),
                            //     },
                            //     String::from(""),
                            //     &HashMap::new()
                            // ),
                            Either::Right(v) => v,
                        });

                        sha.update(serde_json::to_vec(&panel.data.0).unwrap());

                        if let Some(script) = script.as_deref() {
                            sha.update(script);
                        }

                        found_panels.push(CompiledWidgetPanel {
                            id: panel.id,
                            content: panel.data.0.clone(),
                            script,
                        });
                    }
                }

                NewAddonCompiledWidget {
                    addon_id: addon.id,
                    widget_id: widget.pk,
                    compiled_id: compiled.pk,
                    data: widget.data.0,
                    script,
                    hash: format!("{:X}", sha.finalize()),
                    title: widget.title,
                    description: widget.description,
                    thumbnail: None,
                    settings: CompiledWidgetSettings {
                        action_bar: widget.settings.0.action_bar,
                        presets: widget.settings.0.presets,
                        variables: widget.settings.0.variables,
                        panels: found_panels,
                    },
                }
                .insert(trx)
                .await?;
            }

            addon.version = version;

            addon.update(trx).await?;

            eyre::Ok(())
        })
    })
    .await?;

    Ok(Json(WrappingResponse::okay("ok")))
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonInstall {
    website_id: WebsiteUuid,
    member_id: MemberUuid,

    // TODO: Both of these are said Models'
    member: MemberModel,
    website: WebsiteModel,
}

pub async fn website_addon_install(
    State(db): State<SqlitePool>,
    Path(addon_uuid): Path<AddonUuid>,
    Json(value): Json<AddonInstall>,
) -> Result<JsonResponse<AddonInstallResponse>> {
    let mut acq = db.acquire().await?;

    let addon = AddonModel::find_one_by_guid(*addon_uuid, &mut acq)
        .await?
        .context("Addon not found")?;

    // TODO: Get an optional current version of the addon instance if it exists. So we can update it instead of installing a new one.
    let active_instances = query_active_addon_list(value.website_id, &mut acq).await?;

    // Get newest version
    let mut compiled = AddonCompiledModel::get_all(addon.id, 0, 1, &mut acq).await?;

    let Some(compiled) = compiled.pop() else {
        return Err(eyre::eyre!("Addon doesn't exist"))?;
    };

    // Check if we have an active instance of the addon
    if let Some(instance) = active_instances.iter().find(|v| v.addon.guid == addon.guid) {
        if instance.instance_version != compiled.version {
            // TODO: We need a way to revert changes if something goes wrong. ---- OR EVEN BETTER, execute the if's below in the transaction.
            // We have an active instance, but the version is different.
            // We should remove addon pages

            warn!(
                "Addon updating not implemented yet. Instance: {}, Version: {}",
                instance.instance_guid, instance.instance_version
            );

            return Ok(Json(WrappingResponse::error("NOT IMPLEMENTED YET")));
        } else {
            // We have an active instance, and the version is the same.
            // We can skip the installation process.
            return Ok(Json(WrappingResponse::error("Already installed")));
        }
    }

    let instance = user_install_addon(*addon_uuid, value, compiled.version, &mut acq).await?;

    #[derive(Serialize)]
    struct PublicPage {
        type_of: webby_api::WebsitePageType,
        addon_uuid: AddonUuid,
        path: String,
        display_name: String,
        data: webby_storage::DisplayStore,
    }

    let widget_pages = AddonCompiledPage::find_by_compiled_id(compiled.pk, &mut acq).await?;

    // ========================================
    // Continue with the installation process
    // ========================================

    // TODO: Currently we continue inside the website. We should have an API endpoint to create webpages from here.

    Ok(Json(WrappingResponse::okay(AddonInstallResponse {
        instance_uuid: instance.public_id,
        new_pages: serde_json::to_value(
            widget_pages
                .into_iter()
                .map(|p| PublicPage {
                    type_of: p.type_of,
                    addon_uuid,
                    path: p.path,
                    display_name: p.display_name,
                    data: p.data.0,
                })
                .collect::<Vec<PublicPage>>(),
        )?,
    })))
}

pub async fn user_install_addon(
    guid: Uuid,
    value: AddonInstall,
    version: String,
    db: &mut SqliteConnection,
) -> Result<AddonInstanceModel> {
    let Some(addon) = AddonModel::find_one_by_guid(guid, db).await? else {
        return Err(eyre::eyre!("Addon not found"))?;
    };

    // TODO: Check if website already has addon installed
    // TODO: Ensure member_id is owner of website or has admin

    // TODO: Utilize perms
    let _perms = AddonPermissionModel::find_by_scope_addon_id(addon.id, "member", db).await?;

    // 1. Insert Website Addon
    let mut inst = NewAddonInstanceModel {
        addon_id: addon.id,
        website_id: value.website.pk,
        website_uuid: *value.website_id,
        version,
    }
    .insert(db)
    .await?;

    if let Some(url) = addon.action_url {
        // 2. Send install request
        let resp = CLIENT
            .post(format!("{url}/registration"))
            .json(&RegisterNewJson {
                instance_id: inst.public_id,
                version: inst.version.clone(),

                owner_id: value.member_id,
                website_id: value.website_id,

                // TODO: Use Permissions
                member: MemberPartial {
                    uuid: value.member.id.into(),
                    role: value.member.role,
                    display_name: value.member.display_name,
                    tag: Some(value.member.tag),
                    email: Some(value.member.email),
                    created_at: value.member.created_at,
                    updated_at: value.member.updated_at,
                },
                website: WebsitePartial {
                    public_id: value.website.id.into(),
                    name: value.website.name,
                    url: value.website.url,
                    theme_id: value.website.theme_id,
                    created_at: value.website.created_at,
                    updated_at: value.website.updated_at,
                },
            })
            .send()
            .await?;

        // TODO: Create Addon Template Pages & Widget info in main program
        // TODO: Its' possible for the registration to succeed but WrappingResponse wil be an Error

        if resp.status().is_success() {
            // 3. Get Response - Can have multiple resolutions.
            //  - Could want to redirect the user to finish on another site.
            //  - Could be finished now
            //  - Could be step 1 and require multiple setup requests & permission steps.
            let resp: WrappingResponse<InstallResponse> = resp.json().await?;

            match resp {
                WrappingResponse::Resp(InstallResponse::Complete) => {
                    inst.is_setup = true;
                    inst.update(db).await?;
                }

                WrappingResponse::Resp(InstallResponse::Redirect(_url)) => {
                    // TODO
                }

                WrappingResponse::Error(e) => return Err(eyre::eyre!("{}", e))?,
            }

            Ok(inst)
        } else {
            // TODO: Remove once registration is fully working
            inst.delete(db).await?;

            let resp = resp.text().await?;

            Err(eyre::eyre!("Addon Install Failed: {resp}"))?
        }
    } else {
        inst.is_setup = true;
        inst.update(db).await?;

        Ok(inst)
    }
}

async fn get_addon_overview(
    Path(guid): Path<Uuid>,
    State(db): State<SqlitePool>,
) -> Result<JsonResponse<serde_json::Value>> {
    let mut acq = db.acquire().await?;

    let Some(addon) = AddonModel::find_one_by_guid(guid, &mut acq).await? else {
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

#[derive(Serialize, Deserialize)]
pub struct AddonItemJson {
    pub item: String,
}

pub async fn create_addon_item(
    Path(addon_id): Path<AddonUuid>,
    State(db): State<SqlitePool>,
    Json(AddonItemJson { item }): Json<AddonItemJson>,
) -> Result<JsonResponse<&'static str>> {
    let mut acq = db.acquire().await?;

    let Some(addon) = AddonModel::find_one_by_guid(*addon_id, &mut acq).await? else {
        return Err(eyre::eyre!("Addon not found"))?;
    };

    if item == "widget" {
        acq.transaction(|txn| {
            Box::pin(async move {
                // 1. Create Widget Content in database locally.
                let model = NewAddonWidgetContent {
                    addon_id: addon.id,
                    data: webby_storage::DisplayStore::empty_widget(),
                    title: None,
                    description: None,
                    thumbnail: None,
                }
                .insert(txn)
                .await?;

                // 2. Insert Widget Content
                WidgetModel {
                    addon_id: addon.id,
                    widget_id: model.pk,
                    public_id: model.id,
                }
                .insert(txn)
                .await?;

                // Addons should request widget info when needed.

                Result::<_, crate::Error>::Ok(())
            })
        })
        .await?;
    } else if item == "templatePage" {
        let page = DisplayStore::empty_template();
        let rand_num = rand::random::<u8>();

        let count = AddonTemplatePageModel::count_by_addon_id(addon.id, &mut acq).await?;

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
    } else {
        warn!("Unhandled Creation Type: {item}");
    }

    Ok(Json(WrappingResponse::okay("ok")))
}

// TODO: We need to specify a version to get.
// Currently this is only used for when an addon uses another addons widget.
pub async fn get_widget_compiled(
    Path((addon_id, widget_id)): Path<(AddonUuid, AddonWidgetPublicId)>,
    State(db): State<SqlitePool>,
) -> Result<JsonResponse<Option<CompiledAddonWidgetInfo>>> {
    let mut acq = db.acquire().await?;

    let addon = AddonModel::find_one_by_guid(*addon_id, &mut acq)
        .await?
        .context("Addon not found")?;

    let widget = AddonWidgetContent::find_one_by_public_id_no_data(widget_id, &mut acq)
        .await?
        .context("Widget not found")?;

    let addon_compiled = AddonCompiledModel::find_one_by_addon_uuid_and_version(
        widget.addon_id,
        &addon.version,
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

pub async fn get_widget_no_data(
    Path((addon_id, widget_id)): Path<(AddonUuid, AddonWidgetPublicId)>,
    State(db): State<SqlitePool>,
) -> Result<JsonResponse<Option<AddonWidgetNoDataModel>>> {
    let mut acq = db.acquire().await?;

    Ok(Json(WrappingResponse::okay(
        AddonWidgetContent::find_one_by_public_id_no_data(widget_id, &mut acq).await?,
    )))
}

pub async fn get_widget_data(
    Path((addon_id, widget_id)): Path<(AddonUuid, AddonWidgetPublicId)>,
    State(db): State<SqlitePool>,
) -> Result<JsonResponse<Option<DisplayStore>>> {
    let mut acq = db.acquire().await?;

    if let Some(found) = AddonWidgetContent::find_one_by_public_id(widget_id, &mut acq).await? {
        Ok(Json(WrappingResponse::okay(Some(found.data.0))))
    } else {
        Ok(Json(WrappingResponse::okay(None)))
    }
}

pub async fn update_widget(
    Path((addon_id, widget_id)): Path<(AddonUuid, AddonWidgetPublicId)>,
    State(db): State<SqlitePool>,
    Json(update): Json<webby_api::UpdateWidget>,
) -> Result<JsonResponse<&'static str>> {
    let mut acq = db.acquire().await?;

    let Some(mut found) = AddonWidgetContent::find_one_by_public_id(widget_id, &mut acq).await?
    else {
        return Err(eyre::eyre!("Widget doesn't exist"))?;
    };

    let mut updated = false;

    if let Some(store) = update.store {
        if matches!(&store, DisplayStore::Widget { .. }) {
            found.data.0 = store;
            updated = true;
        }
    }

    if let Some(variables) = update.variables {
        found.settings.0.variables = variables;
        updated = true;
    }

    if let Some(presets) = update.presets {
        found.settings.0.presets = presets;
        updated = true;
    }

    if let Some(action_bar) = update.action_bar {
        found.settings.0.action_bar = action_bar;
        updated = true;
    }

    if updated {
        found.update(&mut acq).await?;
    }
    Ok(Json(WrappingResponse::okay("ok")))
}

pub async fn get_widget_panel_list(
    Path((addon_id, widget_id)): Path<(AddonUuid, AddonWidgetPublicId)>,
    State(db): State<SqlitePool>,
) -> Result<JsonListResponse<AddonWidgetPanelNoDataModel>> {
    let mut acq = db.acquire().await?;

    let panels = AddonWidgetPanelContentModel::get_all_no_data(widget_id, &mut acq).await?;

    Ok(Json(WrappingResponse::okay(ListResponse::all(panels))))
}

pub async fn create_website_panel(
    Path((addon_id, widget_id)): Path<(AddonUuid, AddonWidgetPublicId)>,
    State(db): State<SqlitePool>,
) -> Result<JsonResponse<AddonWidgetPanelContentModel>> {
    let mut acq = db.acquire().await?;

    let Some(addon) = AddonModel::find_one_by_guid(*addon_id, &mut acq).await? else {
        return Err(eyre::eyre!("Addon not found"))?;
    };

    let widget = AddonWidgetContent::find_one_by_public_id_no_data(widget_id, &mut acq)
        .await?
        .context("Widget doesn't exist")?;

    let model = NewAddonWidgetPanelContentModel {
        addon_id: addon.id,
        addon_widget_id: widget.pk,
        title: Some(String::from("New Panel")),
        data: WidgetPanelContent::empty(),
    }
    .insert(&mut acq)
    .await?;

    Ok(Json(WrappingResponse::okay(model)))
}

pub async fn get_widget_panel_data(
    Path((addon_id, _widget_id, panel_id)): Path<(
        AddonUuid,
        AddonWidgetPublicId,
        AddonWidgetPanelPublicId,
    )>,
    State(db): State<SqlitePool>,
) -> Result<JsonResponse<WidgetPanelContent>> {
    let mut acq = db.acquire().await?;

    let found = AddonWidgetPanelContentModel::find_one_by_public_id(panel_id, &mut acq)
        .await?
        .context("Panel doesn't exist")?;

    Ok(Json(WrappingResponse::okay(found.data.0)))
}

pub async fn update_widget_panel_data(
    Path((addon_id, _widget_id, panel_id)): Path<(
        AddonUuid,
        AddonWidgetPublicId,
        AddonWidgetPanelPublicId,
    )>,
    State(db): State<SqlitePool>,
    Json(store): Json<webby_api::UpdateWidgetPanel>,
) -> Result<JsonResponse<&'static str>> {
    let mut acq = db.acquire().await?;

    let mut found = AddonWidgetPanelContentModel::find_one_by_public_id(panel_id, &mut acq)
        .await?
        .context("Panel doesn't exist")?;

    if let Some(store) = store.contents {
        found.data.0 = store;
    }
    // found.settings.0;

    found.update(&mut acq).await?;

    Ok(Json(WrappingResponse::okay("ok")))
}
