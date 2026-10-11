use image::{Rgba, RgbaImage};
use skinstrike::document::{LayerContent, export_png, load_document, new_document, save_document};
use std::path::PathBuf;

fn main() -> Result<(), String> {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("Usage: cargo run --example authoring_smoke -- <new-output-directory>")?;
    if output.exists() {
        return Err("Output directory already exists; choose a new one".into());
    }
    std::fs::create_dir(&output).map_err(|e| e.to_string())?;
    let base_path = output.join("synthetic-base.png");
    let decal_path = output.join("synthetic-artwork.png");
    let project_path = output.join("synthetic.skinstrike.json");
    let export_path = output.join("synthetic-export.png");

    let mut base = RgbaImage::new(512, 512);
    for (x, y, pixel) in base.enumerate_pixels_mut() {
        let grid = x % 64 < 2 || y % 64 < 2;
        let gray = if grid { 90 } else { 38 + (x / 10) as u8 };
        *pixel = Rgba([
            gray,
            gray.saturating_add(8),
            gray.saturating_add(15),
            if x < 28 || y < 28 { 90 } else { 255 },
        ]);
    }
    base.save(&base_path).map_err(|e| e.to_string())?;
    let mut decal = RgbaImage::new(128, 128);
    for (x, y, pixel) in decal.enumerate_pixels_mut() {
        let dx = x as f32 - 63.5;
        let dy = y as f32 - 63.5;
        let radius = (dx * dx + dy * dy).sqrt();
        let alpha = ((65.0 - radius) * 18.0).clamp(0.0, 255.0) as u8;
        *pixel = Rgba([239, 92, 53, alpha]);
    }
    decal.save(&decal_path).map_err(|e| e.to_string())?;
    let mut project = new_document(&base_path, &project_path)?;
    project.add_color_layer("Warm finish", [137, 79, 72], 48)?;
    let index = project.add_image_layer(&decal_path)?;
    project.layers[index].name = "Rotated emblem".into();
    if let LayerContent::Image {
        x,
        y,
        scale,
        rotation_deg,
        ..
    } = &mut project.layers[index].content
    {
        *x = 145.0;
        *y = 150.0;
        *scale = 1.7;
        *rotation_deg = 18.0;
    }
    save_document(&project, &project_path, false)?;
    let reopened = load_document(&project_path)?;
    export_png(&reopened, &project_path, &export_path)?;
    println!("Project: {}", project_path.display());
    println!("PNG: {}", export_path.display());
    Ok(())
}
