use skinstrike::ui::SkinStrikeApp;

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "SkinStrike - texture inspection",
        eframe::NativeOptions::default(),
        Box::new(|_| Ok(Box::new(SkinStrikeApp::default()))),
    )
}
