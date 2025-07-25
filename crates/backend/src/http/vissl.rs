use addon_common::{JsonResponse, WrappingResponse};
use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use database::{
    AddonModel, AddonWidgetContent, AddonWidgetPanelContentModel, NewVisslCodeAddonModel,
    NewVisslCodeAddonPanelModel, VisslCodeAddonModel, VisslCodeAddonPanelModel,
};
use eyre::ContextCompat;
use global_common::{
    id::{AddonUuid, AddonWidgetPanelPublicId, AddonWidgetPublicId},
    Either,
};
use scripting::json::VisslContent;
use sqlx::SqlitePool;

use crate::Result;

pub fn routes() -> Router<SqlitePool> {
    Router::new()
        .route(
            "/widget/:widget_id",
            get(get_widget_script).post(update_widget_script),
        )
        .route("/widget/:widget_id/compile", get(compile_widget_script))
        // Addon Panel Scripting
        .route(
            "/widget/:widget_id/panel/:panel_id",
            get(get_widget_panel_script).post(update_widget_panel_script),
        )
        .route(
            "/widget/:widget_id/panel/:panel_id/compile",
            get(compile_widget_panel_script),
        )
}

// Addon Scripting

// TODO: Access Checks

// TODO: Code should be compiled on save.

async fn compile_widget_script(
    State(db): State<SqlitePool>,
    Path((addon_id, widget_id)): Path<(AddonUuid, AddonWidgetPublicId)>,
) -> Result<Json<Option<String>>> {
    let mut acq = db.acquire().await?;

    let addon = AddonModel::find_one_by_guid(*addon_id, &mut acq)
        .await?
        .context("Addon not found")?;

    let addon_widget = AddonWidgetContent::find_one_by_public_id_no_data(widget_id, &mut acq)
        .await?
        .context("Addon Widget page not found")?;

    if let Some(found) =
        VisslCodeAddonModel::find_one_addon_widget(addon.id, Some(addon_widget.pk), &mut acq)
            .await?
    {
        match found.take_data() {
            Either::Left(_) => Ok(Json(None)),
            Either::Right(script) => Ok(Json(Some(scripting::swc::compile(script)?))),
        }
    } else {
        Ok(Json(None))
    }
}

async fn get_widget_script(
    State(db): State<SqlitePool>,
    Path((addon_id, widget_id)): Path<(AddonUuid, AddonWidgetPublicId)>,
) -> Result<JsonResponse<Option<Either<VisslContent, String>>>> {
    let mut acq = db.acquire().await?;

    let addon = AddonModel::find_one_by_guid(*addon_id, &mut acq)
        .await?
        .context("Addon not found")?;

    let addon_widget = AddonWidgetContent::find_one_by_public_id_no_data(widget_id, &mut acq)
        .await?
        .context("Addon Widget page not found")?;

    if let Some(found) =
        VisslCodeAddonModel::find_one_addon_widget(addon.id, Some(addon_widget.pk), &mut acq)
            .await?
    {
        Ok(Json(WrappingResponse::okay(Some(found.take_data()))))
    } else {
        Ok(Json(WrappingResponse::okay(None)))
    }
}

async fn update_widget_script(
    State(db): State<SqlitePool>,
    Path((addon_id, widget_id)): Path<(AddonUuid, AddonWidgetPublicId)>,

    Json(visual_or_script): Json<Either<VisslContent, String>>,
) -> Result<JsonResponse<&'static str>> {
    let mut acq = db.acquire().await?;

    let addon = AddonModel::find_one_by_guid(*addon_id, &mut acq)
        .await?
        .context("Addon not found")?;

    let addon_widget = AddonWidgetContent::find_one_by_public_id_no_data(widget_id, &mut acq)
        .await?
        .context("Addon Widget page not found")?;

    match visual_or_script {
        Either::Left(code) => {
            // TODO: `code` Validations

            if let Some(mut model) = VisslCodeAddonModel::find_one_addon_widget(
                addon.id,
                Some(addon_widget.pk),
                &mut acq,
            )
            .await?
            {
                if code.cards.is_empty() && code.links.is_empty() && code.variables.is_empty() {
                    VisslCodeAddonModel::delete_by_id(model.pk(), &mut acq).await?;
                } else {
                    if let VisslCodeAddonModel::Visual { visual_data, .. } = &mut model {
                        visual_data.0 = code;
                    }

                    model.update(&mut acq).await?;
                }
            } else if !code.cards.is_empty() || !code.links.is_empty() || !code.links.is_empty() {
                NewVisslCodeAddonModel::Visual {
                    addon_id: addon.id,
                    widget_id: Some(addon_widget.pk),
                    visual_data: code,
                }
                .insert(&mut acq)
                .await?;
            }
        }

        Either::Right(script) => {
            if let Some(mut model) = VisslCodeAddonModel::find_one_addon_widget(
                addon.id,
                Some(addon_widget.pk),
                &mut acq,
            )
            .await?
            {
                if script.trim().is_empty() {
                    VisslCodeAddonModel::delete_by_id(model.pk(), &mut acq).await?;
                } else {
                    if let VisslCodeAddonModel::Scripting { script_data, .. } = &mut model {
                        *script_data = script;
                    }

                    model.update(&mut acq).await?;
                }
            } else if !script.trim().is_empty() {
                NewVisslCodeAddonModel::Scripting {
                    addon_id: addon.id,
                    widget_id: Some(addon_widget.pk),
                    script_data: script,
                }
                .insert(&mut acq)
                .await?;
            }
        }
    }

    Ok(Json(WrappingResponse::okay("ok")))
}

// TODO: Access Checks

// TODO: Code should be compiled on save.

async fn compile_widget_panel_script(
    State(db): State<SqlitePool>,
    Path((addon_id, widget_id, panel_id)): Path<(
        AddonUuid,
        AddonWidgetPublicId,
        AddonWidgetPanelPublicId,
    )>,
) -> Result<Json<Option<String>>> {
    let mut acq = db.acquire().await?;

    let addon = AddonModel::find_one_by_guid(*addon_id, &mut acq)
        .await?
        .context("Addon not found")?;

    let _addon_widget = AddonWidgetContent::find_one_by_public_id_no_data(widget_id, &mut acq)
        .await?
        .context("Addon Widget page not found")?;

    let addon_widget_panel =
        AddonWidgetPanelContentModel::find_one_by_public_id(panel_id, &mut acq)
            .await?
            .context("Addon Widget panel not found")?;

    if let Some(found) = VisslCodeAddonPanelModel::find_one_addon_widget(
        addon.id,
        Some(addon_widget_panel.pk),
        &mut acq,
    )
    .await?
    {
        match found.take_data() {
            Either::Left(_) => Ok(Json(None)),
            Either::Right(script) => Ok(Json(Some(scripting::swc::compile(script)?))),
        }
    } else {
        Ok(Json(None))
    }
}

async fn get_widget_panel_script(
    State(db): State<SqlitePool>,
    Path((addon_id, widget_id, panel_id)): Path<(
        AddonUuid,
        AddonWidgetPublicId,
        AddonWidgetPanelPublicId,
    )>,
) -> Result<JsonResponse<Option<Either<VisslContent, String>>>> {
    let mut acq = db.acquire().await?;

    let addon = AddonModel::find_one_by_guid(*addon_id, &mut acq)
        .await?
        .context("Addon not found")?;

    let _addon_widget = AddonWidgetContent::find_one_by_public_id_no_data(widget_id, &mut acq)
        .await?
        .context("Addon Widget page not found")?;

    let addon_widget_panel =
        AddonWidgetPanelContentModel::find_one_by_public_id(panel_id, &mut acq)
            .await?
            .context("Addon Widget panel not found")?;

    if let Some(found) = VisslCodeAddonPanelModel::find_one_addon_widget(
        addon.id,
        Some(addon_widget_panel.pk),
        &mut acq,
    )
    .await?
    {
        Ok(Json(WrappingResponse::okay(Some(found.take_data()))))
    } else {
        Ok(Json(WrappingResponse::okay(None)))
    }
}

async fn update_widget_panel_script(
    State(db): State<SqlitePool>,
    Path((addon_id, widget_id, panel_id)): Path<(
        AddonUuid,
        AddonWidgetPublicId,
        AddonWidgetPanelPublicId,
    )>,

    Json(visual_or_script): Json<Either<VisslContent, String>>,
) -> Result<JsonResponse<&'static str>> {
    let mut acq = db.acquire().await?;

    let addon = AddonModel::find_one_by_guid(*addon_id, &mut acq)
        .await?
        .context("Addon not found")?;

    let addon_widget = AddonWidgetContent::find_one_by_public_id_no_data(widget_id, &mut acq)
        .await?
        .context("Addon Widget page not found")?;

    let addon_widget_panel =
        AddonWidgetPanelContentModel::find_one_by_public_id(panel_id, &mut acq)
            .await?
            .context("Addon Widget panel not found")?;

    match visual_or_script {
        Either::Left(code) => {
            // TODO: `code` Validations

            if let Some(mut model) = VisslCodeAddonPanelModel::find_one_addon_widget(
                addon.id,
                Some(addon_widget_panel.pk),
                &mut acq,
            )
            .await?
            {
                if code.cards.is_empty() && code.links.is_empty() && code.variables.is_empty() {
                    VisslCodeAddonPanelModel::delete_by_id(model.pk(), &mut acq).await?;
                } else {
                    if let VisslCodeAddonPanelModel::Visual { visual_data, .. } = &mut model {
                        visual_data.0 = code;
                    }

                    model.update(&mut acq).await?;
                }
            } else if !code.cards.is_empty() || !code.links.is_empty() || !code.links.is_empty() {
                NewVisslCodeAddonPanelModel::Visual {
                    addon_id: addon.id,
                    widget_id: Some(addon_widget.pk),
                    widget_panel_id: Some(addon_widget_panel.pk),
                    visual_data: code,
                }
                .insert(&mut acq)
                .await?;
            }
        }

        Either::Right(script) => {
            if let Some(mut model) = VisslCodeAddonPanelModel::find_one_addon_widget(
                addon.id,
                Some(addon_widget_panel.pk),
                &mut acq,
            )
            .await?
            {
                if script.trim().is_empty() {
                    VisslCodeAddonPanelModel::delete_by_id(model.pk(), &mut acq).await?;
                } else {
                    if let VisslCodeAddonPanelModel::Scripting { script_data, .. } = &mut model {
                        *script_data = script;
                    }

                    model.update(&mut acq).await?;
                }
            } else if !script.trim().is_empty() {
                NewVisslCodeAddonPanelModel::Scripting {
                    addon_id: addon.id,
                    widget_id: Some(addon_widget.pk),
                    widget_panel_id: Some(addon_widget_panel.pk),
                    script_data: script,
                }
                .insert(&mut acq)
                .await?;
            }
        }
    }

    Ok(Json(WrappingResponse::okay("ok")))
}
