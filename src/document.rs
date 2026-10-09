use crate::inspect_png;
use image::{DynamicImage, ImageFormat, ImageReader};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ColorLayer {
    pub name: String,
    pub rgb: [u8; 3],
    pub opacity: u8,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FinishDocument {
    pub schema_version: u32,
    pub base_png: String,
    pub base_sha256: String,
    pub layers: Vec<ColorLayer>,
}

fn base_path(document: &FinishDocument, document_path: &Path) -> PathBuf {
    let stored = Path::new(&document.base_png);
    if stored.is_absolute() {
        stored.to_path_buf()
    } else {
        document_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(stored)
    }
}

pub fn new_document(base_png: &Path, document_path: &Path) -> Result<FinishDocument, String> {
    let report = inspect_png(base_png)?;
    let base = base_png.canonicalize().map_err(|error| error.to_string())?;
    let parent = document_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let stored = base
        .strip_prefix(&parent)
        .map(|path| path.to_path_buf())
        .unwrap_or(base);
    Ok(FinishDocument {
        schema_version: 1,
        base_png: stored.to_string_lossy().into_owned(),
        base_sha256: report.source_sha256,
        layers: Vec::new(),
    })
}

pub fn load_document(path: &Path) -> Result<FinishDocument, String> {
    let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
    let document: FinishDocument =
        serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    if document.schema_version != 1 {
        return Err("unsupported finish document version".into());
    }
    let actual = inspect_png(&base_path(&document, path))?;
    if actual.source_sha256 != document.base_sha256 {
        return Err("base texture changed since the finish document was saved".into());
    }
    Ok(document)
}

pub fn save_document(
    document: &FinishDocument,
    path: &Path,
    overwrite: bool,
) -> Result<(), String> {
    if overwrite && path.exists() {
        let existing = load_document(path)?;
        if existing.base_sha256 != document.base_sha256 || existing.base_png != document.base_png {
            return Err("destination belongs to a different finish project".into());
        }
    }
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut pending = tempfile::NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    let bytes = serde_json::to_vec_pretty(document).map_err(|error| error.to_string())?;
    pending
        .write_all(&bytes)
        .and_then(|_| pending.as_file().sync_all())
        .map_err(|error| error.to_string())?;
    if overwrite {
        pending
            .persist(path)
            .map_err(|error| error.error.to_string())?;
    } else {
        pending
            .persist_noclobber(path)
            .map_err(|error| error.error.to_string())?;
    }
    Ok(())
}

impl FinishDocument {
    pub fn add_color_layer(&mut self, name: &str, rgb: [u8; 3], opacity: u8) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("layer name is required".into());
        }
        self.layers.push(ColorLayer {
            name: name.into(),
            rgb,
            opacity,
            enabled: true,
        });
        Ok(())
    }
}

pub fn export_png(
    document: &FinishDocument,
    document_path: &Path,
    output: &Path,
) -> Result<(), String> {
    let base = base_path(document, document_path);
    let report = inspect_png(&base)?;
    if report.source_sha256 != document.base_sha256 {
        return Err("base texture changed".into());
    }
    let mut pixels = ImageReader::open(&base)
        .map_err(|error| error.to_string())?
        .decode()
        .map_err(|error| error.to_string())?
        .to_rgba8();
    for layer in document.layers.iter().filter(|layer| layer.enabled) {
        let strength = u32::from(layer.opacity);
        for pixel in pixels.pixels_mut() {
            for channel in 0..3 {
                pixel.0[channel] = ((u32::from(pixel.0[channel]) * (255 - strength)
                    + u32::from(layer.rgb[channel]) * strength
                    + 127)
                    / 255) as u8;
            }
        }
    }
    let parent = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut pending = tempfile::NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    DynamicImage::ImageRgba8(pixels)
        .write_to(&mut pending, ImageFormat::Png)
        .map_err(|error| error.to_string())?;
    pending
        .as_file()
        .sync_all()
        .map_err(|error| error.to_string())?;
    pending
        .persist_noclobber(output)
        .map_err(|error| error.error.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{export_png, load_document, new_document, save_document};
    use image::{ImageReader, Rgba, RgbaImage};

    #[test]
    fn color_layer_survives_reopen_and_exports_changed_rgb_with_original_alpha() {
        let dir = tempfile::TempDir::new().unwrap();
        let base = dir.path().join("base.png");
        let mut image = RgbaImage::from_pixel(2, 1, Rgba([10, 20, 30, 255]));
        image.put_pixel(1, 0, Rgba([10, 20, 30, 0]));
        image.save(&base).unwrap();
        let path = dir.path().join("finish.skinstrike.json");
        let mut document = new_document(&base, &path).unwrap();
        document
            .add_color_layer("red wash", [210, 20, 30], 128)
            .unwrap();
        save_document(&document, &path, false).unwrap();
        let reopened = load_document(&path).unwrap();
        assert_eq!(reopened.layers.len(), 1);
        let output = dir.path().join("export.png");
        export_png(&reopened, &path, &output).unwrap();
        let result = ImageReader::open(&output)
            .unwrap()
            .decode()
            .unwrap()
            .to_rgba8();
        assert!(result.get_pixel(0, 0).0[0] > 10);
        assert_eq!(result.get_pixel(0, 0).0[3], 255);
        assert_eq!(result.get_pixel(1, 0).0[3], 0);
        assert!(export_png(&reopened, &path, &output).is_err());
    }

    #[test]
    fn base_hash_prevents_silent_source_swap() {
        let dir = tempfile::TempDir::new().unwrap();
        let base = dir.path().join("base.png");
        RgbaImage::from_pixel(1, 1, Rgba([1, 2, 3, 255]))
            .save(&base)
            .unwrap();
        let path = dir.path().join("finish.skinstrike.json");
        let document = new_document(&base, &path).unwrap();
        save_document(&document, &path, false).unwrap();
        RgbaImage::from_pixel(1, 1, Rgba([3, 2, 1, 255]))
            .save(&base)
            .unwrap();
        assert!(load_document(&path).unwrap_err().contains("changed"));
    }

    #[test]
    fn refuses_to_overwrite_unrelated_json() {
        let dir = tempfile::TempDir::new().unwrap();
        let base = dir.path().join("base.png");
        RgbaImage::from_pixel(1, 1, Rgba([1, 2, 3, 255]))
            .save(&base)
            .unwrap();
        let path = dir.path().join("existing.json");
        std::fs::write(&path, b"{\"unrelated\":true}").unwrap();
        let document = new_document(&base, &path).unwrap();
        assert!(save_document(&document, &path, true).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"{\"unrelated\":true}");
    }
}
