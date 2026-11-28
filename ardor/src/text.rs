//! Caption text rendering: a contained FFI port of Pillow
//! 13.0.0's text path (`libImagingFt` / `_imagingft.c`).
//!
//! The sheet gate (RECORD-T3) holds caption bands to pixel
//! identity with the lanes' Pillow-built sheets, and Pillow's
//! text layout has behaviours no off-the-shelf Rust text stack
//! reproduces: shaping by libraqm (HarfBuzz) — the layout engine
//! Pillow selects whenever libraqm is present, as it is on the
//! estate's machines — 26.6 fixed-point positions rounded with
//! `PIXEL(x) = ((x + 32) & -64) >> 6`, per-glyph offsets from the
//! shaper, mask pixels merged with MULDIV255. So this module
//! drives the system FreeType and libraqm exactly the way Pillow
//! does and ports the arithmetic. libraqm is dlopen'd (never
//! link-time) so the rest of the server runs on hosts without
//! it; text rendering itself refuses explicitly when libraqm is
//! absent rather than silently diverging from Pillow's shaping.
//! All `unsafe` in the crate lives here; every other module is
//! safe Rust. GRAY bitmaps only: any other pixel mode is an
//! explicit error (the estate caption font, DejaVu Sans-Bold,
//! renders GRAY).

use std::ffi::CString;
use std::marker::PhantomData;
use std::os::raw::c_long;

use freetype_sys as ft;

use crate::raster::RgbImage;

/// Pillow's PIXEL macro: round a 26.6 fixed-point value to whole
/// pixels.
fn pixel(x: i64) -> i64 {
    ((x + 32) & -64) >> 6
}

fn muldiv255(a: u32, b: u32) -> u32 {
    let tmp = a * b + 128;
    ((tmp >> 8) + tmp) >> 8
}

fn clip8(v: u32) -> u8 {
    if v == 0 {
        0
    } else if v < 256 {
        v as u8
    } else {
        255
    }
}

fn div255(a: u32) -> u32 {
    let tmp = a + 128;
    ((tmp >> 8) + tmp) >> 8
}

/// Blend one channel the way ImagingFill2 + fill_mask_L do for an
/// RGB target: out = DIV255(out * (255 - m) + ink * m).
fn blend(m: u8, out: u8, ink: u8) -> u8 {
    let m = m as u32;
    div255(out as u32 * (255 - m) + ink as u32 * m) as u8
}

/// A rendered single-line alpha mask plus the anchor offsets
/// Pillow's `font.getmask2` returns for anchor "la".
pub struct TextMask {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub x_offset: i64,
    pub y_offset: i64,
}

/// A FreeType library handle. Owns the FT_Library; faces borrow it.
pub struct TextEngine {
    lib: ft::FT_Library,
}

impl TextEngine {
    pub fn new() -> Result<TextEngine, String> {
        let mut lib: ft::FT_Library = std::ptr::null_mut();
        let err = unsafe { ft::FT_Init_FreeType(&mut lib) };
        if err != 0 || lib.is_null() {
            return Err(format!("FreeType init failed (error {err})"));
        }
        Ok(TextEngine { lib })
    }

    /// Load a face at a nominal pixel size, the way Pillow's
    /// `FT_Request_Size(FT_SIZE_REQUEST_TYPE_NOMINAL)` does.
    pub fn face(&self, path: &str, size_px: i64) -> Result<Face<'_>, String> {
        let cpath = CString::new(path).map_err(|_| "font path has NUL".to_string())?;
        let mut face: ft::FT_Face = std::ptr::null_mut();
        let err = unsafe { ft::FT_New_Face(self.lib, cpath.as_ptr(), 0, &mut face) };
        if err != 0 || face.is_null() {
            return Err(format!("cannot load font {path} (error {err})"));
        }
        let mut request = ft::FT_Size_RequestRec {
            size_request_type: ft::FT_SIZE_REQUEST_TYPE_NOMINAL,
            width: (size_px << 6) as c_long,
            height: (size_px << 6) as c_long,
            horiResolution: 0,
            vertResolution: 0,
        };
        let err = unsafe { ft::FT_Request_Size(face, &mut request) };
        if err != 0 {
            unsafe { ft::FT_Done_Face(face) };
            return Err(format!(
                "cannot set font size {size_px} on {path} (error {err})"
            ));
        }
        Ok(Face {
            face,
            _lib: PhantomData,
        })
    }
}

impl Drop for TextEngine {
    fn drop(&mut self) {
        unsafe { ft::FT_Done_FreeType(self.lib) };
    }
}

struct GlyphInfo {
    index: u32,
    x_advance: i64, // 26.6
    y_advance: i64, // 26.6
    x_offset: i64,  // 26.6
    y_offset: i64,  // 26.6
}

// ------------------------------------------------------------------
// libraqm via dlopen: the shaper Pillow uses when it is present.
// ------------------------------------------------------------------

#[repr(C)]
struct RaqmGlyph {
    index: std::os::raw::c_uint,
    x_advance: std::os::raw::c_int,
    y_advance: std::os::raw::c_int,
    x_offset: std::os::raw::c_int,
    y_offset: std::os::raw::c_int,
    cluster: u32,
    ftface: ft::FT_Face,
}

type RaqmCreate = unsafe extern "C" fn() -> *mut std::ffi::c_void;
type RaqmDestroy = unsafe extern "C" fn(*mut std::ffi::c_void);
type RaqmSetText = unsafe extern "C" fn(*mut std::ffi::c_void, *const u32, usize) -> bool;
type RaqmSetParDirection = unsafe extern "C" fn(*mut std::ffi::c_void, std::os::raw::c_int) -> bool;
type RaqmSetFreetypeFace = unsafe extern "C" fn(*mut std::ffi::c_void, ft::FT_Face) -> bool;
type RaqmLayoutFn = unsafe extern "C" fn(*mut std::ffi::c_void) -> bool;
type RaqmGetGlyphs = unsafe extern "C" fn(*mut std::ffi::c_void, *mut usize) -> *const RaqmGlyph;

struct Raqm {
    create: RaqmCreate,
    destroy: RaqmDestroy,
    set_text: RaqmSetText,
    set_par_direction: RaqmSetParDirection,
    set_freetype_face: RaqmSetFreetypeFace,
    layout: RaqmLayoutFn,
    get_glyphs: RaqmGetGlyphs,
}

unsafe extern "C" {
    fn dlopen(
        filename: *const std::os::raw::c_char,
        flag: std::os::raw::c_int,
    ) -> *mut std::ffi::c_void;
    fn dlsym(
        handle: *mut std::ffi::c_void,
        symbol: *const std::os::raw::c_char,
    ) -> *mut std::ffi::c_void;
}

fn load_raqm() -> Option<Raqm> {
    const RTLD_NOW: std::os::raw::c_int = 2;
    let mut handle: *mut std::ffi::c_void = std::ptr::null_mut();
    for name in ["libraqm.so.0", "libraqm.so"] {
        let cname = CString::new(name).ok()?;
        handle = unsafe { dlopen(cname.as_ptr(), RTLD_NOW) };
        if !handle.is_null() {
            break;
        }
    }
    if handle.is_null() {
        return None;
    }
    let sym = |name: &str| -> Option<*mut std::ffi::c_void> {
        let cname = CString::new(name).ok()?;
        let ptr = unsafe { dlsym(handle, cname.as_ptr()) };
        if ptr.is_null() { None } else { Some(ptr) }
    };
    // The handle is intentionally never dlclose'd: the function
    // pointers live for the process lifetime.
    Some(Raqm {
        create: unsafe {
            std::mem::transmute::<*mut std::ffi::c_void, RaqmCreate>(sym("raqm_create")?)
        },
        destroy: unsafe {
            std::mem::transmute::<*mut std::ffi::c_void, RaqmDestroy>(sym("raqm_destroy")?)
        },
        set_text: unsafe {
            std::mem::transmute::<*mut std::ffi::c_void, RaqmSetText>(sym("raqm_set_text")?)
        },
        set_par_direction: unsafe {
            std::mem::transmute::<*mut std::ffi::c_void, RaqmSetParDirection>(sym(
                "raqm_set_par_direction",
            )?)
        },
        set_freetype_face: unsafe {
            std::mem::transmute::<*mut std::ffi::c_void, RaqmSetFreetypeFace>(sym(
                "raqm_set_freetype_face",
            )?)
        },
        layout: unsafe {
            std::mem::transmute::<*mut std::ffi::c_void, RaqmLayoutFn>(sym("raqm_layout")?)
        },
        get_glyphs: unsafe {
            std::mem::transmute::<*mut std::ffi::c_void, RaqmGetGlyphs>(sym("raqm_get_glyphs")?)
        },
    })
}

fn raqm() -> Option<&'static Raqm> {
    static RAQM: std::sync::OnceLock<Option<Raqm>> = std::sync::OnceLock::new();
    RAQM.get_or_init(load_raqm).as_ref()
}

/// A sized face. Drop runs FT_Done_Face.
pub struct Face<'a> {
    face: ft::FT_Face,
    _lib: PhantomData<&'a TextEngine>,
}

impl Drop for Face<'_> {
    fn drop(&mut self) {
        unsafe { ft::FT_Done_Face(self.face) };
    }
}

impl Face<'_> {
    fn load(&self, index: u32, render: bool) -> Result<(), String> {
        let flags = if render {
            ft::FT_LOAD_DEFAULT | ft::FT_LOAD_RENDER
        } else {
            ft::FT_LOAD_DEFAULT
        };
        let err = unsafe { ft::FT_Load_Glyph(self.face, index, flags) };
        if err != 0 {
            return Err(format!("glyph load failed (error {err})"));
        }
        Ok(())
    }

    /// Pillow's `text_layout_raqm`: shape with libraqm over the
    /// sized face (UTF-32 text, default paragraph direction, no
    /// language or features — the ImageDraw/getmask2 defaults).
    fn layout(&self, text: &str) -> Result<Vec<GlyphInfo>, String> {
        if text.is_empty() {
            return Ok(Vec::new());
        }
        let rq = raqm().ok_or(
            "libraqm is not available on this host; caption rendering requires it \
             (Pillow shapes with libraqm whenever it is present, and a fallback \
             would silently diverge)",
        )?;
        let handle = unsafe { (rq.create)() };
        if handle.is_null() {
            return Err("raqm_create failed".to_string());
        }
        let codepoints: Vec<u32> = text.chars().map(|c| c as u32).collect();
        let result = (|| {
            if !unsafe { (rq.set_text)(handle, codepoints.as_ptr(), codepoints.len()) } {
                return Err("raqm_set_text failed".to_string());
            }
            // RAQM_DIRECTION_DEFAULT = 0
            if !unsafe { (rq.set_par_direction)(handle, 0) } {
                return Err("raqm_set_par_direction failed".to_string());
            }
            if !unsafe { (rq.set_freetype_face)(handle, self.face) } {
                return Err("raqm_set_freetype_face failed".to_string());
            }
            if !unsafe { (rq.layout)(handle) } {
                return Err("raqm_layout failed".to_string());
            }
            let mut count: usize = 0;
            let glyphs = unsafe { (rq.get_glyphs)(handle, &mut count) };
            if glyphs.is_null() {
                return Err("raqm_get_glyphs failed".to_string());
            }
            let slice = unsafe { std::slice::from_raw_parts(glyphs, count) };
            Ok(slice
                .iter()
                .map(|g| GlyphInfo {
                    index: g.index,
                    x_advance: g.x_advance as i64,
                    y_advance: g.y_advance as i64,
                    x_offset: g.x_offset as i64,
                    y_offset: g.y_offset as i64,
                })
                .collect())
        })();
        unsafe { (rq.destroy)(handle) };
        result
    }

    /// Pillow's `font_getsize` bounding box + anchor resolution
    /// for anchor "la" (left-ascender), single-line text.
    /// Returns (width, height, x_offset, y_offset).
    fn bbox(&self, glyphs: &[GlyphInfo]) -> Result<(i64, i64, i64, i64), String> {
        let face = self.face;
        let (mut x_min, mut x_max, mut y_min, mut y_max) = (0i64, 0i64, 0i64, 0i64);
        let mut position: i64 = 0;
        for g in glyphs {
            let px = pixel(position + g.x_offset);
            let py = pixel(g.y_offset);
            position += g.x_advance;
            let advanced = pixel(position);
            if advanced > x_max {
                x_max = advanced;
            }
            self.load(g.index, false)?;
            let mut glyph: ft::FT_Glyph = std::ptr::null_mut();
            let err = unsafe { ft::FT_Get_Glyph((*face).glyph, &mut glyph) };
            if err != 0 {
                return Err(format!("FT_Get_Glyph failed (error {err})"));
            }
            let mut cbox = ft::FT_BBox {
                xMin: 0,
                yMin: 0,
                xMax: 0,
                yMax: 0,
            };
            unsafe {
                ft::FT_Glyph_Get_CBox(glyph, ft::FT_GLYPH_BBOX_PIXELS, &mut cbox);
                ft::FT_Done_Glyph(glyph);
            }
            if cbox.xMax + px > x_max {
                x_max = cbox.xMax + px;
            }
            if cbox.xMin + px < x_min {
                x_min = cbox.xMin + px;
            }
            if cbox.yMax + py > y_max {
                y_max = cbox.yMax + py;
            }
            if cbox.yMin + py < y_min {
                y_min = cbox.yMin + py;
            }
        }
        let ascender = unsafe { (*(*face).size).metrics.ascender };
        let y_anchor = pixel(ascender);
        Ok((x_max - x_min, y_max - y_min, x_min, y_anchor - y_max))
    }

    /// Render the text to an alpha mask, porting Pillow's
    /// `font_render`: pass 1 finds the mask origin from rendered
    /// bitmap bearings, pass 2 merges glyph bitmaps.
    pub fn render_mask(&self, text: &str) -> Result<TextMask, String> {
        let glyphs = self.layout(text)?;
        let (width, height, x_offset, y_offset) = self.bbox(&glyphs)?;
        let mut mask = TextMask {
            width: width.max(0) as u32,
            height: height.max(0) as u32,
            data: Vec::new(),
            x_offset,
            y_offset,
        };
        if glyphs.is_empty() || width <= 0 || height <= 0 {
            return Ok(mask);
        }
        let face = self.face;
        // Pass 1: mask origin from rendered bearings.
        let (mut x, mut y, mut x_min, mut y_max) = (0i64, 0i64, 0i64, 0i64);
        for g in &glyphs {
            let px = pixel(x + g.x_offset);
            let py = pixel(y + g.y_offset);
            self.load(g.index, true)?;
            let (left, top) = unsafe {
                (
                    (*(*face).glyph).bitmap_left as i64,
                    (*(*face).glyph).bitmap_top as i64,
                )
            };
            if top + py > y_max {
                y_max = top + py;
            }
            if left + px < x_min {
                x_min = left + px;
            }
            x += g.x_advance;
            y += g.y_advance;
        }
        // Pass 2: merge bitmaps.
        let w = mask.width as usize;
        let h = mask.height as usize;
        let mut data = vec![0u8; w * h];
        let mut pen_y = -y_max * 64;
        let mut pen = -x_min * 64;
        for g in &glyphs {
            let px = pixel(pen + g.x_offset);
            let py = pixel(pen_y + g.y_offset);
            self.load(g.index, true)?;
            let (buffer, pixel_mode, bw, bh, pitch, left, top) = unsafe {
                let slot = (*face).glyph;
                (
                    (*slot).bitmap.buffer,
                    (*slot).bitmap.pixel_mode,
                    (*slot).bitmap.width as i64,
                    (*slot).bitmap.rows as i64,
                    (*slot).bitmap.pitch as i64,
                    (*slot).bitmap_left as i64,
                    (*slot).bitmap_top as i64,
                )
            };
            if !buffer.is_null() {
                if pixel_mode != ft::FT_PIXEL_MODE_GRAY as i8 {
                    return Err(format!(
                        "unsupported glyph pixel mode {pixel_mode} (GRAY only)"
                    ));
                }
                let xx = px + left;
                let yy = -(py + top);
                let x0 = if xx < 0 { -xx } else { 0 };
                let mut x1 = bw;
                if xx + x1 > w as i64 {
                    x1 = w as i64 - xx;
                }
                for row in 0..bh {
                    let ty = yy + row;
                    if ty < 0 || ty >= h as i64 {
                        continue;
                    }
                    for k in x0..x1 {
                        let src_alpha = unsafe { *buffer.offset((row * pitch + k) as isize) };
                        if src_alpha == 0 {
                            continue;
                        }
                        let idx = ty as usize * w + (xx + k) as usize;
                        let target = data[idx];
                        data[idx] = if target > 0 {
                            clip8(
                                src_alpha as u32 + muldiv255(target as u32, 255 - src_alpha as u32),
                            )
                        } else {
                            src_alpha
                        };
                    }
                }
            }
            pen += g.x_advance;
            pen_y += g.y_advance;
        }
        mask.data = data;
        Ok(mask)
    }
}

/// Composite a mask onto an RGB image at (dx, dy) with the given
/// ink colour — Pillow's `ImagingDrawWideText` fill for a text
/// mask on an RGB image.
pub fn composite_mask(img: &mut RgbImage, mask: &TextMask, dx: i64, dy: i64, ink: [u8; 3]) {
    if mask.data.is_empty() {
        return;
    }
    for my in 0..mask.height as i64 {
        let ty = dy + my;
        if ty < 0 || ty >= img.height as i64 {
            continue;
        }
        for mx in 0..mask.width as i64 {
            let m = mask.data[my as usize * mask.width as usize + mx as usize];
            if m == 0 {
                continue; // BLEND with m=0 is the identity
            }
            let tx = dx + mx;
            if tx < 0 || tx >= img.width as i64 {
                continue;
            }
            let i = (ty as usize * img.width as usize + tx as usize) * 3;
            for c in 0..3 {
                img.px[i + c] = blend(m, img.px[i + c], ink[c]);
            }
        }
    }
}

/// Draw one caption line onto an image at Pillow's ImageDraw
/// anchor "la" position (x, y): the mask lands at
/// (x + x_offset, y + y_offset).
pub fn draw_caption(
    face: &Face<'_>,
    img: &mut RgbImage,
    x: i64,
    y: i64,
    text: &str,
    ink: [u8; 3],
) -> Result<(), String> {
    let mask = face.render_mask(text)?;
    composite_mask(img, &mask, x + mask.x_offset, y + mask.y_offset, ink);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    const FONT: &str = "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf";

    #[test]
    fn mask_44_matches_pillow() {
        // Pillow 13.0.0 getmask2 on primo: size (1000, 33),
        // offset (0, 8), sha256 of the mask bytes below.
        let engine = TextEngine::new().expect("freetype init");
        let face = engine.face(FONT, 44).expect("face");
        let mask = face
            .render_mask("SOLENNE c1 - seed 274101 - CANDIDATE")
            .expect("render");
        assert_eq!((mask.width, mask.height), (1000, 33));
        assert_eq!((mask.x_offset, mask.y_offset), (0, 8));
        let digest = Sha256::digest(&mask.data);
        assert_eq!(
            format!("{digest:x}"),
            "285543c27e9cd6acaaee934961c0277e6ec921d58b2eff68fbe50aaae92192b2"
        );
    }

    #[test]
    fn mask_30_matches_pillow() {
        let engine = TextEngine::new().expect("freetype init");
        let face = engine.face(FONT, 30).expect("face");
        let mask = face
            .render_mask("TAMSIN - SET-1 c1 - DESIGN SOURCE")
            .expect("render");
        assert_eq!((mask.width, mask.height), (612, 22));
        assert_eq!((mask.x_offset, mask.y_offset), (0, 6));
        let digest = Sha256::digest(&mask.data);
        assert_eq!(
            format!("{digest:x}"),
            "b60daa81777d1ecc9ea8b41aa39d9264f5b2cd8d4677aac9c68d83264767f698"
        );
    }

    #[test]
    fn empty_text_is_empty_mask() {
        let engine = TextEngine::new().expect("freetype init");
        let face = engine.face(FONT, 30).expect("face");
        let mask = face.render_mask("").expect("render");
        assert_eq!((mask.width, mask.height), (0, 0));
        assert!(mask.data.is_empty());
    }

    #[test]
    fn missing_font_file_errors() {
        let engine = TextEngine::new().expect("freetype init");
        assert!(engine.face("/nonexistent/NoSuchFont.ttf", 30).is_err());
    }

    #[test]
    fn composite_blend_endpoints() {
        // m=0 leaves the pixel, m=255 paints the ink exactly.
        assert_eq!(blend(0, 18, 240), 18);
        assert_eq!(blend(255, 18, 240), 240);
        assert_eq!(blend(0, 255, 0), 255);
        assert_eq!(blend(255, 0, 137), 137);
    }
}
