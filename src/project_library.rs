//! A local, bounded project index. Covers are rendered from each saved document.
use crate::document::{FinishDocument, load_document, render_document, save_document};
use image::RgbaImage;
use std::io::Write;
use std::path::{Path, PathBuf};

const SUFFIX: &str = ".skinstrike.json";
const MAX_PAGE_SIZE: usize = 24;

pub struct ProjectLibrary {
    root: PathBuf,
}

pub struct GalleryCard {
    pub path: PathBuf,
    pub title: String,
    pub collection: String,
    pub thumbnail: Option<RgbaImage>,
    pub error: Option<String>,
}

pub struct GalleryPage {
    pub total: usize,
    pub cards: Vec<GalleryCard>,
}

impl ProjectLibrary {
    pub fn create(root: &Path) -> Result<Self, String> {
        std::fs::create_dir_all(root).map_err(|e| format!("Cannot create library: {e}"))?;
        Self::open(root)
    }

    pub fn open(root: &Path) -> Result<Self, String> {
        if !root.is_dir() {
            return Err("Library folder is missing; choose another folder".into());
        }
        let root = root
            .canonicalize()
            .map_err(|e| format!("Cannot open library: {e}"))?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn create_collection(&self, name: &str) -> Result<PathBuf, String> {
        let name = safe_name(name)?;
        let path = self.root.join(name);
        if path.exists() {
            return Err("Collection already exists".into());
        }
        std::fs::create_dir(&path).map_err(|e| format!("Cannot create collection: {e}"))?;
        Ok(path)
    }

    pub fn collections(&self) -> Result<Vec<String>, String> {
        let mut found = Vec::new();
        for entry in std::fs::read_dir(&self.root).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if safe_name(&name).is_ok() {
                    found.push(name);
                }
            }
        }
        found.sort_by_key(|s| s.to_lowercase());
        Ok(found)
    }

    pub fn save_project(
        &self,
        collection: &str,
        title: &str,
        doc: &FinishDocument,
    ) -> Result<PathBuf, String> {
        let collection = safe_name(collection)?;
        let title = safe_name(title)?;
        let folder = self.root.join(collection);
        if !folder.is_dir()
            || folder
                .symlink_metadata()
                .map_err(|e| e.to_string())?
                .file_type()
                .is_symlink()
        {
            return Err("Collection is missing or unavailable".into());
        }
        if !doc.project_id.chars().all(|c| c.is_ascii_hexdigit()) || doc.project_id.len() < 16 {
            return Err("Invalid project identity".into());
        }
        let path = folder.join(format!("{} -- {}{}", title, &doc.project_id[..16], SUFFIX));
        let identity_suffix = format!(" -- {}{}", &doc.project_id[..16], SUFFIX);
        for entry in std::fs::read_dir(&folder).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if entry.file_type().map_err(|e| e.to_string())?.is_file()
                && entry
                    .file_name()
                    .to_string_lossy()
                    .ends_with(&identity_suffix)
                && entry.path() != path
            {
                return Err("This project already has a file in the collection; use its existing title or save a separate copy".into());
            }
        }
        if path
            .symlink_metadata()
            .is_ok_and(|m| m.file_type().is_symlink())
        {
            return Err("Project path cannot be a link".into());
        }
        save_document(doc, &path, path.exists())?;
        Ok(path)
    }

    pub fn page(
        &self,
        collection: Option<&str>,
        query: &str,
        offset: usize,
        limit: usize,
    ) -> Result<GalleryPage, String> {
        if limit == 0 || limit > MAX_PAGE_SIZE {
            return Err("Page size must be 1 to 24".into());
        }
        let collections = match collection {
            Some(name) => {
                let name = safe_name(name)?;
                if !self.collections()?.contains(&name.to_string()) {
                    return Err("Collection is missing".into());
                }
                vec![name.to_string()]
            }
            None => self.collections()?,
        };
        let query = query.trim().to_lowercase();
        let mut found = Vec::new();
        for name in collections {
            for entry in std::fs::read_dir(self.root.join(&name)).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                if !entry.file_type().map_err(|e| e.to_string())?.is_file() {
                    continue;
                }
                let file = entry.file_name().to_string_lossy().into_owned();
                let Some(stem) = file.strip_suffix(SUFFIX) else {
                    continue;
                };
                let title = stem
                    .rsplit_once(" -- ")
                    .map_or(stem, |(t, _)| t)
                    .to_string();
                if !title.to_lowercase().contains(&query) {
                    continue;
                }
                let changed = entry.metadata().ok().and_then(|m| m.modified().ok());
                found.push((entry.path(), name.clone(), title, changed));
            }
        }
        found.sort_by(|a, b| b.3.cmp(&a.3).then_with(|| a.0.cmp(&b.0)));
        let total = found.len();
        let mut cards = Vec::new();
        for (path, collection, title, _) in found.into_iter().skip(offset).take(limit) {
            let rendered = load_document(&path).and_then(|doc| render_document(&doc, Some(256)));
            let (thumbnail, error) = match rendered {
                Ok(image) => (Some(image), None),
                Err(e) => (None, Some(e)),
            };
            cards.push(GalleryCard {
                path,
                title,
                collection,
                thumbnail,
                error,
            });
        }
        Ok(GalleryPage { total, cards })
    }
}

fn safe_name(input: &str) -> Result<&str, String> {
    let name = input.trim();
    if name.is_empty()
        || name.len() > 80
        || name != input
        || name.ends_with('.')
        || name.ends_with(' ')
        || name
            .chars()
            .any(|c| c.is_control() || "<>:\"/\\|?*".contains(c))
        || name == "."
        || name == ".."
    {
        return Err("Use a simple collection or project name".into());
    }
    let stem = name.split('.').next().unwrap_or(name).to_ascii_uppercase();
    if matches!(
        stem.as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    ) {
        return Err("That name is reserved by Windows".into());
    }
    Ok(name)
}

/// Persist the selected local folder outside the project library itself.
pub fn write_library_preference(config: &Path, root: &Path) -> Result<(), String> {
    let parent = config.parent().ok_or("Settings folder is missing")?;
    std::fs::create_dir_all(parent).map_err(|e| format!("Cannot create settings folder: {e}"))?;
    let canonical = root
        .canonicalize()
        .map_err(|e| format!("Library folder is missing: {e}"))?;
    if !canonical.is_dir() {
        return Err("Library folder is missing".into());
    }
    let mut pending = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    pending
        .write_all(canonical.to_string_lossy().as_bytes())
        .map_err(|e| e.to_string())?;
    pending
        .flush()
        .and_then(|_| pending.as_file().sync_all())
        .map_err(|e| e.to_string())?;
    pending.persist(config).map_err(|e| e.error.to_string())?;
    Ok(())
}

pub fn read_library_preference(config: &Path) -> Result<Option<PathBuf>, String> {
    if !config.exists() {
        return Ok(None);
    }
    let bytes = std::fs::read(config).map_err(|e| e.to_string())?;
    if bytes.len() > 4096 {
        return Err("Library setting is too long".into());
    }
    let value = String::from_utf8(bytes).map_err(|e| e.to_string())?;
    let path = PathBuf::from(value);
    if !path.is_absolute() {
        return Err("Library setting must be an absolute folder".into());
    }
    Ok(Some(path))
}
