use std::io::Cursor;

use eyre::Result;
use image::{imageops::FilterType, load_from_memory, DynamicImage, ImageFormat};
use sha2::{Digest, Sha256};

use super::{get_full_file_path, get_thumb_file_path, StorageService, UploadResponse};

pub fn crop_to_webp_or_jpg(value: &[u8], (max_width, ratio): (u32, f64)) -> Result<Vec<u8>> {
    let image = load_from_memory(value)?;

    let width = image.width().min(max_width) as f64;
    let image = image.crop_imm(0, 0, width as u32, (width * ratio).ceil() as u32);

    let mut webp = Cursor::new(Vec::new());
    if let Err(e) = image.write_to(&mut webp, ImageFormat::WebP) {
        error!("Failed to write image to webp: {e} Trying with jpeg.");

        webp.get_mut().clear();
        webp.set_position(0);

        image.write_to(&mut webp, ImageFormat::Jpeg)?;
    }

    Ok(webp.into_inner())
}

pub fn convert_to_webp_or_jpg(value: Vec<u8>) -> Result<Vec<u8>> {
    let image = load_from_memory(&value)?;

    let mut webp = Cursor::new(Vec::new());
    if let Err(e) = image.write_to(&mut webp, ImageFormat::WebP) {
        error!("Failed to write image to webp: {e} Trying with jpeg.");

        webp.get_mut().clear();
        webp.set_position(0);

        image.write_to(&mut webp, ImageFormat::Jpeg)?;
    }

    Ok(webp.into_inner())
}

pub fn resize_image_for_attachment_thumbnail(
    image: &DynamicImage,
    dimensions: (u32, u32),
) -> Result<(Vec<u8>, &'static str)> {
    let image = image.resize_to_fill(dimensions.0, dimensions.0, FilterType::Lanczos3);

    let mut image_type = "webp";

    let mut webp = Cursor::new(Vec::new());
    if let Err(e) = image.write_to(&mut webp, ImageFormat::WebP) {
        error!("Failed to write image to webp: {e} -- Trying with jpeg.");

        webp.get_mut().clear();
        webp.set_position(0);

        image.write_to(&mut webp, ImageFormat::Jpeg)?;
        image_type = "jpeg";
    }

    Ok((webp.into_inner(), image_type))
}

pub async fn process_image(
    store_path: &str,
    full_file_name: &str,
    mut image_original_u8: Vec<u8>,
    should_optimize: bool,
    // TODO: Currently we don't also create a thumbnail if this is set.
    set_dimensions: Option<(u32, u32)>,
    storage: &StorageService,
) -> Result<UploadResponse> {
    let (file_name, file_type) = if let Some(split) = full_file_name.rsplit_once('.') {
        split
    } else {
        (full_file_name, "")
    };

    let mut file_type = file_type.to_string();

    let mut file_size = image_original_u8.len();
    let mut original_hash = format!("{:X}", Sha256::digest(&image_original_u8));

    let (media_width, media_height) = if should_optimize && set_dimensions.is_none() {
        let file_format = ImageFormat::WebP;

        // TODO: Actually Optimize
        let image = load_from_memory(&image_original_u8)?;

        let mut data = Cursor::new(Vec::new());
        image.write_to(&mut data, file_format)?;

        // Ensure the "optimization" is actually smaller
        if image_original_u8.len() < data.get_ref().len() {
            debug!(
                "Ignoring Optimization. Size is greater than original: {} < {}",
                image_original_u8.len(),
                data.get_ref().len()
            );

            (image.width(), image.height())
        } else {
            debug!(
                "Optimizing. Decreased size by {} bytes",
                image_original_u8.len() - data.get_ref().len()
            );

            original_hash = format!("{:X}", Sha256::digest(data.get_ref()));
            file_size = data.get_ref().len();
            image_original_u8 = data.into_inner();
            file_type = file_format.extensions_str()[0].to_string();

            (image.width(), image.height())
        }
    } else {
        let mut image = load_from_memory(&image_original_u8)?;

        if let Some(dim) = set_dimensions {
            image = image.resize_to_fill(dim.0, dim.0, FilterType::Lanczos3)
        }

        (image.width(), image.height())
    };

    if set_dimensions.is_none() {
        let (thumbnail_original_data, thumbnail_original_type) =
            resize_image_for_attachment_thumbnail(
                &load_from_memory(&image_original_u8)?,
                (150, 150),
            )?;

        let thumb_file_path = get_thumb_file_path(&store_path);

        storage
            .upload(
                thumb_file_path.clone(),
                mime_guess::from_ext(&thumbnail_original_type).first_or_octet_stream(),
                thumbnail_original_data,
            )
            .await?;
    }

    let full_file_path = get_full_file_path(&store_path);

    storage
        .upload(
            full_file_path.clone(),
            mime_guess::from_ext(&file_type).first_or_octet_stream(),
            image_original_u8,
        )
        .await?;

    Ok(UploadResponse {
        file_name: file_name.to_string(),
        file_type: file_type.clone(),
        file_size: file_size as i64,
        media_width: Some(media_width as i32),
        media_height: Some(media_height as i32),
        hash: original_hash,
        has_thumbnail: set_dimensions.is_none(),
    })
}
