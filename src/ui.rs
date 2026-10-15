use crate::document::{
    AlphaMode, DocumentState, FinishDocument, FinishStyleIntent, LayerContent, PaintStroke,
    RenderCache, apply_uv_reference_cached, export_png_with_cache, load_document, new_document,
    prepare_render_cache, render_document_cached, save_document, save_document_with_cancel,
    validate_export_destination_quick,
};
use crate::project_library::{
    GalleryPage, ProjectLibrary, read_library_preference, write_library_preference,
};
use eframe::egui::{self, Color32, ColorImage, RichText, TextureHandle, TextureOptions};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ViewChannel {
    Rgb,
    Rgba,
    Alpha,
}
fn render_preview_image(
    document: &FinishDocument,
    cache: &RenderCache,
    view: ViewChannel,
    cancelled: &dyn Fn() -> bool,
) -> Result<ColorImage, String> {
    let mut image = render_document_cached(document, Some(1024), cache, cancelled)?;
    for pixel in image.pixels_mut() {
        match view {
            ViewChannel::Rgb => pixel.0[3] = 255,
            ViewChannel::Rgba => {}
            ViewChannel::Alpha => {
                let alpha = pixel.0[3];
                pixel.0 = [alpha, alpha, alpha, 255];
            }
        }
    }
    if view != ViewChannel::Alpha {
        apply_uv_reference_cached(document, &mut image, cache, cancelled)?;
    }
    Ok(ColorImage::from_rgba_unmultiplied(
        [image.width() as usize, image.height() as usize],
        image.as_raw(),
    ))
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Tool {
    Move,
    Pan,
    Brush,
}
#[derive(Clone, Copy)]
enum Pending {
    New,
    Open,
    LibraryOpen,
    Close,
}
#[derive(Clone, Copy)]
enum LayerAction {
    Up,
    Down,
    Duplicate,
    Delete,
}

#[derive(Clone, Copy)]
enum AuxKind {
    Alpha,
    UvReference,
}

enum IoRequest {
    New(PathBuf),
    Open(PathBuf),
    Import(PathBuf),
    ImportAux(PathBuf, AuxKind),
    Save(PathBuf),
    SaveToLibrary(PathBuf, String, String),
}

enum IoOutput {
    Opened {
        document: FinishDocument,
        cache: RenderCache,
        path: Option<PathBuf>,
        migrated: bool,
    },
    Imported {
        document: FinishDocument,
        cache: RenderCache,
        selected: usize,
        path: PathBuf,
    },
    ImportedAux {
        document: FinishDocument,
        cache: RenderCache,
        kind: AuxKind,
        path: PathBuf,
    },
    Saved {
        path: PathBuf,
        state: DocumentState,
        project_id: String,
    },
}

struct IoJob {
    receiver: Receiver<Result<IoOutput, String>>,
    cancel: Arc<AtomicBool>,
    edit_revision: u64,
}

struct PreviewJob {
    receiver: Receiver<Result<ColorImage, String>>,
    cancel: Arc<AtomicBool>,
    preview_revision: u64,
}
struct GalleryJob {
    receiver: Receiver<Result<(Vec<String>, GalleryPage), String>>,
    generation: u64,
}
pub struct SkinStrikeApp {
    document: Option<FinishDocument>,
    cache: Option<RenderCache>,
    edit_revision: u64,
    preview_revision: u64,
    io_job: Option<IoJob>,
    preview_job: Option<PreviewJob>,
    continue_after_save: Option<Pending>,
    project_path: Option<PathBuf>,
    saved_state: Option<DocumentState>,
    undo: Vec<DocumentState>,
    drag_before: Option<DocumentState>,
    paint_points: Vec<[f32; 2]>,
    paint_layer: Option<usize>,
    brush_color: [u8; 3],
    brush_radius: f32,
    redo: Vec<DocumentState>,
    selected: Option<usize>,
    texture: Option<TextureHandle>,
    preview_dirty_at: Option<Instant>,
    zoom: f32,
    pan: egui::Vec2,
    tool: Tool,
    view: ViewChannel,
    pending: Option<Pending>,
    allow_close: bool,
    confirm_export: Option<PathBuf>,
    export_job: Option<Receiver<Result<PathBuf, String>>>,
    export_cancel: Option<Arc<AtomicBool>>,
    export_revision: Option<u64>,
    status: String,
    error: Option<String>,
    library_root: Option<PathBuf>,
    gallery_open: bool,
    gallery_job: Option<GalleryJob>,
    gallery_page: Option<GalleryPage>,
    gallery_textures: Vec<Option<TextureHandle>>,
    gallery_collections: Vec<String>,
    gallery_collection: Option<String>,
    gallery_query: String,
    gallery_offset: usize,
    gallery_generation: u64,
    gallery_dirty_at: Option<Instant>,
    library_target: Option<PathBuf>,
    new_collection: String,
    save_to_library: bool,
    library_title: String,
    library_save_collection: String,
    logo_texture: Option<TextureHandle>,
    show_about: bool,
    style_ready: bool,
}

fn preference_path() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(|root| {
        PathBuf::from(root)
            .join("SkinStrike")
            .join("library-root.txt")
    })
}

impl Default for SkinStrikeApp {
    fn default() -> Self {
        let library_root =
            preference_path().and_then(|path| read_library_preference(&path).ok().flatten());
        Self {
            document: None,
            cache: None,
            edit_revision: 0,
            preview_revision: 0,
            io_job: None,
            preview_job: None,
            continue_after_save: None,
            project_path: None,
            saved_state: None,
            undo: Vec::new(),
            drag_before: None,
            paint_points: Vec::new(),
            paint_layer: None,
            brush_color: [240, 123, 54],
            brush_radius: 12.0,
            redo: Vec::new(),
            selected: None,
            texture: None,
            preview_dirty_at: None,
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
            tool: Tool::Move,
            view: ViewChannel::Rgb,
            pending: None,
            allow_close: false,
            confirm_export: None,
            export_job: None,
            export_cancel: None,
            export_revision: None,
            status: "Open a base PNG or a SkinStrike project to begin.".into(),
            error: None,
            library_root,
            gallery_open: true,
            gallery_job: None,
            gallery_page: None,
            gallery_textures: Vec::new(),
            gallery_collections: Vec::new(),
            gallery_collection: None,
            gallery_query: String::new(),
            gallery_offset: 0,
            gallery_generation: 0,
            gallery_dirty_at: Some(Instant::now()),
            library_target: None,
            new_collection: String::new(),
            save_to_library: false,
            library_title: String::new(),
            library_save_collection: String::new(),
            logo_texture: None,
            show_about: false,
            style_ready: false,
        }
    }
}

impl SkinStrikeApp {
    /// Allow a one-shot capture window to close its synthetic unsaved document.
    pub fn allow_capture_close(&mut self) {
        self.allow_close = true;
    }

    fn dirty(&self) -> bool {
        match (&self.document, &self.saved_state) {
            (Some(doc), Some(saved)) => doc.state() != *saved,
            (Some(_), None) => true,
            _ => false,
        }
    }

    fn current_project_title(&self) -> String {
        let filename = self
            .project_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("Untitled finish");
        let stem = filename
            .strip_suffix(".skinstrike.json")
            .or_else(|| filename.strip_suffix(".json"))
            .unwrap_or(filename);
        stem.rsplit_once(" -- ")
            .and_then(|(title, id)| {
                (id.len() == 16 && id.chars().all(|c| c.is_ascii_hexdigit())).then_some(title)
            })
            .unwrap_or(stem)
            .to_string()
    }

    fn install_prepared(
        &mut self,
        document: FinishDocument,
        path: Option<PathBuf>,
        cache: RenderCache,
    ) {
        let dims = (document.width, document.height);
        self.saved_state = path.as_ref().map(|_| document.state());
        self.project_path = path;
        self.document = Some(document);
        self.gallery_open = false;
        self.texture = None;
        self.cache = Some(cache);
        self.selected = None;
        self.undo.clear();
        self.drag_before = None;
        self.paint_points.clear();
        self.paint_layer = None;
        self.redo.clear();
        self.zoom = 1.0;
        self.pan = egui::Vec2::ZERO;
        self.error = None;
        self.edit_revision += 1;
        self.mark_preview();
        self.status = format!(
            "Canvas {} x {} ready. Artwork is embedded when saved.",
            dims.0, dims.1
        );
    }

    fn install(&mut self, document: FinishDocument, path: Option<PathBuf>) -> Result<(), String> {
        let cache = prepare_render_cache(&document, &|| false)?;
        self.install_prepared(document, path, cache);
        Ok(())
    }

    pub fn open_texture(&mut self, path: &Path) -> Result<(), String> {
        let document = new_document(path, path)?;
        self.install(document, None)
    }

    /// Open a chosen library for native review without changing the saved folder preference.
    pub fn preview_library_at(&mut self, root: &Path) -> Result<(), String> {
        let library = ProjectLibrary::open(root)?;
        self.library_root = Some(library.root().to_path_buf());
        self.gallery_open = true;
        self.gallery_page = None;
        self.gallery_collection = None;
        self.gallery_offset = 0;
        self.schedule_gallery_refresh();
        Ok(())
    }

    pub fn open_project(&mut self, path: &Path) -> Result<(), String> {
        let document = load_document(path)?;
        let migrated = std::fs::read(path)
            .ok()
            .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
            .and_then(|v| v.get("schema_version").and_then(|n| n.as_u64()))
            .is_some_and(|v| v == 1);
        self.install(
            document,
            if migrated {
                None
            } else {
                Some(path.to_path_buf())
            },
        )?;
        if migrated {
            self.status = "Legacy project imported. Save as a portable v2 project.".into();
        }
        Ok(())
    }

    fn mark_preview(&mut self) {
        if let Some(job) = &self.preview_job {
            job.cancel.store(true, Ordering::Relaxed);
        }
        self.preview_revision += 1;
        self.preview_dirty_at = Some(Instant::now());
    }
    fn commit(&mut self, before: DocumentState) {
        let Some(doc) = self.document.as_mut() else {
            return;
        };
        if doc.state() == before {
            return;
        }
        if let Err(error) = crate::document::validate_edit_state(doc) {
            doc.restore(before);
            self.selected = self.selected.filter(|index| *index < doc.layers.len());
            self.error = Some(error);
            self.mark_preview();
            return;
        }
        self.error = None;
        self.undo.push(before);
        if self.undo.len() > 48 {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.edit_revision += 1;
        self.mark_preview();
    }

    fn undo(&mut self) {
        if let (Some(doc), Some(previous)) = (self.document.as_mut(), self.undo.pop()) {
            self.redo.push(doc.state());
            doc.restore(previous);
            self.selected = self.selected.filter(|i| *i < doc.layers.len());
            self.edit_revision += 1;
            self.mark_preview();
        }
    }

    fn redo(&mut self) {
        if let (Some(doc), Some(next)) = (self.document.as_mut(), self.redo.pop()) {
            self.undo.push(doc.state());
            doc.restore(next);
            self.selected = self.selected.filter(|i| *i < doc.layers.len());
            self.edit_revision += 1;
            self.mark_preview();
        }
    }
    pub fn add_color(&mut self) -> Result<(), String> {
        let doc = self.document.as_mut().ok_or("Open a base PNG first")?;
        let before = doc.state();
        let name = doc.unique_layer_name("Color wash")?;
        doc.add_color_layer(&name, [196, 84, 76], 128)?;
        self.selected = Some(doc.layers.len() - 1);
        self.commit(before);
        Ok(())
    }
    pub fn add_paint(&mut self) -> Result<(), String> {
        let doc = self.document.as_mut().ok_or("Open a base PNG first")?;
        let before = doc.state();
        let name = doc.unique_layer_name("Brushwork")?;
        self.selected = Some(doc.add_paint_layer(&name)?);
        self.tool = Tool::Brush;
        self.commit(before);
        Ok(())
    }

    fn finish_stroke(&mut self) {
        if self.paint_points.is_empty() {
            return;
        }
        let points = std::mem::take(&mut self.paint_points);
        let stroke_layer = self.paint_layer.take();
        let Some(doc) = self.document.as_mut() else {
            return;
        };
        let Some(index) = stroke_layer.filter(|layer| Some(*layer) == self.selected) else {
            return;
        };
        let before = doc.state();
        let stroke = PaintStroke {
            rgb: self.brush_color,
            radius: self.brush_radius,
            points,
        };
        match doc.add_paint_stroke(index, stroke) {
            Ok(()) => self.commit(before),
            Err(error) => self.error = Some(error),
        }
    }
    pub fn import_artwork(&mut self, path: &Path) -> Result<(), String> {
        let doc = self.document.as_mut().ok_or("Open a base PNG first")?;
        let before = doc.state();
        let selected = doc.add_image_layer(path)?;
        let asset_id = match &doc.layers[selected].content {
            LayerContent::Image { asset_id, .. } => asset_id.clone(),
            _ => unreachable!(),
        };
        let cache = self
            .cache
            .as_mut()
            .ok_or("Decoded image cache is unavailable")?;
        if let Err(error) = cache.insert_asset(doc, &asset_id) {
            doc.restore(before);
            return Err(error);
        }
        self.selected = Some(selected);
        self.commit(before);
        self.status = format!("Imported {} as an editable artwork layer.", path.display());
        Ok(())
    }
    pub fn save_project_at(&mut self, path: &Path) -> Result<(), String> {
        let doc = self.document.as_ref().ok_or("Nothing to save")?;
        save_document(doc, path, path.exists())?;
        self.project_path = Some(path.to_path_buf());
        self.saved_state = Some(doc.state());
        self.status = format!("Saved portable project: {}", path.display());
        self.error = None;
        Ok(())
    }

    fn save(&mut self, force_dialog: bool) -> Result<(), String> {
        let path = if !force_dialog {
            self.project_path.clone()
        } else {
            None
        };
        let path = match path {
            Some(p) => p,
            None => match rfd::FileDialog::new()
                .add_filter("SkinStrike project", &["json"])
                .set_file_name("finish.skinstrike.json")
                .save_file()
            {
                Some(p) => p,
                None => return Ok(()),
            },
        };
        self.start_io(IoRequest::Save(path))
    }

    fn start_io(&mut self, request: IoRequest) -> Result<(), String> {
        if self.io_job.is_some() {
            return Err("Another file operation is running; cancel or wait for it".into());
        }
        let document = self.document.clone();
        let existing_cache = self.cache.clone();
        let edit_revision = self.edit_revision;
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = cancel.clone();
        let (tx, rx) = mpsc::channel();
        self.status = match &request {
            IoRequest::New(_) => "Importing base PNG...".into(),
            IoRequest::Open(_) => "Opening and verifying project...".into(),
            IoRequest::Import(_) => "Importing artwork...".into(),
            IoRequest::ImportAux(_, AuxKind::Alpha) => "Importing alpha data...".into(),
            IoRequest::ImportAux(_, AuxKind::UvReference) => "Importing UV reference...".into(),
            IoRequest::Save(_) => "Saving project...".into(),
            IoRequest::SaveToLibrary(_, _, _) => "Saving project to collection...".into(),
        };
        self.error = None;
        std::thread::spawn(move || {
            let cancelled = || worker_cancel.load(Ordering::Relaxed);
            let result: Result<IoOutput, String> = (|| {
                if cancelled() {
                    return Err("Operation cancelled".into());
                }
                match request {
                    IoRequest::New(path) => {
                        let next = new_document(&path, &path)?;
                        let cache = prepare_render_cache(&next, &cancelled)?;
                        if cancelled() {
                            return Err("Operation cancelled".into());
                        }
                        Ok(IoOutput::Opened {
                            document: next,
                            cache,
                            path: None,
                            migrated: false,
                        })
                    }
                    IoRequest::Open(path) => {
                        let next = load_document(&path)?;
                        if cancelled() {
                            return Err("Operation cancelled".into());
                        }
                        let migrated = std::fs::read(&path)
                            .ok()
                            .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
                            .and_then(|v| v.get("schema_version").and_then(|n| n.as_u64()))
                            .is_some_and(|v| v == 1);
                        let cache = prepare_render_cache(&next, &cancelled)?;
                        if cancelled() {
                            return Err("Operation cancelled".into());
                        }
                        Ok(IoOutput::Opened {
                            document: next,
                            cache,
                            path: if migrated { None } else { Some(path) },
                            migrated,
                        })
                    }
                    IoRequest::Import(path) => {
                        let mut next = document.ok_or("Open a base PNG first")?;
                        let selected = next.add_image_layer(&path)?;
                        if cancelled() {
                            return Err("Operation cancelled".into());
                        }
                        let asset_id = match &next.layers[selected].content {
                            LayerContent::Image { asset_id, .. } => asset_id.clone(),
                            _ => unreachable!(),
                        };
                        let mut cache = match existing_cache {
                            Some(cache) => cache,
                            None => prepare_render_cache(&next, &cancelled)?,
                        };
                        cache.insert_asset(&next, &asset_id)?;
                        if cancelled() {
                            return Err("Operation cancelled".into());
                        }
                        Ok(IoOutput::Imported {
                            document: next,
                            cache,
                            selected,
                            path,
                        })
                    }
                    IoRequest::ImportAux(path, kind) => {
                        let mut next = document.ok_or("Open a base PNG first")?;
                        let id = match kind {
                            AuxKind::Alpha => {
                                next.set_alpha_data(&path)?;
                                next.alpha_data.as_ref().unwrap().asset_id.clone()
                            }
                            AuxKind::UvReference => {
                                next.set_uv_reference(&path)?;
                                next.uv_reference.as_ref().unwrap().asset_id.clone()
                            }
                        };
                        if cancelled() {
                            return Err("Operation cancelled".into());
                        }
                        let mut cache = match existing_cache {
                            Some(cache) => cache,
                            None => prepare_render_cache(&next, &cancelled)?,
                        };
                        cache.insert_asset(&next, &id)?;
                        if cancelled() {
                            return Err("Operation cancelled".into());
                        }
                        Ok(IoOutput::ImportedAux {
                            document: next,
                            cache,
                            kind,
                            path,
                        })
                    }
                    IoRequest::Save(path) => {
                        let current = document.ok_or("Nothing to save")?;
                        let state = current.state();
                        save_document_with_cancel(&current, &path, path.exists(), &cancelled)?;
                        Ok(IoOutput::Saved {
                            path,
                            state,
                            project_id: current.project_id,
                        })
                    }
                    IoRequest::SaveToLibrary(root, collection, title) => {
                        let current = document.ok_or("Nothing to save")?;
                        if cancelled() {
                            return Err("Operation cancelled".into());
                        }
                        let state = current.state();
                        let path = ProjectLibrary::open(&root)?.save_project(
                            &collection,
                            &title,
                            &current,
                        )?;
                        Ok(IoOutput::Saved {
                            path,
                            state,
                            project_id: current.project_id,
                        })
                    }
                }
            })();
            let _ = tx.send(result);
        });
        self.io_job = Some(IoJob {
            receiver: rx,
            cancel,
            edit_revision,
        });
        Ok(())
    }

    fn poll_io(&mut self, ctx: &egui::Context) {
        let result = match self.io_job.as_ref() {
            Some(job) => match job.receiver.try_recv() {
                Ok(value) => Some(value),
                Err(mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(Duration::from_millis(50));
                    None
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    Some(Err("File worker stopped unexpectedly".into()))
                }
            },
            None => None,
        };
        let Some(result) = result else {
            return;
        };
        let job = self.io_job.take().unwrap();
        match result {
            Err(error) => {
                if error == "Operation cancelled" {
                    self.status = "File operation cancelled; current work kept.".into();
                    self.error = None;
                } else {
                    self.error = Some(error);
                }
                self.continue_after_save = None;
            }
            Ok(IoOutput::Opened {
                document,
                cache,
                path,
                migrated,
            }) => {
                if job.cancel.load(Ordering::Relaxed) || job.edit_revision != self.edit_revision {
                    self.status = "Outdated import discarded; current work kept.".into();
                    return;
                }
                self.install_prepared(document, path, cache);
                if migrated {
                    self.status = "Legacy project imported. Save as a portable v2 project.".into();
                }
            }
            Ok(IoOutput::Imported {
                document,
                cache,
                selected,
                path,
            }) => {
                if job.cancel.load(Ordering::Relaxed) || job.edit_revision != self.edit_revision {
                    self.status = "Outdated artwork import discarded; current work kept.".into();
                    return;
                }
                let before = self.document.as_ref().map(|d| d.state());
                self.document = Some(document);
                self.cache = Some(cache);
                self.selected = Some(selected);
                if let Some(before) = before {
                    self.commit(before);
                }
                self.status = format!("Imported {} as an editable artwork layer.", path.display());
            }
            Ok(IoOutput::ImportedAux {
                document,
                cache,
                kind,
                path,
            }) => {
                if job.cancel.load(Ordering::Relaxed) || job.edit_revision != self.edit_revision {
                    self.status =
                        "Outdated channel/reference import discarded; current work kept.".into();
                    return;
                }
                let before = self.document.as_ref().map(|d| d.state());
                self.document = Some(document);
                self.cache = Some(cache);
                if let Some(before) = before {
                    self.commit(before);
                }
                self.status = format!(
                    "Imported {} from {}.",
                    match kind {
                        AuxKind::Alpha => "grayscale alpha data",
                        AuxKind::UvReference => "editor-only UV reference",
                    },
                    path.display()
                );
            }
            Ok(IoOutput::Saved {
                path,
                state,
                project_id,
            }) => {
                if self
                    .document
                    .as_ref()
                    .is_some_and(|d| d.project_id == project_id)
                {
                    self.project_path = Some(path.clone());
                    self.saved_state = Some(state);
                    self.status = if self.dirty() {
                        format!(
                            "Saved earlier revision to {}; newer edits remain unsaved.",
                            path.display()
                        )
                    } else {
                        format!("Saved portable project: {}", path.display())
                    };
                    self.error = None;
                    self.schedule_gallery_refresh();
                    if let Some(action) = self.continue_after_save.take() {
                        if self.dirty() {
                            self.pending = Some(action);
                        } else if matches!(action, Pending::Close) {
                            self.allow_close = true;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        } else {
                            self.perform(action);
                        }
                    }
                } else {
                    self.status = format!(
                        "Earlier project saved to {}; current project kept.",
                        path.display()
                    );
                }
            }
        }
    }

    fn request(&mut self, action: Pending) {
        if self.io_job.is_some() {
            self.error = Some("A file operation is running; cancel or wait for it".into());
        } else if self.dirty() {
            self.pending = Some(action);
        } else {
            self.perform(action);
        }
    }

    fn perform(&mut self, action: Pending) {
        let request = match action {
            Pending::New => rfd::FileDialog::new()
                .add_filter("PNG image", &["png"])
                .pick_file()
                .map(IoRequest::New),
            Pending::Open => rfd::FileDialog::new()
                .add_filter("SkinStrike project", &["json"])
                .pick_file()
                .map(IoRequest::Open),
            Pending::LibraryOpen => self.library_target.take().map(IoRequest::Open),
            Pending::Close => None,
        };
        if let Some(request) = request {
            if let Err(error) = self.start_io(request) {
                self.error = Some(error);
            }
        }
    }
    fn export_to(&mut self, path: PathBuf, overwrite: bool) {
        if self.export_job.is_some() {
            return;
        }
        let (Some(document), Some(cache)) = (self.document.clone(), self.cache.clone()) else {
            self.error = Some("Nothing to export".into());
            return;
        };
        let protected: Vec<PathBuf> = self.project_path.iter().cloned().collect();
        let references: Vec<&Path> = protected.iter().map(PathBuf::as_path).collect();
        if let Err(error) =
            validate_export_destination_quick(&document, &path, overwrite, &references)
        {
            self.error = Some(error);
            return;
        }
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = cancel.clone();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let references: Vec<&Path> = protected.iter().map(PathBuf::as_path).collect();
            let cancelled = || worker_cancel.load(Ordering::Relaxed);
            let result =
                export_png_with_cache(&document, &cache, &path, overwrite, &references, &cancelled)
                    .map(|_| path);
            let _ = tx.send(result);
        });
        self.export_job = Some(rx);
        self.export_cancel = Some(cancel);
        self.export_revision = Some(self.edit_revision);
        self.status = "Rendering full-resolution PNG in the background...".into();
        self.error = None;
    }

    fn choose_export(&mut self) {
        if self.document.is_none() || self.export_job.is_some() {
            return;
        }
        let Some(path) = rfd::FileDialog::new()
            .add_filter("PNG image", &["png"])
            .set_file_name("finish-preview.png")
            .save_file()
        else {
            return;
        };
        let overwrite = path.exists();
        let protected: Vec<&Path> = self.project_path.iter().map(PathBuf::as_path).collect();
        if let Some(document) = &self.document {
            if let Err(error) =
                validate_export_destination_quick(document, &path, overwrite, &protected)
            {
                self.error = Some(error);
                return;
            }
        }
        if overwrite {
            self.confirm_export = Some(path);
        } else {
            self.export_to(path, false);
        }
    }

    fn poll_export(&mut self, ctx: &egui::Context) {
        let result = match self.export_job.as_ref() {
            Some(rx) => match rx.try_recv() {
                Ok(result) => Some(result),
                Err(mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(Duration::from_millis(50));
                    None
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    Some(Err("Export worker stopped unexpectedly".into()))
                }
            },
            None => None,
        };
        let Some(result) = result else {
            return;
        };
        self.export_job = None;
        self.export_cancel = None;
        let revision = self.export_revision.take();
        match result {
            Ok(path) => {
                self.status = if revision != Some(self.edit_revision) {
                    format!(
                        "Exported earlier revision to {}; newer edits remain in the canvas.",
                        path.display()
                    )
                } else {
                    format!("Exported PNG: {}", path.display())
                };
                self.error = None;
            }
            Err(error) if error == "Operation cancelled" => {
                self.status = "Export cancelled; output was not replaced.".into();
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
    }
    fn poll_preview(&mut self, ctx: &egui::Context) {
        let result = match self.preview_job.as_ref() {
            Some(job) => match job.receiver.try_recv() {
                Ok(result) => Some(result),
                Err(mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(Duration::from_millis(40));
                    None
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    Some(Err("Preview worker stopped unexpectedly".into()))
                }
            },
            None => None,
        };
        let Some(result) = result else {
            return;
        };
        let job = self.preview_job.take().unwrap();
        if job.cancel.load(Ordering::Relaxed) || job.preview_revision != self.preview_revision {
            return;
        }
        match result {
            Ok(image) => match &mut self.texture {
                Some(handle) => handle.set(image, TextureOptions::LINEAR),
                None => {
                    self.texture =
                        Some(ctx.load_texture("SkinStrike canvas", image, TextureOptions::LINEAR))
                }
            },
            Err(error) => {
                self.texture = None;
                self.error = Some(format!("Preview unavailable: {error}"));
            }
        }
    }

    fn update_preview(&mut self, ctx: &egui::Context) {
        if self.preview_job.is_some() {
            return;
        }
        let Some(started) = self.preview_dirty_at else {
            return;
        };
        if started.elapsed() < Duration::from_millis(60) {
            ctx.request_repaint_after(Duration::from_millis(65));
            return;
        }
        self.preview_dirty_at = None;
        let (Some(document), Some(cache)) = (self.document.clone(), self.cache.clone()) else {
            return;
        };
        let view = self.view;
        let preview_revision = self.preview_revision;
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = cancel.clone();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let cancelled = || worker_cancel.load(Ordering::Relaxed);
            let result = render_preview_image(&document, &cache, view, &cancelled);
            let _ = tx.send(result);
        });
        self.preview_job = Some(PreviewJob {
            receiver: rx,
            cancel,
            preview_revision,
        });
        ctx.request_repaint_after(Duration::from_millis(40));
    }
    fn schedule_gallery_refresh(&mut self) {
        self.gallery_generation = self.gallery_generation.wrapping_add(1);
        self.gallery_dirty_at = Some(Instant::now());
        self.gallery_page = None;
        self.gallery_textures.clear();
    }

    fn choose_library(&mut self) {
        let Some(root) = rfd::FileDialog::new().pick_folder() else {
            return;
        };
        match ProjectLibrary::open(&root) {
            Ok(library) => {
                if let Some(config) = preference_path() {
                    if let Err(error) = write_library_preference(&config, library.root()) {
                        self.error = Some(error);
                        return;
                    }
                }
                self.library_root = Some(library.root().to_path_buf());
                self.gallery_collection = None;
                self.gallery_offset = 0;
                self.gallery_page = None;
                self.schedule_gallery_refresh();
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
    }

    fn update_gallery(&mut self, ctx: &egui::Context) {
        if let Some(job) = self.gallery_job.as_ref() {
            let outcome = match job.receiver.try_recv() {
                Ok(result) => Some(result),
                Err(mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(Duration::from_millis(60));
                    None
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    Some(Err("Library scan stopped unexpectedly".into()))
                }
            };
            if let Some(outcome) = outcome {
                let job = self.gallery_job.take().unwrap();
                if job.generation == self.gallery_generation {
                    match outcome {
                        Ok((collections, page)) => {
                            self.gallery_textures = page
                                .cards
                                .iter()
                                .enumerate()
                                .map(|(i, card)| {
                                    card.thumbnail.as_ref().map(|img| {
                                        ctx.load_texture(
                                            format!("skin-cover-{}-{}", job.generation, i),
                                            ColorImage::from_rgba_unmultiplied(
                                                [img.width() as usize, img.height() as usize],
                                                img.as_raw(),
                                            ),
                                            TextureOptions::LINEAR,
                                        )
                                    })
                                })
                                .collect();
                            self.gallery_collections = collections;
                            self.gallery_page = Some(page);
                            self.error = None;
                        }
                        Err(error) => self.error = Some(error),
                    }
                }
            }
        }
        if self.gallery_job.is_none()
            && self
                .gallery_dirty_at
                .is_some_and(|at| at.elapsed() >= Duration::from_millis(180))
        {
            self.gallery_dirty_at = None;
            let Some(root) = self.library_root.clone() else {
                return;
            };
            let collection = self.gallery_collection.clone();
            let query = self.gallery_query.clone();
            let offset = self.gallery_offset;
            let generation = self.gallery_generation;
            let (tx, rx) = mpsc::channel();
            std::thread::spawn(move || {
                let outcome = (|| {
                    let library = ProjectLibrary::open(&root)?;
                    let collections = library.collections()?;
                    let page = library.page(collection.as_deref(), &query, offset, 12)?;
                    Ok((collections, page))
                })();
                let _ = tx.send(outcome);
            });
            self.gallery_job = Some(GalleryJob {
                receiver: rx,
                generation,
            });
        }
    }

    fn gallery(&mut self, ctx: &egui::Context) {
        if self.logo_texture.is_none() {
            if let Ok(image) =
                image::load_from_memory(include_bytes!("../assets/branding/SkinStrike.png"))
            {
                let image = image.to_rgba8();
                self.logo_texture = Some(ctx.load_texture(
                    "skinstrike-approved-logo",
                    ColorImage::from_rgba_unmultiplied(
                        [image.width() as usize, image.height() as usize],
                        image.as_raw(),
                    ),
                    TextureOptions::LINEAR,
                ));
            }
        }
        let mut open_path = None;
        let mut choose = false;
        let mut refresh = false;
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(20.0);
                ui.horizontal(|ui| {
                    if let Some(logo) = &self.logo_texture { ui.image((logo.id(), egui::vec2(92.0, 92.0))); }
                    ui.vertical(|ui| {
                        ui.heading(RichText::new("SkinStrike").size(30.0).strong());
                        ui.label("Your finishes, organized by collection");
                        ui.small("A PWDS Strike app");
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("About").clicked() { self.show_about = true; }
                        if self.document.is_some() && ui.button("Return to editor").clicked() { self.gallery_open = false; }
                        if ui.button("New from PNG").clicked() { self.request(Pending::New); }
                    });
                });
                ui.add_space(20.0);
                ui.horizontal(|ui| {
                    if ui.button("Choose library folder").clicked() { choose = true; }
                    if ui.button("Refresh").clicked() { refresh = true; }
                    if let Some(root) = &self.library_root {
                        let folder = root.file_name().unwrap_or_default().to_string_lossy();
                        ui.small(format!("Library: {folder}")).on_hover_text(root.display().to_string());
                    }
                });
                if let Some(error) = &self.error { ui.colored_label(Color32::from_rgb(255, 140, 130), error); }
                let Some(_) = &self.library_root else {
                    ui.add_space(40.0);
                    ui.heading("Choose a local folder to start your library");
                    ui.label("Collections and portable project files stay in the folder you select.");
                    return;
                };
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    ui.label("New collection");
                    ui.add(egui::TextEdit::singleline(&mut self.new_collection).desired_width(170.0).hint_text("Collection name"));
                    if ui.button("Create").clicked() {
                        if let Some(root) = &self.library_root {
                            match ProjectLibrary::open(root).and_then(|library| library.create_collection(&self.new_collection)) {
                                Ok(_) => { self.new_collection.clear(); refresh = true; self.error = None; },
                                Err(error) => self.error = Some(error),
                            }
                        }
                    }
                    ui.separator();
                    if self.document.is_some() && ui.button("Save current finish to collection").clicked() {
                        self.library_title = self.current_project_title();
                        self.library_save_collection = self.gallery_collection.clone()
                            .or_else(|| self.gallery_collections.first().cloned()).unwrap_or_default();
                        self.save_to_library = true;
                    }
                });
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if ui.selectable_label(self.gallery_collection.is_none(), "All projects").clicked() {
                        self.gallery_collection = None; self.gallery_offset = 0; refresh = true;
                    }
                    for name in &self.gallery_collections {
                        if ui.selectable_label(self.gallery_collection.as_ref() == Some(name), name).clicked() {
                            self.gallery_collection = Some(name.clone()); self.gallery_offset = 0; refresh = true;
                        }
                    }
                });
                ui.add_space(8.0);
                if ui.add(egui::TextEdit::singleline(&mut self.gallery_query).hint_text("Search project titles")).changed() {
                    self.gallery_offset = 0; refresh = true;
                }
                ui.separator();
                if self.gallery_job.is_some() || self.gallery_dirty_at.is_some() { ui.label("Loading your projects..."); }
                if let Some(job) = &self.io_job {
                    ui.horizontal(|ui| {
                        ui.label(&self.status);
                        if ui.button("Cancel file work").clicked() { job.cancel.store(true, Ordering::Relaxed); }
                    });
                }
                if let Some(page) = &self.gallery_page {
                    ui.small(format!("{} projects", page.total));
                    if page.cards.is_empty() && self.gallery_job.is_none() { ui.label("No projects here yet. Save a finish into a collection to see its real cover."); }
                    for row in (0..page.cards.len()).step_by(4) {
                        ui.columns(4, |columns| {
                            for (column, index) in (row..(row+4).min(page.cards.len())).enumerate() {
                                let card = &page.cards[index];
                                egui::Frame::group(columns[column].style()).show(&mut columns[column], |ui| {
                                    if let Some(texture) = self.gallery_textures.get(index).and_then(|t| t.as_ref()) {
                                        let [width, height] = texture.size();
                                        let scale = (165.0 / width as f32).min(165.0 / height as f32);
                                        ui.vertical_centered(|ui| {
                                            ui.image((texture.id(), egui::vec2(width as f32 * scale, height as f32 * scale)));
                                        });
                                    } else { ui.allocate_space(egui::vec2(165.0, 165.0)); }
                                    ui.label(RichText::new(&card.title).strong());
                                    ui.small(&card.collection);
                                    if let Some(error) = &card.error { ui.colored_label(Color32::LIGHT_RED, format!("Cannot preview: {error}")); }
                                    else if ui.button("Open in editor").clicked() { open_path = Some(card.path.clone()); }
                                });
                            }
                        });
                        ui.add_space(8.0);
                    }
                    ui.horizontal(|ui| {
                        if ui.add_enabled(self.gallery_offset > 0, egui::Button::new("Previous")).clicked() {
                            self.gallery_offset = self.gallery_offset.saturating_sub(12); refresh = true;
                        }
                        ui.label(format!("Page {}", self.gallery_offset / 12 + 1));
                        if ui.add_enabled(self.gallery_offset + 12 < page.total, egui::Button::new("Next")).clicked() {
                            self.gallery_offset += 12; refresh = true;
                        }
                    });
                }
            });
        });
        if choose {
            self.choose_library();
        }
        if refresh {
            self.schedule_gallery_refresh();
        }
        if let Some(path) = open_path {
            self.library_target = Some(path);
            self.request(Pending::LibraryOpen);
        }
    }
    fn shortcuts(&mut self, ctx: &egui::Context) {
        let (save, open, new, undo, redo, zoom_in, zoom_out, fit) = ctx.input(|i| {
            (
                i.modifiers.command && i.key_pressed(egui::Key::S),
                i.modifiers.command && i.key_pressed(egui::Key::O),
                i.modifiers.command && i.key_pressed(egui::Key::N),
                i.modifiers.command && i.key_pressed(egui::Key::Z) && !i.modifiers.shift,
                (i.modifiers.command && i.key_pressed(egui::Key::Y))
                    || (i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::Z)),
                i.key_pressed(egui::Key::Plus),
                i.key_pressed(egui::Key::Minus),
                i.key_pressed(egui::Key::Num0),
            )
        });
        if save {
            if let Err(e) = self.save(false) {
                self.error = Some(e);
            }
        }
        if open {
            self.request(Pending::Open);
        }
        if new {
            self.request(Pending::New);
        }
        if undo {
            self.undo();
        }
        if redo {
            self.redo();
        }
        if zoom_in {
            self.zoom = (self.zoom * 1.25).min(16.0);
        }
        if zoom_out {
            self.zoom = (self.zoom / 1.25).max(0.1);
        }
        if fit {
            self.zoom = 1.0;
            self.pan = egui::Vec2::ZERO;
        }
    }

    fn top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.add_space(7.0);
            ui.horizontal(|ui| {
                if ui.button("Projects").clicked() {
                    self.gallery_open = true;
                    self.schedule_gallery_refresh();
                }
                ui.separator();
                ui.label(RichText::new("Editor").strong().size(17.0));
                ui.separator();
                if ui.button("About").clicked() {
                    self.show_about = true;
                }
                ui.separator();
                if ui.button("New from PNG").on_hover_text("Ctrl+N").clicked() {
                    self.request(Pending::New);
                }
                if ui.button("Open project").on_hover_text("Ctrl+O").clicked() {
                    self.request(Pending::Open);
                }
                ui.separator();
                if ui
                    .add_enabled(self.document.is_some(), egui::Button::new("Save"))
                    .on_hover_text("Ctrl+S")
                    .clicked()
                {
                    if let Err(e) = self.save(false) {
                        self.error = Some(e);
                    }
                }
                if ui
                    .add_enabled(self.document.is_some(), egui::Button::new("Save as"))
                    .clicked()
                {
                    if let Err(e) = self.save(true) {
                        self.error = Some(e);
                    }
                }
                if ui
                    .add_enabled(
                        self.document.is_some() && self.library_root.is_some(),
                        egui::Button::new("Save to collection"),
                    )
                    .clicked()
                {
                    self.library_title = self.current_project_title();
                    self.library_save_collection = self
                        .gallery_collection
                        .clone()
                        .or_else(|| self.gallery_collections.first().cloned())
                        .unwrap_or_default();
                    self.save_to_library = true;
                }
                if ui
                    .add_enabled(self.document.is_some(), egui::Button::new("Export PNG"))
                    .clicked()
                {
                    self.choose_export();
                }
                ui.separator();
                if ui
                    .add_enabled(!self.undo.is_empty(), egui::Button::new("Undo"))
                    .clicked()
                {
                    self.undo();
                }
                if ui
                    .add_enabled(!self.redo.is_empty(), egui::Button::new("Redo"))
                    .clicked()
                {
                    self.redo();
                }
            });
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                let title = self.current_project_title();
                ui.label(
                    RichText::new(format!(
                        "{}{}",
                        title,
                        if self.dirty() { "  * unsaved" } else { "" }
                    ))
                    .color(Color32::from_rgb(169, 188, 213)),
                );
                if let Some(doc) = &self.document {
                    ui.label(format!(
                        "  |  {} x {}  |  {} layers",
                        doc.width,
                        doc.height,
                        doc.layers.len()
                    ));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("Portable layered project  |  PNG preview export");
                });
            });
            ui.add_space(5.0);
        });
    }

    fn left_bar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("tools")
            .resizable(false)
            .exact_width(116.0)
            .show(ctx, |ui| {
                ui.add_space(12.0);
                ui.heading("Tools");
                ui.add_space(12.0);
                ui.selectable_value(&mut self.tool, Tool::Move, "Move");
                ui.small("Drag artwork");
                ui.add_space(10.0);
                ui.selectable_value(&mut self.tool, Tool::Pan, "Pan");
                ui.small("Drag canvas");
                ui.add_space(10.0);
                ui.selectable_value(&mut self.tool, Tool::Brush, "Brush");
                ui.small("Paint by hand");
                if self.tool == Tool::Brush {
                    ui.separator();
                    ui.label("Color");
                    ui.color_edit_button_srgb(&mut self.brush_color);
                    ui.add(egui::Slider::new(&mut self.brush_radius, 0.5..=64.0).text("Size"));
                    ui.small("Select a visible, unlocked paint layer.");
                }
                ui.separator();
                if ui.button("Fit").clicked() {
                    self.zoom = 1.0;
                    self.pan = egui::Vec2::ZERO;
                }
                ui.label(format!("{:.0}%", self.zoom * 100.0));
                ui.small("Wheel to zoom");
            });
    }

    fn layer_action(&mut self, action: LayerAction) -> Result<(), String> {
        let Some(index) = self.selected else {
            return Ok(());
        };
        let Some(doc) = self.document.as_mut() else {
            return Ok(());
        };
        if index >= doc.layers.len() {
            return Ok(());
        }
        if !matches!(action, LayerAction::Duplicate) && doc.layers[index].locked {
            return Err("Unlock the layer before moving or deleting it".into());
        }
        if (matches!(action, LayerAction::Up)
            && index + 1 < doc.layers.len()
            && doc.layers[index + 1].locked)
            || (matches!(action, LayerAction::Down) && index > 0 && doc.layers[index - 1].locked)
        {
            return Err("Unlock the adjacent layer before reordering".into());
        }
        let before = doc.state();
        match action {
            LayerAction::Up if index + 1 < doc.layers.len() => {
                doc.layers.swap(index, index + 1);
                self.selected = Some(index + 1);
            }
            LayerAction::Down if index > 0 => {
                doc.layers.swap(index, index - 1);
                self.selected = Some(index - 1);
            }
            LayerAction::Duplicate => {
                if doc.layers.len() >= 128 {
                    return Err("Project supports at most 128 layers".into());
                }
                let mut copy = doc.layers[index].clone();
                copy.locked = false;
                let root = if let Some((prefix, suffix)) = copy.name.rsplit_once(" copy") {
                    if suffix.is_empty() || suffix.trim().parse::<u32>().is_ok() {
                        prefix
                    } else {
                        &copy.name
                    }
                } else {
                    &copy.name
                };
                let preferred: String = format!("{root} copy").chars().take(80).collect();
                copy.name = doc.unique_layer_name(&preferred)?;
                doc.layers.insert(index + 1, copy);
                self.selected = Some(index + 1);
            }
            LayerAction::Delete => {
                doc.layers.remove(index);
                self.selected = if doc.layers.is_empty() {
                    None
                } else {
                    Some(index.min(doc.layers.len() - 1))
                };
            }
            _ => {}
        }
        self.commit(before);
        Ok(())
    }
    fn right_bar(&mut self, ctx: &egui::Context) {
        let mut action = None;
        let mut import_artwork = false;
        let mut import_alpha = false;
        let mut import_uv = false;
        let mut add_color = false;
        let mut add_paint = false;
        let mut before = self.document.as_ref().map(|d| d.state());
        let mut edited = false;
        egui::SidePanel::right("layers")
            .resizable(true)
            .default_width(325.0)
            .min_width(280.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(9.0);
                    ui.heading("Layers");
                    if self.document.is_none() {
                        ui.label("Start with a base PNG. Then add artwork and color layers.");
                        return;
                    }
                    ui.horizontal(|ui| {
                        if ui.button("+ Artwork").clicked() { import_artwork = true; }
                        if ui.button("+ Color").clicked() { add_color = true; }
                        if ui.button("+ Paint").clicked() { add_paint = true; }
                    });
                    ui.small("Top layers appear first. Click a row to edit it.");
                    ui.separator();
                    let doc = self.document.as_mut().unwrap();
                    egui::ScrollArea::vertical().max_height(225.0).show(ui, |ui| {
                        for i in (0..doc.layers.len()).rev() {
                            ui.horizontal(|ui| {
                                edited |= ui.checkbox(&mut doc.layers[i].visible, "").on_hover_text("Show or hide layer").changed();
                                let lock_label = if doc.layers[i].locked { "LOCK" } else { "EDIT" };
                                if ui.small_button(lock_label).on_hover_text("Toggle layer edit lock").clicked() {
                                    doc.layers[i].locked = !doc.layers[i].locked;
                                    edited = true;
                                }
                                let icon = match doc.layers[i].content {
                                    LayerContent::Color { .. } => "COLOR",
                                    LayerContent::Image { .. } => "IMAGE",
                                    LayerContent::Paint { .. } => "PAINT",
                                };
                                if ui.selectable_label(self.selected == Some(i), format!("{}  {}", icon, doc.layers[i].name)).clicked() {
                                    self.selected = Some(i);
                                }
                            });
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui.add_enabled(self.selected.is_some(), egui::Button::new("Up")).clicked() { action = Some(LayerAction::Up); }
                        if ui.add_enabled(self.selected.is_some(), egui::Button::new("Down")).clicked() { action = Some(LayerAction::Down); }
                        if ui.add_enabled(self.selected.is_some(), egui::Button::new("Copy")).clicked() { action = Some(LayerAction::Duplicate); }
                        if ui.add_enabled(self.selected.is_some(), egui::Button::new("Delete")).clicked() { action = Some(LayerAction::Delete); }
                    });
                    ui.separator();
                    ui.heading("Properties");
                    if let Some(index) = self.selected.filter(|i| *i < doc.layers.len()) {
                        let layer = &mut doc.layers[index];
                        if layer.locked { ui.small("Layer locked. Unlock it in the layer list to edit properties."); }
                        ui.add_enabled_ui(!layer.locked, |ui| {
                        ui.label("Name");
                        edited |= ui.text_edit_singleline(&mut layer.name).changed();
                        edited |= ui.add(egui::Slider::new(&mut layer.opacity, 0..=255).text("Opacity")).changed();
                        match &mut layer.content {
                            LayerContent::Color { rgb } => {
                                ui.label("Wash color");
                                edited |= ui.color_edit_button_srgb(rgb).changed();
                                ui.small("Blends RGB without changing the current alpha channel.");
                            }
                            LayerContent::Image { asset_id, x, y, scale, rotation_deg } => {
                                let source = doc.assets.get(asset_id).map(|a| a.name.as_str()).unwrap_or("Missing artwork");
                                ui.small(format!("Source: {source}"));
                                ui.label("Position on canvas (pixels)");
                                ui.horizontal(|ui| {
                                    ui.label("X");
                                    edited |= ui.add(egui::DragValue::new(x).speed(1.0)).changed();
                                    ui.label("Y");
                                    edited |= ui.add(egui::DragValue::new(y).speed(1.0)).changed();
                                });
                                edited |= ui.add(egui::Slider::new(scale, 0.05..=10.0).logarithmic(true).text("Scale")).changed();
                                edited |= ui.add(egui::Slider::new(rotation_deg, -180.0..=180.0).text("Rotate")).changed();
                                ui.small("Select Move and drag on the canvas to position artwork.");
                            }
                            LayerContent::Paint { strokes } => {
                                ui.small(format!("{} saved strokes", strokes.len()));
                                ui.small("Choose Brush and draw on the canvas. Each gesture is one undo step.");
                            }
                        }
                        });
                    } else {
                        ui.small("Select a layer to edit its properties.");
                    }
                    ui.separator();
                    ui.heading("Alpha channel");
                    edited |= ui.radio_value(&mut doc.alpha_mode, AlphaMode::PreserveBase, "Preserve base alpha").changed();
                    edited |= ui.radio_value(&mut doc.alpha_mode, AlphaMode::CompositeCoverage, "Composite coverage").changed();
                    ui.add_enabled_ui(doc.alpha_data.is_some(), |ui| {
                        edited |= ui.radio_value(&mut doc.alpha_mode, AlphaMode::ImportedData, "Imported grayscale data").changed();
                    });
                    ui.small(match doc.alpha_mode {
                        AlphaMode::PreserveBase => "Output alpha is exactly the base PNG's alpha data.",
                        AlphaMode::CompositeCoverage => "Artwork uses ordinary source-over transparency, not a CS2 finish mask.",
                        AlphaMode::ImportedData => "Output alpha comes from the imported grayscale image; RGB artwork still edits color.",
                    });
                    if ui.button("Import alpha data PNG...").clicked() { import_alpha = true; }
                    if let Some(data) = &mut doc.alpha_data {
                        let name = doc.assets.get(&data.asset_id).map(|a| a.name.as_str()).unwrap_or("Missing alpha data");
                        ui.small(format!("Data source: {name}"));
                        edited |= ui.checkbox(&mut data.invert, "Invert data values").changed();
                        if ui.button("Remove alpha data").clicked() {
                            doc.alpha_data = None;
                            if doc.alpha_mode == AlphaMode::ImportedData { doc.alpha_mode = AlphaMode::PreserveBase; }
                            edited = true;
                        }
                    }
                    ui.small("Data import requires canvas-sized opaque grayscale PNG (R=G=B, A=255).");
                    ui.separator();
                    ui.heading("Material intent");
                    egui::ComboBox::from_label("Finish style to check in CS2")
                        .selected_text(match doc.style_intent {
                            FinishStyleIntent::Unspecified => "Unspecified",
                            FinishStyleIntent::CustomPaint => "Custom Paint",
                            FinishStyleIntent::Hydrographic => "Hydrographic",
                            FinishStyleIntent::AnodizedMulticolored => "Anodized Multicolored",
                            FinishStyleIntent::Gunsmith => "Gunsmith",
                            FinishStyleIntent::Other => "Other",
                        })
                        .show_ui(ui, |ui| {
                            for (style, name) in [
                                (FinishStyleIntent::Unspecified, "Unspecified"),
                                (FinishStyleIntent::CustomPaint, "Custom Paint"),
                                (FinishStyleIntent::Hydrographic, "Hydrographic"),
                                (FinishStyleIntent::AnodizedMulticolored, "Anodized Multicolored"),
                                (FinishStyleIntent::Gunsmith, "Gunsmith"),
                                (FinishStyleIntent::Other, "Other"),
                            ] {
                                edited |= ui.selectable_value(&mut doc.style_intent, style, name).changed();
                            }
                        });
                    ui.small(match doc.style_intent {
                        FinishStyleIntent::CustomPaint | FinishStyleIntent::Hydrographic => "Valve documents optional durability and masking data in alpha for this style.",
                        FinishStyleIntent::AnodizedMulticolored | FinishStyleIntent::Gunsmith => "Valve documents style-specific roughness and masking ranges in alpha.",
                        _ => "Alpha meaning depends on the finish style; inspect it in CS2 Workshop Tools.",
                    });
                    ui.small("This style note is project metadata; it does not generate a CS2 material.");
                    ui.separator();
                    ui.heading("UV reference");
                    if ui.button("Import UV guide PNG...").clicked() { import_uv = true; }
                    if let Some(reference) = &mut doc.uv_reference {
                        let name = doc.assets.get(&reference.asset_id).map(|a| a.name.as_str()).unwrap_or("Missing UV guide");
                        ui.small(format!("Guide: {name}"));
                        edited |= ui.checkbox(&mut reference.visible, "Show over canvas").changed();
                        edited |= ui.add(egui::Slider::new(&mut reference.opacity, 0..=255).text("Guide opacity")).changed();
                        if ui.button("Remove UV guide").clicked() {
                            doc.uv_reference = None;
                            edited = true;
                        }
                    }
                    ui.small("Canvas-sized 2D guide. Shown in RGB/RGBA, hidden in Alpha data view, never exported. No model or engine parity.");
                });
            });
        if edited {
            if let Some(doc) = &self.document {
                if let Err(error) = crate::document::validate_edit_state(doc) {
                    if let Some(old) = before.take() {
                        self.document.as_mut().unwrap().restore(old);
                    }
                    self.error = Some(error);
                } else if let Some(old) = before.take() {
                    self.commit(old);
                }
            }
        }
        if let Some(action) = action {
            if let Err(error) = self.layer_action(action) {
                self.error = Some(error);
            }
        }
        if add_color {
            if let Err(error) = self.add_color() {
                self.error = Some(error);
            }
        }
        if add_paint {
            if let Err(error) = self.add_paint() {
                self.error = Some(error);
            }
        }
        for (pick, kind) in [
            (import_artwork, None),
            (import_alpha, Some(AuxKind::Alpha)),
            (import_uv, Some(AuxKind::UvReference)),
        ] {
            if !pick {
                continue;
            }
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("PNG image", &["png"])
                .pick_file()
            {
                let request = match kind {
                    Some(kind) => IoRequest::ImportAux(path, kind),
                    None => IoRequest::Import(path),
                };
                if let Err(error) = self.start_io(request) {
                    self.error = Some(error);
                }
            }
        }
    }
    fn canvas(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let area = ui.available_rect_before_wrap();
            let response = ui.interact(area, ui.id().with("canvas"), egui::Sense::click_and_drag());
            let painter = ui.painter_at(area);
            painter.rect_filled(area, 0.0, Color32::from_rgb(23, 27, 35));
            let Some((width, height)) = self.document.as_ref().map(|d| (d.width, d.height)) else {
                painter.text(area.center(), egui::Align2::CENTER_CENTER,
                    "OPEN A BASE PNG\n\nBuild color and artwork layers, then save a portable project.",
                    egui::FontId::proportional(19.0), Color32::from_rgb(176, 193, 213));
                return;
            };
            let fit = (area.width() * 0.9 / width as f32).min(area.height() * 0.9 / height as f32).max(0.001);
            let mut scale = fit * self.zoom;
            if response.hovered() {
                let scroll = ctx.input(|i| i.raw_scroll_delta.y);
                if scroll.abs() > 0.0 {
                    let pointer = ctx.input(|i| i.pointer.hover_pos()).unwrap_or(area.center());
                    let old_origin = area.center() - egui::vec2(width as f32 * scale, height as f32 * scale) / 2.0 + self.pan;
                    let uv = (pointer - old_origin) / scale;
                    self.zoom = (self.zoom * (scroll / 400.0).exp()).clamp(0.1, 16.0);
                    scale = fit * self.zoom;
                    let new_centered = area.center() - egui::vec2(width as f32 * scale, height as f32 * scale) / 2.0;
                    self.pan = pointer - new_centered - uv * scale;
                }
            }
            let pan_drag = self.tool == Tool::Pan || response.dragged_by(egui::PointerButton::Middle);
            if self.tool != Tool::Brush || (self.paint_layer.is_some() && self.paint_layer != self.selected) {
                self.paint_points.clear();
                self.paint_layer = None;
            }
            let origin = area.center() - egui::vec2(width as f32 * scale, height as f32 * scale) / 2.0 + self.pan;
            let image_rect = egui::Rect::from_min_size(origin, egui::vec2(width as f32 * scale, height as f32 * scale));
            if self.tool == Tool::Brush && !pan_drag {
                let paintable = self.document.as_ref().and_then(|d| self.selected.and_then(|i| d.layers.get(i)))
                    .is_some_and(|l| l.visible && !l.locked && matches!(&l.content, LayerContent::Paint { .. }));
                if paintable && (response.drag_started() || response.dragged() || response.clicked()) {
                    if let Some(pointer) = ctx.input(|i| i.pointer.interact_pos()) {
                        if image_rect.contains(pointer) || !self.paint_points.is_empty() {
                            if self.paint_points.is_empty() { self.paint_layer = self.selected; }
                            let point = [((pointer.x-origin.x)/scale).clamp(0.0, width as f32 - 0.01),
                                         ((pointer.y-origin.y)/scale).clamp(0.0, height as f32 - 0.01)];
                            if self.paint_points.last().is_none_or(|p| (p[0]-point[0]).abs()+(p[1]-point[1]).abs() > 0.3)
                                && self.paint_points.len() < 2048 { self.paint_points.push(point); }
                        }
                    }
                }
                if response.drag_stopped() || response.clicked() { self.finish_stroke(); }
            }
            if response.drag_started() && !pan_drag && self.tool == Tool::Move {
                if let (Some(doc), Some(index)) = (&self.document, self.selected) {
                    if doc.layers.get(index).is_some_and(|l| !l.locked && matches!(&l.content, LayerContent::Image { .. })) {
                        self.drag_before = Some(doc.state());
                    }
                }
            }
            if response.dragged() {
                let delta = ctx.input(|i| i.pointer.delta());
                if pan_drag { self.pan += delta; }
                else if self.tool == Tool::Move { if let Some(index) = self.selected {
                    if let Some(doc) = &mut self.document {
                        if let Some(layer) = doc.layers.get_mut(index) {
                            if !layer.locked {
                                if let LayerContent::Image { x, y, .. } = &mut layer.content {
                                    *x += delta.x / scale;
                                    *y += delta.y / scale;
                                    self.edit_revision += 1;
                                    self.mark_preview();
                                }
                            }
                        }
                    }
                }}
            }
            if response.drag_stopped() {
                if let Some(before) = self.drag_before.take() { self.commit(before); }
            }
            let clip = image_rect.intersect(area);
            if clip.is_positive() {
                let tile = 18.0;
                let x0 = ((clip.left() - image_rect.left()) / tile).floor() as i32;
                let x1 = ((clip.right() - image_rect.left()) / tile).ceil() as i32;
                let y0 = ((clip.top() - image_rect.top()) / tile).floor() as i32;
                let y1 = ((clip.bottom() - image_rect.top()) / tile).ceil() as i32;
                if self.view == ViewChannel::Rgba && (x1-x0).saturating_mul(y1-y0) < 10000 {
                    for ty in y0..y1 { for tx in x0..x1 {
                        let rect = egui::Rect::from_min_size(image_rect.min + egui::vec2(tx as f32 * tile, ty as f32 * tile), egui::vec2(tile, tile)).intersect(image_rect);
                        painter.rect_filled(rect, 0.0, if (tx+ty)%2 == 0 { Color32::from_gray(92) } else { Color32::from_gray(128) });
                    }}
                }
                if self.texture.is_none() {
                    painter.text(image_rect.center(), egui::Align2::CENTER_CENTER,
                        if self.error.is_some() { "PREVIEW UNAVAILABLE" } else { "Rendering preview..." },
                        egui::FontId::proportional(17.0), Color32::from_rgb(185, 195, 210));
                }
                if let Some(texture) = &self.texture {
                    painter.image(texture.id(), image_rect,
                        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                        Color32::WHITE);
                    if self.preview_dirty_at.is_some() || self.preview_job.is_some() {
                        let badge = egui::Rect::from_min_size(image_rect.min, egui::vec2(190.0, 29.0));
                        painter.rect_filled(badge, 0.0, Color32::from_rgba_unmultiplied(13, 20, 30, 220));
                        painter.text(badge.min + egui::vec2(9.0, 15.0), egui::Align2::LEFT_CENTER,
                            "Preview updating...", egui::FontId::proportional(13.0), Color32::LIGHT_BLUE);
                    }
                    if self.view != ViewChannel::Alpha && self.document.as_ref().and_then(|d| d.uv_reference.as_ref()).is_some_and(|r| r.visible && r.opacity > 0)
                        && image_rect.width() > 200.0 && image_rect.height() > 35.0 {
                        let badge = egui::Rect::from_min_size(
                            egui::pos2(image_rect.left() + 4.0, image_rect.bottom() - 28.0),
                            egui::vec2(190.0, 24.0));
                        painter.rect_filled(badge, 0.0, Color32::from_rgba_unmultiplied(13, 20, 30, 220));
                        painter.text(badge.min + egui::vec2(8.0, 12.0), egui::Align2::LEFT_CENTER,
                            "UV GUIDE | PREVIEW ONLY", egui::FontId::proportional(12.0), Color32::LIGHT_BLUE);
                    }
                }
                if !self.paint_points.is_empty() {
                    let stroke = egui::Stroke::new((self.brush_radius*scale*2.0).clamp(1.0, 128.0),
                        Color32::from_rgb(self.brush_color[0], self.brush_color[1], self.brush_color[2]));
                    for pair in self.paint_points.windows(2) {
                        let a = origin + egui::vec2(pair[0][0]*scale, pair[0][1]*scale);
                        let b = origin + egui::vec2(pair[1][0]*scale, pair[1][1]*scale);
                        painter.line_segment([a,b], stroke);
                    }
                    if self.paint_points.len() == 1 {
                        let p = self.paint_points[0];
                        painter.circle_filled(origin + egui::vec2(p[0]*scale, p[1]*scale),
                            (self.brush_radius*scale).max(0.5), stroke.color);
                    }
                }
                painter.rect_stroke(image_rect, 0.0, egui::Stroke::new(1.0_f32, Color32::from_rgb(99, 121, 146)));
            }
            ui.allocate_rect(area, egui::Sense::hover());
        });
    }

    fn footer(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(self.view == ViewChannel::Rgb, "RGB")
                    .on_hover_text("Show color even where alpha is zero")
                    .clicked()
                {
                    self.view = ViewChannel::Rgb;
                    self.mark_preview();
                }
                if ui
                    .selectable_label(self.view == ViewChannel::Rgba, "RGBA")
                    .on_hover_text("Show image transparency")
                    .clicked()
                {
                    self.view = ViewChannel::Rgba;
                    self.mark_preview();
                }
                if ui
                    .selectable_label(self.view == ViewChannel::Alpha, "Alpha data")
                    .on_hover_text("Inspect alpha values as grayscale data")
                    .clicked()
                {
                    self.view = ViewChannel::Alpha;
                    self.mark_preview();
                }
                ui.separator();
                if self.io_job.is_some() || self.export_job.is_some() || self.preview_job.is_some()
                {
                    if ui.button("Cancel work").clicked() {
                        if let Some(job) = &self.io_job {
                            job.cancel.store(true, Ordering::Relaxed);
                        }
                        if let Some(cancel) = &self.export_cancel {
                            cancel.store(true, Ordering::Relaxed);
                        }
                        if let Some(job) = &self.preview_job {
                            job.cancel.store(true, Ordering::Relaxed);
                        }
                        if self.preview_job.is_some() {
                            self.texture = None;
                            self.preview_dirty_at = None;
                        }
                        self.status = "Cancelling current work...".into();
                    }
                    ui.separator();
                }
                if self.document.is_some()
                    && self.texture.is_none()
                    && self.preview_job.is_none()
                    && self.preview_dirty_at.is_none()
                {
                    if ui.button("Retry preview").clicked() {
                        self.mark_preview();
                    }
                }
                if let Some(error) = &self.error {
                    ui.colored_label(Color32::from_rgb(255, 128, 128), error);
                } else {
                    ui.label(&self.status);
                }
            });
        });
    }

    fn dialogs(&mut self, ctx: &egui::Context) {
        if self.show_about {
            let mut open = self.show_about;
            egui::Window::new("About SkinStrike").open(&mut open).resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
                    ui.heading("SkinStrike");
                    ui.label("A PWDS Strike app");
                    ui.separator();
                    ui.label("Local layered texture authoring and a visual project library.");
                    ui.small("Creator prototype. Exported PNGs and material choices require review in CS2 Workshop Tools.");
                });
            self.show_about = open;
        }
        if self.save_to_library {
            egui::Window::new("Save finish to collection")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    if self.gallery_collections.is_empty() {
                        ui.label("Create a collection on the Projects screen first.");
                    } else {
                        ui.label("Project title");
                        ui.text_edit_singleline(&mut self.library_title);
                        egui::ComboBox::from_label("Collection")
                            .selected_text(&self.library_save_collection)
                            .show_ui(ui, |ui| {
                                for name in &self.gallery_collections {
                                    ui.selectable_value(
                                        &mut self.library_save_collection,
                                        name.clone(),
                                        name,
                                    );
                                }
                            });
                        if ui.button("Save project").clicked() {
                            if let Some(root) = &self.library_root {
                                let request = IoRequest::SaveToLibrary(
                                    root.clone(),
                                    self.library_save_collection.clone(),
                                    self.library_title.clone(),
                                );
                                match self.start_io(request) {
                                    Ok(()) => self.save_to_library = false,
                                    Err(error) => self.error = Some(error),
                                }
                            }
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        self.save_to_library = false;
                    }
                });
        }
        if let Some(pending) = self.pending {
            egui::Window::new("Unsaved changes").collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
                ui.label("This project has unsaved edits. Save it before leaving, or discard those edits.");
                ui.horizontal(|ui| {
                    if ui.button("Save and continue").clicked() {
                        match self.save(false) {
                            Ok(()) if self.io_job.is_some() => {
                                self.pending = None;
                                self.continue_after_save = Some(pending);
                            }
                            Ok(()) => {}
                            Err(e) => self.error = Some(e),
                        }
                    }
                    if ui.button("Discard edits").clicked() {
                        self.pending = None;
                        if matches!(pending, Pending::Close) { self.allow_close = true; ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
                        else { self.perform(pending); }
                    }
                    if ui.button("Cancel").clicked() { self.pending = None; }
                });
            });
        }
        if let Some(path) = self.confirm_export.clone() {
            egui::Window::new("Replace PNG?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(format!("Replace the existing PNG at {}?", path.display()));
                    ui.horizontal(|ui| {
                        if ui.button("Replace").clicked() {
                            self.confirm_export = None;
                            self.export_to(path, true);
                        }
                        if ui.button("Cancel").clicked() {
                            self.confirm_export = None;
                        }
                    });
                });
        }
    }
}

impl eframe::App for SkinStrikeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.style_ready {
            let mut style = (*ctx.style()).clone();
            style.visuals = egui::Visuals::dark();
            style.visuals.selection.bg_fill = Color32::from_rgb(126, 69, 43);
            style.visuals.widgets.active.bg_fill = Color32::from_rgb(126, 69, 43);
            style.spacing.item_spacing = egui::vec2(8.0, 6.0);
            style.spacing.button_padding = egui::vec2(9.0, 5.0);
            style
                .text_styles
                .insert(egui::TextStyle::Body, egui::FontId::proportional(14.0));
            style
                .text_styles
                .insert(egui::TextStyle::Button, egui::FontId::proportional(14.0));
            style
                .text_styles
                .insert(egui::TextStyle::Small, egui::FontId::proportional(11.5));
            style
                .text_styles
                .insert(egui::TextStyle::Heading, egui::FontId::proportional(21.0));
            ctx.set_style(style);
            self.style_ready = true;
        }
        if ctx.input(|i| i.viewport().close_requested())
            && (self.io_job.is_some() || self.export_job.is_some())
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.error = Some("Wait for file work to finish or cancel it before closing".into());
        } else if ctx.input(|i| i.viewport().close_requested())
            && self.dirty()
            && self.pending.is_none()
            && !self.allow_close
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.pending = Some(Pending::Close);
        }
        self.shortcuts(ctx);
        self.poll_io(ctx);
        self.poll_export(ctx);
        self.poll_preview(ctx);
        self.update_preview(ctx);
        self.update_gallery(ctx);
        if self.gallery_open {
            self.gallery(ctx);
        } else {
            self.top_bar(ctx);
            self.footer(ctx);
            self.left_bar(ctx);
            self.right_bar(ctx);
            self.canvas(ctx);
        }
        self.dialogs(ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::{Pending, SkinStrikeApp};
    use crate::project_library::ProjectLibrary;
    use image::{Rgba, RgbaImage};

    #[test]
    fn view_model_cancel_and_undo_preserve_document() {
        let dir = tempfile::TempDir::new().unwrap();
        let base = dir.path().join("base.png");
        let art = dir.path().join("art.png");
        RgbaImage::from_pixel(2, 2, Rgba([1, 2, 3, 255]))
            .save(&base)
            .unwrap();
        RgbaImage::from_pixel(1, 1, Rgba([200, 1, 1, 128]))
            .save(&art)
            .unwrap();
        let mut app = SkinStrikeApp::default();
        app.open_texture(&base).unwrap();
        assert!(app.dirty());
        app.import_artwork(&art).unwrap();
        assert_eq!(app.document.as_ref().unwrap().layers.len(), 1);
        app.undo();
        assert!(app.document.as_ref().unwrap().layers.is_empty());
        app.redo();
        assert_eq!(app.document.as_ref().unwrap().layers.len(), 1);
        let path = dir.path().join("portable.json");
        app.save_project_at(&path).unwrap();
        assert!(!app.dirty());
        let before = app.document.as_ref().unwrap().state();
        assert!(app.import_artwork(&dir.path().join("missing.png")).is_err());
        assert_eq!(app.document.as_ref().unwrap().state(), before);
        app.open_project(&path).unwrap();
        assert!(!app.dirty());
        assert_eq!(app.current_project_title(), "portable");
    }

    #[test]
    fn brush_gesture_is_one_undoable_edit_and_locked_layer_keeps_work() {
        let dir = tempfile::TempDir::new().unwrap();
        let base = dir.path().join("base.png");
        RgbaImage::from_pixel(16, 16, Rgba([15, 25, 35, 255]))
            .save(&base)
            .unwrap();
        let mut app = SkinStrikeApp::default();
        app.open_texture(&base).unwrap();
        app.add_paint().unwrap();
        app.brush_radius = 2.0;
        app.paint_layer = app.selected;
        app.paint_points = vec![[3.5, 3.5], [9.5, 9.5]];
        app.finish_stroke();
        let doc = app.document.as_ref().unwrap();
        let crate::document::LayerContent::Paint { strokes } = &doc.layers[0].content else {
            panic!("paint layer expected")
        };
        assert_eq!(strokes.len(), 1);
        app.undo();
        let crate::document::LayerContent::Paint { strokes } =
            &app.document.as_ref().unwrap().layers[0].content
        else {
            panic!("paint layer expected")
        };
        assert!(strokes.is_empty());
        app.redo();
        app.document.as_mut().unwrap().layers[0].locked = true;
        let before = app.document.as_ref().unwrap().state();
        app.paint_layer = app.selected;
        app.paint_points = vec![[5.5, 5.5]];
        app.finish_stroke();
        assert_eq!(app.document.as_ref().unwrap().state(), before);
        assert!(app.error.as_ref().unwrap().contains("Unlock"));
    }

    #[test]
    fn saved_library_card_reopens_and_dirty_navigation_waits_for_a_decision() {
        let dir = tempfile::TempDir::new().unwrap();
        let base = dir.path().join("base.png");
        RgbaImage::from_pixel(16, 16, Rgba([20, 30, 40, 255]))
            .save(&base)
            .unwrap();
        let library = ProjectLibrary::create(&dir.path().join("Library")).unwrap();
        library.create_collection("Studies").unwrap();
        let mut app = SkinStrikeApp::default();
        app.open_texture(&base).unwrap();
        app.add_paint().unwrap();
        app.paint_layer = app.selected;
        app.paint_points = vec![[8.5, 8.5]];
        app.finish_stroke();
        let path = library
            .save_project("Studies", "First finish", app.document.as_ref().unwrap())
            .unwrap();
        app.open_project(&path).unwrap();
        assert!(!app.dirty());
        assert_eq!(app.current_project_title(), "First finish");
        app.add_color().unwrap();
        let before = app.document.as_ref().unwrap().state();
        app.library_target = Some(path);
        app.request(Pending::LibraryOpen);
        assert!(matches!(app.pending, Some(Pending::LibraryOpen)));
        assert_eq!(app.document.as_ref().unwrap().state(), before);
        let broken = library
            .root()
            .join("Studies")
            .join("Broken.skinstrike.json");
        std::fs::write(&broken, b"{broken").unwrap();
        assert!(app.open_project(&broken).is_err());
        assert_eq!(app.document.as_ref().unwrap().state(), before);
    }
}

#[cfg(test)]
mod workflow_tests {
    use super::{Pending, SkinStrikeApp};
    use image::{ImageReader, Rgba, RgbaImage};
    use std::time::{Duration, Instant};

    #[test]
    fn cancelled_navigation_keeps_unsaved_work_and_export_runs_on_worker() {
        let dir = tempfile::TempDir::new().unwrap();
        let base = dir.path().join("base.png");
        RgbaImage::from_pixel(128, 128, Rgba([20, 30, 40, 255]))
            .save(&base)
            .unwrap();
        let mut app = SkinStrikeApp::default();
        app.open_texture(&base).unwrap();
        app.add_color().unwrap();
        let state = app.document.as_ref().unwrap().state();
        app.request(Pending::Open);
        assert!(matches!(app.pending, Some(Pending::Open)));
        assert_eq!(app.document.as_ref().unwrap().state(), state);
        app.pending = None;
        assert_eq!(app.document.as_ref().unwrap().state(), state);
        let output = dir.path().join("worker.png");
        let started = Instant::now();
        app.export_to(output.clone(), false);
        assert!(started.elapsed() < Duration::from_secs(2));
        assert!(app.export_job.is_some());
        let result = loop {
            let rx = app.export_job.as_ref().unwrap();
            match rx.try_recv() {
                Ok(done) => break done,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    panic!("export worker disconnected")
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    assert!(
                        started.elapsed() < Duration::from_secs(10),
                        "export timed out"
                    );
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        };
        assert_eq!(result.unwrap(), output);
        let decoded = ImageReader::open(&output).unwrap().decode().unwrap();
        assert_eq!((decoded.width(), decoded.height()), (128, 128));
        assert_eq!(app.document.as_ref().unwrap().state(), state);
    }
}

#[cfg(test)]
mod worker_regressions {
    use super::{IoRequest, SkinStrikeApp};
    use eframe::egui;
    use image::{Rgba, RgbaImage};
    use std::time::{Duration, Instant};

    fn poll_until_idle(app: &mut SkinStrikeApp, ctx: &egui::Context) {
        let started = Instant::now();
        while app.io_job.is_some() {
            app.poll_io(ctx);
            assert!(
                started.elapsed() < Duration::from_secs(10),
                "file worker timed out"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn cancelled_import_and_stale_revision_never_replace_current_work() {
        let dir = tempfile::TempDir::new().unwrap();
        let base = dir.path().join("base.png");
        let art = dir.path().join("art.png");
        RgbaImage::from_pixel(16, 16, Rgba([1, 2, 3, 255]))
            .save(&base)
            .unwrap();
        RgbaImage::from_pixel(16, 16, Rgba([200, 2, 3, 255]))
            .save(&art)
            .unwrap();
        let ctx = egui::Context::default();
        let mut app = SkinStrikeApp::default();
        app.open_texture(&base).unwrap();
        let original = app.document.as_ref().unwrap().state();
        app.start_io(IoRequest::Import(art.clone())).unwrap();
        app.io_job
            .as_ref()
            .unwrap()
            .cancel
            .store(true, std::sync::atomic::Ordering::Relaxed);
        poll_until_idle(&mut app, &ctx);
        assert_eq!(app.document.as_ref().unwrap().state(), original);
        assert!(app.undo.is_empty());

        app.start_io(IoRequest::Import(art)).unwrap();
        app.add_color().unwrap();
        let current = app.document.as_ref().unwrap().state();
        poll_until_idle(&mut app, &ctx);
        assert_eq!(app.document.as_ref().unwrap().state(), current);
        assert_eq!(app.undo.len(), 1);
    }

    #[test]
    fn preview_revision_discard_and_export_path_guard() {
        let dir = tempfile::TempDir::new().unwrap();
        let base = dir.path().join("base.png");
        RgbaImage::from_pixel(32, 32, Rgba([1, 2, 3, 255]))
            .save(&base)
            .unwrap();
        let ctx = egui::Context::default();
        let mut app = SkinStrikeApp::default();
        app.open_texture(&base).unwrap();
        std::thread::sleep(Duration::from_millis(65));
        app.update_preview(&ctx);
        assert!(app.preview_job.is_some());
        app.mark_preview();
        let started = Instant::now();
        while app.preview_job.is_some() {
            app.poll_preview(&ctx);
            assert!(started.elapsed() < Duration::from_secs(10));
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(app.texture.is_none());
        assert!(app.preview_dirty_at.is_some());
        let project = dir.path().join("work.json");
        app.save_project_at(&project).unwrap();
        app.export_to(project.clone(), true);
        assert!(app.export_job.is_none());
        assert!(app.error.as_ref().unwrap().contains(".png extension"));
        app.export_to(base.clone(), true);
        assert!(app.export_job.is_none());
        assert!(app.error.as_ref().unwrap().contains("base PNG"));
        assert!(project.is_file());
        assert!(base.is_file());
    }
}

#[cfg(test)]
mod layer_ui_regressions {
    use super::{LayerAction, SkinStrikeApp};
    use image::{Rgba, RgbaImage};

    #[test]
    fn copy_creates_unique_name_and_rejects_limit_without_history_change() {
        let dir = tempfile::TempDir::new().unwrap();
        let base = dir.path().join("base.png");
        RgbaImage::from_pixel(1, 1, Rgba([1, 2, 3, 255]))
            .save(&base)
            .unwrap();
        let mut app = SkinStrikeApp::default();
        app.open_texture(&base).unwrap();
        app.add_color().unwrap();
        app.layer_action(LayerAction::Duplicate).unwrap();
        let names: Vec<_> = app
            .document
            .as_ref()
            .unwrap()
            .layers
            .iter()
            .map(|l| l.name.clone())
            .collect();
        assert_eq!(names, vec!["Color wash", "Color wash copy"]);
        app.layer_action(LayerAction::Duplicate).unwrap();
        assert_eq!(
            app.document.as_ref().unwrap().layers[2].name,
            "Color wash copy 2"
        );
        let doc = app.document.as_mut().unwrap();
        for n in 4..=128 {
            doc.add_color_layer(&format!("Layer {n}"), [1, 2, 3], 1)
                .unwrap();
        }
        app.selected = Some(0);
        let before = app.document.as_ref().unwrap().state();
        let undo_len = app.undo.len();
        assert!(
            app.layer_action(LayerAction::Duplicate)
                .unwrap_err()
                .contains("128 layers")
        );
        assert_eq!(app.document.as_ref().unwrap().state(), before);
        assert_eq!(app.undo.len(), undo_len);
    }
}

#[cfg(test)]
mod async_state_regressions {
    use super::{IoRequest, SkinStrikeApp};
    use eframe::egui;
    use image::{Rgba, RgbaImage};
    use std::time::{Duration, Instant};

    #[test]
    fn save_snapshot_does_not_mark_newer_edits_clean() {
        let dir = tempfile::TempDir::new().unwrap();
        let base = dir.path().join("base.png");
        let project = dir.path().join("work.json");
        RgbaImage::from_pixel(16, 16, Rgba([1, 2, 3, 255]))
            .save(&base)
            .unwrap();
        let mut app = SkinStrikeApp::default();
        app.open_texture(&base).unwrap();
        app.add_color().unwrap();
        app.start_io(IoRequest::Save(project.clone())).unwrap();
        app.add_color().unwrap();
        let started = Instant::now();
        let ctx = egui::Context::default();
        while app.io_job.is_some() {
            app.poll_io(&ctx);
            assert!(started.elapsed() < Duration::from_secs(10));
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(app.dirty());
        assert_eq!(app.document.as_ref().unwrap().layers.len(), 2);
        assert_eq!(skinstrike_layer_count(&project), 1);
    }

    fn skinstrike_layer_count(path: &std::path::Path) -> usize {
        crate::document::load_document(path).unwrap().layers.len()
    }

    #[test]
    fn failed_new_preview_clears_previous_texture() {
        let dir = tempfile::TempDir::new().unwrap();
        let base = dir.path().join("base.png");
        RgbaImage::from_pixel(16, 16, Rgba([1, 2, 3, 255]))
            .save(&base)
            .unwrap();
        let ctx = egui::Context::default();
        let mut app = SkinStrikeApp::default();
        app.open_texture(&base).unwrap();
        std::thread::sleep(Duration::from_millis(70));
        app.update_preview(&ctx);
        let started = Instant::now();
        while app.preview_job.is_some() {
            app.poll_preview(&ctx);
            assert!(started.elapsed() < Duration::from_secs(10));
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(app.texture.is_some());
        app.add_color().unwrap();
        app.document.as_mut().unwrap().layers[0].name.clear();
        app.mark_preview();
        std::thread::sleep(Duration::from_millis(70));
        app.update_preview(&ctx);
        let started = Instant::now();
        while app.preview_job.is_some() {
            app.poll_preview(&ctx);
            assert!(started.elapsed() < Duration::from_secs(10));
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(app.texture.is_none());
        assert!(app.error.as_ref().unwrap().contains("Preview unavailable"));
    }
}

#[cfg(test)]
mod channel_ui_regressions {
    use super::{AuxKind, IoRequest, SkinStrikeApp};
    use crate::document::AlphaMode;
    use eframe::egui;
    use image::{Rgba, RgbaImage};
    use std::time::{Duration, Instant};

    fn finish_io(app: &mut SkinStrikeApp, ctx: &egui::Context) {
        let started = Instant::now();
        while app.io_job.is_some() {
            app.poll_io(ctx);
            assert!(started.elapsed() < Duration::from_secs(10));
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn alpha_import_worker_undo_redo_and_bad_input_preserve_state() {
        let dir = tempfile::TempDir::new().unwrap();
        let base = dir.path().join("base.png");
        let alpha = dir.path().join("alpha.png");
        let bad = dir.path().join("bad.png");
        RgbaImage::from_pixel(16, 16, Rgba([30, 50, 70, 255]))
            .save(&base)
            .unwrap();
        RgbaImage::from_pixel(16, 16, Rgba([90, 90, 90, 255]))
            .save(&alpha)
            .unwrap();
        RgbaImage::from_pixel(16, 16, Rgba([90, 20, 90, 255]))
            .save(&bad)
            .unwrap();
        let ctx = egui::Context::default();
        let mut app = SkinStrikeApp::default();
        app.open_texture(&base).unwrap();
        let before = app.document.as_ref().unwrap().state();
        app.start_io(IoRequest::ImportAux(alpha, AuxKind::Alpha))
            .unwrap();
        finish_io(&mut app, &ctx);
        assert_eq!(
            app.document.as_ref().unwrap().alpha_mode,
            AlphaMode::ImportedData
        );
        assert!(app.dirty());
        assert_eq!(app.undo.len(), 1);
        app.undo();
        assert_eq!(app.document.as_ref().unwrap().state(), before);
        app.redo();
        assert_eq!(
            app.document.as_ref().unwrap().alpha_mode,
            AlphaMode::ImportedData
        );
        let current = app.document.as_ref().unwrap().state();
        let undo_len = app.undo.len();
        app.start_io(IoRequest::ImportAux(bad, AuxKind::Alpha))
            .unwrap();
        finish_io(&mut app, &ctx);
        assert_eq!(app.document.as_ref().unwrap().state(), current);
        assert_eq!(app.undo.len(), undo_len);
        assert!(app.error.as_ref().unwrap().contains("opaque grayscale"));
    }

    #[test]
    fn cancelled_and_stale_uv_imports_keep_current_document() {
        let dir = tempfile::TempDir::new().unwrap();
        let base = dir.path().join("base.png");
        let guide = dir.path().join("guide.png");
        RgbaImage::from_pixel(16, 16, Rgba([30, 50, 70, 255]))
            .save(&base)
            .unwrap();
        RgbaImage::from_pixel(16, 16, Rgba([255, 240, 20, 90]))
            .save(&guide)
            .unwrap();
        let ctx = egui::Context::default();
        let mut app = SkinStrikeApp::default();
        app.open_texture(&base).unwrap();
        let original = app.document.as_ref().unwrap().state();
        app.start_io(IoRequest::ImportAux(guide.clone(), AuxKind::UvReference))
            .unwrap();
        app.io_job
            .as_ref()
            .unwrap()
            .cancel
            .store(true, std::sync::atomic::Ordering::Relaxed);
        finish_io(&mut app, &ctx);
        assert_eq!(app.document.as_ref().unwrap().state(), original);
        app.start_io(IoRequest::ImportAux(guide, AuxKind::UvReference))
            .unwrap();
        app.add_color().unwrap();
        let current = app.document.as_ref().unwrap().state();
        finish_io(&mut app, &ctx);
        assert_eq!(app.document.as_ref().unwrap().state(), current);
        assert!(app.document.as_ref().unwrap().uv_reference.is_none());
    }
}

#[cfg(test)]
mod layer_lock_regressions {
    use super::{LayerAction, SkinStrikeApp};
    use image::{Rgba, RgbaImage};

    #[test]
    fn locked_layers_preserve_order_and_cannot_be_deleted_until_unlocked() {
        let dir = tempfile::TempDir::new().unwrap();
        let base = dir.path().join("base.png");
        RgbaImage::from_pixel(2, 2, Rgba([1, 2, 3, 255]))
            .save(&base)
            .unwrap();
        let mut app = SkinStrikeApp::default();
        app.open_texture(&base).unwrap();
        app.add_color().unwrap();
        app.add_color().unwrap();
        app.document.as_mut().unwrap().layers[0].locked = true;
        app.selected = Some(0);
        let before = app.document.as_ref().unwrap().state();
        let undo_len = app.undo.len();
        assert!(
            app.layer_action(LayerAction::Delete)
                .unwrap_err()
                .contains("Unlock")
        );
        assert!(
            app.layer_action(LayerAction::Up)
                .unwrap_err()
                .contains("Unlock")
        );
        assert_eq!(app.document.as_ref().unwrap().state(), before);
        assert_eq!(app.undo.len(), undo_len);
        app.selected = Some(1);
        assert!(
            app.layer_action(LayerAction::Down)
                .unwrap_err()
                .contains("adjacent")
        );
        app.selected = Some(0);
        app.layer_action(LayerAction::Duplicate).unwrap();
        assert!(!app.document.as_ref().unwrap().layers[1].locked);
    }
}

#[cfg(test)]
mod alpha_preview_regressions {
    use super::{ViewChannel, render_preview_image};
    use crate::document::{export_png, new_document, prepare_render_cache};
    use image::{ImageReader, Rgba, RgbaImage};

    #[test]
    fn alpha_view_stays_exact_grayscale_when_colored_uv_guide_is_enabled() {
        let dir = tempfile::TempDir::new().unwrap();
        let base_path = dir.path().join("base.png");
        let data_path = dir.path().join("alpha.png");
        let guide_path = dir.path().join("guide.png");
        let project_path = dir.path().join("work.json");
        let values = [33u8, 66, 99, 132, 165, 222];
        RgbaImage::from_pixel(3, 2, Rgba([80, 90, 100, 255]))
            .save(&base_path)
            .unwrap();
        let mut data = RgbaImage::new(3, 2);
        for (pixel, value) in data.pixels_mut().zip(values) {
            *pixel = Rgba([value, value, value, 255]);
        }
        data.save(&data_path).unwrap();
        RgbaImage::from_pixel(3, 2, Rgba([0, 255, 255, 255]))
            .save(&guide_path)
            .unwrap();
        let mut doc = new_document(&base_path, &project_path).unwrap();
        doc.set_alpha_data(&data_path).unwrap();
        doc.set_uv_reference(&guide_path).unwrap();
        let cache = prepare_render_cache(&doc, &|| false).unwrap();
        let preview = render_preview_image(&doc, &cache, ViewChannel::Alpha, &|| false).unwrap();
        assert_eq!(preview.pixels.len(), values.len());
        for (pixel, value) in preview.pixels.iter().zip(values) {
            assert_eq!(pixel.to_array(), [value, value, value, 255]);
        }
        let with_guide = render_preview_image(&doc, &cache, ViewChannel::Rgb, &|| false).unwrap();
        doc.uv_reference.as_mut().unwrap().visible = false;
        let without_guide =
            render_preview_image(&doc, &cache, ViewChannel::Rgb, &|| false).unwrap();
        assert_ne!(with_guide.pixels[0], without_guide.pixels[0]);
        doc.uv_reference.as_mut().unwrap().visible = true;
        let exported = dir.path().join("export.png");
        export_png(&doc, &project_path, &exported).unwrap();
        let decoded = ImageReader::open(&exported)
            .unwrap()
            .decode()
            .unwrap()
            .to_rgba8();
        for (pixel, value) in decoded.pixels().zip(values) {
            assert_eq!(pixel.0[3], value);
        }
    }
}
