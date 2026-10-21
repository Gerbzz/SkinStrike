use image::{Rgba, RgbaImage};
use skinstrike::document::{
    FinishDocument, new_document, save_document, save_document_with_cancel,
};
use std::cell::{Cell, RefCell};
use std::path::Path;

fn document(base: &Path, target: &Path) -> FinishDocument {
    RgbaImage::from_pixel(2, 2, Rgba([7, 8, 9, 128]))
        .save(base)
        .unwrap();
    new_document(base, target).unwrap()
}

// The existing cancellation hook runs after the temporary file has been synced.
// Use that boundary to reproduce another writer deterministically, without sleeps.
fn save_with_concurrent_change(
    doc: &FinishDocument,
    target: &Path,
    overwrite: bool,
    change: impl Fn(),
) -> Result<(), String> {
    let calls = Cell::new(0);
    let result = save_document_with_cancel(doc, target, overwrite, &|| {
        calls.set(calls.get() + 1);
        if calls.get() == 3 {
            change();
        }
        false
    });
    assert!(calls.get() >= 3, "save must reach the pre-commit boundary");
    result
}

#[test]
fn staged_save_preserves_a_different_project_moved_into_the_destination() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().join("base.png");
    let target = dir.path().join("finish.json");
    let mut doc = document(&base, &target);
    save_document(&doc, &target, false).unwrap();
    let mut other = doc.clone();
    other.project_id = "fedcba9876543210".repeat(4);
    let replacement = serde_json::to_vec(&other).unwrap();
    doc.add_color_layer("Local edit", [80, 90, 100], 128)
        .unwrap();

    let result = save_with_concurrent_change(&doc, &target, true, || {
        std::fs::write(&target, &replacement).unwrap();
    });

    assert!(
        result.is_err(),
        "save accepted a different destination project"
    );
    assert_eq!(std::fs::read(&target).unwrap(), replacement);
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 3);
}

#[test]
fn staged_save_preserves_concurrent_edits_to_the_same_project() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().join("base.png");
    let target = dir.path().join("finish.json");
    let mut doc = document(&base, &target);
    save_document(&doc, &target, false).unwrap();
    let mut other = doc.clone();
    other
        .add_color_layer("External edit", [11, 22, 33], 255)
        .unwrap();
    let replacement = serde_json::to_vec(&other).unwrap();
    doc.add_color_layer("Local edit", [80, 90, 100], 128)
        .unwrap();

    let result = save_with_concurrent_change(&doc, &target, true, || {
        std::fs::write(&target, &replacement).unwrap();
    });

    assert!(result.is_err(), "save discarded another writer's edits");
    assert_eq!(std::fs::read(&target).unwrap(), replacement);
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 3);
}

#[test]
fn staged_save_preserves_a_newly_created_destination_even_with_overwrite_enabled() {
    for overwrite in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().join("base.png");
        let target = dir.path().join("finish.json");
        let doc = document(&base, &target);
        let replacement = b"another writer's file";

        let result = save_with_concurrent_change(&doc, &target, overwrite, || {
            std::fs::write(&target, replacement).unwrap();
        });

        assert!(
            result.is_err(),
            "save clobbered a newly created destination"
        );
        assert_eq!(std::fs::read(&target).unwrap(), replacement);
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 3);
    }
}

#[test]
fn staged_save_preserves_an_invalid_replacement_file() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().join("base.png");
    let target = dir.path().join("finish.json");
    let doc = document(&base, &target);
    save_document(&doc, &target, false).unwrap();
    let replacement = b"unfinished external project write";

    let result = save_with_concurrent_change(&doc, &target, true, || {
        std::fs::write(&target, replacement).unwrap();
    });

    assert!(
        result.is_err(),
        "save replaced an unrecognized destination file"
    );
    assert_eq!(std::fs::read(&target).unwrap(), replacement);
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 3);
}

#[test]
fn staged_save_does_not_recreate_a_deleted_destination() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().join("base.png");
    let target = dir.path().join("finish.json");
    let doc = document(&base, &target);
    save_document(&doc, &target, false).unwrap();

    let result = save_with_concurrent_change(&doc, &target, true, || {
        std::fs::remove_file(&target).unwrap();
    });

    assert!(
        result.is_err(),
        "save recreated a file removed by another writer"
    );
    assert!(!target.exists());
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 2);
}

#[test]
fn staged_save_preserves_unknown_json_fields_added_by_another_writer() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().join("base.png");
    let target = dir.path().join("finish.json");
    let doc = document(&base, &target);
    save_document(&doc, &target, false).unwrap();
    let mut value = serde_json::to_value(&doc).unwrap();
    value["external_metadata"] = serde_json::json!({"artist_note": "Keep this"});
    let replacement = serde_json::to_vec(&value).unwrap();

    let result = save_with_concurrent_change(&doc, &target, true, || {
        std::fs::write(&target, &replacement).unwrap();
    });

    assert!(result.is_err(), "save discarded unknown destination data");
    assert_eq!(std::fs::read(&target).unwrap(), replacement);
}

#[test]
fn overlapping_skinstrike_save_is_rejected_before_it_can_replace_the_destination() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().join("base.png");
    let target = dir.path().join("finish.json");
    let mut doc = document(&base, &target);
    save_document(&doc, &target, false).unwrap();
    let mut competing = doc.clone();
    competing
        .add_color_layer("Competing", [1, 2, 3], 255)
        .unwrap();
    doc.add_color_layer("Winner", [80, 90, 100], 128).unwrap();
    let attempt = RefCell::new(None);

    let result = save_with_concurrent_change(&doc, &target, true, || {
        attempt.replace(Some(save_document(&competing, &target, true)));
    });

    assert!(
        attempt
            .into_inner()
            .unwrap()
            .unwrap_err()
            .contains("being saved")
    );
    result.unwrap();
    assert_eq!(skinstrike::document::load_document(&target).unwrap(), doc);
    // Dropping the guard must release the OS lock and permit the next save.
    save_document(&competing, &target, true).unwrap();
}

#[test]
fn cancellation_after_final_verification_keeps_the_previous_project() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().join("base.png");
    let target = dir.path().join("finish.json");
    let mut doc = document(&base, &target);
    save_document(&doc, &target, false).unwrap();
    let previous = std::fs::read(&target).unwrap();
    doc.add_color_layer("Local edit", [80, 90, 100], 128)
        .unwrap();
    let calls = Cell::new(0);

    let result = save_document_with_cancel(&doc, &target, true, &|| {
        calls.set(calls.get() + 1);
        calls.get() == 4
    });

    assert!(result.unwrap_err().contains("cancelled"));
    assert_eq!(calls.get(), 4);
    assert_eq!(std::fs::read(&target).unwrap(), previous);
    save_document(&doc, &target, true).unwrap();
}

#[test]
fn stable_sidecar_is_not_truncated_and_independent_handles_exclude_saves() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().join("base.png");
    let target = dir.path().join("finish.json");
    let lock_path = dir.path().join(".finish.json.skinstrike-save.lock");
    let doc = document(&base, &target);
    let sentinel = b"persistent lock identity; do not truncate or remove";
    std::fs::write(&lock_path, sentinel).unwrap();
    save_document(&doc, &target, false).unwrap();
    assert_eq!(std::fs::read(&lock_path).unwrap(), sentinel);
    let previous = std::fs::read(&target).unwrap();

    let guard = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&lock_path)
        .unwrap();
    guard.try_lock().unwrap();
    assert!(
        save_document(&doc, &target, true)
            .unwrap_err()
            .contains("being saved")
    );
    assert_eq!(std::fs::read(&target).unwrap(), previous);
    drop(guard);

    save_document(&doc, &target, true).unwrap();
    assert_eq!(std::fs::read(&lock_path).unwrap(), sentinel);
}

#[test]
fn characterizes_external_writes_after_last_check_as_outside_the_lock_protocol() {
    // This is a limitation characterization, not a promise of cross-process CAS.
    // A raw writer that ignores the sidecar can still change the destination
    // after the last fingerprint check. Atomic rename has no expected hash.
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().join("base.png");
    let target = dir.path().join("finish.json");
    let mut doc = document(&base, &target);
    save_document(&doc, &target, false).unwrap();
    doc.add_color_layer("Local edit", [80, 90, 100], 128)
        .unwrap();
    let calls = Cell::new(0);

    let result = save_document_with_cancel(&doc, &target, true, &|| {
        calls.set(calls.get() + 1);
        if calls.get() == 4 {
            std::fs::write(&target, b"nonparticipating external writer").unwrap();
        }
        false
    });

    result.unwrap();
    assert_eq!(calls.get(), 4);
    assert_eq!(skinstrike::document::load_document(&target).unwrap(), doc);
}

#[test]
fn reserved_lock_filename_cannot_be_replaced_by_a_project_save() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().join("base.png");
    let target = dir.path().join(".finish.json.SKINSTRIKE-SAVE.LOCK");
    let doc = document(&base, &target);
    let previous = b"existing save lock";
    std::fs::write(&target, previous).unwrap();

    assert!(
        save_document(&doc, &target, true)
            .unwrap_err()
            .contains("reserved")
    );
    assert_eq!(std::fs::read(&target).unwrap(), previous);
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 2);
}
