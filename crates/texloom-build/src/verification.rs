//! Same-renderer displacement comparison, independent of the UI and source planner.
//!
//! Evidence must come from a trusted inspector. In particular `target` is a
//! provenance assertion, not a client-supplied selection rectangle. This module
//! is supplied by the native document gate from a checked isolated inspector;
//! a bare comparison result is not itself document-commit authorization.
use std::collections::BTreeMap;

pub type Result<T> = std::result::Result<T, &'static str>;
const EPSILON: f64 = 0.000001;
const NEIGHBOR_TOLERANCE: f64 = 0.02;
const TARGET_TOLERANCE: f64 = 0.25;
const MAX_PIXELS: usize = 64_000_000;

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
impl Rect {
    fn finite(self) -> bool {
        [self.x, self.y, self.width, self.height]
            .iter()
            .all(|v| v.is_finite())
            && self.width >= 0.0
            && self.height >= 0.0
    }
    fn translated(self, dx: f64, dy: f64) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
            ..self
        }
    }
    fn near(self, other: Self, tolerance: f64) -> bool {
        [
            (self.x, other.x),
            (self.y, other.y),
            (self.width, other.width),
            (self.height, other.height),
        ]
        .iter()
        .all(|(a, b)| (a - b).abs() <= tolerance)
    }
    fn intersects(self, other: Self) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }
}
#[derive(Clone, Debug)]
pub struct Page {
    pub width: f64,
    pub height: f64,
    pub crop: [f64; 4],
    pub rotation: i32,
    pub annotations: usize,
}
#[derive(Clone, Debug)]
pub struct Glyph {
    pub text: String,
    /// Zero-based physical page. Geometry is top-left PDF bp, never CSS pixels.
    pub page: usize,
    pub bounds: Rect,
    pub target: bool,
}
#[derive(Clone, Debug)]
pub struct Graphic {
    pub kind: i32,
    pub page: usize,
    pub bounds: Rect,
}
#[derive(Clone, Debug)]
pub struct Raster {
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub channels: usize,
    pub scale: f64,
    /// The inspector must render onto opaque white, with identical settings.
    pub pixels: Vec<u8>,
}
#[derive(Clone, Debug)]
pub struct Inspection {
    pub pages: Vec<Page>,
    pub glyphs: Vec<Glyph>,
    pub graphics: Vec<Graphic>,
    pub rasters: Vec<Raster>,
}
#[derive(Clone, Copy, Debug)]
pub enum Operation {
    Identity,
    Displace { dx: f64, dy: f64 },
}
#[derive(Debug)]
pub struct Verified {
    pub glyphs: usize,
    pub target_glyphs: usize,
    pub pixels: usize,
    pub max_geometry_error_bp: f64,
}
fn require(ok: bool, reason: &'static str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(reason)
    }
}
fn inside(r: Rect, p: &Page) -> bool {
    r.finite()
        && r.x >= -EPSILON
        && r.y >= -EPSILON
        && r.x + r.width <= p.width + EPSILON
        && r.y + r.height <= p.height + EPSILON
}
pub fn validate(view: &Inspection) -> Result<usize> {
    require(
        !view.pages.is_empty() && view.pages.len() <= 20 && view.rasters.len() == view.pages.len(),
        "Invalid page/raster count",
    )?;
    require(
        view.glyphs.len() <= 50_000 && view.graphics.len() <= 10_000,
        "Inspection exceeds object budget",
    )?;
    let mut pixels = 0usize;
    for (p, r) in view.pages.iter().zip(&view.rasters) {
        require(
            p.width.is_finite()
                && p.height.is_finite()
                && p.width > 0.0
                && p.height > 0.0
                && p.width <= 14_400.0
                && p.height <= 14_400.0,
            "Invalid page dimensions",
        )?;
        require(
            p.rotation == 0
                && p.crop.iter().all(|n| n.is_finite())
                && p.crop
                    .iter()
                    .zip([0.0, 0.0, p.width, p.height])
                    .all(|(a, b)| (a - b).abs() <= EPSILON),
            "Crop or rotation requires a coordinate adapter",
        )?;
        require(
            p.annotations <= 10_000,
            "Excessive annotation count",
        )?;
        require(
            r.scale.is_finite() && r.scale > 0.0 && r.scale <= 4.0 && [3, 4].contains(&r.channels),
            "Unsupported raster format",
        )?;
        require(
            (r.width as f64 - p.width * r.scale).abs() <= 1.0
                && (r.height as f64 - p.height * r.scale).abs() <= 1.0,
            "Raster dimensions do not match page",
        )?;
        let count = r
            .width
            .checked_mul(r.height)
            .ok_or("Raster budget overflow")?;
        require(
            count > 0 && count <= 16_000_000,
            "Page exceeds raster budget",
        )?;
        pixels = pixels.checked_add(count).ok_or("Raster budget overflow")?;
        require(pixels <= MAX_PIXELS, "Document exceeds raster budget")?;
        require(
            r.width
                .checked_mul(r.channels)
                .is_some_and(|n| r.stride >= n)
                && r.stride.checked_mul(r.height) == Some(r.pixels.len()),
            "Truncated or invalid raster buffer",
        )?;
    }
    for g in &view.glyphs {
        require(
            !g.text.trim().is_empty()
                && g.text.len() <= 64
                && view.pages.get(g.page).is_some_and(|p| inside(g.bounds, p))
                && g.bounds.width > 0.0
                && g.bounds.height > 0.0,
            "Invalid glyph evidence",
        )?;
    }
    for g in &view.graphics {
        require(
            view.pages.get(g.page).is_some_and(|p| inside(g.bounds, p)),
            "Invalid graphic evidence",
        )?;
    }
    Ok(pixels)
}
type Key<'a> = (&'a str, usize, bool);
fn groups(view: &Inspection) -> BTreeMap<Key<'_>, Vec<usize>> {
    let mut groups = BTreeMap::new();
    for (i, g) in view.glyphs.iter().enumerate() {
        groups
            .entry((g.text.as_str(), g.page, g.target))
            .or_insert_with(Vec::new)
            .push(i);
    }
    groups
}
fn match_glyphs(a: &Inspection, b: &Inspection, dx: f64, dy: f64, identity: bool) -> Result<f64> {
    require(a.glyphs.len() == b.glyphs.len(), "Glyph count changed")?;
    let old = groups(a);
    let new = groups(b);
    let mut cost = 0usize;
    require(
        old.keys().eq(new.keys()),
        "Text, page or target identity changed",
    )?;
    let mut max_error = 0.0f64;
    for (key, before) in &old {
        let after = &new[key];
        require(before.len() == after.len(), "Glyph multiplicity changed")?;
        cost = cost
            .checked_add(
                before
                    .len()
                    .checked_mul(after.len())
                    .ok_or("Matching budget overflow")?,
            )
            .ok_or("Matching budget overflow")?;
        require(cost <= 2_000_000, "Ambiguous text exceeds matching budget")?;
        let mut remaining = after.clone();
        for &i in before {
            let g = &a.glyphs[i];
            let expected = if g.target {
                g.bounds.translated(dx, dy)
            } else {
                g.bounds
            };
            let tolerance = if identity {
                NEIGHBOR_TOLERANCE
            } else if g.target {
                TARGET_TOLERANCE
            } else {
                NEIGHBOR_TOLERANCE
            };
            let matches: Vec<_> = remaining
                .iter()
                .copied()
                .filter(|j| expected.near(b.glyphs[*j].bounds, tolerance))
                .collect();
            let &j = matches
                .first()
                .ok_or("Target or neighbor has no match at its expected position")?;
            require(
                matches
                    .iter()
                    .all(|n| b.glyphs[*n].bounds.near(b.glyphs[j].bounds, EPSILON)),
                "Ambiguous glyph correspondence",
            )?;
            let next = b.glyphs[j].bounds;
            for delta in [
                next.x - expected.x,
                next.y - expected.y,
                next.width - expected.width,
                next.height - expected.height,
            ] {
                max_error = max_error.max(delta.abs());
            }
            remaining.retain(|n| *n != j);
        }
    }
    Ok(max_error)
}
fn match_graphics(a: &Inspection, b: &Inspection) -> Result<()> {
    require(
        a.graphics.len() == b.graphics.len(),
        "Non-text object count changed",
    )?;
    require(
        a.graphics
            .len()
            .checked_mul(b.graphics.len())
            .is_some_and(|n| n <= 2_000_000),
        "Non-text matching budget exceeded",
    )?;
    let mut remaining: Vec<_> = (0..b.graphics.len()).collect();
    for old in &a.graphics {
        let position = remaining
            .iter()
            .position(|j| {
                let new = &b.graphics[*j];
                old.kind == new.kind && old.page == new.page && old.bounds.near(new.bounds, EPSILON)
            })
            .ok_or("Non-text object changed")?;
        remaining.swap_remove(position);
    }
    Ok(())
}
#[derive(Clone, Copy)]
struct Window {
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
}
fn window(bounds: Rect, r: &Raster) -> Result<Window> {
    let x0 = (bounds.x * r.scale).floor() - 1.0;
    let y0 = (bounds.y * r.scale).floor() - 1.0;
    let x1 = ((bounds.x + bounds.width) * r.scale).ceil() + 1.0;
    let y1 = ((bounds.y + bounds.height) * r.scale).ceil() + 1.0;
    require(
        x0 >= 0.0 && y0 >= 0.0 && x1 <= r.width as f64 && y1 <= r.height as f64,
        "Target raster leaves page",
    )?;
    Ok(Window {
        x0: x0 as usize,
        y0: y0 as usize,
        x1: x1 as usize,
        y1: y1 as usize,
    })
}
fn isolated(w: Window, page: usize, raster: &Raster, a: &Inspection, b: &Inspection) -> Result<()> {
    let mask = Rect {
        x: w.x0 as f64 / raster.scale,
        y: w.y0 as f64 / raster.scale,
        width: (w.x1 - w.x0) as f64 / raster.scale,
        height: (w.y1 - w.y0) as f64 / raster.scale,
    };
    for view in [a, b] {
        require(
            !view
                .glyphs
                .iter()
                .any(|g| g.page == page && !g.target && mask.intersects(g.bounds)),
            "Target raster overlaps unselected text",
        )?;
        require(
            !view
                .graphics
                .iter()
                .any(|g| g.page == page && mask.intersects(g.bounds)),
            "Target raster overlaps a non-text object",
        )?;
    }
    Ok(())
}
/// Fail closed for unsupported coordinates, annotations, overlap, fractional
/// raster shifts or bounded-resource exhaustion. No tolerances are supplied by
/// the frontend. Identity requires complete same-renderer pixel equality.
pub fn compare(a: &Inspection, b: &Inspection, operation: Operation) -> Result<Verified> {
    compare_impl(a,b,operation,None)
}
/// `prediction` is a separately inspected baseline whose selected PDF text
/// objects were translated by the pinned worker. No candidate pixel is used to
/// construct it. Its original geometry and the complete resulting raster are
/// checked, including all pixels outside and within the target.
pub fn compare_prediction(a:&Inspection,b:&Inspection,dx:f64,dy:f64,prediction:&Inspection)->Result<Verified>{
    validate(prediction)?;
    require(a.pages.len()==prediction.pages.len(),"Prediction page count changed")?;
    match_glyphs(a,prediction,0.0,0.0,true)?;match_graphics(a,prediction)?;
    for (p,q) in a.pages.iter().zip(&prediction.pages){require(p.width==q.width&&p.height==q.height&&p.crop==q.crop&&p.rotation==q.rotation&&p.annotations==q.annotations,"Prediction page geometry changed")?;}
    compare_impl(a,b,Operation::Displace{dx,dy},Some(prediction))
}
fn compare_impl(a: &Inspection, b: &Inspection, operation: Operation,prediction:Option<&Inspection>) -> Result<Verified> {
    let pixels = validate(a)?;
    validate(b)?;
    require(a.pages.len() == b.pages.len(), "Page count changed")?;
    for (p, q) in a.pages.iter().zip(&b.pages) {
        require(p.annotations==0&&q.annotations==0,"Annotations require a separate movement adapter")?;
        require(
            (p.width - q.width).abs() <= EPSILON && (p.height - q.height).abs() <= EPSILON,
            "Page dimensions changed",
        )?;
    }
    let (dx, dy, identity) = match operation {
        Operation::Identity => (0.0, 0.0, true),
        Operation::Displace { dx, dy } => (dx, dy, false),
    };
    require(
        dx.is_finite() && dy.is_finite() && dx.abs() <= 2_000.0 && dy.abs() <= 2_000.0,
        "Invalid displacement",
    )?;
    let targets: Vec<_> = a
        .glyphs
        .iter()
        .enumerate()
        .filter(|(_, g)| g.target)
        .map(|(i, _)| i)
        .collect();
    require(
        identity || !targets.is_empty(),
        "Target identity is missing",
    )?;
    if !identity {
        require(
            targets
                .iter()
                .all(|i| a.glyphs[*i].page == a.glyphs[targets[0]].page),
            "Target crosses physical pages",
        )?;
    }
    let max_geometry_error_bp = match_glyphs(a, b, dx, dy, identity)?;
    match_graphics(a, b)?;
    let object_count = a.glyphs.len() + b.glyphs.len() + a.graphics.len() + b.graphics.len();
    require(
        identity
            || targets
                .len()
                .checked_mul(object_count)
                .and_then(|n| n.checked_mul(2))
                .is_some_and(|n| n <= 20_000_000),
        "Target isolation budget exceeded",
    )?;
    for (page, (old, new)) in a.rasters.iter().zip(&b.rasters).enumerate() {
        require(
            old.width == new.width
                && old.height == new.height
                && old.stride == new.stride
                && old.channels == new.channels
                && old.scale == new.scale,
            "Raster configuration changed",
        )?;
        let mut expected = if let Some(prediction)=prediction {let p=&prediction.rasters[page];require(p.width==old.width&&p.height==old.height&&p.stride==old.stride&&p.channels==old.channels&&p.scale==old.scale,"Prediction raster configuration changed")?;p.pixels.clone()}else{old.pixels.clone()};
        if !identity && targets.iter().any(|i| a.glyphs[*i].page == page) {
            if prediction.is_some(){
                for i in &targets {let glyph=&a.glyphs[*i];if glyph.page!=page{continue}let w=window(glyph.bounds,old)?;let d=window(glyph.bounds.translated(dx,dy),old)?;isolated(w,page,old,a,b)?;isolated(d,page,old,a,b)?;}
            }else{
            let shift_x = dx * old.scale;
            let shift_y = dy * old.scale;
            require(
                (shift_x - shift_x.round()).abs() <= EPSILON
                    && (shift_y - shift_y.round()).abs() <= EPSILON,
                "Fractional-pixel appearance needs a separate adapter",
            )?;
            let (sx, sy) = (shift_x.round() as isize, shift_y.round() as isize);
            let mut windows = Vec::new();
            let mut work = 0usize;
            for i in &targets {
                let glyph = &a.glyphs[*i];
                if glyph.page != page {
                    continue;
                }
                let w = window(glyph.bounds, old)?;
                require(
                    (w.x0 as isize) + sx >= 0
                        && (w.y0 as isize) + sy >= 0
                        && (w.x1 as isize) + sx <= old.width as isize
                        && (w.y1 as isize) + sy <= old.height as isize,
                    "Target raster leaves page",
                )?;
                let dest = Window {
                    x0: ((w.x0 as isize) + sx) as usize,
                    y0: ((w.y0 as isize) + sy) as usize,
                    x1: ((w.x1 as isize) + sx) as usize,
                    y1: ((w.y1 as isize) + sy) as usize,
                };
                isolated(w, page, old, a, b)?;
                isolated(dest, page, old, a, b)?;
                work = work
                    .checked_add((w.x1 - w.x0) * (w.y1 - w.y0))
                    .ok_or("Target raster budget overflow")?;
                require(work <= MAX_PIXELS, "Target raster budget exceeded")?;
                windows.push((w, dest));
            }
            // Reconstruct the entire expected image. Merely masking old/new
            // rectangles could accept duplicate ink left in the vacated area.
            // No other paint may intersect these windows; the worker must have
            // complete object coverage and render onto opaque white.
            for (w, _) in &windows {
                for y in w.y0..w.y1 {
                    expected[y * old.stride + w.x0 * old.channels
                        ..y * old.stride + w.x1 * old.channels]
                        .fill(255);
                }
            }
            for (w, d) in &windows {
                for offset in 0..w.y1 - w.y0 {
                    let from = (w.y0 + offset) * old.stride + w.x0 * old.channels;
                    let to = (d.y0 + offset) * old.stride + d.x0 * old.channels;
                    let length = (w.x1 - w.x0) * old.channels;
                    expected[to..to + length].copy_from_slice(&old.pixels[from..from + length]);
                }
            }
            }
        }
        for y in 0..old.height {
            let start = y * old.stride;
            let end = start + old.width * old.channels;
            require(
                expected[start..end] == new.pixels[start..end],
                "Raster appearance changed beyond the exact operation",
            )?;
        }
    }
    Ok(Verified {
        glyphs: a.glyphs.len(),
        target_glyphs: targets.len(),
        pixels,
        max_geometry_error_bp,
    })
}
