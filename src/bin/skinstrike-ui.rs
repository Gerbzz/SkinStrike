use skinstrike::ui::SkinStrikeApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([960.0, 620.0]),
        ..Default::default()
    };
    eframe::run_native(
        "SkinStrike - finish canvas",
        options,
        Box::new(|_| Ok(Box::new(SkinStrikeApp::default()))),
    )
}
