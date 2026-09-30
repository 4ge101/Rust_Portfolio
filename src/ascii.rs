use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use image::imageops::{self, FilterType};
use image::{DynamicImage, GrayImage};

const CELL_ASPECT: f32 = 0.5;

const MAX_SOURCE_DIMENSION: u32 = 640;

const PALETTE: &str = " .'`,:;-~+*=?%S#@";

const TONE_GAMMA: f32 = 1.6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Options {
    pub cols: u16,
    pub rows: u16,
    pub invert: bool,
}

pub fn fit(src_w: u32, src_h: u32, cols: u16, rows: u16) -> (u32, u32) {
    let (cols, rows) = (u32::from(cols.max(1)), u32::from(rows.max(1)));
    let (sw, sh) = (src_w.max(1) as f32, src_h.max(1) as f32);

    let rows_for_full_width = cols as f32 * (sh / sw) * CELL_ASPECT;
    if rows_for_full_width <= rows as f32 {
        (cols, (rows_for_full_width.round() as u32).max(1))
    } else {
        let cols_for_full_height = rows as f32 * (sw / sh) / CELL_ASPECT;
        ((cols_for_full_height.round() as u32).clamp(1, cols), rows)
    }
}

pub fn render(gray: &GrayImage, options: Options) -> Vec<String> {
    if options.cols == 0 || options.rows == 0 || gray.width() == 0 || gray.height() == 0 {
        return Vec::new();
    }
    let (w, h) = fit(gray.width(), gray.height(), options.cols, options.rows);
    let small = imageops::resize(gray, w, h, FilterType::Lanczos3);

    let (lo, hi) = percentile_range(&small, 0.02, 0.98);

    let palette: Vec<char> = PALETTE.chars().collect();
    let top = palette.len() - 1;

    (0..h)
        .map(|y| {
            (0..w)
                .map(|x| {
                    let value = small.get_pixel(x, y).0[0];
                    let mut t = normalise(value, lo, hi);
                    if options.invert {
                        t = 1.0 - t;
                    }
                    t = t.powf(TONE_GAMMA);
                    palette[(t * top as f32).round() as usize]
                })
                .collect()
        })
        .collect()
}

fn normalise(value: u8, lo: u8, hi: u8) -> f32 {
    if hi <= lo.saturating_add(4) {
        return f32::from(value) / 255.0;
    }
    ((f32::from(value) - f32::from(lo)) / (f32::from(hi) - f32::from(lo))).clamp(0.0, 1.0)
}

fn percentile_range(gray: &GrayImage, low: f32, high: f32) -> (u8, u8) {
    let mut histogram = [0u32; 256];
    for pixel in gray.pixels() {
        histogram[usize::from(pixel.0[0])] += 1;
    }
    let total = gray.width() * gray.height();
    let at = |fraction: f32| {
        let target = (total as f32 * fraction) as u32;
        let mut seen = 0;
        for (value, count) in histogram.iter().enumerate() {
            seen += count;
            if seen > target {
                return value as u8;
            }
        }
        255
    };
    (at(low), at(high))
}

#[cfg(bundled_portrait)]
const BUNDLED_PORTRAIT: Option<&[u8]> = Some(include_bytes!("../assets/profile.png"));
#[cfg(not(bundled_portrait))]
const BUNDLED_PORTRAIT: Option<&[u8]> = None;

pub struct Portrait {
    gray: GrayImage,
    cache: Option<(Options, Vec<String>)>,
}

impl Portrait {
    fn from_image(image: DynamicImage) -> Self {
        let image = if image.width().max(image.height()) > MAX_SOURCE_DIMENSION {
            image.resize(
                MAX_SOURCE_DIMENSION,
                MAX_SOURCE_DIMENSION,
                FilterType::Lanczos3,
            )
        } else {
            image
        };
        Self {
            gray: image.to_luma8(),
            cache: None,
        }
    }

    pub fn from_path(path: &Path) -> Result<Self> {
        let image = image::open(path).with_context(|| format!("cannot read {}", path.display()))?;
        Ok(Self::from_image(image))
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let image =
            image::load_from_memory(bytes).context("bundled portrait is not a valid image")?;
        Ok(Self::from_image(image))
    }

    pub fn lines(&mut self, options: Options) -> &[String] {
        let stale = self
            .cache
            .as_ref()
            .is_none_or(|(cached, _)| *cached != options);
        if stale {
            self.cache = Some((options, render(&self.gray, options)));
        }
        &self.cache.as_ref().expect("cache filled above").1
    }
}

pub struct PortraitLoad {
    pub portrait: Option<Portrait>,
    pub note: Option<String>,
    pub source: Option<String>,
}

pub fn resolve(explicit: Option<&Path>, data_dir: Option<&Path>) -> PortraitLoad {
    let mut note = None;

    if let Some(path) = explicit {
        match Portrait::from_path(path) {
            Ok(portrait) => return found(portrait, path.display().to_string()),
            Err(err) => note = Some(format!("--image: {err:#}")),
        }
    }

    if let Some(dir) = data_dir {
        let candidates: Vec<PathBuf> = ["profile.jpg", "profile.jpeg", "profile.png"]
            .iter()
            .map(|name| dir.join(name))
            .filter(|path| path.is_file())
            .collect();
        if let Some(path) = candidates.first() {
            match Portrait::from_path(path) {
                Ok(portrait) => return found(portrait, path.display().to_string()),
                Err(err) => note = Some(format!("{err:#}")),
            }
        }
    }

    if let Some(bytes) = BUNDLED_PORTRAIT {
        match Portrait::from_bytes(bytes) {
            Ok(portrait) => return found(portrait, "bundled assets/profile.png".to_owned()),
            Err(err) => note = Some(format!("{err:#}")),
        }
    }

    PortraitLoad {
        portrait: None,
        note: Some(note.unwrap_or_else(|| {
            "no portrait: add assets/profile.png and rebuild, or pass --image PATH".to_owned()
        })),
        source: None,
    }
}

fn found(portrait: Portrait, source: String) -> PortraitLoad {
    PortraitLoad {
        portrait: Some(portrait),
        note: None,
        source: Some(source),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Luma;

    fn ramp(w: u32, h: u32) -> GrayImage {
        GrayImage::from_fn(w, h, |x, _| Luma([(x * 255 / (w - 1)) as u8]))
    }

    fn options(cols: u16, rows: u16) -> Options {
        Options {
            cols,
            rows,
            invert: false,
        }
    }

    #[test]
    fn fit_corrects_for_cell_aspect() {
        assert_eq!(fit(100, 100, 40, 40), (40, 20));

        assert_eq!(fit(200, 100, 40, 40), (40, 10));
    }

    #[test]
    fn fit_respects_the_row_limit() {
        assert_eq!(fit(100, 400, 40, 20), (10, 20));
        let (w, h) = fit(1000, 1000, 500, 5);
        assert!(h <= 5 && w <= 500);
    }

    #[test]
    fn output_matches_requested_geometry() {
        let lines = render(&ramp(100, 100), options(40, 40));
        assert_eq!(lines.len(), 20);
        assert!(lines.iter().all(|line| line.chars().count() == 40));
    }

    #[test]
    fn brightness_maps_to_density() {
        let lines = render(&ramp(128, 64), options(32, 16));
        let palette: Vec<char> = PALETTE.chars().collect();
        for line in &lines {
            let chars: Vec<char> = line.chars().collect();
            assert_eq!(chars[0], palette[0], "black is blank");
            assert_eq!(
                *chars.last().unwrap(),
                *palette.last().unwrap(),
                "white is densest"
            );
            let rank = |c: &char| palette.iter().position(|p| p == c).unwrap();
            assert!(chars
                .windows(2)
                .all(|pair| rank(&pair[0]) <= rank(&pair[1])));
        }
    }

    #[test]
    fn invert_flips_the_ramp() {
        let normal = render(&ramp(64, 32), options(16, 8));
        let mut flipped_options = options(16, 8);
        flipped_options.invert = true;
        let flipped = render(&ramp(64, 32), flipped_options);
        let palette: Vec<char> = PALETTE.chars().collect();
        assert_eq!(normal[0].chars().next(), Some(palette[0]));
        assert_eq!(flipped[0].chars().next(), palette.last().copied());
    }

    #[test]
    fn degenerate_inputs_do_not_panic() {
        let flat = GrayImage::from_pixel(10, 10, Luma([128]));
        assert!(!render(&flat, options(20, 10)).is_empty());
        assert!(render(&flat, options(0, 10)).is_empty());
        let tiny = GrayImage::from_pixel(1, 1, Luma([0]));
        assert_eq!(render(&tiny, options(1, 1)).len(), 1);
    }

    #[test]
    fn missing_image_is_a_note_not_an_error() {
        let loaded = resolve(Some(Path::new("/definitely/not/here.png")), None);
        assert!(loaded.note.is_some());
        if BUNDLED_PORTRAIT.is_none() {
            assert!(loaded.portrait.is_none());
        }
    }

    #[test]
    fn portrait_render_is_cached() {
        let mut portrait = Portrait {
            gray: ramp(64, 64),
            cache: None,
        };
        let o = options(20, 10);
        let first = portrait.lines(o).to_vec();
        assert_eq!(portrait.lines(o), first.as_slice());
        let wider = options(30, 10);
        assert_ne!(portrait.lines(wider), first.as_slice());
    }
}
