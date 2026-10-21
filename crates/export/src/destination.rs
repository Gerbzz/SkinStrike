//! Portable output-name checks and native no-replace publication.

use std::collections::HashSet;

use crate::ExportError;

// Deliberately conservative: reject case aliases even on case-sensitive filesystems.
// Uppercase expansion also catches common aliases such as Straße / STRASSE.
// This is a portability check, not full Unicode normalization.
fn name_key(name: &str) -> String {
    name.to_uppercase().to_lowercase()
}

pub(super) fn validate_names(names: &[String]) -> Result<(), ExportError> {
    let mut seen = HashSet::new();
    for name in names {
        let stem = name
            .split('.')
            .next()
            .unwrap_or_default()
            .to_ascii_uppercase();
        let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || ["COM", "LPT"].iter().any(|&prefix| {
                stem.strip_prefix(prefix).is_some_and(|n| {
                    matches!(n, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
                })
            });
        if name.is_empty()
            || matches!(name.as_str(), "." | "..")
            || name.ends_with(['.', ' '])
            || name.len() > 255
            || name.encode_utf16().count() > 255
            || reserved
            || name.chars().any(|c| {
                c.is_control() || matches!(c, '/' | '\\' | ':' | '<' | '>' | '"' | '|' | '?' | '*')
            })
        {
            return Err(ExportError::UnsafeOutput(name.clone()));
        }
        if !seen.insert(name_key(name)) {
            return Err(ExportError::DuplicateOutput(name.clone()));
        }
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn preflight_directory(
    dir: &std::path::Path,
    names: &[String],
) -> Result<(), ExportError> {
    let planned: HashSet<_> = names.iter().map(|name| name_key(name)).collect();
    let entries = std::fs::read_dir(dir).map_err(|e| ExportError::Folder(dir.to_path_buf(), e))?;
    for entry in entries {
        let entry = entry.map_err(|e| ExportError::Folder(dir.to_path_buf(), e))?;
        // Directory entries include directories and dangling symlinks, unlike Path::exists.
        if entry
            .file_name()
            .to_str()
            .is_some_and(|name| planned.contains(&name_key(name)))
        {
            return Err(ExportError::ExistingOutput(entry.path()));
        }
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn stage_output(
    dir: &std::path::Path,
    bytes: &[u8],
) -> std::io::Result<tempfile::NamedTempFile> {
    use std::io::Write;

    let mut staged = tempfile::Builder::new()
        .prefix(".subcraft-export-")
        .suffix(".tmp")
        .tempfile_in(dir)?;
    staged.write_all(bytes)?;
    staged.as_file().sync_all()?;
    Ok(staged)
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn publish_output(
    staged: tempfile::NamedTempFile,
    destination: &std::path::Path,
) -> Result<(), ExportError> {
    // Preflight alone is racy. persist_noclobber also refuses a destination created
    // afterwards. On failure its returned NamedTempFile is dropped and cleaned up.
    // Some filesystems use link/unlink: a crash can leave a second temporary link.
    staged
        .persist_noclobber(destination)
        .map(|_| ())
        .map_err(|e| ExportError::Write(destination.to_path_buf(), e.error.to_string()))
}
