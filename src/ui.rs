use crate::document::{FinishDocument, export_png, load_document, new_document, save_document};
use crate::{TextureReport, inspect_png, write_new_report};
use eframe::egui;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct SkinStrikeApp {
    pub source: Option<PathBuf>,
    pub report: Option<TextureReport>,
    pub output_path: String,
    pub document: Option<FinishDocument>,
    pub project_path: String,
    pub export_path: String,
    pub layer_name: String,
    pub layer_rgb: [u8; 3],
    pub layer_opacity: u8,
    pub status: Option<String>,
    pub error: Option<String>,
}

impl SkinStrikeApp {
    pub fn open_texture(&mut self, path: &Path) -> Result<(), String> {
        match inspect_png(path) {
            Ok(report) => {
                let project_path = path.with_extension("skinstrike.json");
                let document = match new_document(path, &project_path) {
                    Ok(document) => document,
                    Err(error) => { self.error = Some(error.clone()); return Err(error); }
                };
                self.source = Some(path.to_path_buf());
                self.output_path = path
                    .with_extension("texture-report.json")
                    .display()
                    .to_string();
                self.project_path = project_path.display().to_string();
                self.export_path = path
                    .with_extension("finish-export.png")
                    .display()
                    .to_string();
                self.document = Some(document);
                self.layer_name = "Color wash".into();
                self.layer_rgb = [200, 80, 80];
                self.layer_opacity = 128;
                self.report = Some(report);
                self.status = Some(
                    "PNG decoded and measured. Valve material output is not built yet.".into(),
                );
                self.error = None;
                Ok(())
            }
            Err(error) => {
                self.error = Some(error.clone());
                Err(error)
            }
        }
    }

    pub fn save_report(&mut self) -> Result<(), String> {
        let report = self
            .report
            .as_ref()
            .ok_or_else(|| "open a PNG first".to_string())?;
        match write_new_report(report, Path::new(&self.output_path)) {
            Ok(()) => {
                self.status = Some(format!("Report saved: {}", self.output_path));
                self.error = None;
                Ok(())
            }
            Err(error) => {
                self.error = Some(error.clone());
                Err(error)
            }
        }
    }

    pub fn open_project(&mut self, path: &Path) -> Result<(), String> {
        match load_document(path) {
            Ok(document) => {
                self.project_path = path.display().to_string();
                self.export_path = path.with_extension("export.png").display().to_string();
                self.document = Some(document);
                self.error = None;
                self.status = Some("Finish project reopened".into());
                Ok(())
            }
            Err(error) => {
                self.error = Some(error.clone());
                Err(error)
            }
        }
    }

    pub fn add_layer(&mut self) -> Result<(), String> {
        let result = self.document.as_mut()
            .ok_or_else(|| "open a PNG or finish project first".to_string())
            .and_then(|document| {
                document.add_color_layer(&self.layer_name, self.layer_rgb, self.layer_opacity)?;
                Ok(document.layers.len())
            });
        match result {
            Ok(count) => {
                self.status = Some(format!("{count} color layers"));
                self.error = None;
                Ok(())
            }
            Err(error) => { self.error = Some(error.clone()); Err(error) }
        }
    }

    pub fn save_project(&mut self) -> Result<(), String> {
        let document = self
            .document
            .as_ref()
            .ok_or_else(|| "open a PNG or finish project first".to_string())?;
        let path = Path::new(&self.project_path);
        match save_document(document, path, path.exists()) {
            Ok(()) => {
                self.status = Some(format!("Finish project saved: {}", self.project_path));
                self.error = None;
                Ok(())
            }
            Err(error) => {
                self.error = Some(error.clone());
                Err(error)
            }
        }
    }

    pub fn export_finish(&mut self) -> Result<(), String> {
        let document = self
            .document
            .as_ref()
            .ok_or_else(|| "open a PNG or finish project first".to_string())?;
        match export_png(
            document,
            Path::new(&self.project_path),
            Path::new(&self.export_path),
        ) {
            Ok(()) => {
                self.status = Some(format!("Finish PNG exported: {}", self.export_path));
                self.error = None;
                Ok(())
            }
            Err(error) => {
                self.error = Some(error.clone());
                Err(error)
            }
        }
    }
}

impl eframe::App for SkinStrikeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("SkinStrike");
            ui.label("Weapon-finish color layers and texture inspection");
            if ui.button("Open PNG texture").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("PNG texture", &["png"])
                    .pick_file()
                {
                    let _ = self.open_texture(&path);
                }
            }
            if ui.button("Open finish project").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Finish document", &["json"])
                    .pick_file()
                {
                    let _ = self.open_project(&path);
                }
            }
            if let Some(error) = &self.error {
                ui.colored_label(egui::Color32::LIGHT_RED, error);
            }
            if let Some(status) = &self.status {
                ui.colored_label(egui::Color32::LIGHT_GREEN, status);
            }
            if let Some(report) = &self.report {
                ui.separator();
                if let Some(source) = &self.source {
                    ui.small(source.display().to_string());
                }
                ui.label(format!("{} × {} pixels", report.width, report.height));
                ui.label(format!(
                    "Opaque: {} · partial alpha: {} · transparent: {}",
                    report.fully_opaque_pixels,
                    report.partially_transparent_pixels,
                    report.fully_transparent_pixels
                ));
                ui.small(format!("SHA-256: {}", report.source_sha256));
                ui.horizontal(|ui| {
                    ui.label("New report");
                    ui.text_edit_singleline(&mut self.output_path);
                });
                if ui.button("Save inspection report").clicked() {
                    let _ = self.save_report();
                }
            }
            if let Some(document) = &self.document {
                ui.separator();
                ui.heading("Color layers");
                ui.small(format!("Base texture: {}", document.base_png));
                for layer in &document.layers {
                    ui.label(format!(
                        "{} · RGB {:?} · opacity {}",
                        layer.name, layer.rgb, layer.opacity
                    ));
                }
                ui.horizontal(|ui| {
                    ui.label("Name");
                    ui.text_edit_singleline(&mut self.layer_name);
                    ui.color_edit_button_srgb(&mut self.layer_rgb);
                    ui.add(egui::Slider::new(&mut self.layer_opacity, 0..=255).text("Opacity"));
                });
                if ui.button("Add color layer").clicked() {
                    let _ = self.add_layer();
                }
                ui.horizontal(|ui| {
                    ui.label("Finish project");
                    ui.text_edit_singleline(&mut self.project_path);
                });
                if ui.button("Save finish project").clicked() {
                    let _ = self.save_project();
                }
                ui.horizontal(|ui| {
                    ui.label("Export PNG");
                    ui.text_edit_singleline(&mut self.export_path);
                });
                if ui.button("Export layered finish").clicked() {
                    let _ = self.export_finish();
                }
                ui.small("Export is a layered PNG preview, not a Valve material package.");
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::SkinStrikeApp;
    use image::{Rgba, RgbaImage};
    use std::path::Path;

    #[test]
    fn view_model_opens_texture_and_saves_report() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("finish.png");
        RgbaImage::from_pixel(2, 2, Rgba([1, 2, 3, 255]))
            .save(&path)
            .unwrap();
        let mut app = SkinStrikeApp::default();
        app.open_texture(&path).unwrap();
        assert_eq!(app.report.as_ref().unwrap().pixels, 4);
        app.save_report().unwrap();
        assert!(Path::new(&app.output_path).is_file());
        app.add_layer().unwrap();
        app.save_project().unwrap();
        app.open_project(&Path::new(&app.project_path).to_path_buf())
            .unwrap();
        assert_eq!(app.document.as_ref().unwrap().layers.len(), 1);
        app.layer_name.clear();
        assert!(app.add_layer().is_err());
        assert!(app.error.as_ref().unwrap().contains("layer name"));
        assert_eq!(app.document.as_ref().unwrap().layers.len(), 1);
        app.export_finish().unwrap();
        assert!(Path::new(&app.export_path).is_file());
    }
}
