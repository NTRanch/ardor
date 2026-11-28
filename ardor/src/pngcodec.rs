//! PNG codec for the media tools: decode to RGBA, encode from
//! packed RGB/RGBA, and the channel-max pixel diff.
//!
//! The decoder is a deliberate port of the ardor-verify kit's
//! stdlib decoder (bit depth 8, colour types 0/2/3/4/6,
//! non-interlaced, all five filters) so the MCP tools read pixels
//! exactly the way the gated CLI does — pixels, never file md5
//! (rig-lessons L-37). The encoder writes filter-0 rows; PNG is
//! lossless, so decoded pixels are encoder-independent.

use std::io::Read;

use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;

const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

/// A decoded PNG: dimensions plus RGBA bytes (4 per pixel,
/// row-major, top row first).
#[derive(Debug)]
pub struct PngRgba {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

fn chunk_data<'a>(data: &'a [u8], wanted: &[u8; 4]) -> Vec<&'a [u8]> {
    let mut found = Vec::new();
    if data.len() < 8 || &data[..8] != PNG_SIGNATURE {
        return found;
    }
    let mut pos = 8usize;
    while pos + 8 <= data.len() {
        let length =
            u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
        let ctype = &data[pos + 4..pos + 8];
        let end = pos + 8 + length;
        if end > data.len() {
            break;
        }
        if ctype == wanted {
            found.push(&data[pos + 8..end]);
        }
        pos += 12 + length;
    }
    found
}

/// Decode PNG bytes to RGBA. Mirrors the kit decoder's supported
/// set and its failure modes (as typed errors, not panics).
pub fn decode_png_rgba(data: &[u8]) -> Result<PngRgba, String> {
    if data.len() < 8 || &data[..8] != PNG_SIGNATURE {
        return Err("not a PNG".to_string());
    }
    let mut meta: Option<(u32, u32, u8)> = None;
    let mut plte: Vec<u8> = Vec::new();
    let mut idat: Vec<u8> = Vec::new();
    let mut pos = 8usize;
    while pos + 8 <= data.len() {
        let length =
            u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
        let ctype = &data[pos + 4..pos + 8];
        let end = pos + 8 + length;
        if end > data.len() {
            return Err("truncated chunk".to_string());
        }
        let chunk = &data[pos + 8..end];
        match ctype {
            b"IHDR" => {
                if chunk.len() < 13 {
                    return Err("short IHDR".to_string());
                }
                let w = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                let h = u32::from_be_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
                let bd = chunk[8];
                let ct = chunk[9];
                let inter = chunk[12];
                if bd != 8 {
                    return Err(format!("bit depth {bd} unsupported"));
                }
                if inter != 0 {
                    return Err("interlaced PNG unsupported".to_string());
                }
                meta = Some((w, h, ct));
            }
            b"PLTE" => plte = chunk.to_vec(),
            b"IDAT" => idat.extend_from_slice(chunk),
            _ => {}
        }
        pos += 12 + length;
    }
    let (w, h, ct) = meta.ok_or("missing IHDR")?;
    let nch: usize = match ct {
        0 => 1,
        2 => 3,
        3 => 1,
        4 => 2,
        6 => 4,
        other => return Err(format!("colour type {other} unsupported")),
    };
    let mut decoder = ZlibDecoder::new(&idat[..]);
    let mut raw = Vec::new();
    decoder
        .read_to_end(&mut raw)
        .map_err(|e| format!("zlib inflate failed: {e}"))?;
    let (w_us, h_us) = (w as usize, h as usize);
    let stride = w_us * nch;
    if raw.len() < h_us * (stride + 1) {
        return Err("short image data".to_string());
    }
    let mut out = vec![0u8; h_us * stride];
    let mut prev = vec![0u8; stride];
    let mut p = 0usize;
    for y in 0..h_us {
        let f = raw[p];
        p += 1;
        let mut row = raw[p..p + stride].to_vec();
        p += stride;
        match f {
            0 => {}
            1 => {
                for i in nch..stride {
                    row[i] = row[i].wrapping_add(row[i - nch]);
                }
            }
            2 => {
                for i in 0..stride {
                    row[i] = row[i].wrapping_add(prev[i]);
                }
            }
            3 => {
                for i in 0..stride {
                    let left = if i >= nch { row[i - nch] } else { 0 } as u16;
                    row[i] = row[i].wrapping_add(((left + prev[i] as u16) >> 1) as u8);
                }
            }
            4 => {
                for i in 0..stride {
                    let a = if i >= nch { row[i - nch] } else { 0 } as i32;
                    let b = prev[i] as i32;
                    let c = if i >= nch { prev[i - nch] } else { 0 } as i32;
                    let pa = (b - c).abs();
                    let pb = (a - c).abs();
                    let pc = (a + b - 2 * c).abs();
                    let pr = if pa <= pb && pa <= pc {
                        a
                    } else if pb <= pc {
                        b
                    } else {
                        c
                    };
                    row[i] = row[i].wrapping_add(pr as u8);
                }
            }
            other => return Err(format!("unknown filter {other}")),
        }
        out[y * stride..(y + 1) * stride].copy_from_slice(&row);
        prev = row;
    }
    if ct == 6 {
        return Ok(PngRgba {
            width: w,
            height: h,
            rgba: out,
        });
    }
    let mut rgba = vec![0u8; w_us * h_us * 4];
    match ct {
        2 => {
            for i in 0..w_us * h_us {
                rgba[4 * i..4 * i + 3].copy_from_slice(&out[3 * i..3 * i + 3]);
                rgba[4 * i + 3] = 255;
            }
        }
        0 => {
            for i in 0..w_us * h_us {
                rgba[4 * i] = out[i];
                rgba[4 * i + 1] = out[i];
                rgba[4 * i + 2] = out[i];
                rgba[4 * i + 3] = 255;
            }
        }
        4 => {
            for i in 0..w_us * h_us {
                rgba[4 * i] = out[2 * i];
                rgba[4 * i + 1] = out[2 * i];
                rgba[4 * i + 2] = out[2 * i];
                rgba[4 * i + 3] = out[2 * i + 1];
            }
        }
        3 => {
            for i in 0..w_us * h_us {
                let idx = out[i] as usize * 3;
                if idx + 3 > plte.len() {
                    return Err("palette index out of range".to_string());
                }
                rgba[4 * i..4 * i + 3].copy_from_slice(&plte[idx..idx + 3]);
                rgba[4 * i + 3] = 255;
            }
        }
        _ => unreachable!(),
    }
    Ok(PngRgba {
        width: w,
        height: h,
        rgba,
    })
}

fn png_chunk(out: &mut Vec<u8>, ctype: &[u8; 4], payload: &[u8]) {
    out.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    out.extend_from_slice(ctype);
    out.extend_from_slice(payload);
    let mut crc_data = Vec::with_capacity(4 + payload.len());
    crc_data.extend_from_slice(ctype);
    crc_data.extend_from_slice(payload);
    out.extend_from_slice(&crc32(&crc_data).to_be_bytes());
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn encode_png(width: u32, height: u32, colour_type: u8, rows: &[u8], stride: usize) -> Vec<u8> {
    let mut raw = Vec::with_capacity(height as usize * (stride + 1));
    for y in 0..height as usize {
        raw.push(0);
        raw.extend_from_slice(&rows[y * stride..(y + 1) * stride]);
    }
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(6));
    use std::io::Write;
    encoder.write_all(&raw).expect("zlib encode to memory");
    let idat = encoder.finish().expect("zlib finish");
    let mut out = Vec::new();
    out.extend_from_slice(PNG_SIGNATURE);
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.extend_from_slice(&[8, colour_type, 0, 0, 0]);
    png_chunk(&mut out, b"IHDR", &ihdr);
    png_chunk(&mut out, b"IDAT", &idat);
    png_chunk(&mut out, b"IEND", &[]);
    out
}

/// Encode packed RGB bytes as a colour-type-2 PNG.
pub fn encode_png_rgb(width: u32, height: u32, rgb: &[u8]) -> Vec<u8> {
    encode_png(width, height, 2, rgb, width as usize * 3)
}

/// Encode RGBA bytes as a colour-type-6 PNG.
pub fn encode_png_rgba(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
    encode_png(width, height, 6, rgba, width as usize * 4)
}

/// The raw IHDR chunk payload and concatenated IDAT payload of a
/// PNG — the kit's fast identity path: equal IDAT under the same
/// IHDR means equal pixels, no decode needed.
pub fn png_idat(data: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let ihdr = chunk_data(data, b"IHDR")
        .into_iter()
        .next()
        .unwrap_or(&[])
        .to_vec();
    let mut idat = Vec::new();
    for part in chunk_data(data, b"IDAT") {
        idat.extend_from_slice(part);
    }
    (ihdr, idat)
}

/// Channel-max pixel diff between two PNG files: a pixel counts
/// as differing when any channel differs. Returns
/// (differing_px, total_px, max_channel_delta).
pub fn pixel_diff(a: &[u8], b: &[u8]) -> Result<(u64, u64, u32), String> {
    let (ihdr_a, idat_a) = png_idat(a);
    let (ihdr_b, idat_b) = png_idat(b);
    if ihdr_a == ihdr_b && idat_a == idat_b && ihdr_a.len() >= 8 {
        let w = u32::from_be_bytes([ihdr_a[0], ihdr_a[1], ihdr_a[2], ihdr_a[3]]) as u64;
        let h = u32::from_be_bytes([ihdr_a[4], ihdr_a[5], ihdr_a[6], ihdr_a[7]]) as u64;
        return Ok((0, w * h, 0));
    }
    let pa = decode_png_rgba(a)?;
    let pb = decode_png_rgba(b)?;
    if (pa.width, pa.height) != (pb.width, pb.height) {
        return Err(format!(
            "dimension mismatch {}x{} vs {}x{}",
            pa.width, pa.height, pb.width, pb.height
        ));
    }
    let mut differing = 0u64;
    let mut max_delta = 0u32;
    for i in (0..pa.rgba.len()).step_by(4) {
        let d = (0..4)
            .map(|c| (pa.rgba[i + c] as i32 - pb.rgba[i + c] as i32).unsigned_abs())
            .max()
            .unwrap_or(0);
        if d > 0 {
            differing += 1;
            if d > max_delta {
                max_delta = d;
            }
        }
    }
    Ok((differing, pa.width as u64 * pa.height as u64, max_delta))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TINY_RGB: &[u8] = include_bytes!("../tests/fixtures/tiny_rgb2x2.png");
    const TINY_RGBA: &[u8] = include_bytes!("../tests/fixtures/tiny_rgba2x2.png");
    const TINY_GRAY: &[u8] = include_bytes!("../tests/fixtures/tiny_gray1x1.png");
    const TINY_LA: &[u8] = include_bytes!("../tests/fixtures/tiny_la2x1.png");
    const TINY_PAL: &[u8] = include_bytes!("../tests/fixtures/tiny_pal2x1.png");

    #[test]
    fn decode_rgb_fixture_pixels() {
        let img = decode_png_rgba(TINY_RGB).expect("decode");
        assert_eq!((img.width, img.height), (2, 2));
        assert_eq!(
            img.rgba,
            vec![
                255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255
            ]
        );
    }

    #[test]
    fn decode_rgba_fixture_pixels() {
        let img = decode_png_rgba(TINY_RGBA).expect("decode");
        assert_eq!(
            img.rgba,
            vec![255, 0, 0, 128, 0, 255, 0, 255, 0, 0, 255, 0, 10, 20, 30, 40]
        );
    }

    #[test]
    fn decode_gray_la_palette_fixtures() {
        let g = decode_png_rgba(TINY_GRAY).expect("gray");
        assert_eq!(g.rgba, vec![77, 77, 77, 255]);
        let la = decode_png_rgba(TINY_LA).expect("la");
        assert_eq!(la.rgba, vec![50, 50, 50, 60, 200, 200, 200, 210]);
        let p = decode_png_rgba(TINY_PAL).expect("palette");
        assert_eq!(p.rgba, vec![9, 8, 7, 255, 100, 110, 120, 255]);
    }

    #[test]
    fn encode_decode_roundtrip_preserves_pixels() {
        let rgb: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let bytes = encode_png_rgb(2, 2, &rgb);
        let back = decode_png_rgba(&bytes).expect("roundtrip");
        assert_eq!(
            back.rgba,
            vec![1, 2, 3, 255, 4, 5, 6, 255, 7, 8, 9, 255, 10, 11, 12, 255]
        );
    }

    #[test]
    fn pixel_diff_identical_is_zero() {
        let (d, px, md) = pixel_diff(TINY_RGB, TINY_RGB).expect("diff");
        assert_eq!((d, px, md), (0, 4, 0));
    }

    #[test]
    fn pixel_diff_catches_one_corrupted_pixel() {
        // Adversarial control: one pixel of the RGBA fixture changed
        // by +17 on one channel must read as exactly 1 differing px.
        let mut corrupt = decode_png_rgba(TINY_RGBA).expect("decode");
        corrupt.rgba[1] = corrupt.rgba[1].wrapping_add(17);
        let bytes = encode_png_rgba(2, 2, &corrupt.rgba);
        let (d, px, md) = pixel_diff(TINY_RGBA, &bytes).expect("diff");
        assert_eq!((d, px, md), (1, 4, 17));
    }

    #[test]
    fn decode_rejects_non_png_and_truncated() {
        assert!(decode_png_rgba(b"this is not a png").is_err());
        assert!(decode_png_rgba(&TINY_RGB[..20]).is_err());
    }

    #[test]
    fn decode_rejects_interlaced() {
        // Adversarial: flip the IHDR interlace byte (offset 28).
        let mut bytes = TINY_RGB.to_vec();
        bytes[28] = 1;
        let err = decode_png_rgba(&bytes).expect_err("interlaced must fail");
        assert!(err.contains("interlaced"), "{err}");
    }

    #[test]
    fn pixel_diff_dimension_mismatch_errors() {
        let err = pixel_diff(TINY_RGB, TINY_GRAY).expect_err("dims must fail");
        assert!(err.contains("dimension mismatch"), "{err}");
    }
}
