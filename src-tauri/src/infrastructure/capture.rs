use std::{env, fs, path::Path};

use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose};
use uuid::Uuid;
use xcap::{
    Monitor, XCapError,
    image::{RgbaImage, imageops},
};

use crate::{
    application::{
        errors::{ApplicationError, ApplicationErrorKind},
        ports::capture::ScreenCapturePort,
    },
    domain::{
        entities::capture_record::{CaptureRecord, OcrStatus},
        value_objects::identifiers::CaptureId,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureRegionCoordinates {
    pub logical_x: i32,
    pub logical_y: i32,
    pub logical_width: u32,
    pub logical_height: u32,
    pub physical_x: i32,
    pub physical_y: i32,
    pub physical_width: u32,
    pub physical_height: u32,
    pub monitor_logical_x: i32,
    pub monitor_logical_y: i32,
    pub monitor_logical_width: u32,
    pub monitor_logical_height: u32,
    pub monitor_physical_x: i32,
    pub monitor_physical_y: i32,
    pub monitor_physical_width: u32,
    pub monitor_physical_height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CaptureRegion {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

#[derive(Debug, Default)]
pub struct XCapCapturePort;

#[async_trait]
impl ScreenCapturePort for XCapCapturePort {
    async fn capture_area(&self) -> Result<CaptureRecord, ApplicationError> {
        let (path, width, height) = tauri::async_runtime::spawn_blocking(capture_primary_screen)
            .await
            .map_err(|error| {
                ApplicationError::new(
                    ApplicationErrorKind::Unavailable,
                    format!("capture task panicked: {error}"),
                )
            })??;

        let id =
            CaptureId::new(Uuid::new_v4().to_string()).expect("uuid is always a valid capture id");

        Ok(CaptureRecord {
            id,
            session_id: None,
            image_path: path,
            width,
            height,
            ocr_status: OcrStatus::Pending,
        })
    }
}

/// Synchronous screen capture — safe to call from `spawn_blocking` or shortcut handler threads.
pub fn capture_primary_screen() -> Result<(String, u32, u32), ApplicationError> {
    let monitor = primary_monitor()?;
    let image = monitor
        .capture_image()
        .map_err(|error| map_xcap_error(error, "screen capture failed"))?;
    let width = image.width();
    let height = image.height();
    let path = persist_image(&image, "apollo")?;

    Ok((path, width, height))
}

/// Capture a rectangular region of the screen and return the saved PNG path,
/// the captured width/height, and a base64-encoded PNG ready to render in a
/// webview as `data:image/png;base64,...`.
pub fn capture_screen_region_sync(
    coordinates: CaptureRegionCoordinates,
) -> Result<(String, u32, u32, String), ApplicationError> {
    let monitor_region = monitor_lookup_region(coordinates);
    let monitor = resolve_monitor_for_region(monitor_region)?;
    let image = monitor
        .capture_image()
        .map_err(|error| map_xcap_error(error, "region capture failed"))?;
    let local_region = relative_logical_region_within_monitor(coordinates)?;
    let cropped = crop_monitor_image(&image, local_region, coordinates)?;

    let captured_width = cropped.width();
    let captured_height = cropped.height();
    let path = persist_image(&cropped, "apollo_region")?;
    let bytes = fs::read(&path).map_err(|error| {
        ApplicationError::new(
            ApplicationErrorKind::Unavailable,
            format!("failed to read region capture: {error}"),
        )
    })?;
    let base64 = general_purpose::STANDARD.encode(&bytes);
    let data_url = format!("data:image/png;base64,{base64}");

    Ok((path, captured_width, captured_height, data_url))
}

/// Synchronous OCR extraction — safe to call from shortcut handler threads.
pub fn extract_text_sync(image_path: &str, lang: &str) -> Result<String, ApplicationError> {
    let output = std::process::Command::new("tesseract")
        .arg(image_path)
        .arg("stdout")
        .arg("-l")
        .arg(lang)
        .output()
        .map_err(|error| {
            ApplicationError::new(
                ApplicationErrorKind::Unavailable,
                format!("tesseract execution failed: {error}"),
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(ApplicationError::new(
            ApplicationErrorKind::Unavailable,
            if stderr.is_empty() {
                "OCR failed".to_string()
            } else {
                stderr
            },
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn primary_monitor() -> Result<Monitor, ApplicationError> {
    let monitors =
        Monitor::all().map_err(|error| map_xcap_error(error, "screen enumeration failed"))?;

    monitors
        .into_iter()
        .find(|monitor| monitor.is_primary().unwrap_or(false))
        .or_else(|| {
            Monitor::all()
                .ok()
                .and_then(|mut monitors| monitors.drain(..).next())
        })
        .ok_or_else(|| {
            ApplicationError::new(
                ApplicationErrorKind::Unavailable,
                "no monitor found for capture",
            )
        })
}

fn monitor_lookup_region(coordinates: CaptureRegionCoordinates) -> CaptureRegion {
    #[cfg(target_os = "windows")]
    {
        CaptureRegion {
            x: coordinates.physical_x,
            y: coordinates.physical_y,
            width: coordinates.physical_width,
            height: coordinates.physical_height,
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        CaptureRegion {
            x: coordinates.logical_x,
            y: coordinates.logical_y,
            width: coordinates.logical_width,
            height: coordinates.logical_height,
        }
    }
}

fn relative_logical_region_within_monitor(
    coordinates: CaptureRegionCoordinates,
) -> Result<CaptureRegion, ApplicationError> {
    let region = CaptureRegion {
        x: coordinates.logical_x - coordinates.monitor_logical_x,
        y: coordinates.logical_y - coordinates.monitor_logical_y,
        width: coordinates.logical_width,
        height: coordinates.logical_height,
    };

    if region.width == 0 || region.height == 0 {
        return Err(ApplicationError::new(
            ApplicationErrorKind::Validation,
            "selection area must have a positive size",
        ));
    }

    relative_region_within_monitor(
        region,
        CaptureRegion {
            x: 0,
            y: 0,
            width: coordinates.monitor_logical_width,
            height: coordinates.monitor_logical_height,
        },
    )
}

fn relative_region_within_monitor(
    region: CaptureRegion,
    bounds: CaptureRegion,
) -> Result<CaptureRegion, ApplicationError> {
    let local_x = region.x - bounds.x;
    let local_y = region.y - bounds.y;

    if local_x < 0
        || local_y < 0
        || local_x as u32 >= bounds.width
        || local_y as u32 >= bounds.height
    {
        return Err(ApplicationError::new(
            ApplicationErrorKind::Validation,
            "selection area falls outside the resolved monitor bounds",
        ));
    }

    let max_width = bounds.width.saturating_sub(local_x as u32);
    let max_height = bounds.height.saturating_sub(local_y as u32);
    let width = region.width.min(max_width);
    let height = region.height.min(max_height);

    if width == 0 || height == 0 {
        return Err(ApplicationError::new(
            ApplicationErrorKind::Validation,
            "selection area must overlap the resolved monitor bounds",
        ));
    }

    Ok(CaptureRegion {
        x: local_x,
        y: local_y,
        width,
        height,
    })
}

fn resolve_monitor_for_region(region: CaptureRegion) -> Result<Monitor, ApplicationError> {
    Monitor::from_point(region.x, region.y)
        .or_else(|_| {
            let center_x = region.x.saturating_add((region.width / 2) as i32);
            let center_y = region.y.saturating_add((region.height / 2) as i32);
            Monitor::from_point(center_x, center_y)
        })
        .map_err(|error| map_xcap_error(error, "failed to resolve monitor for selection"))
}

fn crop_monitor_image(
    image: &RgbaImage,
    local_region: CaptureRegion,
    coordinates: CaptureRegionCoordinates,
) -> Result<RgbaImage, ApplicationError> {
    if coordinates.monitor_logical_width == 0 || coordinates.monitor_logical_height == 0 {
        return Err(ApplicationError::new(
            ApplicationErrorKind::Validation,
            "resolved monitor geometry must have a positive size",
        ));
    }

    let scale_x = image.width() as f32 / coordinates.monitor_logical_width as f32;
    let scale_y = image.height() as f32 / coordinates.monitor_logical_height as f32;

    let crop_x = ((local_region.x as f32) * scale_x).round().max(0.0) as u32;
    let crop_y = ((local_region.y as f32) * scale_y).round().max(0.0) as u32;
    let max_width = image.width().saturating_sub(crop_x);
    let max_height = image.height().saturating_sub(crop_y);
    let crop_width = ((local_region.width as f32) * scale_x)
        .round()
        .clamp(1.0, max_width as f32) as u32;
    let crop_height = ((local_region.height as f32) * scale_y)
        .round()
        .clamp(1.0, max_height as f32) as u32;

    Ok(imageops::crop_imm(image, crop_x, crop_y, crop_width, crop_height).to_image())
}

fn persist_image(image: &RgbaImage, prefix: &str) -> Result<String, ApplicationError> {
    let filename = format!("{prefix}_{}.png", Uuid::new_v4());
    let path = env::temp_dir().join(filename);

    image.save(&path).map_err(|error| {
        ApplicationError::new(
            ApplicationErrorKind::Unavailable,
            format!("failed to save capture: {error}"),
        )
    })?;

    Ok(path_to_string(&path))
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn map_xcap_error(error: XCapError, context: &str) -> ApplicationError {
    let kind = match error {
        XCapError::InvalidCaptureRegion(_) => ApplicationErrorKind::Validation,
        XCapError::NotSupported => ApplicationErrorKind::Unavailable,
        _ => ApplicationErrorKind::Unavailable,
    };

    ApplicationError::new(kind, format!("{context}: {error}"))
}

#[cfg(test)]
mod tests {
    use xcap::image::{Rgba, RgbaImage};

    use crate::application::errors::ApplicationErrorKind;

    use super::{
        CaptureRegion, CaptureRegionCoordinates, crop_monitor_image,
        relative_logical_region_within_monitor, relative_region_within_monitor,
    };

    #[test]
    fn resolves_selection_relative_to_the_monitor_in_logical_space() {
        let region = relative_logical_region_within_monitor(CaptureRegionCoordinates {
            logical_x: 2020,
            logical_y: 40,
            logical_width: 320,
            logical_height: 200,
            physical_x: 4040,
            physical_y: 80,
            physical_width: 640,
            physical_height: 400,
            monitor_logical_x: 1920,
            monitor_logical_y: 0,
            monitor_logical_width: 1920,
            monitor_logical_height: 1080,
            monitor_physical_x: 3840,
            monitor_physical_y: 0,
            monitor_physical_width: 3840,
            monitor_physical_height: 2160,
        })
        .expect("region should be valid");

        assert_eq!(
            region,
            CaptureRegion {
                x: 100,
                y: 40,
                width: 320,
                height: 200,
            }
        );
    }

    #[test]
    fn clips_selection_to_monitor_bounds_when_rounding_reaches_the_edge() {
        let local = relative_region_within_monitor(
            CaptureRegion {
                x: 1910,
                y: 20,
                width: 40,
                height: 40,
            },
            CaptureRegion {
                x: 1900,
                y: 0,
                width: 45,
                height: 200,
            },
        )
        .expect("selection should be clipped");

        assert_eq!(
            local,
            CaptureRegion {
                x: 10,
                y: 20,
                width: 35,
                height: 40,
            }
        );
    }

    #[test]
    fn rejects_selection_that_starts_outside_the_monitor_bounds() {
        let error = relative_region_within_monitor(
            CaptureRegion {
                x: -10,
                y: 20,
                width: 40,
                height: 40,
            },
            CaptureRegion {
                x: 0,
                y: 0,
                width: 100,
                height: 100,
            },
        )
        .expect_err("selection should be invalid");

        assert_eq!(error.kind, ApplicationErrorKind::Validation);
    }

    #[test]
    fn crops_using_the_monitor_logical_geometry_instead_of_xcap_region_coordinates() {
        let image = RgbaImage::from_fn(400, 200, |x, y| Rgba([x as u8, y as u8, 0, 255]));
        let cropped = crop_monitor_image(
            &image,
            CaptureRegion {
                x: 10,
                y: 20,
                width: 50,
                height: 25,
            },
            CaptureRegionCoordinates {
                logical_x: 110,
                logical_y: 70,
                logical_width: 50,
                logical_height: 25,
                physical_x: 220,
                physical_y: 140,
                physical_width: 100,
                physical_height: 50,
                monitor_logical_x: 100,
                monitor_logical_y: 50,
                monitor_logical_width: 200,
                monitor_logical_height: 100,
                monitor_physical_x: 200,
                monitor_physical_y: 100,
                monitor_physical_width: 400,
                monitor_physical_height: 200,
            },
        )
        .expect("crop should succeed");

        assert_eq!(cropped.width(), 100);
        assert_eq!(cropped.height(), 50);
        assert_eq!(cropped.get_pixel(0, 0), &Rgba([20, 40, 0, 255]));
    }
}
