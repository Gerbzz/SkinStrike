use image::ImageReader;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

#[cfg(feature = "native-ui")]
pub mod ui;

pub mod document;
pub mod project_library;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TextureReport {
    pub schema_version: u32,
    pub source_sha256: String,
    pub width: u32,
    pub height: u32,
    pub pixels: u64,
    pub fully_transparent_pixels: u64,
    pub partially_transparent_pixels: u64,
    pub fully_opaque_pixels: u64,
}

pub fn inspect_png(path: &Path) -> Result<TextureReport, String> {
    if !path.is_file()
        || !path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("png"))
    {
        return Err("select an existing PNG texture".into());
    }
    let mut file = File::open(path).map_err(|error| error.to_string())?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    let image = ImageReader::open(path)
        .map_err(|error| error.to_string())?
        .decode()
        .map_err(|error| format!("PNG decode failed: {error}"))?
        .to_rgba8();
    let (width, height) = image.dimensions();
    if width == 0 || height == 0 {
        return Err("texture dimensions must be positive".into());
    }
    let mut zero = 0;
    let mut partial = 0;
    let mut opaque = 0;
    for pixel in image.pixels() {
        match pixel.0[3] {
            0 => zero += 1,
            255 => opaque += 1,
            _ => partial += 1,
        }
    }
    Ok(TextureReport {
        schema_version: 1,
        source_sha256: format!("{:x}", hash.finalize()),
        width,
        height,
        pixels: u64::from(width) * u64::from(height),
        fully_transparent_pixels: zero,
        partially_transparent_pixels: partial,
        fully_opaque_pixels: opaque,
    })
}

pub fn write_new_report(report: &TextureReport, output: &Path) -> Result<(), String> {
    let parent = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut pending = tempfile::NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    let bytes = serde_json::to_vec_pretty(report).map_err(|error| error.to_string())?;
    pending
        .write_all(&bytes)
        .and_then(|_| pending.as_file().sync_all())
        .map_err(|error| error.to_string())?;
    pending
        .persist_noclobber(output)
        .map_err(|error| error.error.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{TextureReport, inspect_png, write_new_report};
    use image::{Rgba, RgbaImage};

    #[test]
    fn decodes_texture_and_records_alpha_coverage() {
        let dir = tempfile::TempDir::new().unwrap();
        let input = dir.path().join("finish.png");
        let mut image = RgbaImage::from_pixel(2, 2, Rgba([50, 80, 120, 255]));
        image.put_pixel(0, 0, Rgba([0, 0, 0, 0]));
        image.put_pixel(1, 0, Rgba([255, 0, 0, 128]));
        image.save(&input).unwrap();
        let report = inspect_png(&input).unwrap();
        assert_eq!((report.width, report.height, report.pixels), (2, 2, 4));
        assert_eq!(
            (
                report.fully_transparent_pixels,
                report.partially_transparent_pixels,
                report.fully_opaque_pixels
            ),
            (1, 1, 2)
        );
        let output = dir.path().join("finish-report.json");
        write_new_report(&report, &output).unwrap();
        let reopened: TextureReport =
            serde_json::from_slice(&std::fs::read(&output).unwrap()).unwrap();
        assert_eq!(reopened, report);
        assert!(write_new_report(&report, &output).is_err());
    }

    #[test]
    fn rejects_corrupt_png() {
        let dir = tempfile::TempDir::new().unwrap();
        let input = dir.path().join("broken.png");
        std::fs::write(&input, b"not a png").unwrap();
        assert!(inspect_png(&input).is_err());
    }
}
