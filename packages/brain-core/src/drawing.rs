//! Drawing organ — external visual-motor memory (DESIGN.md §9).
//!
//! M4 core (headless): an operation-graph canvas model (strokes, layers,
//! transforms — the artifact stays editable and structurally usable),
//! deterministic stroke feature extraction, motif memory via streaming
//! clustering, aesthetic preference signals, and binding of drawing sessions
//! into the substrate as visual-spatial memory. The editor UI (blend modes,
//! stabilizer, symmetry, reference boards, canvas history, etc.) is the
//! desktop-shell milestone; it consumes exactly this model.
//!
//! All analysis is local and deterministic — no cloud, no hidden uploads.

use serde::{Deserialize, Serialize};

// --- canvas model ------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Layer {
    pub id: u64,
    pub name: String,
    pub opacity: f32,
    pub blend: BlendMode,
    pub visible: bool,
    pub parent: Option<u64>, // group layer id
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StrokePoint {
    pub x: f32,
    pub y: f32,
    pub pressure: f32, // 0..1
    pub t: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Stroke {
    pub id: u64,
    pub layer_id: u64,
    pub brush: u32,
    pub color: [u8; 4], // RGBA
    pub width: f32,
    pub points: Vec<StrokePoint>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum CanvasOp {
    Stroke(Stroke),
    AddLayer(Layer),
    GroupLayer { id: u64, parent: u64 },
    SetOpacity { layer_id: u64, opacity: f32 },
    SetBlend { layer_id: u64, blend: BlendMode },
    Transform { layer_id: u64, dx: f32, dy: f32, scale: f32, rotation: f32 },
    DeleteLayer { id: u64 },
}

/// A reference-board asset (user-provided image/video). The brain never stores
/// raw media: only a vault reference plus extracted feature summaries
/// (palette, layout, motion classes) — §5.8. Full decoding lives in the
/// media sidecar (tools/media-extract.py) or the desktop shell.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReferenceAsset {
    pub id: u64,
    pub kind: String, // "image" | "video"
    pub name: String,
    pub vault_ref: String,
    pub features: Vec<f32>,
    pub width: u32,
    pub height: u32,
    pub added: u64,
}

/// The canvas: the op graph is the source of truth (editable history, undo,
/// revision). `layers`/`strokes` are the materialized replay of layer/stroke
/// ops — deterministic by construction (pure fold in op order).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Canvas {
    pub id: u64,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub ops: Vec<CanvasOp>,
    pub layers: Vec<Layer>,
    pub strokes: Vec<Stroke>,
    pub refs: Vec<ReferenceAsset>,
    pub created: u64,
    pub updated: u64,
}

impl Canvas {
    pub fn replay_digest(&self) -> u64 {
        let mut h: u64 = 0x9e37_79b9_7f4a_7c15;
        for op in self.ops.iter() {
            for b in format!("{:?}", op).as_bytes() {
                h = (h ^ *b as u64).wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        h
    }
}

// --- stroke features (deterministic, 16 dims) --------------------------------

pub const FEATURE_DIM: usize = 16;

/// Direction histogram (8 bins) + curvature + pressure + geometry + color.
/// Pure function of the stroke — deterministic, local, no learned params.
pub fn stroke_features(s: &Stroke) -> Vec<f32> {
    let mut f = vec![0.0f32; FEATURE_DIM];
    let n = s.points.len();
    if n < 2 {
        return f;
    }
    // 8-bin direction histogram + curvature (turning angles)
    let mut bins = [0.0f32; 8];
    let mut turns = Vec::new();
    let mut prev_ang = 0.0f32;
    for i in 1..n {
        let dx = s.points[i].x - s.points[i - 1].x;
        let dy = s.points[i].y - s.points[i - 1].y;
        let d = (dx * dx + dy * dy).sqrt();
        if d < 1e-4 {
            continue;
        }
        let ang = dy.atan2(dx); // -PI..PI
        let bin = (((ang + std::f32::consts::PI) / (2.0 * std::f32::consts::PI)) * 8.0) as usize % 8;
        bins[bin] += 1.0;
        if i >= 2 {
            let mut turn = ang - prev_ang;
            while turn > std::f32::consts::PI {
                turn -= 2.0 * std::f32::consts::PI;
            }
            while turn < -std::f32::consts::PI {
                turn += 2.0 * std::f32::consts::PI;
            }
            turns.push(turn.abs());
        }
        prev_ang = ang;
    }
    let total: f32 = bins.iter().sum();
    if total > 0.0 {
        for b in bins.iter_mut() {
            *b /= total;
        }
    }
    f[0..8].copy_from_slice(&bins);

    let tmean = if turns.is_empty() {
        0.0
    } else {
        turns.iter().sum::<f32>() / turns.len() as f32
    };
    let tvar = if turns.is_empty() {
        0.0
    } else {
        turns.iter().map(|t| (t - tmean) * (t - tmean)).sum::<f32>() / turns.len() as f32
    };
    f[8] = tmean / std::f32::consts::PI;
    f[9] = tvar.sqrt() / std::f32::consts::PI;

    let pm = s.points.iter().map(|p| p.pressure).sum::<f32>() / n as f32;
    let pv = s
        .points
        .iter()
        .map(|p| (p.pressure - pm) * (p.pressure - pm))
        .sum::<f32>()
        / n as f32;
    f[10] = pm;
    f[11] = pv.sqrt();

    // path length + bbox aspect
    let mut len = 0.0f32;
    for i in 1..n {
        let dx = s.points[i].x - s.points[i - 1].x;
        let dy = s.points[i].y - s.points[i - 1].y;
        len += (dx * dx + dy * dy).sqrt();
    }
    f[12] = len.ln().max(0.0) / 8.0; // log length, normalized
    let minx = s.points.iter().map(|p| p.x).fold(f32::MAX, f32::min);
    let maxx = s.points.iter().map(|p| p.x).fold(f32::MIN, f32::max);
    let miny = s.points.iter().map(|p| p.y).fold(f32::MAX, f32::min);
    let maxy = s.points.iter().map(|p| p.y).fold(f32::MIN, f32::max);
    let (bw, bh) = ((maxx - minx).max(1e-4), (maxy - miny).max(1e-4));
    f[13] = (bw / bh).ln().abs().min(3.0) / 3.0;

    // color: hue-ish + warmth
    let r = s.color[0] as f32 / 255.0;
    let g = s.color[1] as f32 / 255.0;
    let b = s.color[2] as f32 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let hue = if delta < 1e-4 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };
    f[14] = (hue / 360.0).clamp(0.0, 1.0);
    f[15] = s.width.ln().max(0.0) / 4.0;
    f
}

/// Cosine similarity over normalized feature vectors.
pub fn features_cosine(a: &[f32], b: &[f32]) -> f32 {
    let mut na = 0.0f32;
    let mut nb = 0.0f32;
    let mut dot = 0.0f32;
    for i in 0..a.len().min(b.len()) {
        na += a[i] * a[i];
        nb += b[i] * b[i];
        dot += a[i] * b[i];
    }
    let den = na.sqrt() * nb.sqrt();
    if den < 1e-9 {
        0.0
    } else {
        dot / den
    }
}

// --- motif memory ------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Motif {
    pub id: u64,
    pub centroid: Vec<f32>,
    pub strokes: Vec<u64>,
    pub first_seen: u64,
    pub last_seen: u64,
    pub salience: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct MotifMemory {
    pub motifs: Vec<Motif>,
    pub next_id: u64,
}

impl MotifMemory {
    /// Streaming clustering: join the nearest motif (cosine ≥ MATCH) or start
    /// a new one. Returns the motif id the stroke landed in.
    pub fn ingest_stroke(&mut self, stroke_id: u64, features: &[f32], now: u64) -> u64 {
        let mut best: Option<(usize, f32)> = None;
        for (i, m) in self.motifs.iter().enumerate() {
            let c = features_cosine(&m.centroid, features);
            if best.map(|(_, bc)| c > bc).unwrap_or(true) {
                best = Some((i, c));
            }
        }
        match best {
            Some((i, c)) if c >= 0.75 => {
                let m = &mut self.motifs[i];
                let n = m.strokes.len() as f32;
                for (k, v) in m.centroid.iter_mut().enumerate() {
                    *v = (*v * n + features[k]) / (n + 1.0);
                }
                m.strokes.push(stroke_id);
                m.last_seen = now;
                m.salience = (m.salience + 0.1).min(1.0);
                m.id
            }
            _ => {
                let id = self.next_id;
                self.next_id += 1;
                self.motifs.push(Motif {
                    id,
                    centroid: features.to_vec(),
                    strokes: vec![stroke_id],
                    first_seen: now,
                    last_seen: now,
                    salience: 0.2,
                });
                id
            }
        }
    }

    pub fn top(&self, k: usize) -> Vec<&Motif> {
        let mut v: Vec<&Motif> = self.motifs.iter().collect();
        v.sort_by(|a, b| {
            b.salience
                .partial_cmp(&a.salience)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        v.into_iter().take(k).collect()
    }
}

// --- aesthetic preference signals --------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AestheticSignals {
    /// Quantized color usage: palette index (16×16×16 RGB cube) → count.
    pub palette: std::collections::HashMap<u32, u32>,
    pub width_tendency: f32,
    pub pressure_tendency: f32,
    pub stroke_count: u32,
    pub symmetry_use: u32,
    pub motif_engagement: Vec<(u64, u32)>, // motif id → strokes
}

fn quantize_color(rgba: [u8; 4]) -> u32 {
    ((rgba[0] as u32 >> 4) << 8) | ((rgba[1] as u32 >> 4) << 4) | (rgba[2] as u32 >> 4)
}

// --- the organ ---------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DrawingOrgan {
    pub canvases: Vec<Canvas>,
    pub motifs: MotifMemory,
    pub aesthetic: AestheticSignals,
    pub next_canvas_id: u64,
    pub next_layer_id: u64,
    pub next_stroke_id: u64,
    pub next_ref_id: u64,
}

impl DrawingOrgan {
    pub fn new() -> Self {
        DrawingOrgan {
            canvases: Vec::new(),
            motifs: MotifMemory::default(),
            aesthetic: AestheticSignals::default(),
            next_canvas_id: 1,
            next_layer_id: 1,
            next_stroke_id: 1,
            next_ref_id: 1,
        }
    }

    pub fn create_canvas(&mut self, name: &str, width: u32, height: u32, tick: u64) -> Canvas {
        let canvas = Canvas {
            id: self.next_canvas_id,
            name: name.to_string(),
            width,
            height,
            ops: Vec::new(),
            layers: Vec::new(),
            strokes: Vec::new(),
            refs: Vec::new(),
            created: tick,
            updated: tick,
        };
        self.next_canvas_id += 1;
        self.canvases.push(canvas.clone());
        canvas
    }

    pub fn add_layer(&mut self, canvas_id: u64, name: &str, tick: u64) -> Option<u64> {
        let canvas = self.canvases.iter_mut().find(|c| c.id == canvas_id)?;
        let layer = Layer {
            id: self.next_layer_id,
            name: name.to_string(),
            opacity: 1.0,
            blend: BlendMode::Normal,
            visible: true,
            parent: None,
        };
        self.next_layer_id += 1;
        canvas.ops.push(CanvasOp::AddLayer(layer.clone()));
        canvas.layers.push(layer.clone());
        canvas.updated = tick;
        Some(layer.id)
    }

    /// Apply a stroke: op-graph append, feature extraction, motif ingestion,
    /// aesthetic signals. Returns (stroke id, motif id, features).
    pub fn add_stroke(
        &mut self,
        canvas_id: u64,
        layer_id: u64,
        brush: u32,
        color: [u8; 4],
        width: f32,
        points: Vec<StrokePoint>,
        tick: u64,
    ) -> Option<(u64, u64, Vec<f32>)> {
        let stroke = Stroke {
            id: self.next_stroke_id,
            layer_id,
            brush,
            color,
            width,
            points,
        };
        self.next_stroke_id += 1;
        let features = stroke_features(&stroke);
        let motif_id = self.motifs.ingest_stroke(stroke.id, &features, tick);
        self.aesthetic
            .palette
            .entry(quantize_color(color))
            .and_modify(|c| *c += 1)
            .or_insert(1);
        let n = self.aesthetic.stroke_count as f32;
        self.aesthetic.width_tendency = (self.aesthetic.width_tendency * n + width) / (n + 1.0);
        self.aesthetic.pressure_tendency = (self.aesthetic.pressure_tendency * n
            + stroke.points.iter().map(|p| p.pressure).sum::<f32>() / stroke.points.len().max(1) as f32)
            / (n + 1.0);
        self.aesthetic.stroke_count += 1;
        match self.aesthetic.motif_engagement.iter_mut().find(|(m, _)| *m == motif_id) {
            Some((_, c)) => *c += 1,
            None => self.aesthetic.motif_engagement.push((motif_id, 1)),
        }
        let stroke_id = stroke.id;
        let canvas = self.canvases.iter_mut().find(|c| c.id == canvas_id)?;
        canvas.ops.push(CanvasOp::Stroke(stroke.clone()));
        canvas.strokes.push(stroke);
        canvas.updated = tick;
        Some((stroke_id, motif_id, features))
    }

    pub fn transform_layer(
        &mut self,
        canvas_id: u64,
        layer_id: u64,
        dx: f32,
        dy: f32,
        scale: f32,
        rotation: f32,
    ) -> bool {
        let Some(canvas) = self.canvases.iter_mut().find(|c| c.id == canvas_id) else {
            return false;
        };
        canvas.ops.push(CanvasOp::Transform {
            layer_id,
            dx,
            dy,
            scale,
            rotation,
        });
        true
    }

    /// Reference-board entry: vault pointer + extracted features, never raw
    /// media (§5.8). `features` come from the media sidecar or desktop shell.
    pub fn add_reference(
        &mut self,
        canvas_id: u64,
        kind: &str,
        name: &str,
        vault_ref: &str,
        features: Vec<f32>,
        width: u32,
        height: u32,
        tick: u64,
    ) -> Option<u64> {
        let asset = ReferenceAsset {
            id: self.next_ref_id,
            kind: kind.to_string(),
            name: name.to_string(),
            vault_ref: vault_ref.to_string(),
            features,
            width,
            height,
            added: tick,
        };
        self.next_ref_id += 1;
        let canvas = self.canvases.iter_mut().find(|c| c.id == canvas_id)?;
        canvas.refs.push(asset.clone());
        canvas.updated = tick;
        Some(asset.id)
    }

    pub fn digest(&self) -> u64 {
        let mut h: u64 = 0x5851_f42d_4c95_7f2d;
        for c in self.canvases.iter() {
            for b in c.replay_digest().to_le_bytes() {
                h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        for m in self.motifs.motifs.iter() {
            for b in m.id.to_le_bytes() {
                h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
            }
            for b in m.strokes.len().to_le_bytes() {
                h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
            }
            for v in m.centroid.iter() {
                for b in v.to_bits().to_le_bytes() {
                    h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
                }
            }
        }
        h
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn circle_stroke(id_seed: f32) -> Stroke {
        let mut pts = Vec::new();
        for i in 0..24 {
            let a = i as f32 / 24.0 * 2.0 * std::f32::consts::PI;
            pts.push(StrokePoint {
                x: 100.0 + id_seed + 40.0 * a.cos(),
                y: 100.0 + 40.0 * a.sin(),
                pressure: 0.6,
                t: i as u32,
            });
        }
        Stroke {
            id: 0,
            layer_id: 1,
            brush: 1,
            color: [200, 80, 40, 255],
            width: 3.0,
            points: pts,
        }
    }

    fn zigzag_stroke(seed: f32) -> Stroke {
        let mut pts = Vec::new();
        for i in 0..20 {
            pts.push(StrokePoint {
                x: 10.0 + i as f32 * 10.0 + seed,
                y: 50.0 + if i % 2 == 0 { 30.0 } else { 5.0 },
                pressure: 0.3 + 0.4 * ((i % 3) as f32 / 3.0),
                t: i as u32,
            });
        }
        Stroke {
            id: 0,
            layer_id: 1,
            brush: 2,
            color: [60, 120, 220, 255],
            width: 2.0,
            points: pts,
        }
    }

    #[test]
    fn features_are_deterministic_and_sensitive() {
        let a1 = stroke_features(&circle_stroke(0.0));
        let a2 = stroke_features(&circle_stroke(0.0));
        assert_eq!(a1, a2, "same stroke → same features");
        let z = stroke_features(&zigzag_stroke(0.0));
        assert!(
            features_cosine(&a1, &z) < 0.8,
            "different shapes → different features: {}",
            features_cosine(&a1, &z)
        );
        assert_eq!(a1.len(), FEATURE_DIM);
    }

    #[test]
    fn motifs_separate_shape_families() {
        let mut mm = MotifMemory::default();
        let mut ids = Vec::new();
        for i in 0..4 {
            ids.push(mm.ingest_stroke(i as u64 * 2, &stroke_features(&circle_stroke(i as f32 * 2.0)), 10));
            ids.push(mm.ingest_stroke(i as u64 * 2 + 1, &stroke_features(&zigzag_stroke(i as f32 * 2.0)), 10));
        }
        // Two distinct families → at least two motifs; each stroke of a family
        // should land in the same motif as its siblings.
        assert!(mm.motifs.len() >= 2, "families separated: {}", mm.motifs.len());
        let circles: Vec<u64> = ids.iter().step_by(2).copied().collect();
        assert!(circles.iter().all(|m| *m == circles[0]), "circles share a motif");
        let zigs: Vec<u64> = ids.iter().skip(1).step_by(2).copied().collect();
        assert!(zigs.iter().all(|m| *m == zigs[0]), "zigzags share a motif");
        assert_ne!(circles[0], zigs[0], "families are distinct");
    }

    #[test]
    fn op_graph_replays_deterministically() {
        let mut a = DrawingOrgan::new();
        let mut b = DrawingOrgan::new();
        let ca = a.create_canvas("Sketch", 512, 512, 0);
        let cb = b.create_canvas("Sketch", 512, 512, 0);
        let la = a.add_layer(ca.id, "Line", 0).unwrap();
        let lb = b.add_layer(cb.id, "Line", 0).unwrap();
        for i in 0..3 {
            let pts = vec![
                StrokePoint { x: i as f32 * 10.0, y: 0.0, pressure: 0.5, t: 0 },
                StrokePoint { x: i as f32 * 10.0 + 10.0, y: 10.0, pressure: 0.8, t: 1 },
            ];
            a.add_stroke(ca.id, la, 1, [10, 20, 30, 255], 2.0, pts.clone(), 5);
            b.add_stroke(cb.id, lb, 1, [10, 20, 30, 255], 2.0, pts, 5);
        }
        a.transform_layer(ca.id, la, 3.0, -2.0, 1.1, 0.1);
        b.transform_layer(cb.id, lb, 3.0, -2.0, 1.1, 0.1);
        assert_eq!(a.canvases[0].replay_digest(), b.canvases[0].replay_digest());
        assert_eq!(a.digest(), b.digest());
        assert_eq!(a.canvases[0].strokes.len(), 3);
        assert_eq!(a.canvases[0].ops.len(), 5); // 1 layer + 3 strokes + 1 transform
    }

    #[test]
    fn aesthetic_signals_track_usage() {
        let mut organ = DrawingOrgan::new();
        let c = organ.create_canvas("Palette Test", 128, 128, 0);
        let l = organ.add_layer(c.id, "P", 0).unwrap();
        let pts = vec![
            StrokePoint { x: 0.0, y: 0.0, pressure: 0.5, t: 0 },
            StrokePoint { x: 5.0, y: 5.0, pressure: 0.5, t: 1 },
        ];
        for _ in 0..3 {
            organ.add_stroke(c.id, l, 1, [255, 0, 0, 255], 4.0, pts.clone(), 1);
        }
        organ.add_stroke(c.id, l, 2, [0, 0, 255, 255], 1.0, pts, 2);
        assert_eq!(organ.aesthetic.stroke_count, 4);
        assert_eq!(organ.aesthetic.palette.len(), 2, "two quantized colors");
        assert!((organ.aesthetic.width_tendency - 3.25).abs() < 1e-5, "rolling width mean");
    }
}
