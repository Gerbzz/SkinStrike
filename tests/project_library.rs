use image::{Rgba, RgbaImage};
use skinstrike::document::{PaintStroke, load_document, new_document};
use skinstrike::project_library::ProjectLibrary;

#[test]
fn collection_project_and_painted_cover_persist_across_library_reopen() {
    let temp = tempfile::TempDir::new().unwrap();
    let base = temp.path().join("base.png");
    RgbaImage::from_pixel(16, 16, Rgba([20, 30, 40, 255]))
        .save(&base)
        .unwrap();
    let library_root = temp.path().join("library");
    let library = ProjectLibrary::create(&library_root).unwrap();
    library.create_collection("Favorites").unwrap();
    let mut document = new_document(&base, &temp.path().join("source.json")).unwrap();
    let paint = document.add_paint_layer("Brushwork").unwrap();
    document
        .add_paint_stroke(
            paint,
            PaintStroke {
                rgb: [234, 67, 19],
                radius: 2.0,
                points: vec![[8.5, 8.5]],
            },
        )
        .unwrap();

    let saved = library
        .save_project("Favorites", "Copper Study", &document)
        .unwrap();
    let reopened = ProjectLibrary::open(&library_root).unwrap();
    assert!(
        reopened
            .collections()
            .unwrap()
            .contains(&"Favorites".to_string())
    );
    let page = reopened.page(Some("Favorites"), "copper", 0, 12).unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.cards[0].title, "Copper Study");
    assert_eq!(
        page.cards[0].thumbnail.as_ref().unwrap().get_pixel(8, 8).0,
        [234, 67, 19, 255]
    );
    assert_eq!(
        load_document(&saved).unwrap().project_id,
        document.project_id
    );

    // Repeating Save updates the same project, keeping one collection card.
    document
        .add_paint_stroke(
            paint,
            PaintStroke {
                rgb: [19, 120, 234],
                radius: 2.0,
                points: vec![[3.5, 3.5]],
            },
        )
        .unwrap();
    assert_eq!(
        library
            .save_project("Favorites", "Copper Study", &document)
            .unwrap(),
        saved
    );
    let updated = reopened.page(Some("Favorites"), "copper", 0, 12).unwrap();
    assert_eq!(updated.total, 1);
    assert_eq!(
        updated.cards[0]
            .thumbnail
            .as_ref()
            .unwrap()
            .get_pixel(3, 3)
            .0,
        [19, 120, 234, 255]
    );
}

#[test]
fn invalid_folders_and_corrupt_projects_have_explicit_errors() {
    let temp = tempfile::TempDir::new().unwrap();
    let root = temp.path().join("library");
    assert!(ProjectLibrary::open(&root).is_err());
    let library = ProjectLibrary::create(&root).unwrap();
    assert!(library.create_collection("../outside").is_err());
    assert!(library.create_collection("CON").is_err());
    assert!(!temp.path().join("outside").exists());
    library.create_collection("Workshop").unwrap();
    std::fs::write(
        root.join("Workshop").join("Broken.skinstrike.json"),
        b"{broken",
    )
    .unwrap();
    let page = library.page(Some("Workshop"), "", 0, 12).unwrap();
    assert_eq!(page.total, 1);
    assert!(page.cards[0].error.is_some());
    assert!(page.cards[0].thumbnail.is_none());
}
