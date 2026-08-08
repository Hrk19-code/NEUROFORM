//! Face-state emitter — renders the file's live state as JSON the face app
//! (tools/face/index.html) consumes: voice plan params, affect, fatigue,
//! posture, emotional coloring. The face is expressive puppetry: everything
//! visible maps to simulated state, nothing more.

use brain_core::voice::VoicePlan;

pub fn face_state(
    plan: &VoicePlan,
    valence: f32,
    arousal: f32,
    fatigue: f32,
    posture: &str,
    curiosity: f32,
) -> serde_json::Value {
    serde_json::json!({
        "text": plan.text,
        "coloring": plan.emotional_coloring,
        "mouth": {
            "open": (0.15 + 0.85 * plan.params.energy).clamp(0.0, 1.0),
            "rate": plan.params.rate,
            "speaking": plan.params.energy > 0.2,
        },
        "expression": {
            "valence": valence,          // -1..1 → smile/frown
            "arousal": arousal,          // 0..1 → brow tension
            "fatigue": fatigue,          // 0..1 → blink rate, lid droop
            "curiosity": curiosity,      // 0..1 → gaze wander
            "brows": (arousal * 0.7 + valence.abs() * 0.3).clamp(0.0, 1.0),
        },
        "posture": posture,              // still|upright|lying|moving|transport
        "voice": {
            "pitch": plan.params.pitch,
            "warmth": plan.params.warmth,
            "brightness": plan.params.brightness,
            "breathiness": plan.params.breathiness,
        },
        "tick": 0,
    })
}
