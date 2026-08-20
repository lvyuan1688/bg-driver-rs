//! bg-screenshot-diff: perceptual screenshot differ.
//!
//! Compares two `Screenshot`s and returns a `DiffResult` with:
//!   - `mean_delta` — mean per-channel abs diff across all pixels
//!   - `max_delta`  — max per-channel abs diff
//!   - `changed_pct` — % of pixels whose abs diff exceeds `threshold`
//!
//! Useful for detecting "did anything actually change on screen?" after
//! an action, without needing a full image library.

use bg_driver::Screenshot;

/// Perceptual diff between two screenshots.
#[derive(Debug, Clone, PartialEq)]
pub struct DiffResult {
    pub mean_delta: f64,
    pub max_delta: u8,
    pub changed_pct: f64,
    pub width: u32,
    pub height: u32,
}

/// Compare `a` and `b` with a per-channel threshold (default 10).
/// Returns `Err` if dimensions or rgba lengths mismatch.
pub fn diff(a: &Screenshot, b: &Screenshot, threshold: u8) -> Result<DiffResult, String> {
    if a.width != b.width || a.height != b.height {
        return Err(format!(
            "dimension mismatch: {}x{} vs {}x{}",
            a.width, a.height, b.width, b.height
        ));
    }
    if a.rgba.len() != b.rgba.len() {
        return Err(format!(
            "rgba length mismatch: {} vs {}",
            a.rgba.len(),
            b.rgba.len()
        ));
    }

    let mut sum_delta: u64 = 0;
    let mut max_delta: u8 = 0;
    let mut changed_pixels: u64 = 0;
    let mut pixels_compared: u64 = 0;

    // Compare pixel-by-pixel. Each pixel is 4 bytes (RGBA).
    let chunk_size = 4;
    let mut i = 0;
    while i + chunk_size <= a.rgba.len() && i + chunk_size <= b.rgba.len() {
        let mut pixel_delta: u32 = 0;
        for j in 0..chunk_size {
            let d = (a.rgba[i + j] as i32 - b.rgba[i + j] as i32).unsigned_abs();
            pixel_delta += d as u32;
            if d > max_delta as i32 {
                max_delta = d as u8;
            }
        }
        sum_delta += pixel_delta as u64;
        pixels_compared += 1;
        if pixel_delta > threshold as u32 * chunk_size as u32 {
            changed_pixels += 1;
        }
        i += chunk_size;
    }

    let total_possible = pixels_compared.saturating_mul(chunk_size as u64) * 255;
    let mean_delta = if total_possible > 0 {
        sum_delta as f64 / (pixels_compared * chunk_size as u64) as f64
    } else {
        0.0
    };
    let changed_pct = if pixels_compared > 0 {
        (changed_pixels as f64 / pixels_compared as f64) * 100.0
    } else {
        0.0
    };

    Ok(DiffResult {
        mean_delta,
        max_delta,
        changed_pct,
        width: a.width,
        height: a.height,
    })
}

/// Convenience: returns true if `changed_pct` exceeds `min_pct`.
pub fn changed_significantly(r: &DiffResult, min_pct: f64) -> bool {
    r.changed_pct >= min_pct
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shot(rgba: Vec<u8>, w: u32, h: u32) -> Screenshot {
        Screenshot { width: w, height: h, rgba }
    }

    #[test]
    fn identical_screenshots_zero_diff() {
        let a = shot(vec![10, 20, 30, 255, 40, 50, 60, 255], 2, 1);
        let b = shot(vec![10, 20, 30, 255, 40, 50, 60, 255], 2, 1);
        let r = diff(&a, &b, 10).unwrap();
        assert_eq!(r.mean_delta, 0.0);
        assert_eq!(r.max_delta, 0);
        assert_eq!(r.changed_pct, 0.0);
    }

    #[test]
    fn different_pixels_counted() {
        let a = shot(vec![10, 20, 30, 255, 40, 50, 60, 255], 2, 1);
        let b = shot(vec![10, 20, 30, 255, 80, 90, 100, 255], 2, 1);
        let r = diff(&a, &b, 10).unwrap();
        assert!(r.mean_delta > 0.0);
        assert!(r.max_delta >= 40);
        assert_eq!(r.changed_pct, 50.0); // 1 of 2 pixels changed
    }

    #[test]
    fn dimension_mismatch_errors() {
        let a = shot(vec![0, 0, 0, 0], 1, 1);
        let b = shot(vec![0, 0, 0, 0, 0, 0, 0, 0], 2, 1);
        assert!(diff(&a, &b, 10).is_err());
    }

    #[test]
    fn threshold_below_change_flags_pixel() {
        let a = shot(vec![10, 20, 30, 255], 1, 1);
        let b = shot(vec![15, 25, 35, 255], 1, 1);
        // threshold=3 means a delta of 5 per channel should flag.
        let r = diff(&a, &b, 3).unwrap();
        assert_eq!(r.changed_pct, 100.0);
    }

    #[test]
    fn changed_significantly_helper() {
        let r = DiffResult {
            mean_delta: 10.0,
            max_delta: 50,
            changed_pct: 25.0,
            width: 100,
            height: 100,
        };
        assert!(changed_significantly(&r, 20.0));
        assert!(!changed_significantly(&r, 30.0));
    }

    #[test]
    fn empty_rgba_returns_zero_diff() {
        let a = shot(vec![], 0, 0);
        let b = shot(vec![], 0, 0);
        let r = diff(&a, &b, 10).unwrap();
        assert_eq!(r.mean_delta, 0.0);
    }
}
