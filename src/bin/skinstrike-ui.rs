use eframe::egui;
use skinstrike::ui::SkinStrikeApp;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct CaptureApp {
    inner: SkinStrikeApp,
    path: PathBuf,
    outcome: Arc<Mutex<Option<Result<(), String>>>>,
    started: Instant,
    requested: bool,
    finished: bool,
}

impl CaptureApp {
    fn write_png(path: &std::path::Path, screenshot: &egui::ColorImage) -> Result<(), String> {
        let [width, height] = screenshot.size;
        let bytes = width
            .checked_mul(height)
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or("Screenshot dimensions overflow")?;
        let mut pixels = Vec::with_capacity(bytes);
        for color in &screenshot.pixels {
            pixels.extend_from_slice(&color.to_array());
        }
        if pixels.len() != bytes {
            return Err("Screenshot pixel count does not match dimensions".into());
        }
        let width = u32::try_from(width).map_err(|_| "Screenshot width exceeds PNG limits")?;
        let height = u32::try_from(height).map_err(|_| "Screenshot height exceeds PNG limits")?;
        let image = image::RgbaImage::from_raw(width, height, pixels)
            .ok_or("Could not construct screenshot image")?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|error| format!("Create screenshot {}: {error}", path.display()))?;
        image::DynamicImage::ImageRgba8(image)
            .write_to(&mut file, image::ImageFormat::Png)
            .map_err(|error| format!("Encode screenshot: {error}"))?;
        file.sync_all()
            .map_err(|error| format!("Sync screenshot: {error}"))?;
        Ok(())
    }

    fn finish(&mut self, ctx: &egui::Context, result: Result<(), String>) {
        self.finished = true;
        *self.outcome.lock().unwrap() = Some(result);
        self.inner.allow_capture_close();
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

impl eframe::App for CaptureApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        eframe::App::update(&mut self.inner, ctx, frame);
        if self.finished {
            return;
        }
        if self.started.elapsed() > Duration::from_secs(15) {
            self.finish(ctx, Err("Timed out waiting for screenshot".into()));
            return;
        }
        if self.requested {
            let screenshot = ctx.input(|input| {
                input.events.iter().find_map(|event| match event {
                    egui::Event::Screenshot { image, .. } => Some(Arc::clone(image)),
                    _ => None,
                })
            });
            if let Some(screenshot) = screenshot {
                let result = Self::write_png(&self.path, &screenshot);
                self.finish(ctx, result);
            } else {
                ctx.request_repaint();
            }
            return;
        }
        if self.started.elapsed() >= Duration::from_secs(3) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
            self.requested = true;
            ctx.request_repaint();
        } else {
            ctx.request_repaint_after(Duration::from_millis(50));
        }
    }
}

fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    let capture = match (args.next(), args.next(), args.next(), args.next()) {
        (None, None, None, None) => None,
        (Some(flag), Some(output), Some(texture), None) if flag == "--capture" => {
            let output = PathBuf::from(output);
            if output.extension().is_none_or(|ext| ext != "png") {
                eprintln!("Capture path must end in .png");
                return ExitCode::FAILURE;
            }
            Some((output, PathBuf::from(texture)))
        }
        _ => {
            eprintln!("Usage: skinstrike-ui [--capture FILE.png TEXTURE.png]");
            return ExitCode::FAILURE;
        }
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([960.0, 620.0]),
        ..Default::default()
    };
    if let Some((path, texture)) = capture {
        let mut inner = SkinStrikeApp::default();
        if let Err(error) = inner.open_texture(&texture) {
            eprintln!("SkinStrike fixture: {error}");
            return ExitCode::FAILURE;
        }
        let outcome = Arc::new(Mutex::new(None));
        let app = CaptureApp {
            inner,
            path,
            outcome: Arc::clone(&outcome),
            started: Instant::now(),
            requested: false,
            finished: false,
        };
        match eframe::run_native(
            "SkinStrike - finish canvas",
            options,
            Box::new(move |_| Ok(Box::new(app))),
        ) {
            Ok(()) => match outcome.lock().unwrap().take() {
                Some(Ok(())) => ExitCode::SUCCESS,
                Some(Err(error)) => {
                    eprintln!("SkinStrike capture: {error}");
                    ExitCode::FAILURE
                }
                None => {
                    eprintln!("SkinStrike capture: window closed before screenshot completed");
                    ExitCode::FAILURE
                }
            },
            Err(error) => {
                eprintln!("SkinStrike window: {error}");
                ExitCode::FAILURE
            }
        }
    } else {
        match eframe::run_native(
            "SkinStrike - finish canvas",
            options,
            Box::new(|_| Ok(Box::new(SkinStrikeApp::default()))),
        ) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("SkinStrike window: {error}");
                ExitCode::FAILURE
            }
        }
    }
}
