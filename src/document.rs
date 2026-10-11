use image::{DynamicImage, ImageFormat, ImageReader, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{Cursor, Read, Write};
use std::path::Path;
use std::sync::Arc;

const MAX_PNG_BYTES: usize = 32 * 1024 * 1024;
const MAX_DIMENSION: u32 = 4096;
const MAX_TOTAL_PIXELS: u64 = 64 * 1024 * 1024;
const MAX_PROJECT_BYTES: u64 = 256 * 1024 * 1024;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum AlphaMode {
    #[default]
    PreserveBase,
    CompositeCoverage,
    ImportedData,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Asset {
    pub name: String,
    #[serde(default)]
    pub source_origin: Option<String>,
    pub sha256: String,
    pub png: Arc<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LayerContent {
    Color {
        rgb: [u8; 3],
    },
    Image {
        asset_id: String,
        x: f32,
        y: f32,
        scale: f32,
        rotation_deg: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Layer {
    pub name: String,
    pub visible: bool,
    #[serde(default)]
    pub locked: bool,
    pub opacity: u8,
    pub content: LayerContent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AlphaData {
    pub asset_id: String,
    pub invert: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UvReference {
    pub asset_id: String,
    pub visible: bool,
    pub opacity: u8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum FinishStyleIntent {
    #[default]
    Unspecified,
    CustomPaint,
    Hydrographic,
    AnodizedMulticolored,
    Gunsmith,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FinishDocument {
    pub schema_version: u32,
    pub project_id: String,
    #[serde(default)]
    pub source_origin: Option<String>,
    pub width: u32,
    pub height: u32,
    pub alpha_mode: AlphaMode,
    #[serde(default)]
    pub alpha_data: Option<AlphaData>,
    #[serde(default)]
    pub uv_reference: Option<UvReference>,
    #[serde(default)]
    pub style_intent: FinishStyleIntent,
    pub assets: BTreeMap<String, Asset>,
    pub layers: Vec<Layer>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DocumentState {
    pub alpha_mode: AlphaMode,
    pub alpha_data: Option<AlphaData>,
    pub uv_reference: Option<UvReference>,
    pub style_intent: FinishStyleIntent,
    pub layers: Vec<Layer>,
}

impl FinishDocument {
    pub fn state(&self) -> DocumentState {
        DocumentState {
            alpha_mode: self.alpha_mode,
            alpha_data: self.alpha_data.clone(),
            uv_reference: self.uv_reference.clone(),
            style_intent: self.style_intent,
            layers: self.layers.clone(),
        }
    }

    pub fn restore(&mut self, state: DocumentState) {
        self.alpha_mode = state.alpha_mode;
        self.alpha_data = state.alpha_data;
        self.uv_reference = state.uv_reference;
        self.style_intent = state.style_intent;
        self.layers = state.layers;
    }

    pub fn add_color_layer(&mut self, name: &str, rgb: [u8; 3], opacity: u8) -> Result<(), String> {
        self.check_layer_slot(name)?;
        self.layers.push(Layer {
            name: name.trim().into(),
            visible: true,
            locked: false,
            opacity,
            content: LayerContent::Color { rgb },
        });
        Ok(())
    }

    fn check_layer_slot(&self, name: &str) -> Result<(), String> {
        if self.layers.len() >= 128 {
            return Err("Project supports at most 128 layers".into());
        }
        validate_name(name)?;
        if self
            .layers
            .iter()
            .any(|layer| layer.name.trim().eq_ignore_ascii_case(name.trim()))
        {
            return Err(format!("Layer name '{}' is already in use", name.trim()));
        }
        Ok(())
    }

    pub fn unique_layer_name(&self, preferred: &str) -> Result<String, String> {
        validate_name(preferred)?;
        if !self
            .layers
            .iter()
            .any(|layer| layer.name.trim().eq_ignore_ascii_case(preferred.trim()))
        {
            return Ok(preferred.trim().into());
        }
        for number in 2..=129 {
            let suffix = format!(" {number}");
            let keep = 80 - suffix.chars().count();
            let prefix: String = preferred.trim().chars().take(keep).collect();
            let candidate = format!("{prefix}{suffix}");
            if !self
                .layers
                .iter()
                .any(|layer| layer.name.trim().eq_ignore_ascii_case(&candidate))
            {
                return Ok(candidate);
            }
        }
        Err("No unused layer name is available".into())
    }

    pub fn add_image_layer(&mut self, path: &Path) -> Result<usize, String> {
        if self.layers.len() >= 128 {
            return Err("Project supports at most 128 layers".into());
        }
        let preferred = path.file_stem().unwrap_or_default().to_string_lossy();
        let name = self.unique_layer_name(if preferred.trim().is_empty() {
            "Artwork"
        } else {
            &preferred
        })?;
        let png = read_png(path)?;
        let image = decode_png(&png)?;
        let digest = hash(&png);
        let id = format!("art-{digest}");
        let mut total_pixels = self.used_asset_pixels()?;
        if !self.assets.contains_key(&id) {
            total_pixels += u64::from(image.width()) * u64::from(image.height());
        }
        if total_pixels > MAX_TOTAL_PIXELS {
            return Err("Project images exceed 64 megapixels of decoded assets".into());
        }
        self.assets.entry(id.clone()).or_insert_with(|| Asset {
            name: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            source_origin: path
                .canonicalize()
                .ok()
                .map(|p| p.to_string_lossy().into_owned()),
            sha256: digest,
            png: Arc::new(png),
        });
        self.layers.push(Layer {
            name,
            visible: true,
            locked: false,
            opacity: 255,
            content: LayerContent::Image {
                asset_id: id,
                x: (self.width as f32 - image.width() as f32) / 2.0,
                y: (self.height as f32 - image.height() as f32) / 2.0,
                scale: 1.0,
                rotation_deg: 0.0,
            },
        });
        Ok(self.layers.len() - 1)
    }

    pub fn set_alpha_data(&mut self, path: &Path) -> Result<(), String> {
        let png = read_png(path)?;
        let image = decode_png(&png)?;
        if image.dimensions() != (self.width, self.height) {
            return Err("Alpha data PNG must exactly match the canvas dimensions".into());
        }
        validate_alpha_pixels(&image)?;
        let digest = hash(&png);
        let id = format!("alpha-{digest}");
        let mut next = self.clone();
        next.assets
            .insert(id.clone(), imported_asset(path, png, digest));
        next.alpha_data = Some(AlphaData {
            asset_id: id,
            invert: false,
        });
        next.alpha_mode = AlphaMode::ImportedData;
        validate_edit_state(&next)?;
        *self = next;
        Ok(())
    }

    pub fn set_uv_reference(&mut self, path: &Path) -> Result<(), String> {
        let png = read_png(path)?;
        let image = decode_png(&png)?;
        if image.dimensions() != (self.width, self.height) {
            return Err("UV reference PNG must exactly match the canvas dimensions".into());
        }
        let digest = hash(&png);
        let id = format!("uv-{digest}");
        let mut next = self.clone();
        next.assets
            .insert(id.clone(), imported_asset(path, png, digest));
        next.uv_reference = Some(UvReference {
            asset_id: id,
            visible: true,
            opacity: 128,
        });
        validate_edit_state(&next)?;
        *self = next;
        Ok(())
    }

    fn used_asset_pixels(&self) -> Result<u64, String> {
        let mut ids = BTreeSet::from(["base".to_string()]);
        if let Some(data) = &self.alpha_data {
            ids.insert(data.asset_id.clone());
        }
        if let Some(reference) = &self.uv_reference {
            ids.insert(reference.asset_id.clone());
        }
        for layer in &self.layers {
            if let LayerContent::Image { asset_id, .. } = &layer.content {
                ids.insert(asset_id.clone());
            }
        }
        ids.iter().try_fold(0u64, |sum, id| {
            let asset = self
                .assets
                .get(id)
                .ok_or_else(|| format!("Artwork asset missing: {id}"))?;
            let (w, h) = png_dimensions(&asset.png)?;
            Ok(sum + u64::from(w) * u64::from(h))
        })
    }
}
fn imported_asset(path: &Path, png: Vec<u8>, digest: String) -> Asset {
    Asset {
        name: path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned(),
        source_origin: path
            .canonicalize()
            .ok()
            .map(|p| p.to_string_lossy().into_owned()),
        sha256: digest,
        png: Arc::new(png),
    }
}

fn validate_alpha_pixels(image: &RgbaImage) -> Result<(), String> {
    if image.pixels().any(|pixel| {
        let [r, g, b, a] = pixel.0;
        r != g || g != b || a != 255
    }) {
        return Err("Alpha data PNG must be opaque grayscale (R=G=B, A=255)".into());
    }
    Ok(())
}

fn validate_name(name: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        Err("Layer name is required".into())
    } else if name.chars().count() > 80 {
        Err("Layer name must be at most 80 characters".into())
    } else {
        Ok(())
    }
}

pub fn validate_edit_state(document: &FinishDocument) -> Result<(), String> {
    if document.layers.len() > 128 {
        return Err("Project supports at most 128 layers".into());
    }
    let mut names = BTreeSet::new();
    for layer in &document.layers {
        validate_name(&layer.name)?;
        if !names.insert(layer.name.trim().to_lowercase()) {
            return Err(format!("Layer name '{}' is already in use", layer.name));
        }
        if let LayerContent::Image {
            asset_id,
            x,
            y,
            scale,
            rotation_deg,
        } = &layer.content
        {
            if !x.is_finite()
                || !y.is_finite()
                || !scale.is_finite()
                || !rotation_deg.is_finite()
                || *scale < 0.01
                || *scale > 100.0
                || x.abs() > 100_000.0
                || y.abs() > 100_000.0
            {
                return Err(format!("Invalid transform in layer '{}'", layer.name));
            }
            if !document.assets.contains_key(asset_id) {
                return Err(format!("Artwork asset missing for layer '{}'", layer.name));
            }
        }
    }
    if document.alpha_mode == AlphaMode::ImportedData && document.alpha_data.is_none() {
        return Err("Imported alpha mode requires an alpha data PNG".into());
    }
    for (label, id) in [
        (
            "Alpha data",
            document.alpha_data.as_ref().map(|a| &a.asset_id),
        ),
        (
            "UV reference",
            document.uv_reference.as_ref().map(|u| &u.asset_id),
        ),
    ] {
        if let Some(id) = id {
            let asset = document
                .assets
                .get(id)
                .ok_or_else(|| format!("{label} asset missing"))?;
            if png_dimensions(&asset.png)? != (document.width, document.height) {
                return Err(format!(
                    "{label} PNG must exactly match the canvas dimensions"
                ));
            }
        }
    }
    if document.used_asset_pixels()? > MAX_TOTAL_PIXELS {
        return Err("Project images exceed 64 megapixels of decoded assets".into());
    }
    Ok(())
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn png_dimensions(bytes: &[u8]) -> Result<(u32, u32), String> {
    if bytes.len() < 24 || &bytes[0..8] != b"\x89PNG\r\n\x1a\n" || &bytes[12..16] != b"IHDR" {
        return Err("PNG header is missing or invalid".into());
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
    let height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
    if width == 0 || height == 0 || width > MAX_DIMENSION || height > MAX_DIMENSION {
        return Err("PNG dimensions must be 1-4096 pixels per side".into());
    }
    Ok((width, height))
}

fn decode_png(bytes: &[u8]) -> Result<RgbaImage, String> {
    if bytes.len() > MAX_PNG_BYTES {
        return Err("PNG exceeds 32 MiB import limit".into());
    }
    let expected = png_dimensions(bytes)?;
    let mut reader = ImageReader::with_format(Cursor::new(bytes), ImageFormat::Png);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_DIMENSION);
    limits.max_image_height = Some(MAX_DIMENSION);
    limits.max_alloc = Some(96 * 1024 * 1024);
    reader.limits(limits);
    let image = reader
        .decode()
        .map_err(|error| format!("PNG decode failed: {error}"))?
        .to_rgba8();
    if image.dimensions() != expected {
        return Err("PNG decoded dimensions differ from header".into());
    }
    Ok(image)
}

fn read_png(path: &Path) -> Result<Vec<u8>, String> {
    if !path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("png"))
    {
        return Err("Choose a PNG image".into());
    }
    let size = std::fs::metadata(path)
        .map_err(|e| format!("Cannot read {}: {e}", path.display()))?
        .len();
    if size > MAX_PNG_BYTES as u64 {
        return Err("PNG exceeds 32 MiB import limit".into());
    }
    let bytes = std::fs::read(path).map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
    png_dimensions(&bytes)?;
    Ok(bytes)
}
pub fn new_document(base_png: &Path, _document_path: &Path) -> Result<FinishDocument, String> {
    let png = read_png(base_png)?;
    let image = decode_png(&png)?;
    let digest = hash(&png);
    let mut assets = BTreeMap::new();
    assets.insert(
        "base".into(),
        Asset {
            name: base_png
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            source_origin: base_png
                .canonicalize()
                .ok()
                .map(|p| p.to_string_lossy().into_owned()),
            sha256: digest.clone(),
            png: Arc::new(png),
        },
    );
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    Ok(FinishDocument {
        schema_version: 2,
        project_id: hash(format!("{digest}:{nonce}:{}", std::process::id()).as_bytes()),
        source_origin: base_png
            .canonicalize()
            .ok()
            .map(|p| p.to_string_lossy().into_owned()),
        width: image.width(),
        height: image.height(),
        alpha_mode: AlphaMode::PreserveBase,
        alpha_data: None,
        uv_reference: None,
        style_intent: FinishStyleIntent::Unspecified,
        assets,
        layers: Vec::new(),
    })
}

#[derive(Deserialize)]
struct LegacyColorLayer {
    name: String,
    rgb: [u8; 3],
    opacity: u8,
    enabled: bool,
}
#[derive(Deserialize)]
struct LegacyDocument {
    schema_version: u32,
    base_png: String,
    base_sha256: String,
    layers: Vec<LegacyColorLayer>,
}

pub fn load_document(path: &Path) -> Result<FinishDocument, String> {
    let length = std::fs::metadata(path)
        .map_err(|e| format!("Cannot open project: {e}"))?
        .len();
    if length > MAX_PROJECT_BYTES {
        return Err("Project exceeds 256 MiB limit".into());
    }
    let bytes = std::fs::read(path).map_err(|e| format!("Cannot open project: {e}"))?;
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|e| format!("Invalid project JSON: {e}"))?;
    let version = value
        .get("schema_version")
        .and_then(|v| v.as_u64())
        .ok_or("Project version is missing")?;
    let document = match version {
        1 => {
            let old: LegacyDocument = serde_json::from_value(value)
                .map_err(|e| format!("Invalid legacy project: {e}"))?;
            if old.schema_version != 1 {
                return Err("Unsupported project version".into());
            }
            let stored = Path::new(&old.base_png);
            let source = if stored.is_absolute() {
                stored.to_path_buf()
            } else {
                path.parent().unwrap_or(Path::new(".")).join(stored)
            };
            let mut next = new_document(&source, path)
                .map_err(|e| format!("Legacy project source missing or invalid: {e}"))?;
            if next.assets["base"].sha256 != old.base_sha256 {
                return Err("Legacy base texture changed since save".into());
            }
            for layer in old.layers {
                next.add_color_layer(&layer.name, layer.rgb, layer.opacity)?;
                next.layers.last_mut().unwrap().visible = layer.enabled;
            }
            next
        }
        2 => serde_json::from_value(value).map_err(|e| format!("Invalid project: {e}"))?,
        _ => return Err(format!("Unsupported project version {version}")),
    };
    validate_document(&document)?;
    Ok(document)
}

pub fn validate_document(document: &FinishDocument) -> Result<(), String> {
    if document.schema_version != 2 || document.project_id.len() < 16 {
        return Err("Invalid project identity or version".into());
    }
    validate_edit_state(document)?;
    let base = document
        .assets
        .get("base")
        .ok_or("Base image missing from project")?;
    let decoded = verify_asset(base)?;
    if (decoded.width(), decoded.height()) != (document.width, document.height) {
        return Err("Base image dimensions differ from project canvas".into());
    }
    let mut checked = BTreeSet::new();
    for layer in &document.layers {
        if let LayerContent::Image { asset_id, .. } = &layer.content {
            if checked.insert(asset_id) {
                verify_asset(
                    document
                        .assets
                        .get(asset_id)
                        .ok_or("Artwork image missing")?,
                )?;
            }
        }
    }
    if let Some(data) = &document.alpha_data {
        let alpha = verify_asset(
            document
                .assets
                .get(&data.asset_id)
                .ok_or("Alpha data asset missing")?,
        )?;
        validate_alpha_pixels(&alpha)?;
    }
    if let Some(reference) = &document.uv_reference {
        verify_asset(
            document
                .assets
                .get(&reference.asset_id)
                .ok_or("UV reference asset missing")?,
        )?;
    }
    Ok(())
}
fn verify_asset(asset: &Asset) -> Result<RgbaImage, String> {
    if hash(&asset.png) != asset.sha256 {
        return Err(format!(
            "Embedded image '{}' failed integrity check",
            asset.name
        ));
    }
    decode_png(&asset.png)
}

pub fn save_document(
    document: &FinishDocument,
    path: &Path,
    overwrite: bool,
) -> Result<(), String> {
    save_document_with_cancel(document, path, overwrite, &|| false)
}

pub fn save_document_with_cancel(
    document: &FinishDocument,
    path: &Path,
    overwrite: bool,
    cancelled: &dyn Fn() -> bool,
) -> Result<(), String> {
    if cancelled() {
        return Err("Operation cancelled".into());
    }
    validate_document(document)?;
    if cancelled() {
        return Err("Operation cancelled".into());
    }
    if path.exists() {
        if !overwrite {
            return Err("Project already exists; choose another path".into());
        }
        let existing = load_document(path)?;
        if existing.project_id != document.project_id {
            return Err("Destination belongs to a different project".into());
        }
    }
    let mut compact = document.clone();
    let mut used = BTreeSet::from(["base".to_string()]);
    if let Some(data) = &compact.alpha_data {
        used.insert(data.asset_id.clone());
    }
    if let Some(reference) = &compact.uv_reference {
        used.insert(reference.asset_id.clone());
    }
    for layer in &compact.layers {
        if let LayerContent::Image { asset_id, .. } = &layer.content {
            used.insert(asset_id.clone());
        }
    }
    compact.assets.retain(|key, _| used.contains(key));
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut pending =
        tempfile::NamedTempFile::new_in(parent).map_err(|e| format!("Cannot save project: {e}"))?;
    serde_json::to_writer(&mut pending, &compact)
        .map_err(|e| format!("Cannot encode project: {e}"))?;
    if pending
        .as_file()
        .metadata()
        .map_err(|e| format!("Cannot inspect project size: {e}"))?
        .len()
        > MAX_PROJECT_BYTES
    {
        return Err("Project exceeds 256 MiB limit; reduce embedded images".into());
    }
    pending
        .flush()
        .and_then(|_| pending.as_file().sync_all())
        .map_err(|e| format!("Cannot save project: {e}"))?;
    if cancelled() {
        return Err("Operation cancelled".into());
    }
    if overwrite {
        pending
            .persist(path)
            .map_err(|e| format!("Cannot replace project: {}", e.error))?;
    } else {
        pending
            .persist_noclobber(path)
            .map_err(|e| format!("Cannot create project: {}", e.error))?;
    }
    Ok(())
}

fn mix_rgb(dst: &mut Rgba<u8>, src: [u8; 3], opacity: u8, source_alpha: u8, mode: AlphaMode) {
    let amount = f32::from(opacity) / 255.0 * f32::from(source_alpha) / 255.0;
    match mode {
        AlphaMode::PreserveBase | AlphaMode::ImportedData => {
            for c in 0..3 {
                dst.0[c] = (f32::from(dst.0[c]) * (1.0 - amount) + f32::from(src[c]) * amount)
                    .round() as u8;
            }
        }
        AlphaMode::CompositeCoverage => {
            let da = f32::from(dst.0[3]) / 255.0;
            let out_a = amount + da * (1.0 - amount);
            if out_a > 0.0 {
                for c in 0..3 {
                    dst.0[c] = ((f32::from(src[c]) * amount
                        + f32::from(dst.0[c]) * da * (1.0 - amount))
                        / out_a)
                        .round() as u8;
                }
            }
            dst.0[3] = (out_a * 255.0).round() as u8;
        }
    }
}

#[derive(Clone, Default)]
pub struct RenderCache {
    images: BTreeMap<String, Arc<RgbaImage>>,
}

impl RenderCache {
    pub fn insert_asset(&mut self, document: &FinishDocument, id: &str) -> Result<(), String> {
        if !self.images.contains_key(id) {
            let asset = document
                .assets
                .get(id)
                .ok_or_else(|| format!("Artwork asset missing: {id}"))?;
            self.images
                .insert(id.into(), Arc::new(verify_asset(asset)?));
        }
        Ok(())
    }
}

pub fn prepare_render_cache(
    document: &FinishDocument,
    cancelled: &dyn Fn() -> bool,
) -> Result<RenderCache, String> {
    if document.schema_version != 2 || document.project_id.len() < 16 {
        return Err("Invalid project identity or version".into());
    }
    validate_edit_state(document)?;
    let mut cache = RenderCache::default();
    cache.insert_asset(document, "base")?;
    let base = &cache.images["base"];
    if (base.width(), base.height()) != (document.width, document.height) {
        return Err("Base image dimensions differ from project canvas".into());
    }
    for layer in &document.layers {
        if cancelled() {
            return Err("Operation cancelled".into());
        }
        if let LayerContent::Image { asset_id, .. } = &layer.content {
            cache.insert_asset(document, asset_id)?;
        }
    }
    if let Some(data) = &document.alpha_data {
        if cancelled() {
            return Err("Operation cancelled".into());
        }
        cache.insert_asset(document, &data.asset_id)?;
        validate_alpha_pixels(&cache.images[&data.asset_id])?;
    }
    if let Some(reference) = &document.uv_reference {
        if cancelled() {
            return Err("Operation cancelled".into());
        }
        cache.insert_asset(document, &reference.asset_id)?;
    }
    Ok(cache)
}

pub fn render_document(
    document: &FinishDocument,
    preview_max: Option<u32>,
) -> Result<RgbaImage, String> {
    let cache = prepare_render_cache(document, &|| false)?;
    render_document_cached(document, preview_max, &cache, &|| false)
}

pub fn render_document_cached(
    document: &FinishDocument,
    preview_max: Option<u32>,
    cache: &RenderCache,
    cancelled: &dyn Fn() -> bool,
) -> Result<RgbaImage, String> {
    if cancelled() {
        return Err("Operation cancelled".into());
    }
    validate_edit_state(document)?;
    let base = cache
        .images
        .get("base")
        .ok_or("Base image missing from render cache")?;
    let factor = preview_max
        .map(|m| (m as f32 / document.width.max(document.height) as f32).min(1.0))
        .unwrap_or(1.0);
    let out_w = ((document.width as f32 * factor).round() as u32).max(1);
    let out_h = ((document.height as f32 * factor).round() as u32).max(1);
    let mut output = if factor == 1.0 {
        (**base).clone()
    } else {
        image::imageops::resize(&**base, out_w, out_h, image::imageops::FilterType::Triangle)
    };
    for layer in document
        .layers
        .iter()
        .filter(|l| l.visible && l.opacity > 0)
    {
        if cancelled() {
            return Err("Operation cancelled".into());
        }
        match &layer.content {
            LayerContent::Color { rgb } => {
                for pixel in output.pixels_mut() {
                    mix_rgb(pixel, *rgb, layer.opacity, 255, AlphaMode::PreserveBase);
                }
            }
            LayerContent::Image {
                asset_id,
                x,
                y,
                scale,
                rotation_deg,
            } => {
                let art = cache
                    .images
                    .get(asset_id)
                    .ok_or("Artwork image missing from render cache")?;
                let (sin, cos) = rotation_deg.to_radians().sin_cos();
                let center_x = *x + art.width() as f32 * *scale / 2.0;
                let center_y = *y + art.height() as f32 * *scale / 2.0;
                let half_w = art.width() as f32 * *scale / 2.0;
                let half_h = art.height() as f32 * *scale / 2.0;
                let radius_x = cos.abs() * half_w + sin.abs() * half_h;
                let radius_y = sin.abs() * half_w + cos.abs() * half_h;
                let min_x = (((center_x - radius_x) * factor).floor() as i32 - 1).max(0) as u32;
                let min_y = (((center_y - radius_y) * factor).floor() as i32 - 1).max(0) as u32;
                let max_x = (((center_x + radius_x) * factor).ceil() as i32 + 1).max(0) as u32;
                let max_y = (((center_y + radius_y) * factor).ceil() as i32 + 1).max(0) as u32;
                for py in min_y..max_y.min(out_h) {
                    if py % 64 == 0 && cancelled() {
                        return Err("Operation cancelled".into());
                    }
                    for px in min_x..max_x.min(out_w) {
                        let dx = (px as f32 + 0.5) / factor - center_x;
                        let dy = (py as f32 + 0.5) / factor - center_y;
                        let sx = (cos * dx + sin * dy) / *scale + art.width() as f32 / 2.0 - 0.5;
                        let sy = (-sin * dx + cos * dy) / *scale + art.height() as f32 / 2.0 - 0.5;
                        if sx < -0.5
                            || sy < -0.5
                            || sx >= art.width() as f32 - 0.5
                            || sy >= art.height() as f32 - 0.5
                        {
                            continue;
                        }
                        let sample = sample_bilinear(&art, sx, sy);
                        if sample[3] != 0 {
                            mix_rgb(
                                output.get_pixel_mut(px, py),
                                [sample[0], sample[1], sample[2]],
                                layer.opacity,
                                sample[3],
                                document.alpha_mode,
                            );
                        }
                    }
                }
            }
        }
    }
    if document.alpha_mode == AlphaMode::ImportedData {
        let data = document
            .alpha_data
            .as_ref()
            .ok_or("Alpha data PNG missing")?;
        let alpha = cache
            .images
            .get(&data.asset_id)
            .ok_or("Alpha data image missing from render cache")?;
        let scaled = if factor == 1.0 {
            None
        } else {
            Some(image::imageops::resize(
                &**alpha,
                out_w,
                out_h,
                image::imageops::FilterType::Triangle,
            ))
        };
        let map = scaled.as_ref().unwrap_or(alpha.as_ref());
        for (dst, source) in output.pixels_mut().zip(map.pixels()) {
            dst.0[3] = if data.invert {
                255 - source.0[0]
            } else {
                source.0[0]
            };
        }
    }
    Ok(output)
}

pub fn apply_uv_reference_cached(
    document: &FinishDocument,
    preview: &mut RgbaImage,
    cache: &RenderCache,
    cancelled: &dyn Fn() -> bool,
) -> Result<(), String> {
    let Some(reference) = &document.uv_reference else {
        return Ok(());
    };
    if !reference.visible || reference.opacity == 0 {
        return Ok(());
    }
    let original = cache
        .images
        .get(&reference.asset_id)
        .ok_or("UV reference missing from render cache")?;
    let scaled = if original.dimensions() == preview.dimensions() {
        None
    } else {
        Some(image::imageops::resize(
            &**original,
            preview.width(),
            preview.height(),
            image::imageops::FilterType::Triangle,
        ))
    };
    let guide = scaled.as_ref().unwrap_or(original.as_ref());
    for y in 0..preview.height() {
        if y % 64 == 0 && cancelled() {
            return Err("Operation cancelled".into());
        }
        for x in 0..preview.width() {
            let sample = guide.get_pixel(x, y).0;
            if sample[3] != 0 {
                mix_rgb(
                    preview.get_pixel_mut(x, y),
                    [sample[0], sample[1], sample[2]],
                    reference.opacity,
                    sample[3],
                    AlphaMode::CompositeCoverage,
                );
            }
        }
    }
    Ok(())
}

fn sample_bilinear(image: &RgbaImage, x: f32, y: f32) -> [u8; 4] {
    let x = x.clamp(0.0, (image.width() - 1) as f32);
    let y = y.clamp(0.0, (image.height() - 1) as f32);
    let x0 = x.floor() as u32;
    let y0 = y.floor() as u32;
    let x1 = (x0 + 1).min(image.width() - 1);
    let y1 = (y0 + 1).min(image.height() - 1);
    let tx = x - x0 as f32;
    let ty = y - y0 as f32;
    let samples = [
        (image.get_pixel(x0, y0), (1.0 - tx) * (1.0 - ty)),
        (image.get_pixel(x1, y0), tx * (1.0 - ty)),
        (image.get_pixel(x0, y1), (1.0 - tx) * ty),
        (image.get_pixel(x1, y1), tx * ty),
    ];
    let alpha: f32 = samples
        .iter()
        .map(|(pixel, weight)| f32::from(pixel[3]) * weight)
        .sum();
    if alpha == 0.0 {
        return [0, 0, 0, 0];
    }
    let mut result = [0; 4];
    for c in 0..3 {
        let premultiplied: f32 = samples
            .iter()
            .map(|(pixel, weight)| f32::from(pixel[c]) * f32::from(pixel[3]) * weight)
            .sum();
        result[c] = (premultiplied / alpha).round() as u8;
    }
    result[3] = alpha.round() as u8;
    result
}
fn path_identity(path: &Path) -> Result<String, String> {
    let resolved = if path.exists() {
        path.canonicalize()
            .map_err(|e| format!("Cannot resolve {}: {e}", path.display()))?
    } else {
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let directory = parent
            .canonicalize()
            .map_err(|e| format!("Cannot resolve output folder: {e}"))?;
        directory.join(path.file_name().ok_or("Output filename is missing")?)
    };
    Ok(resolved.to_string_lossy().to_lowercase())
}

pub fn validate_export_destination_quick(
    document: &FinishDocument,
    output: &Path,
    overwrite: bool,
    protected_paths: &[&Path],
) -> Result<(), String> {
    if !output
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("png"))
    {
        return Err("Export destination must have a .png extension".into());
    }
    let identity = path_identity(output)?;
    for protected in protected_paths {
        if identity == path_identity(protected)? {
            return Err(format!(
                "Export destination is a protected project or source: {}",
                protected.display()
            ));
        }
    }
    if let Some(source) = &document.source_origin {
        let source = Path::new(source);
        if source.exists() && identity == path_identity(source)? {
            return Err("Export destination is the imported base PNG; choose another path".into());
        }
    }
    for asset in document.assets.values() {
        if let Some(source) = &asset.source_origin {
            let source = Path::new(source);
            if source.exists() && identity == path_identity(source)? {
                return Err(format!(
                    "Export destination is imported source '{}'; choose another path",
                    asset.name
                ));
            }
        }
    }
    if output.exists() {
        if !overwrite {
            return Err("PNG already exists; choose another path or confirm replacement".into());
        }
        let mut signature = [0u8; 8];
        let mut existing = std::fs::File::open(output)
            .map_err(|e| format!("Cannot inspect existing output: {e}"))?;
        existing
            .read_exact(&mut signature)
            .map_err(|_| "Existing output is not a PNG; refusing to replace it".to_string())?;
        if &signature != b"\x89PNG\r\n\x1a\n" {
            return Err("Existing output is not a PNG; refusing to replace it".into());
        }
        let size = std::fs::metadata(output)
            .map_err(|e| format!("Cannot inspect existing output: {e}"))?
            .len();
        if size > MAX_PNG_BYTES as u64 {
            return Err("Existing PNG exceeds 32 MiB; choose another output path".into());
        }
    }
    Ok(())
}

pub fn validate_export_destination(
    document: &FinishDocument,
    output: &Path,
    overwrite: bool,
    protected_paths: &[&Path],
) -> Result<(), String> {
    validate_export_destination_quick(document, output, overwrite, protected_paths)?;
    if output.exists() {
        let bytes =
            std::fs::read(output).map_err(|e| format!("Cannot inspect existing output: {e}"))?;
        decode_png(&bytes)
            .map_err(|error| format!("Existing PNG is invalid; refusing to replace it: {error}"))?;
        let existing_hash = hash(&bytes);
        if document
            .assets
            .values()
            .any(|asset| asset.sha256 == existing_hash)
        {
            return Err(
                "Existing PNG matches an embedded source image; choose another output path".into(),
            );
        }
    }
    Ok(())
}
pub fn export_png(
    document: &FinishDocument,
    document_path: &Path,
    output: &Path,
) -> Result<(), String> {
    export_png_with_overwrite_protected(document, output, false, &[document_path])
}

pub fn export_png_with_overwrite(
    document: &FinishDocument,
    output: &Path,
    overwrite: bool,
) -> Result<(), String> {
    export_png_with_overwrite_protected(document, output, overwrite, &[])
}

pub fn export_png_with_overwrite_protected(
    document: &FinishDocument,
    output: &Path,
    overwrite: bool,
    protected_paths: &[&Path],
) -> Result<(), String> {
    validate_export_destination(document, output, overwrite, protected_paths)?;
    let cache = prepare_render_cache(document, &|| false)?;
    export_png_with_cache(
        document,
        &cache,
        output,
        overwrite,
        protected_paths,
        &|| false,
    )
}

pub fn export_png_with_cache(
    document: &FinishDocument,
    cache: &RenderCache,
    output: &Path,
    overwrite: bool,
    protected_paths: &[&Path],
    cancelled: &dyn Fn() -> bool,
) -> Result<(), String> {
    validate_export_destination(document, output, overwrite, protected_paths)?;
    if cancelled() {
        return Err("Operation cancelled".into());
    }
    let image = render_document_cached(document, None, cache, cancelled)?;
    if cancelled() {
        return Err("Operation cancelled".into());
    }
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut pending =
        tempfile::NamedTempFile::new_in(parent).map_err(|e| format!("Cannot export PNG: {e}"))?;
    DynamicImage::ImageRgba8(image)
        .write_to(&mut pending, ImageFormat::Png)
        .map_err(|e| format!("PNG encode failed: {e}"))?;
    pending
        .as_file()
        .sync_all()
        .map_err(|e| format!("Cannot export PNG: {e}"))?;
    if cancelled() {
        return Err("Operation cancelled".into());
    }
    validate_export_destination(document, output, overwrite, protected_paths)?;
    if overwrite {
        pending
            .persist(output)
            .map_err(|e| format!("Cannot replace PNG: {}", e.error))?;
    } else {
        pending
            .persist_noclobber(output)
            .map_err(|e| format!("Cannot create PNG: {}", e.error))?;
    }
    Ok(())
}
#[cfg(test)]
mod authoring_tests {
    use super::*;
    use image::{ImageReader, Rgba};

    fn png(path: &Path, pixels: &[[u8; 4]]) {
        let mut image = RgbaImage::new(pixels.len() as u32, 1);
        for (x, pixel) in pixels.iter().enumerate() {
            image.put_pixel(x as u32, 0, Rgba(*pixel));
        }
        image.save(path).unwrap();
    }

    #[test]
    fn artwork_layers_export_expected_rgb_and_preserve_base_alpha() {
        let dir = tempfile::TempDir::new().unwrap();
        let base = dir.path().join("base.png");
        let art = dir.path().join("decal.png");
        png(&base, &[[10, 20, 30, 128], [50, 60, 70, 0]]);
        png(&art, &[[210, 100, 50, 128]]);
        let project = dir.path().join("finish.json");
        let mut doc = new_document(&base, &project).unwrap();
        let index = doc.add_image_layer(&art).unwrap();
        if let LayerContent::Image { x, y, .. } = &mut doc.layers[index].content {
            *x = 0.0;
            *y = 0.0;
        }
        let output = dir.path().join("export.png");
        export_png(&doc, &project, &output).unwrap();
        let decoded = ImageReader::open(&output)
            .unwrap()
            .decode()
            .unwrap()
            .to_rgba8();
        let p = decoded.get_pixel(0, 0).0;
        assert!((109..=111).contains(&p[0]), "{p:?}");
        assert_eq!(p[3], 128);
        assert_eq!(decoded.get_pixel(1, 0).0[3], 0);
        assert!(export_png(&doc, &project, &output).is_err());
        doc.alpha_mode = AlphaMode::CompositeCoverage;
        let second = dir.path().join("coverage.png");
        export_png(&doc, &project, &second).unwrap();
        let coverage = ImageReader::open(&second)
            .unwrap()
            .decode()
            .unwrap()
            .to_rgba8();
        assert_eq!(coverage.get_pixel(0, 0).0[3], 192);
        assert!((143..=145).contains(&coverage.get_pixel(0, 0).0[0]));
    }

    #[test]
    fn portable_project_reopens_after_source_files_are_removed_and_project_moves() {
        let dir = tempfile::TempDir::new().unwrap();
        let source = dir.path().join("source");
        let moved = dir.path().join("moved");
        std::fs::create_dir(&source).unwrap();
        std::fs::create_dir(&moved).unwrap();
        let base = source.join("base.png");
        let art = source.join("art.png");
        png(&base, &[[5, 6, 7, 255]]);
        png(&art, &[[220, 2, 1, 255]]);
        let path = source.join("work.skinstrike.json");
        let mut doc = new_document(&base, &path).unwrap();
        doc.add_image_layer(&art).unwrap();
        save_document(&doc, &path, false).unwrap();
        let target = moved.join("work.skinstrike.json");
        std::fs::copy(&path, &target).unwrap();
        std::fs::remove_file(&base).unwrap();
        std::fs::remove_file(&art).unwrap();
        let reopened = load_document(&target).unwrap();
        assert_eq!(reopened.layers, doc.layers);
        let pixels = render_document(&reopened, None).unwrap();
        assert_eq!(pixels.get_pixel(0, 0).0, [220, 2, 1, 255]);
    }

    #[test]
    fn missing_or_changed_embedded_artwork_is_actionable_and_does_not_overwrite() {
        let dir = tempfile::TempDir::new().unwrap();
        let base = dir.path().join("base.png");
        let art = dir.path().join("art.png");
        png(&base, &[[1, 2, 3, 255]]);
        png(&art, &[[4, 5, 6, 255]]);
        let path = dir.path().join("work.json");
        let mut doc = new_document(&base, &path).unwrap();
        doc.add_image_layer(&art).unwrap();
        save_document(&doc, &path, false).unwrap();
        let untouched = std::fs::read(&path).unwrap();
        let key = match &doc.layers[0].content {
            LayerContent::Image { asset_id, .. } => asset_id.clone(),
            _ => unreachable!(),
        };
        doc.assets.remove(&key);
        assert!(
            save_document(&doc, &path, true)
                .unwrap_err()
                .contains("missing")
        );
        assert_eq!(std::fs::read(&path).unwrap(), untouched);
        let mut value: serde_json::Value = serde_json::from_slice(&untouched).unwrap();
        value["assets"].as_object_mut().unwrap().remove(&key);
        let broken = dir.path().join("broken.json");
        std::fs::write(&broken, serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(load_document(&broken).unwrap_err().contains("missing"));
        let mut malformed = load_document(&path).unwrap();
        if let LayerContent::Image { scale, .. } = &mut malformed.layers[0].content {
            *scale = f32::NAN;
        }
        assert!(
            validate_document(&malformed)
                .unwrap_err()
                .contains("Invalid transform")
        );
        std::fs::write(dir.path().join("invalid.png"), b"garbage").unwrap();
        assert!(new_document(&dir.path().join("invalid.png"), &path).is_err());
    }

    #[test]
    fn visibility_order_and_snapshot_restore_are_non_destructive() {
        let dir = tempfile::TempDir::new().unwrap();
        let base = dir.path().join("base.png");
        png(&base, &[[10, 10, 10, 255]]);
        let mut doc = new_document(&base, &dir.path().join("x.json")).unwrap();
        doc.add_color_layer("red", [210, 10, 10], 255).unwrap();
        let prior = doc.state();
        doc.add_color_layer("blue", [10, 10, 210], 255).unwrap();
        assert_eq!(
            render_document(&doc, None).unwrap().get_pixel(0, 0).0,
            [10, 10, 210, 255]
        );
        doc.layers[1].visible = false;
        assert_eq!(
            render_document(&doc, None).unwrap().get_pixel(0, 0).0,
            [210, 10, 10, 255]
        );
        doc.restore(prior);
        assert_eq!(doc.layers.len(), 1);
        assert_eq!(
            render_document(&doc, None).unwrap().get_pixel(0, 0).0,
            [210, 10, 10, 255]
        );
    }

    #[test]
    fn legacy_project_migrates_only_when_source_hash_matches() {
        let dir = tempfile::TempDir::new().unwrap();
        let base = dir.path().join("base.png");
        png(&base, &[[10, 20, 30, 255]]);
        let bytes = std::fs::read(&base).unwrap();
        let path = dir.path().join("old.json");
        let old = serde_json::json!({
            "schema_version": 1, "base_png": "base.png", "base_sha256": hash(&bytes),
            "layers": [{"name":"wash","rgb":[100,20,30],"opacity":128,"enabled":true}]
        });
        std::fs::write(&path, serde_json::to_vec(&old).unwrap()).unwrap();
        let new = load_document(&path).unwrap();
        assert_eq!(new.schema_version, 2);
        assert_eq!(new.layers.len(), 1);
        assert!(save_document(&new, &path, true).is_err());
        let upgraded = dir.path().join("upgraded.json");
        save_document(&new, &upgraded, false).unwrap();
        assert_eq!(load_document(&upgraded).unwrap().layers, new.layers);
        png(&base, &[[1, 2, 3, 255]]);
        assert!(load_document(&path).unwrap_err().contains("changed"));
    }
}

#[cfg(test)]
mod persistence_tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn same_project_can_save_edits_without_replacing_another_project() {
        let dir = tempfile::TempDir::new().unwrap();
        let base = dir.path().join("base.png");
        RgbaImage::from_pixel(2, 2, Rgba([7, 8, 9, 255]))
            .save(&base)
            .unwrap();
        let path = dir.path().join("first.json");
        let mut first = new_document(&base, &path).unwrap();
        save_document(&first, &path, false).unwrap();
        first.add_color_layer("new", [80, 90, 100], 128).unwrap();
        save_document(&first, &path, true).unwrap();
        assert_eq!(load_document(&path).unwrap().layers, first.layers);
        let second = new_document(&base, &path).unwrap();
        let original = std::fs::read(&path).unwrap();
        assert!(
            save_document(&second, &path, true)
                .unwrap_err()
                .contains("different project")
        );
        assert_eq!(std::fs::read(&path).unwrap(), original);
    }

    #[test]
    fn position_and_scale_change_pixels_without_modifying_embedded_artwork() {
        let dir = tempfile::TempDir::new().unwrap();
        let base = dir.path().join("base.png");
        let art = dir.path().join("art.png");
        RgbaImage::from_pixel(4, 1, Rgba([0, 0, 0, 255]))
            .save(&base)
            .unwrap();
        RgbaImage::from_pixel(1, 1, Rgba([255, 0, 0, 255]))
            .save(&art)
            .unwrap();
        let mut doc = new_document(&base, &dir.path().join("x.json")).unwrap();
        doc.add_image_layer(&art).unwrap();
        let asset = if let LayerContent::Image { asset_id, .. } = &doc.layers[0].content {
            doc.assets[asset_id].png.clone()
        } else {
            unreachable!()
        };
        if let LayerContent::Image { x, y, scale, .. } = &mut doc.layers[0].content {
            *x = 0.0;
            *y = 0.0;
            *scale = 1.0;
        }
        assert_eq!(
            render_document(&doc, None).unwrap().get_pixel(0, 0).0,
            [255, 0, 0, 255]
        );
        if let LayerContent::Image { x, .. } = &mut doc.layers[0].content {
            *x = 2.0;
        }
        let moved = render_document(&doc, None).unwrap();
        assert_eq!(moved.get_pixel(0, 0).0, [0, 0, 0, 255]);
        assert_eq!(moved.get_pixel(2, 0).0, [255, 0, 0, 255]);
        if let LayerContent::Image { asset_id, .. } = &doc.layers[0].content {
            assert_eq!(doc.assets[asset_id].png, asset);
        }
    }
}

#[cfg(test)]
mod review_regressions {
    use super::*;
    use image::Rgba;

    #[test]
    fn export_rejects_project_source_alias_and_non_png_replacement_without_data_loss() {
        let dir = tempfile::TempDir::new().unwrap();
        let nested = dir.path().join("nested");
        std::fs::create_dir(&nested).unwrap();
        let base = dir.path().join("base.png");
        RgbaImage::from_pixel(2, 2, Rgba([1, 2, 3, 255]))
            .save(&base)
            .unwrap();
        let project = dir.path().join("project.json");
        let doc = new_document(&base, &project).unwrap();
        save_document(&doc, &project, false).unwrap();
        let project_bytes = std::fs::read(&project).unwrap();
        let base_bytes = std::fs::read(&base).unwrap();
        assert!(
            export_png_with_overwrite(&doc, &project, true)
                .unwrap_err()
                .contains(".png extension")
        );
        assert_eq!(std::fs::read(&project).unwrap(), project_bytes);
        let source_alias = nested.join("..").join("base.png");
        assert!(
            export_png_with_overwrite(&doc, &source_alias, true)
                .unwrap_err()
                .contains("base PNG")
        );
        assert_eq!(std::fs::read(&base).unwrap(), base_bytes);
        let named_png = dir.path().join("project-copy.png");
        std::fs::copy(&project, &named_png).unwrap();
        assert!(
            export_png_with_overwrite(&doc, &named_png, true)
                .unwrap_err()
                .contains("not a PNG")
        );
        assert_eq!(std::fs::read(&named_png).unwrap(), project_bytes);
        let ordinary = dir.path().join("ordinary.png");
        std::fs::write(&ordinary, b"user notes").unwrap();
        assert!(export_png_with_overwrite(&doc, &ordinary, true).is_err());
        assert_eq!(std::fs::read(&ordinary).unwrap(), b"user notes");
    }

    fn assert_invalid_png_destination_unchanged(bytes: &[u8]) {
        let dir = tempfile::TempDir::new().unwrap();
        let base = dir.path().join("base.png");
        RgbaImage::from_pixel(2, 2, Rgba([1, 2, 3, 255]))
            .save(&base)
            .unwrap();
        let project = dir.path().join("project.json");
        let doc = new_document(&base, &project).unwrap();
        let destination = dir.path().join("existing.png");
        std::fs::write(&destination, bytes).unwrap();
        let error =
            export_png_with_overwrite_protected(&doc, &destination, true, &[project.as_path()])
                .unwrap_err();
        assert!(error.contains("Existing PNG is invalid"), "{error}");
        assert_eq!(std::fs::read(&destination).unwrap(), bytes);
    }

    #[test]
    fn signature_only_png_destination_is_preserved() {
        assert_invalid_png_destination_unchanged(b"\x89PNG\r\n\x1a\n");
    }

    #[test]
    fn truncated_png_destination_is_preserved() {
        let dir = tempfile::TempDir::new().unwrap();
        let valid = dir.path().join("valid.png");
        RgbaImage::from_pixel(2, 2, Rgba([4, 5, 6, 255]))
            .save(&valid)
            .unwrap();
        let mut bytes = std::fs::read(&valid).unwrap();
        bytes.truncate(bytes.len() / 2);
        assert_invalid_png_destination_unchanged(&bytes);
    }
    #[test]
    fn layer_limit_duplicate_names_and_long_import_names_preserve_state() {
        let dir = tempfile::TempDir::new().unwrap();
        let base = dir.path().join("base.png");
        RgbaImage::from_pixel(1, 1, Rgba([1, 2, 3, 255]))
            .save(&base)
            .unwrap();
        let mut doc = new_document(&base, &dir.path().join("work.json")).unwrap();
        doc.add_color_layer("Wash", [1, 2, 3], 1).unwrap();
        let before = doc.state();
        assert!(
            doc.add_color_layer(" wash ", [5, 6, 7], 1)
                .unwrap_err()
                .contains("already in use")
        );
        assert_eq!(doc.state(), before);
        let long = dir.path().join(format!("{}.png", "a".repeat(81)));
        RgbaImage::from_pixel(1, 1, Rgba([5, 6, 7, 255]))
            .save(&long)
            .unwrap();
        assert!(
            doc.add_image_layer(&long)
                .unwrap_err()
                .contains("80 characters")
        );
        assert_eq!(doc.state(), before);
        for n in 2..=128 {
            doc.add_color_layer(&format!("wash {n}"), [1, 2, 3], 1)
                .unwrap();
        }
        let full = doc.state();
        assert!(
            doc.add_color_layer("overflow", [1, 2, 3], 1)
                .unwrap_err()
                .contains("128 layers")
        );
        assert_eq!(doc.state(), full);
        let art = dir.path().join("art.png");
        RgbaImage::from_pixel(1, 1, Rgba([4, 5, 6, 255]))
            .save(&art)
            .unwrap();
        assert!(
            doc.add_image_layer(&art)
                .unwrap_err()
                .contains("128 layers")
        );
        assert_eq!(doc.state(), full);
    }

    #[test]
    fn png_header_rejects_oversize_without_decoding() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("oversize.png");
        let mut header = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
        header.extend_from_slice(&(4097u32.to_be_bytes()));
        header.extend_from_slice(&(1u32.to_be_bytes()));
        std::fs::write(&path, header).unwrap();
        let err = new_document(&path, &dir.path().join("x.json")).unwrap_err();
        assert!(err.contains("1-4096"), "{err}");
    }
}

#[cfg(test)]
mod cancellation_regressions {
    use super::*;
    use image::Rgba;

    #[test]
    fn cancelled_save_leaves_destination_untouched() {
        let dir = tempfile::TempDir::new().unwrap();
        let base = dir.path().join("base.png");
        RgbaImage::from_pixel(2, 2, Rgba([1, 2, 3, 255]))
            .save(&base)
            .unwrap();
        let target = dir.path().join("project.json");
        let doc = new_document(&base, &target).unwrap();
        assert!(
            save_document_with_cancel(&doc, &target, false, &|| true)
                .unwrap_err()
                .contains("cancelled")
        );
        assert!(!target.exists());
        save_document(&doc, &target, false).unwrap();
        let original = std::fs::read(&target).unwrap();
        assert!(save_document_with_cancel(&doc, &target, true, &|| true).is_err());
        assert_eq!(std::fs::read(&target).unwrap(), original);
    }

    #[test]
    fn cached_preview_cancellation_does_not_change_document() {
        let dir = tempfile::TempDir::new().unwrap();
        let base = dir.path().join("base.png");
        RgbaImage::from_pixel(32, 32, Rgba([1, 2, 3, 255]))
            .save(&base)
            .unwrap();
        let doc = new_document(&base, &dir.path().join("x.json")).unwrap();
        let cache = prepare_render_cache(&doc, &|| false).unwrap();
        let state = doc.state();
        assert!(
            render_document_cached(&doc, Some(16), &cache, &|| true)
                .unwrap_err()
                .contains("cancelled")
        );
        assert_eq!(doc.state(), state);
        assert_eq!(
            render_document_cached(&doc, Some(16), &cache, &|| false)
                .unwrap()
                .dimensions(),
            (16, 16)
        );
    }
}

#[cfg(test)]
mod source_path_regressions {
    use super::*;
    use image::Rgba;

    #[test]
    fn imported_artwork_source_is_protected_and_deleted_source_does_not_block_relocated_export() {
        let dir = tempfile::TempDir::new().unwrap();
        let original = dir.path().join("original");
        let relocated = dir.path().join("relocated");
        std::fs::create_dir(&original).unwrap();
        std::fs::create_dir(&relocated).unwrap();
        let base = original.join("base.png");
        let art = original.join("art.png");
        RgbaImage::from_pixel(2, 2, Rgba([1, 2, 3, 255]))
            .save(&base)
            .unwrap();
        RgbaImage::from_pixel(1, 1, Rgba([220, 2, 3, 255]))
            .save(&art)
            .unwrap();
        let project = original.join("work.json");
        let mut doc = new_document(&base, &project).unwrap();
        doc.add_image_layer(&art).unwrap();
        let art_before = std::fs::read(&art).unwrap();
        assert!(
            export_png_with_overwrite(&doc, &art, true)
                .unwrap_err()
                .contains("imported source")
        );
        assert_eq!(std::fs::read(&art).unwrap(), art_before);
        save_document(&doc, &project, false).unwrap();
        let moved = relocated.join("work.json");
        std::fs::copy(&project, &moved).unwrap();
        std::fs::remove_dir_all(&original).unwrap();
        let reopened = load_document(&moved).unwrap();
        let output = relocated.join("export.png");
        export_png(&reopened, &moved, &output).unwrap();
        assert!(output.is_file());
    }
}

#[cfg(test)]
mod file_size_regression {
    use super::*;

    #[test]
    fn oversize_png_file_rejected_by_metadata_before_read() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("oversize.png");
        let file = std::fs::File::create(&path).unwrap();
        file.set_len(MAX_PNG_BYTES as u64 + 1).unwrap();
        let error = new_document(&path, &dir.path().join("project.json")).unwrap_err();
        assert!(error.contains("32 MiB"));
    }
}

#[cfg(test)]
mod asymmetric_rotation_regression {
    use super::*;
    use image::Rgba;

    #[test]
    fn cached_renderer_and_export_preserve_rotated_corner_pixels() {
        let dir = tempfile::TempDir::new().unwrap();
        let base = dir.path().join("base.png");
        let art = dir.path().join("art.png");
        RgbaImage::from_pixel(5, 5, Rgba([10, 20, 30, 77]))
            .save(&base)
            .unwrap();
        let mut artwork = RgbaImage::new(3, 3);
        artwork.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        artwork.put_pixel(2, 0, Rgba([0, 255, 0, 255]));
        artwork.put_pixel(0, 2, Rgba([0, 0, 255, 255]));
        artwork.put_pixel(2, 2, Rgba([255, 255, 0, 255]));
        artwork.save(&art).unwrap();
        let project = dir.path().join("work.json");
        let mut doc = new_document(&base, &project).unwrap();
        let i = doc.add_image_layer(&art).unwrap();
        if let LayerContent::Image {
            x, y, rotation_deg, ..
        } = &mut doc.layers[i].content
        {
            *x = 1.0;
            *y = 1.0;
            *rotation_deg = 90.0;
        }
        let cache = prepare_render_cache(&doc, &|| false).unwrap();
        let rendered = render_document_cached(&doc, None, &cache, &|| false).unwrap();
        for (x, y, expected) in [
            (1, 1, [0, 0, 255, 77]),
            (3, 1, [255, 0, 0, 77]),
            (1, 3, [255, 255, 0, 77]),
            (3, 3, [0, 255, 0, 77]),
            (2, 2, [10, 20, 30, 77]),
        ] {
            assert_eq!(rendered.get_pixel(x, y).0, expected);
        }
        save_document(&doc, &project, false).unwrap();
        let reopened = load_document(&project).unwrap();
        let output = dir.path().join("output.png");
        export_png(&reopened, &project, &output).unwrap();
        assert_eq!(image::open(&output).unwrap().to_rgba8(), rendered);
    }
}

#[cfg(test)]
mod old_project_source_guard {
    use super::*;
    use image::Rgba;

    #[test]
    fn content_hash_protects_source_png_without_origin_metadata() {
        let dir = tempfile::TempDir::new().unwrap();
        let base = dir.path().join("source.png");
        RgbaImage::from_pixel(2, 2, Rgba([1, 2, 3, 255]))
            .save(&base)
            .unwrap();
        let project = dir.path().join("project.json");
        let mut doc = new_document(&base, &project).unwrap();
        doc.source_origin = None;
        for asset in doc.assets.values_mut() {
            asset.source_origin = None;
        }
        save_document(&doc, &project, false).unwrap();
        let reopened = load_document(&project).unwrap();
        let original = std::fs::read(&base).unwrap();
        let error = export_png_with_overwrite(&reopened, &base, true).unwrap_err();
        assert!(error.contains("embedded source image"), "{error}");
        assert_eq!(std::fs::read(&base).unwrap(), original);
    }
}

#[cfg(test)]
mod channel_reference_tests {
    use super::*;
    use image::{ImageReader, Rgba};

    fn pixels(path: &Path, values: &[[u8; 4]]) {
        let mut image = RgbaImage::new(values.len() as u32, 1);
        for (x, rgba) in values.iter().enumerate() {
            image.put_pixel(x as u32, 0, Rgba(*rgba));
        }
        image.save(path).unwrap();
    }

    #[test]
    fn alpha_data_is_exact_and_invert_is_undoable() {
        let dir = tempfile::TempDir::new().unwrap();
        let base = dir.path().join("base.png");
        let alpha = dir.path().join("alpha.png");
        pixels(&base, &[[20, 30, 40, 5], [50, 60, 70, 230]]);
        pixels(&alpha, &[[30, 30, 30, 255], [201, 201, 201, 255]]);
        let mut doc = new_document(&base, &dir.path().join("project.json")).unwrap();
        doc.add_color_layer("Finish", [200, 100, 40], 128).unwrap();
        doc.set_alpha_data(&alpha).unwrap();
        assert_eq!(doc.alpha_mode, AlphaMode::ImportedData);
        let before = doc.state();
        let output = render_document(&doc, None).unwrap();
        assert_eq!(output.get_pixel(0, 0).0[3], 30);
        assert_eq!(output.get_pixel(1, 0).0[3], 201);
        doc.alpha_data.as_mut().unwrap().invert = true;
        let inverted = render_document(&doc, None).unwrap();
        assert_eq!(inverted.get_pixel(0, 0).0[3], 225);
        assert_eq!(inverted.get_pixel(1, 0).0[3], 54);
        doc.restore(before);
        assert_eq!(render_document(&doc, None).unwrap(), output);
    }

    #[test]
    fn invalid_channel_inputs_are_transactional() {
        let dir = tempfile::TempDir::new().unwrap();
        let base = dir.path().join("base.png");
        pixels(&base, &[[1, 2, 3, 255], [4, 5, 6, 255]]);
        let mut doc = new_document(&base, &dir.path().join("work.json")).unwrap();
        let original = doc.clone();
        for bad in [[40, 39, 40, 255], [40, 40, 40, 200]] {
            let path = dir.path().join(format!("bad-{}.png", bad[3]));
            pixels(&path, &[bad, bad]);
            assert!(
                doc.set_alpha_data(&path)
                    .unwrap_err()
                    .contains("opaque grayscale")
            );
            assert_eq!(doc, original);
        }
        let mismatch = dir.path().join("wrong-size.png");
        pixels(&mismatch, &[[1, 1, 1, 255]]);
        assert!(
            doc.set_alpha_data(&mismatch)
                .unwrap_err()
                .contains("canvas dimensions")
        );
        assert!(
            doc.set_uv_reference(&mismatch)
                .unwrap_err()
                .contains("canvas dimensions")
        );
        assert_eq!(doc, original);
        doc.alpha_mode = AlphaMode::ImportedData;
        assert!(
            validate_edit_state(&doc)
                .unwrap_err()
                .contains("requires an alpha data")
        );
    }

    #[test]
    fn portable_uv_preview_never_enters_export() {
        let dir = tempfile::TempDir::new().unwrap();
        let source = dir.path().join("source");
        let moved = dir.path().join("moved");
        std::fs::create_dir(&source).unwrap();
        std::fs::create_dir(&moved).unwrap();
        let base = source.join("base.png");
        let alpha = source.join("data.png");
        let guide = source.join("guide.png");
        pixels(&base, &[[30, 50, 70, 255], [30, 50, 70, 255]]);
        pixels(&alpha, &[[10, 10, 10, 255], [240, 240, 240, 255]]);
        pixels(&guide, &[[255, 0, 0, 255], [0, 255, 0, 0]]);
        let project = source.join("finish.json");
        let mut doc = new_document(&base, &project).unwrap();
        doc.set_alpha_data(&alpha).unwrap();
        doc.set_uv_reference(&guide).unwrap();
        doc.style_intent = FinishStyleIntent::CustomPaint;
        save_document(&doc, &project, false).unwrap();
        std::fs::rename(&project, moved.join("finish.json")).unwrap();
        std::fs::remove_dir_all(&source).unwrap();
        let relocated = moved.join("finish.json");
        let reopened = load_document(&relocated).unwrap();
        assert_eq!(reopened.style_intent, FinishStyleIntent::CustomPaint);
        let cache = prepare_render_cache(&reopened, &|| false).unwrap();
        let clean = render_document_cached(&reopened, None, &cache, &|| false).unwrap();
        let mut preview = clean.clone();
        apply_uv_reference_cached(&reopened, &mut preview, &cache, &|| false).unwrap();
        assert_ne!(preview.get_pixel(0, 0), clean.get_pixel(0, 0));
        assert_eq!(preview.get_pixel(1, 0), clean.get_pixel(1, 0));
        let output = moved.join("export.png");
        export_png(&reopened, &relocated, &output).unwrap();
        let decoded = ImageReader::open(&output)
            .unwrap()
            .decode()
            .unwrap()
            .to_rgba8();
        assert_eq!(decoded, clean);
        assert_eq!(decoded.get_pixel(0, 0).0[3], 10);
        assert_eq!(decoded.get_pixel(1, 0).0[3], 240);
    }
}

#[cfg(test)]
mod schema_compatibility_tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn older_v2_project_without_channel_reference_or_lock_fields_reopens() {
        let dir = tempfile::TempDir::new().unwrap();
        let base = dir.path().join("base.png");
        RgbaImage::from_pixel(2, 2, Rgba([1, 2, 3, 255]))
            .save(&base)
            .unwrap();
        let project = dir.path().join("old-v2.json");
        let mut doc = new_document(&base, &project).unwrap();
        doc.add_color_layer("Color", [20, 30, 40], 255).unwrap();
        let mut value = serde_json::to_value(&doc).unwrap();
        let object = value.as_object_mut().unwrap();
        for field in ["alpha_data", "uv_reference", "style_intent"] {
            object.remove(field);
        }
        object["layers"][0]
            .as_object_mut()
            .unwrap()
            .remove("locked");
        std::fs::write(&project, serde_json::to_vec(&value).unwrap()).unwrap();
        let reopened = load_document(&project).unwrap();
        assert_eq!(reopened.alpha_mode, AlphaMode::PreserveBase);
        assert!(reopened.alpha_data.is_none());
        assert!(reopened.uv_reference.is_none());
        assert_eq!(reopened.style_intent, FinishStyleIntent::Unspecified);
        assert!(!reopened.layers[0].locked);
        assert_eq!(
            render_document(&reopened, None).unwrap(),
            render_document(&doc, None).unwrap()
        );
    }
}

#[cfg(test)]
mod channel_source_guard_tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn alpha_and_uv_source_files_cannot_be_export_destinations() {
        let dir = tempfile::TempDir::new().unwrap();
        let base = dir.path().join("base.png");
        let alpha = dir.path().join("alpha.png");
        let guide = dir.path().join("guide.png");
        RgbaImage::from_pixel(2, 2, Rgba([20, 30, 40, 255]))
            .save(&base)
            .unwrap();
        RgbaImage::from_pixel(2, 2, Rgba([128, 128, 128, 255]))
            .save(&alpha)
            .unwrap();
        RgbaImage::from_pixel(2, 2, Rgba([255, 0, 0, 150]))
            .save(&guide)
            .unwrap();
        let mut doc = new_document(&base, &dir.path().join("work.json")).unwrap();
        doc.set_alpha_data(&alpha).unwrap();
        doc.set_uv_reference(&guide).unwrap();
        for source in [&alpha, &guide] {
            let original = std::fs::read(source).unwrap();
            let error = export_png_with_overwrite(&doc, source, true).unwrap_err();
            assert!(error.contains("imported source"), "{error}");
            assert_eq!(std::fs::read(source).unwrap(), original);
        }
    }
}

#[cfg(test)]
mod pixel_budget_tests {
    use super::*;
    use image::Rgba;

    fn fake_4k_header() -> Arc<Vec<u8>> {
        let mut bytes = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
        bytes.extend_from_slice(&4096u32.to_be_bytes());
        bytes.extend_from_slice(&4096u32.to_be_bytes());
        Arc::new(bytes)
    }

    #[test]
    fn budget_accounts_for_full_canvas_base_alpha_uv_and_art() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("small.png");
        RgbaImage::from_pixel(1, 1, Rgba([1, 2, 3, 255]))
            .save(&path)
            .unwrap();
        let mut doc = new_document(&path, &dir.path().join("work.json")).unwrap();
        doc.width = 4096;
        doc.height = 4096;
        doc.assets.get_mut("base").unwrap().png = fake_4k_header();
        for id in ["alpha", "uv", "art", "extra"] {
            doc.assets.insert(
                id.into(),
                Asset {
                    name: format!("{id}.png"),
                    source_origin: None,
                    sha256: "budget-only".into(),
                    png: fake_4k_header(),
                },
            );
        }
        doc.alpha_data = Some(AlphaData {
            asset_id: "alpha".into(),
            invert: false,
        });
        doc.uv_reference = Some(UvReference {
            asset_id: "uv".into(),
            visible: true,
            opacity: 128,
        });
        doc.alpha_mode = AlphaMode::ImportedData;
        doc.layers.push(Layer {
            name: "Art".into(),
            visible: true,
            locked: false,
            opacity: 255,
            content: LayerContent::Image {
                asset_id: "art".into(),
                x: 0.0,
                y: 0.0,
                scale: 1.0,
                rotation_deg: 0.0,
            },
        });
        assert!(validate_edit_state(&doc).is_ok());
        doc.layers.push(Layer {
            name: "Extra".into(),
            visible: true,
            locked: false,
            opacity: 255,
            content: LayerContent::Image {
                asset_id: "extra".into(),
                x: 0.0,
                y: 0.0,
                scale: 1.0,
                rotation_deg: 0.0,
            },
        });
        assert!(
            validate_edit_state(&doc)
                .unwrap_err()
                .contains("64 megapixels")
        );
    }
}
