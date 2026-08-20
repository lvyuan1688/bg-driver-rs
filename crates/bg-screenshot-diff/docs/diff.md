# Screenshot diff (v0.1.7)

> `crates/bg-screenshot-diff` — perceptual screenshot differ.

## Why

After an agent action, we want to know: did the screen actually change?
A pixel-perfect compare is too strict (anti-aliasing, cursor blink).
A perceptual diff with a threshold is the right tool.

## API

```rust
use bg_screenshot_diff::diff;

let r = diff(&before, &after, 10)?;  // threshold=10
// r.mean_delta     — mean per-channel abs diff
// r.max_delta      — max per-channel abs diff
// r.changed_pct    — % of pixels above threshold
```

## Threshold tuning

| `threshold` | Meaning |
|-------------|---------|
| 0 | any change counts (too strict) |
| 5–10 | typical for "did anything change" |
| 30+ | only large changes count |

## Edge cases

- Dimension mismatch → `Err`
- Empty rgba → zero diff
- All pixels identical → `changed_pct = 0.0`

## What's NOT in v0.1.7

- Block-based diff (split image into NxN blocks, compare block averages — handles JPEG noise)
- SSIM / PSNR perceptual metrics
- Diff heatmap output (PNG with changed pixels highlighted)
- Region-of-interest masking (ignore clock, ads, etc.)
