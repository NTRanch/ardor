//! Packed-RGB raster with a pixel-exact port of Pillow 13.0.0's
//! Lanczos resampler (libImaging/Resample.c, 8bpc path).
//!
//! Why a port instead of a generic resize: the sheet gate
//! (RECORD-T3) holds rebuilt sheets to 0 differing pixels against
//! the lanes' Pillow-built masters, so the resampler must agree
//! with Pillow bit-for-bit — same double-precision coefficient
//! precompute, same 22-bit fixed-point normalisation, same
//! rounding-half seed and clip table semantics. The unit tests pin
//! the port against Pillow 13.0.0 outputs generated on primo.

/// An RGB image, 3 bytes per pixel, row-major, top row first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RgbImage {
    pub width: u32,
    pub height: u32,
    pub px: Vec<u8>,
}

impl RgbImage {
    pub fn new_filled(width: u32, height: u32, rgb: [u8; 3]) -> RgbImage {
        let mut px = Vec::with_capacity(width as usize * height as usize * 3);
        for _ in 0..width as usize * height as usize {
            px.extend_from_slice(&rgb);
        }
        RgbImage { width, height, px }
    }

    /// Drop the alpha channel of an RGBA buffer (Pillow's
    /// `convert("RGB")` semantics for RGBA sources: no compositing).
    pub fn from_rgba(width: u32, height: u32, rgba: &[u8]) -> RgbImage {
        let mut px = vec![0u8; width as usize * height as usize * 3];
        for i in 0..width as usize * height as usize {
            px[3 * i..3 * i + 3].copy_from_slice(&rgba[4 * i..4 * i + 3]);
        }
        RgbImage { width, height, px }
    }

    pub fn pixel(&self, x: u32, y: u32) -> [u8; 3] {
        let i = (y as usize * self.width as usize + x as usize) * 3;
        [self.px[i], self.px[i + 1], self.px[i + 2]]
    }

    /// Paste `src` at (dx, dy), clipped to this image's bounds
    /// (Pillow `Image.paste` without a mask: an exact copy).
    pub fn paste(&mut self, src: &RgbImage, dx: i64, dy: i64) {
        for sy in 0..src.height as i64 {
            let ty = dy + sy;
            if ty < 0 || ty >= self.height as i64 {
                continue;
            }
            for sx in 0..src.width as i64 {
                let tx = dx + sx;
                if tx < 0 || tx >= self.width as i64 {
                    continue;
                }
                let si = (sy as usize * src.width as usize + sx as usize) * 3;
                let ti = (ty as usize * self.width as usize + tx as usize) * 3;
                self.px[ti..ti + 3].copy_from_slice(&src.px[si..si + 3]);
            }
        }
    }
}

fn sinc(x: f64) -> f64 {
    if x == 0.0 {
        return 1.0;
    }
    let x = x * std::f64::consts::PI;
    x.sin() / x
}

fn lanczos_filter(x: f64) -> f64 {
    if (-3.0..3.0).contains(&x) {
        return sinc(x) * sinc(x / 3.0);
    }
    0.0
}

const PRECISION_BITS: i32 = 22; // 32 - 8 - 2, as in Resample.c

/// Pillow's clip8: the accumulator, shifted back by
/// PRECISION_BITS, clamped to 0..=255.
fn clip8(acc: i32) -> u8 {
    (acc >> PRECISION_BITS).clamp(0, 255) as u8
}

struct Coeffs {
    /// Per output sample: (first input index, tap count).
    bounds: Vec<(i32, i32)>,
    /// Per output sample: ksize fixed-point coefficients.
    k: Vec<i32>,
    ksize: usize,
}

fn precompute_coeffs(in_size: i32, out_size: i32) -> Coeffs {
    // Pillow works over the full box (in0 = 0, in1 = in_size).
    let scale = in_size as f64 / out_size as f64;
    let mut filterscale = scale;
    if filterscale < 1.0 {
        filterscale = 1.0;
    }
    let support = 3.0 * filterscale;
    let ksize = (support.ceil() as usize) * 2 + 1;
    let inv_filterscale = 1.0 / filterscale;
    let mut bounds = Vec::with_capacity(out_size as usize);
    let mut kk = vec![0i32; out_size as usize * ksize];
    for xx in 0..out_size {
        let center = (xx as f64 + 0.5) * scale;
        let mut xmin = (center - support + 0.5) as i32;
        if xmin < 0 {
            xmin = 0;
        }
        let mut xmax = (center + support + 0.5) as i32;
        if xmax > in_size {
            xmax = in_size;
        }
        xmax -= xmin;
        let base = xx as usize * ksize;
        let mut ww = 0.0f64;
        let mut raw = vec![0.0f64; ksize];
        for x in 0..xmax {
            let w = lanczos_filter((x as f64 + xmin as f64 - center + 0.5) * inv_filterscale);
            raw[x as usize] = w;
            ww += w;
        }
        if ww != 0.0 {
            for x in 0..xmax {
                raw[x as usize] /= ww;
            }
        }
        for (i, v) in raw.iter().enumerate() {
            kk[base + i] = if *v < 0.0 {
                (-0.5 + v * (1 << PRECISION_BITS) as f64) as i32
            } else {
                (0.5 + v * (1 << PRECISION_BITS) as f64) as i32
            };
        }
        bounds.push((xmin, xmax));
    }
    Coeffs {
        bounds,
        k: kk,
        ksize,
    }
}

fn resample_horizontal(img: &RgbImage, out_width: u32) -> RgbImage {
    let coeffs = precompute_coeffs(img.width as i32, out_width as i32);
    let mut out = RgbImage {
        width: out_width,
        height: img.height,
        px: vec![0u8; out_width as usize * img.height as usize * 3],
    };
    let in_stride = img.width as usize * 3;
    let out_stride = out_width as usize * 3;
    for y in 0..img.height as usize {
        let line_in = &img.px[y * in_stride..(y + 1) * in_stride];
        for xx in 0..out_width as usize {
            let (xmin, xmax) = coeffs.bounds[xx];
            let mut ss = [1 << (PRECISION_BITS - 1); 3];
            for (x, &coef) in coeffs.k[xx * coeffs.ksize..]
                .iter()
                .take(xmax as usize)
                .enumerate()
            {
                let src = (x + xmin as usize) * 3;
                for (c, acc) in ss.iter_mut().enumerate() {
                    *acc += line_in[src + c] as i32 * coef;
                }
            }
            let dst = y * out_stride + xx * 3;
            for c in 0..3 {
                out.px[dst + c] = clip8(ss[c]);
            }
        }
    }
    out
}

fn resample_vertical(img: &RgbImage, out_height: u32) -> RgbImage {
    let coeffs = precompute_coeffs(img.height as i32, out_height as i32);
    let mut out = RgbImage {
        width: img.width,
        height: out_height,
        px: vec![0u8; img.width as usize * out_height as usize * 3],
    };
    let stride = img.width as usize * 3;
    for yy in 0..out_height as usize {
        let (ymin, ymax) = coeffs.bounds[yy];
        let k = &coeffs.k[yy * coeffs.ksize..];
        for xx in 0..stride {
            let mut acc = 1 << (PRECISION_BITS - 1);
            for y in 0..ymax as usize {
                acc += img.px[(y + ymin as usize) * stride + xx] as i32 * k[y];
            }
            out.px[yy * stride + xx] = clip8(acc);
        }
    }
    out
}

/// Resize with Pillow's Lanczos, pass order included: Pillow runs
/// horizontal first unless the height shrinks by more than twice
/// the width shrink. Dimensions must be >= 1.
pub fn resize_lanczos(img: &RgbImage, out_width: u32, out_height: u32) -> RgbImage {
    assert!(
        out_width >= 1 && out_height >= 1,
        "resize target must be >= 1px"
    );
    let need_h = out_width != img.width;
    let need_v = out_height != img.height;
    if !need_h && !need_v {
        return img.clone();
    }
    let horizontal_first = !((img.height as i64 - out_height as i64) > 0
        && (img.height as i64 - out_height as i64) > (img.width as i64 - out_width as i64) * 2);
    let mut cur = img.clone();
    if horizontal_first {
        if need_h {
            cur = resample_horizontal(&cur, out_width);
        }
        if need_v {
            cur = resample_vertical(&cur, out_height);
        }
    } else {
        if need_v {
            cur = resample_vertical(&cur, out_height);
        }
        if need_h {
            cur = resample_horizontal(&cur, out_width);
        }
    }
    cur
}

#[cfg(test)]
mod tests {
    use super::*;

    fn img(w: u32, h: u32, px: &[u8]) -> RgbImage {
        RgbImage {
            width: w,
            height: h,
            px: px.to_vec(),
        }
    }

    #[test]
    fn resize_identity_is_exact() {
        let src = img(
            3,
            2,
            &[
                9, 8, 7, 6, 5, 4, 3, 2, 1, 250, 251, 252, 100, 110, 120, 0, 0, 0,
            ],
        );
        assert_eq!(resize_lanczos(&src, 3, 2), src);
    }

    #[test]
    fn upscale_2x1_to_4x2_matches_pillow() {
        // Pillow 13.0.0 on primo, Image.resize((4,2), LANCZOS).
        let src = img(2, 1, &[0, 0, 0, 255, 255, 255]);
        let out = resize_lanczos(&src, 4, 2);
        assert_eq!(
            out.px,
            vec![
                0, 0, 0, 59, 59, 59, 196, 196, 196, 255, 255, 255, 0, 0, 0, 59, 59, 59, 196, 196,
                196, 255, 255, 255
            ]
        );
    }

    #[test]
    fn downscale_4x1_to_2x1_matches_pillow() {
        let src = img(
            4,
            1,
            &[10, 20, 30, 100, 110, 120, 200, 210, 220, 250, 240, 230],
        );
        let out = resize_lanczos(&src, 2, 1);
        assert_eq!(out.px, vec![62, 73, 85, 222, 223, 224]);
    }

    #[test]
    fn upscale_3x3_to_6x6_matches_pillow() {
        let src = img(
            3,
            3,
            &[
                255, 0, 0, 0, 255, 0, 0, 0, 255, 17, 34, 51, 68, 85, 102, 119, 136, 153, 200, 190,
                180, 90, 80, 70, 1, 2, 3,
            ],
        );
        let out = resize_lanczos(&src, 6, 6);
        assert_eq!(
            out.px,
            vec![
                255, 3, 11, 238, 71, 0, 73, 243, 0, 0, 234, 57, 0, 50, 211, 0, 0, 255, 198, 0, 8,
                164, 50, 4, 58, 189, 14, 19, 203, 82, 27, 83, 205, 39, 35, 247, 50, 0, 18, 51, 29,
                28, 42, 99, 57, 70, 143, 118, 96, 128, 181, 110, 122, 204, 33, 78, 89, 42, 72, 93,
                67, 59, 99, 90, 74, 107, 101, 107, 103, 106, 122, 104, 159, 171, 167, 144, 148,
                148, 114, 99, 108, 64, 49, 57, 37, 36, 31, 30, 35, 23, 246, 222, 210, 213, 192,
                176, 141, 132, 108, 44, 42, 25, 0, 0, 0, 0, 0, 0,
            ]
        );
    }

    #[test]
    fn downscale_3x3_to_2x2_matches_pillow() {
        let src = img(
            3,
            3,
            &[
                255, 0, 0, 0, 255, 0, 0, 0, 255, 17, 34, 51, 68, 85, 102, 119, 136, 153, 200, 190,
                180, 90, 80, 70, 1, 2, 3,
            ],
        );
        let out = resize_lanczos(&src, 2, 2);
        assert_eq!(
            out.px,
            vec![110, 79, 17, 42, 119, 166, 111, 116, 123, 60, 57, 58]
        );
    }

    #[test]
    fn mixed_5x4_to_3x7_matches_pillow() {
        // Vertical-first territory: height grows, width shrinks.
        let src = img(
            5,
            4,
            &[
                0, 0, 0, 37, 91, 53, 74, 182, 106, 111, 17, 159, 148, 108, 212, 185, 199, 9, 222,
                34, 62, 3, 125, 115, 40, 216, 168, 77, 51, 221, 114, 142, 18, 151, 233, 71, 188,
                68, 124, 225, 159, 177, 6, 250, 230, 43, 85, 27, 80, 176, 80, 117, 11, 133, 154,
                102, 186, 191, 193, 239,
            ],
        );
        let out = resize_lanczos(&src, 3, 7);
        assert_eq!(
            out.px,
            vec![
                0, 37, 16, 81, 129, 105, 147, 60, 194, 77, 59, 19, 56, 127, 108, 108, 78, 197, 201,
                110, 25, 50, 125, 114, 56, 124, 203, 204, 170, 31, 141, 131, 120, 52, 187, 209,
                111, 183, 36, 206, 117, 125, 108, 205, 214, 65, 149, 42, 155, 81, 131, 161, 172,
                220, 51, 119, 45, 102, 58, 134, 186, 141, 223,
            ]
        );
    }

    #[test]
    fn constant_image_survives_any_resize() {
        // Adversarial: Lanczos negative lobes must not leak into a
        // flat field at any scale, up or down.
        let src = RgbImage::new_filled(7, 5, [133, 66, 200]);
        for (w, h) in [(1u32, 1u32), (14, 10), (3, 2), (7, 5), (29, 31)] {
            let out = resize_lanczos(&src, w, h);
            assert!(
                out.px.chunks_exact(3).all(|p| p == [133, 66, 200]),
                "resize to {w}x{h} leaked"
            );
        }
    }

    #[test]
    fn paste_clips_at_edges() {
        let mut dst = RgbImage::new_filled(3, 3, [0, 0, 0]);
        let src = RgbImage::new_filled(2, 2, [255, 255, 255]);
        dst.paste(&src, 2, 2);
        assert_eq!(dst.pixel(2, 2), [255, 255, 255]);
        assert_eq!(dst.pixel(1, 1), [0, 0, 0]);
        dst.paste(&src, -1, -1);
        assert_eq!(dst.pixel(0, 0), [255, 255, 255]);
    }
}
