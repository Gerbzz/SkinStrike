use image::{ImageReader, Rgba, RgbaImage};
use skinstrike::document::{
    FinishStyleIntent, LayerContent, apply_uv_reference_cached, export_png, load_document,
    new_document, prepare_render_cache, render_document_cached, save_document,
};
use std::path::PathBuf;

fn main() -> Result<(), String> {
    let root = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("Usage: cargo run --example channel_uv_smoke -- <new-output-directory>")?;
    if root.exists() {
        return Err("Output directory already exists; choose a new directory".into());
    }
    let sources = root.join("temporary-inputs");
    let portable = root.join("portable-project");
    std::fs::create_dir_all(&sources).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&portable).map_err(|e| e.to_string())?;
    let base_path = sources.join("base.png");
    let art_path = sources.join("decal.png");
    let alpha_path = sources.join("alpha-data.png");
    let uv_path = sources.join("uv-guide.png");
    let original_project = sources.join("finish.skinstrike.json");
    let relocated_project = portable.join("finish.skinstrike.json");
    let exported = portable.join("finish-export.png");
    let preview_path = portable.join("editor-preview-offscreen.png");

    let mut base = RgbaImage::new(256, 256);
    let mut alpha = RgbaImage::new(256, 256);
    let mut guide = RgbaImage::new(256, 256);
    for (x, y, pixel) in base.enumerate_pixels_mut() {
        *pixel = Rgba([45 + (x / 4) as u8, 62 + (y / 6) as u8, 96, 255]);
        let value = if x < 85 {
            32
        } else if x < 170 {
            128
        } else {
            224
        };
        alpha.put_pixel(x, y, Rgba([value, value, value, 255]));
        let line = x == 8 || y == 8 || x % 64 == 0 || y % 64 == 0;
        guide.put_pixel(x, y, Rgba([255, 220, 40, if line { 220 } else { 0 }]));
    }
    let mut decal = RgbaImage::new(64, 64);
    for (x, y, pixel) in decal.enumerate_pixels_mut() {
        let dx = x as f32 - 31.5;
        let dy = y as f32 - 31.5;
        let radius = (dx * dx + dy * dy).sqrt();
        *pixel = Rgba([210, 52, 68, if radius < 27.0 { 230 } else { 0 }]);
    }
    base.save(&base_path).map_err(|e| e.to_string())?;
    alpha.save(&alpha_path).map_err(|e| e.to_string())?;
    guide.save(&uv_path).map_err(|e| e.to_string())?;
    decal.save(&art_path).map_err(|e| e.to_string())?;

    let mut project = new_document(&base_path, &original_project)?;
    project.add_color_layer("Blue tint", [58, 108, 165], 80)?;
    let index = project.add_image_layer(&art_path)?;
    project.layers[index].name = "Red emblem".into();
    if let LayerContent::Image {
        x, y, rotation_deg, ..
    } = &mut project.layers[index].content
    {
        *x = 118.0;
        *y = 88.0;
        *rotation_deg = 25.0;
    }
    project.set_alpha_data(&alpha_path)?;
    project.set_uv_reference(&uv_path)?;
    project.style_intent = FinishStyleIntent::CustomPaint;
    save_document(&project, &original_project, false)?;
    std::fs::rename(&original_project, &relocated_project).map_err(|e| e.to_string())?;
    std::fs::remove_dir_all(&sources).map_err(|e| e.to_string())?;

    let reopened = load_document(&relocated_project)?;
    let cache = prepare_render_cache(&reopened, &|| false)?;
    let clean = render_document_cached(&reopened, None, &cache, &|| false)?;
    export_png(&reopened, &relocated_project, &exported)?;
    let decoded = ImageReader::open(&exported)
        .map_err(|e| e.to_string())?
        .decode()
        .map_err(|e| e.to_string())?
        .to_rgba8();
    if decoded != clean
        || decoded.get_pixel(20, 50).0[3] != 32
        || decoded.get_pixel(120, 50).0[3] != 128
        || decoded.get_pixel(220, 50).0[3] != 224
    {
        return Err("Decoded export pixels or exact alpha data did not match".into());
    }
    let mut preview = render_document_cached(&reopened, Some(1024), &cache, &|| false)?;
    for pixel in preview.pixels_mut() {
        pixel.0[3] = 255;
    }
    apply_uv_reference_cached(&reopened, &mut preview, &cache, &|| false)?;
    preview.save(&preview_path).map_err(|e| e.to_string())?;
    if preview.get_pixel(8, 50).0 == decoded.get_pixel(8, 50).0 {
        return Err("Editor UV reference was not visible in the offscreen preview".into());
    }
    println!("Portable project: {}", relocated_project.display());
    println!("Decoded export: {}", exported.display());
    println!(
        "Editor preview (offscreen render, not native GUI): {}",
        preview_path.display()
    );
    Ok(())
}
