//! SkinStrike evaluation boundary. Upstream builds do not enable this module.
//! No environment or user files are modified by path validation.

use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

pub const HOME_ENV: &str = "SKINSTRIKE_CANDIDATE_HOME";
pub const TITLE: &str = "SkinStrike 3D candidate — SubCraft 0.1.7 foundation";

#[derive(Debug, PartialEq, Eq)]
pub struct CandidatePaths {
    pub home: PathBuf,
    pub persistence: PathBuf,
}

impl CandidatePaths {
    pub fn validate(home: &Path, data: &Path, library: &Path) -> Result<Self, String> {
        if !home.is_absolute()
            || home
                .components()
                .any(|part| matches!(part, Component::ParentDir))
        {
            return Err("Candidate home must be an absolute path without parent traversal".into());
        }
        if data != home.join("data") || library != home.join("library") {
            return Err(
                "Candidate data and library must use their isolated home subfolders".into(),
            );
        }
        for path in [
            home.to_path_buf(),
            data.to_path_buf(),
            library.to_path_buf(),
            home.join("ui-state.ron"),
        ] {
            reject_redirected_ancestors(&path)?;
        }
        Ok(Self {
            home: home.into(),
            persistence: home.join("ui-state.ron"),
        })
    }
}

fn reject_redirected_ancestors(path: &Path) -> Result<(), String> {
    let mut cursor = PathBuf::new();
    for component in path.components() {
        cursor.push(component.as_os_str());
        match std::fs::symlink_metadata(&cursor) {
            Ok(metadata) => {
                #[cfg(windows)]
                let redirected = {
                    use std::os::windows::fs::MetadataExt;
                    metadata.file_attributes() & 0x400 != 0 // FILE_ATTRIBUTE_REPARSE_POINT
                };
                #[cfg(not(windows))]
                let redirected = metadata.file_type().is_symlink();
                if redirected {
                    return Err(format!(
                        "Candidate state cannot use a redirected path: {}",
                        cursor.display()
                    ));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "Cannot inspect candidate state path {}: {error}",
                    cursor.display()
                ));
            }
        }
    }
    Ok(())
}

pub fn environment() -> Result<CandidatePaths, String> {
    let required = |name| {
        std::env::var_os(name)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .ok_or_else(|| {
                format!("Missing {name}; use the SkinStrike-3D.ps1 candidate entry point")
            })
    };
    CandidatePaths::validate(
        &required(HOME_ENV)?,
        &required("SUBCRAFT_DATA")?,
        &required("SUBCRAFT_LIBRARY")?,
    )
}

pub fn validate_arguments(arguments: &[OsString]) -> Result<(), String> {
    if arguments.iter().any(|argument| {
        [
            "--apply-update",
            "--resume-update",
            "--check-updates",
            "--auto-update-fixture",
        ]
        .iter()
        .any(|flag| argument == *flag)
    }) {
        return Err("Upstream update operations are disabled in the SkinStrike candidate".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_paths_are_isolated_and_validation_has_no_file_side_effects() {
        let home = std::env::temp_dir().join("skinstrike-candidate-path-validation");
        let paths =
            CandidatePaths::validate(&home, &home.join("data"), &home.join("library")).unwrap();
        assert_eq!(paths.persistence, home.join("ui-state.ron"));
        assert!(
            CandidatePaths::validate(
                &home,
                &home.join("old-subcraft-data"),
                &home.join("library")
            )
            .is_err()
        );
        assert!(
            CandidatePaths::validate(
                Path::new("relative"),
                Path::new("relative/data"),
                Path::new("relative/library")
            )
            .is_err()
        );
        let traversal = home.join("..").join("legacy");
        assert!(
            CandidatePaths::validate(
                &traversal,
                &traversal.join("data"),
                &traversal.join("library")
            )
            .is_err()
        );
    }

    #[test]
    fn candidate_rejects_update_install_and_resume_arguments() {
        for flag in [
            "--apply-update",
            "--resume-update",
            "--check-updates",
            "--auto-update-fixture",
        ] {
            assert!(validate_arguments(&[flag.into()]).is_err());
        }
        assert!(validate_arguments(&["model.glb".into(), "--demo".into()]).is_ok());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn candidate_ui_never_constructs_an_upstream_updater() {
        let session = subcraft_app::Session::new(subcraft_mesh::primitives::cube(2.0), 128);
        assert!(crate::state::UiState::new(session, None).updater.is_none());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn candidate_marker_survives_the_per_frame_title_refresh() {
        let session = subcraft_app::Session::new(subcraft_mesh::primitives::cube(2.0), 128);
        let mut state = crate::state::UiState::new(session, None);
        let model = state.session.scene().mesh.name.clone();
        state
            .session
            .execute(subcraft_app::Command::AddLayer(
                subcraft_app::NewLayer::Paint,
            ))
            .unwrap();
        let title = crate::dialogs::native_title(&state);
        assert!(title.starts_with(TITLE));
        assert!(title.contains(&model));
        assert!(title.ends_with(" *"));
    }

    #[cfg(unix)]
    #[test]
    fn candidate_rejects_an_existing_library_symlink() {
        let temp = std::env::temp_dir();
        let home = temp.join(format!(
            "skinstrike-isolation-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&home).unwrap();
        std::fs::create_dir(home.join("data")).unwrap();
        std::os::unix::fs::symlink(home.join("data"), home.join("library")).unwrap();
        assert!(
            CandidatePaths::validate(&home, &home.join("data"), &home.join("library")).is_err()
        );
        assert!(home.starts_with(&temp));
        std::fs::remove_dir_all(home).unwrap();
    }
}
