"""Approved candidate-only branding; requires Pillow. Does not change donor artwork."""
from pathlib import Path
from hashlib import sha256
from io import BytesIO
import struct
import unittest
from PIL import Image
ROOT = Path(__file__).resolve().parents[1]
APPROVED = 'e1523b9145d862eeb35dd566afe6d697984cf404f30cfd5c8f7d3b1ecefddff4'
class Branding(unittest.TestCase):
    def test_approved_source(self):
        path = ROOT/'assets/branding/SkinStrike.png'
        self.assertTrue(path.exists(), 'approved source missing')
        self.assertEqual(sha256(path.read_bytes()).hexdigest(), APPROVED)
        self.assertEqual(Image.open(BytesIO(path.read_bytes())).size, (512,512))
    def test_candidate_runtime_and_donor_fallback(self):
        code = (ROOT/'crates/ui/src/theme.rs').read_text(encoding="utf-8")
        self.assertIn('#[cfg(feature = "skinstrike-candidate")]\n    let png = include_bytes!("../../../assets/branding/SkinStrike.png")', code)
        self.assertIn('#[cfg(not(feature = "skinstrike-candidate"))]\n    let png = include_bytes!("../assets/icon.png")', code)
        self.assertIn('image::load_from_memory_with_format(png,', code)
        self.assertIn('.with_icon(std::sync::Arc::new(theme::app_icon()))', (ROOT/'crates/ui/src/lib.rs').read_text(encoding="utf-8"))
    def test_candidate_header_and_title(self):
        theme=(ROOT/'crates/ui/src/theme.rs').read_text(encoding="utf-8")
        app=(ROOT/'crates/ui/src/app.rs').read_text(encoding="utf-8")
        dialogs=(ROOT/'crates/ui/src/dialogs.rs').read_text(encoding="utf-8")
        self.assertIn('pub const APP_NAME: &str = "SkinStrike";',theme)
        self.assertIn('pub const APP_NAME: &str = "SubCraft";',theme)
        self.assertIn('theme::brand_mark(ui, r);',app)
        self.assertIn('RichText::new(theme::APP_NAME)',app)
        self.assertIn('format!("{} - {name}{star}", crate::theme::APP_NAME)',dialogs)
        self.assertIn('ColorImage::from_rgba_unmultiplied',theme)
        self.assertIn('image::imageops::FilterType::Lanczos3',theme)
        self.assertIn('rect.width() * ui.ctx().pixels_per_point()',theme)
        self.assertIn('egui::Id::new(("skinstrike-approved-header-logo", pixels))',theme)
    def test_candidate_windows_resource_and_derivatives(self):
        code = (ROOT/'apps/subcraft/build.rs').read_text(encoding="utf-8")
        self.assertIn('CARGO_FEATURE_SKINSTRIKE_CANDIDATE', code)
        self.assertIn('../../assets/branding/SkinStrike.ico', code)
        self.assertIn('"assets/icon.ico"',code)
        path=ROOT/'assets/branding/SkinStrike.ico'
        self.assertTrue(path.exists(), 'candidate ICO missing')
        data=path.read_bytes(); sizes=(16,24,32,48,64,128,256)
        self.assertEqual(struct.unpack_from('<HHH',data),(0,1,len(sizes)))
        source=Image.open(ROOT/'assets/branding/SkinStrike.png').convert('RGBA')
        for i,size in enumerate(sizes):
            w,h,_,_,_,_,length,offset=struct.unpack_from('<BBBBHHII',data,6+16*i)
            self.assertEqual((w or 256,h or 256),(size,size))
            actual=Image.open(BytesIO(data[offset:offset+length])).convert('RGBA')
            self.assertEqual(actual.tobytes(),source.resize((size,size),Image.Resampling.LANCZOS).tobytes())
if __name__=='__main__': unittest.main()
