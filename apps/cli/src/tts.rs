//! TTS voice-over — renders a VoicePlan as actual audible speech (the
//! mouth's final stage, DESIGN.md §7.3). The plan decides *how* the file
//! sounds (rate, pitch, energy from brain state); this module renders it.
//!
//! Pipeline: edge-tts (neural voices; rate + pitch flags map 1:1 from the
//! plan's TtsMapping) → ffmpeg gain (energy_gain_db) → wav for playback.
//! Falls back to Windows SAPI (rate/volume only) if edge-tts is missing.
//! Network-dependent (edge-tts hits the Microsoft service) — intentionally
//! NOT in the canonical suite; verified ad-hoc.

use std::path::Path;
use std::process::Command;

use brain_core::voice::VoicePlan;

/// Pick a voice for the file: explicit flag/env wins; otherwise the
/// identity's pitch mean selects a tendency (low → warm male voice,
/// high → bright female voice) — priors, not locks.
pub fn pick_voice(plan: &VoicePlan, explicit: Option<&str>) -> String {
    if let Some(v) = explicit {
        return v.to_string();
    }
    if let Ok(v) = std::env::var("NEUROFORM_TTS_VOICE") {
        return v;
    }
    match plan.emotional_coloring.as_str() {
        "bright" => "en-US-AvaNeural".to_string(),
        "troubled" | "heavy" => "en-US-ChristopherNeural".to_string(),
        _ => {
            if plan.params.pitch < 0.45 {
                "en-US-AndrewNeural".to_string()
            } else {
                "en-US-AriaNeural".to_string()
            }
        }
    }
}

/// Edge-tts flag strings from the plan (rate multiplier → %, semitones → Hz).
fn edge_flags(plan: &VoicePlan) -> (String, String) {
    let rate_pct = ((plan.tts.rate_mult - 1.0) * 100.0).round() as i32;
    let rate = if rate_pct >= 0 { format!("+{rate_pct}%") } else { format!("{rate_pct}%") };
    let pitch_hz = (plan.tts.pitch_semitones * 2.5).round() as i32;
    let pitch = if pitch_hz >= 0 { format!("+{pitch_hz}Hz") } else { format!("{pitch_hz}Hz") };
    (rate, pitch)
}

/// Generate a spoken render of the plan. Returns the wav path.
/// `python` may be overridden via NEUROFORM_TTS_PYTHON (the interpreter
/// that has edge-tts installed).
pub fn speak(plan: &VoicePlan, voice: &str, out_stem: &Path) -> Result<String, String> {
    let python = std::env::var("NEUROFORM_TTS_PYTHON").unwrap_or_else(|_| "python".to_string());
    let mp3 = out_stem.with_extension("mp3");
    let wav = out_stem.with_extension("wav");
    let (rate, pitch) = edge_flags(plan);

    let status = Command::new(&python)
        .args(["-m", "edge_tts", "--voice", voice])
        .arg(format!("--rate={rate}")) // = form: a value like -27% must not look like an option
        .arg(format!("--pitch={pitch}"))
        .arg("--text")
        .arg(&plan.text)
        .arg("--write-media")
        .arg(&mp3)
        .status()
        .map_err(|e| format!("edge-tts launch failed: {e} (is edge-tts installed? pip install edge-tts)"))?;
    if !status.success() {
        return Err("edge-tts failed".into());
    }

    // Gain stage via ffmpeg (energy_gain_db), then wav for playback.
    let gain = plan.tts.energy_gain_db;
    let mut args = vec![
        "-y".to_string(),
        "-i".to_string(),
        mp3.to_string_lossy().into_owned(),
    ];
    if gain.abs() > 0.5 {
        args.push("-af".to_string());
        args.push(format!("volume={gain:+.1}dB"));
    }
    args.push(wav.to_string_lossy().into_owned());
    let ok = Command::new("ffmpeg").args(&args).status().map_err(|e| format!("ffmpeg failed: {e}"))?;
    if !ok.success() {
        return Err("ffmpeg conversion failed".into());
    }
    let _ = std::fs::remove_file(&mp3);
    Ok(wav.to_string_lossy().into_owned())
}

/// Play a wav synchronously via Windows SAPI/SoundPlayer.
pub fn play(wav: &str) -> Result<(), String> {
    let ps = format!(
        "(New-Object Media.SoundPlayer '{}').PlaySync()",
        wav.replace('\'', "''")
    );
    let status = Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps])
        .status()
        .map_err(|e| format!("powershell launch failed: {e}"))?;
    if !status.success() {
        return Err("playback failed".into());
    }
    Ok(())
}
