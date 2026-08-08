//! neuroform CLI — M1 commands.
//! create / verify / tick / inspect / event / memory / retrieve / chat /
//! attach / detach / teachers / embodiment / audit / life / watch

mod teacher_http;
mod tts;
mod face_state;

use std::path::PathBuf;
use std::process::ExitCode;

use brain_core::capacity::TierName;
use brain_core::embodiment::EmbodimentPreset;
use brain_core::memory::RetrievalBudget;
use brain_core::rng::Rng;
use brain_core::Brain;

const USAGE: &str = "\
neuroform — Neuroform Brain File CLI (M1: core life + embodiment)

USAGE:
  neuroform create <path> [--tier prototype|standard|advanced|experimental]
                          [--embodiment male|female|custom|mixed|non-binary|user-defined]
                          [--seed N] [--passphrase P]
  neuroform verify <path> [--passphrase P]
  neuroform tick <path> --ticks N [--passphrase P] [--save] [--snapshot out.json]
  neuroform inspect <path> [--passphrase P] [--json] [--out file.json]
  neuroform event <path> --text \"...\" [--valence V] [--arousal A] [--source user|system|self|peer|teacher] [--save]
  neuroform memory <path> [--top N] [--passphrase P]
  neuroform retrieve <path> --query \"...\" [--k N] [--passphrase P]
  neuroform chat <path> \"message\" [--valence V] [--teacher NAME] [--passphrase P]
  neuroform attach <path> --teacher <name> [--passphrase P]
  neuroform detach <path> --teacher <name> [--passphrase P]
  neuroform teachers <path> [--passphrase P]
  neuroform llm profiles|save|active|remove|test [--name N] [--endpoint URL] [--model M] [--key K] [--mock] [--active]
  neuroform embodiment <path> [--set male|female|custom|mixed|non-binary|user-defined] [--save] [--passphrase P]
  neuroform audit <path> [--trigger NAME] [--passphrase P]
  neuroform sleep <path> [--cycles 1] [--passphrase P] [--save]
  neuroform dreams <path> [--top N] [--passphrase P]
  neuroform doc new <path> --title T [--mode prose|journal|worldbuilding|lorebook|markdown] [--save] [--passphrase P]
  neuroform doc write <path> --doc N --text \"...\" [--kind para|heading|quote|list|scene-card|entity-card|beat|note] [--save] [--passphrase P]
  neuroform doc style <path> --doc N [--passphrase P]
  neuroform doc ledger <path> [--passphrase P]
  neuroform doc list <path> [--passphrase P]
  neuroform doc assist <path> --doc N \"instruction\" [--teacher NAME] [--passphrase P]
  neuroform draw new <path> --name N [--w 512] [--h 512] [--save] [--passphrase P]
  neuroform draw layer <path> --canvas N --name L [--save] [--passphrase P]
  neuroform draw stroke <path> --canvas N --layer M [--brush 1] [--color RRGGBB] [--width 3] --points \"x,y,p;x,y,p\" [--save] [--passphrase P]
  neuroform draw ref <path> --canvas N --name R --kind image|video --vault-ref FILE [--save] [--passphrase P]
  neuroform draw motifs <path> [--passphrase P]
  neuroform draw canvases <path> [--passphrase P]
  neuroform draw assist <path> --canvas N \"instruction\" [--teacher NAME] [--passphrase P]
  neuroform autonomy <path> [--enable|--disable] [--quiet-start H] [--quiet-end H] [--status] [--save] [--passphrase P]
  neuroform voice status <path> [--passphrase P]
  neuroform voice speak <path> --text \"...\" [--toward N] [--save] [--passphrase P]
  neuroform voice hear <path> --label L --audio FILE.wav [--consent] [--salience 0.7] [--save] [--passphrase P]
  neuroform voice consent <path> [--on|--off] [--id N --on|--off] [--save] [--passphrase P]
  neuroform voice override <path> --param pitch|rate|energy|breathiness|warmth|brightness|roughness --value V --reason \"...\" [--save] [--passphrase P]
  neuroform voice clear <path> --param P [--save] [--passphrase P]
  neuroform life <path> [--days 30] [--seed-stream 42] [--teacher-a NAME] [--teacher-b NAME]
                        [--detach-day 21] [--reattach-day 26] [--sleep-every N] [--autonomy] [--no-autosave] [--passphrase P]
  neuroform watch <path> --ticks N [--interval 100] [--passphrase P]

EXAMPLES:
  neuroform create a.brain --tier standard --embodiment female --seed 42
  neuroform event a.brain --text \"the garden is beautiful today\" --valence 0.6 --save
  neuroform attach a.brain --teacher amber
  neuroform chat a.brain \"hello, what do you remember?\"
  neuroform life a.brain --days 30
";

struct Args {
    command: String,
    positional: Vec<String>,
    flags: Vec<(String, String)>,
}

fn parse_args(argv: &[String]) -> Result<Args, String> {
    if argv.is_empty() {
        return Err(USAGE.to_string());
    }
    let command = argv[0].clone();
    let mut positional = Vec::new();
    let mut flags = Vec::new();
    let mut i = 1;
    while i < argv.len() {
        let a = &argv[i];
        if let Some(rest) = a.strip_prefix("--") {
            if rest.is_empty() {
                return Err("bad flag".into());
            }
            if i + 1 < argv.len() && !argv[i + 1].starts_with("--") {
                flags.push((rest.to_string(), argv[i + 1].clone()));
                i += 2;
            } else {
                flags.push((rest.to_string(), String::new()));
                i += 1;
            }
        } else {
            positional.push(a.clone());
            i += 1;
        }
    }
    Ok(Args {
        command,
        positional,
        flags,
    })
}

fn flag<'a>(args: &'a Args, name: &str) -> Option<&'a str> {
    args.flags
        .iter()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.as_str())
}

fn fnum(args: &Args, name: &str, default: f32) -> f32 {
    flag(args, name)
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

fn fint(args: &Args, name: &str, default: u64) -> u64 {
    flag(args, name)
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

fn fval(args: &Args, name: &str, default: f32) -> f32 {
    flag(args, name)
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let args = match parse_args(&argv) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };
    let result = match args.command.as_str() {
        "create" => cmd_create(&args),
        "verify" => cmd_verify(&args),
        "tick" => cmd_tick(&args),
        "inspect" => cmd_inspect(&args),
        "grow" => cmd_grow(&args),
        "event" => cmd_event(&args),
        "memory" => cmd_memory(&args),
        "retrieve" => cmd_retrieve(&args),
        "chat" => cmd_chat(&args),
        "attach" => cmd_attach(&args),
        "detach" => cmd_detach(&args),
        "teachers" => cmd_teachers(&args),
        "llm" => cmd_llm(&args),
        "embodiment" => cmd_embodiment(&args),
        "audit" => cmd_audit(&args),
        "sleep" => cmd_sleep(&args),
        "dreams" => cmd_dreams(&args),
        "doc" => cmd_doc(&args),
        "draw" => cmd_draw(&args),
        "autonomy" => cmd_autonomy(&args),
        "voice" => cmd_voice(&args),
        "body" => cmd_body(&args),
        "net" => cmd_net(&args),
        "physics" => cmd_physics(&args),
        "serve" => cmd_serve(&args),
        "expose" => cmd_expose(&args),
        "life" => cmd_life(&args),
        "watch" => cmd_watch(&args),
        "help" | "--help" | "-h" => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        other => {
            eprintln!("unknown command: {other}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn load_brain(args: &Args) -> Result<Brain, String> {
    let path = args
        .positional
        .first()
        .ok_or_else(|| "a path argument is required".to_string())?;
    Brain::load(&PathBuf::from(path), flag(args, "passphrase")).map_err(|e| e.to_string())
}

fn load_brain_at(path: &str) -> Result<Brain, String> {
    Brain::load(&PathBuf::from(path), None).map_err(|e| e.to_string())
}

fn maybe_save(brain: &mut Brain, args: &Args) -> Result<(), String> {
    if flag(args, "save").is_some() {
        let path = args
            .positional
            .first()
            .ok_or_else(|| "no path".to_string())?;
        let bytes = brain
            .save(&PathBuf::from(path), flag(args, "passphrase"))
            .map_err(|e| e.to_string())?;
        println!("  saved {bytes} bytes");
    }
    Ok(())
}

fn cmd_create(args: &Args) -> Result<(), String> {
    let path = args
        .positional
        .first()
        .ok_or_else(|| "create requires a path".to_string())?;
    let tier_name = match flag(args, "tier") {
        Some(t) => TierName::from_str(t).ok_or_else(|| format!("unknown tier: {t}"))?,
        None => TierName::Standard,
    };
    let preset = match flag(args, "embodiment") {
        Some(e) => EmbodimentPreset::from_str(e).ok_or_else(|| format!("unknown embodiment: {e}"))?,
        None => EmbodimentPreset::Custom,
    };
    // Chromosomes are the ground truth: if given, they override the preset's
    // implied karyotype and select the gonadal program.
    let karyotype = flag(args, "chromosomes")
        .map(|k| brain_core::embodiment::Karyotype::from_str(k).ok_or_else(|| format!("unknown karyotype: {k}")))
        .transpose()?;
    // Feature encoder — chosen HERE and only here. Immutable for the file's
    // life (BUILD-THE-BODY Phase 0): handcrafted (default, always works),
    // onnx (frozen pretrained vision model — P0 not built yet), or jepa
    // (frozen V-JEPA 2 video encoder; requires the exported ONNX backbone).
    let encoder = flag(args, "encoder").unwrap_or("handcrafted").to_string();
    let encoder_sha = match encoder.as_str() {
        "handcrafted" => None,
        "onnx" => {
            let m = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../models/dinov2-small/onnx/model.onnx");
            if !m.exists() {
                return Err(format!(
                    "encoder \"onnx\": {} missing — download it (BUILD-THE-BODY P0)",
                    m.display()
                ));
            }
            Some(DINO2_SHA.to_string())
        }
        "jepa" => {
            let onnx = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../models/vjepa2-vitl-fpc64-256/vjepa2_backbone.onnx");
            if !onnx.exists() {
                return Err(format!(
                    "encoder \"jepa\": {} missing — run tools/export_vjepa2_onnx.py first",
                    onnx.display()
                ));
            }
            Some("25466aef85727d16546c6cf8c99f12fcfad9cbca8225d45f23685e2e025b786b".to_string())
        }
        other => return Err(format!("unknown encoder: {other} (handcrafted|onnx|jepa)")),
    };
    let seed: u64 = fint(args, "seed", 42);
    let passphrase = flag(args, "passphrase");
    let mut brain = if let Some(k) = karyotype {
        Brain::create_with_encoder_karyotype(tier_name, seed, k, &encoder, encoder_sha.clone())
    } else {
        Brain::create_with_encoder(tier_name, seed, preset, &encoder, encoder_sha.clone())
    };
    let bytes = brain
        .save(&PathBuf::from(path), passphrase)
        .map_err(|e| e.to_string())?;
    let mode = if passphrase.is_some() { "passphrase" } else { "plain-dev (no passphrase)" };
    println!(
        "created {} (tier {}, encoder {}, embodiment {}, chromosomes {}, seed {}, {} bytes, key mode: {})",
        path,
        brain.tier.name,
        brain.encoder(),
        preset.as_str(),
        brain.embodiment.karyotype.as_str(),
        seed,
        bytes,
        mode
    );
    println!("  brain id: {}", brain.brain_id);
    println!("  digest:   {:016x}", brain.digest());
    println!(
        "  modulator deltas: da {:+.3} ne {:+.3} cort {:+.3} oxt {:+.3} avp {:+.3}",
        brain.embodiment.mod_deltas[0],
        brain.embodiment.mod_deltas[2],
        brain.embodiment.mod_deltas[5],
        brain.embodiment.mod_deltas[6],
        brain.embodiment.mod_deltas[7],
    );
    Ok(())
}

/// Frozen DINOv2-small ONNX runtime file (models/dinov2-small/onnx/model.onnx).
const DINO2_SHA: &str = "f22797eabf810a75e41de68d378541ebea372122b25c4ce3ef25ff618250c20a";

/// Streaming sha256 hex of a file (no full-file RAM spike — models are up
/// to 1.3GB). Used by `verify` for runtime model integrity.
fn hash_file(path: &std::path::Path) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    let f = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut r = std::io::BufReader::with_capacity(1 << 20, f);
    let mut h = Sha256::new();
    let mut buf = [0u8; 1 << 16];
    loop {
        use std::io::Read;
        let n = r.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(h.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

// ---------- LLM endpoint manager (Phase L1/L2: llm.json profiles) ----------

#[derive(serde::Serialize, serde::Deserialize, Clone, Default)]
struct LlmProfile {
    endpoint: String,
    model: String,
    #[serde(default)]
    key: String,
    #[serde(default)]
    temperature: f32,
    #[serde(default)]
    max_tokens: u32,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct LlmStore {
    #[serde(default)]
    active: String,
    #[serde(default)]
    profiles: std::collections::HashMap<String, LlmProfile>,
}

/// Sidecar credential store — NEVER inside the .brain, never logged.
fn llm_path() -> std::path::PathBuf {
    std::env::current_dir().unwrap_or_default().join("llm.json")
}

fn llm_load() -> LlmStore {
    std::fs::read(llm_path())
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

fn llm_save(store: &LlmStore) -> Result<(), String> {
    let bytes = serde_json::to_string_pretty(store).map_err(|e| e.to_string())?;
    std::fs::write(llm_path(), bytes).map_err(|e| e.to_string())
}

/// Phase L: teacher resolution — explicit --teacher wins; otherwise the
/// ACTIVE llm.json profile (endpoint/key/model) is attached; else none
/// (honest empty state: organs say "no LLM attached").
fn attach_profile_or_teacher(brain: &mut Brain, args: &Args, explicit: Option<&str>) -> Result<(), String> {
    if let Some(t) = explicit {
        attach_teacher_to(brain, t, args)?;
    } else {
        let store = llm_load();
        if !store.active.is_empty() {
            if let Some(p) = store.profiles.get(&store.active) {
                let t = teacher_http::HttpTeacher::new(&p.endpoint, &p.key, &p.model);
                brain.attach_custom_teacher(Box::new(t));
            }
        }
    }
    Ok(())
}

fn cmd_llm(args: &Args) -> Result<(), String> {
    // positional starts at the subcommand: sub=[0] ("llm" is the command).
    let sub = args
        .positional
        .first()
        .cloned()
        .unwrap_or_else(|| "profiles".to_string());
    let mut store = llm_load();
    match sub.as_str() {
        "profiles" => {
            let mut names: Vec<&String> = store.profiles.keys().collect();
            names.sort();
            for n in names {
                let p = &store.profiles[n];
                let mark = if *n == store.active { "*" } else { " " };
                let host = p.endpoint.replace("https://", "").replace("http://", "");
                let key = if p.key.is_empty() { "no key" } else { "key set" };
                println!("  {mark} {n:<16} {:<20} {} ({key})", p.model, host);
            }
            println!(
                "active: {}",
                if store.active.is_empty() { "(none)" } else { &store.active }
            );
            if store.profiles.is_empty() {
                println!("  (no profiles — llm save --name N --endpoint URL --model M --key K [--active])");
            }
        }
        "save" => {
            let name = flag(args, "name").ok_or_else(|| "llm save requires --name".to_string())?;
            let endpoint = flag(args, "endpoint")
                .ok_or_else(|| "llm save requires --endpoint (https://host/v1)".to_string())?;
            let model = flag(args, "model").ok_or_else(|| "llm save requires --model".to_string())?;
            let key = flag(args, "key").unwrap_or_default().to_string();
            let temperature = fnum(args, "temperature", 0.8);
            let max_tokens = fint(args, "max-tokens", 300).max(1) as u32;
            store.profiles.insert(
                name.to_string(),
                LlmProfile {
                    endpoint: endpoint.to_string(),
                    model: model.to_string(),
                    key,
                    temperature,
                    max_tokens,
                },
            );
            if flag(args, "active").is_some() {
                store.active = name.to_string();
            }
            llm_save(&store)?;
            println!("saved profile \"{name}\" (key in llm.json sidecar — never in the brain, never logged)");
        }
        "active" => {
            if let Some(n) = flag(args, "name") {
                if !store.profiles.contains_key(n) {
                    return Err(format!("no such profile: {n}"));
                }
                store.active = n.to_string();
                llm_save(&store)?;
                println!("active profile: {n}");
            } else {
                println!("active: {}", if store.active.is_empty() { "(none)" } else { &store.active });
            }
        }
        "remove" => {
            let name = flag(args, "name").ok_or_else(|| "llm remove requires --name".to_string())?;
            if store.profiles.remove(name).is_none() {
                return Err(format!("no such profile: {name}"));
            }
            if store.active == name {
                store.active = String::new();
            }
            llm_save(&store)?;
            println!("removed profile \"{name}\"");
        }
        "test" => {
            // One tiny real completion through the FULL state-modulated bridge
            // (a scratch file's utter() — the same code path chat uses), with
            // latency. --mock proves the pipeline offline; otherwise the named
            // or active profile is used. Errors surface verbatim.
            let mut brain = Brain::create(brain_core::capacity::TierName::Standard, 42);
            if flag(args, "mock").is_some() {
                brain.attach_teacher("amber");
            } else {
                let prof = match flag(args, "name") {
                    Some(n) => store
                        .profiles
                        .get(n)
                        .cloned()
                        .ok_or_else(|| format!("no such profile: {n}"))?,
                    None => {
                        let a = store.active.clone();
                        store
                            .profiles
                            .get(&a)
                            .cloned()
                            .ok_or_else(|| "no active profile — save one first (llm save ... --active)".to_string())?
                    }
                };
                let t = teacher_http::HttpTeacher::new(&prof.endpoint, &prof.key, &prof.model);
                brain.attach_custom_teacher(Box::new(t));
            }
            let t0 = std::time::Instant::now();
            let reply = brain.utter("speak", "ping — reply with one word.");
            let ms = t0.elapsed().as_millis();
            let ok = !reply.contains("teacher error");
            println!("llm test: {} ({ms} ms)", if ok { "OK" } else { "FAIL" });
            println!("  reply: {}", reply.chars().take(140).collect::<String>());
            if !ok {
                println!("  (the error above is verbatim from the endpoint — check key/endpoint/model)");
            }
        }
        other => return Err(format!("unknown llm subcommand: {other} (profiles|save|active|remove|test)")),
    }
    Ok(())
}

fn cmd_verify(args: &Args) -> Result<(), String> {
    let path = args
        .positional
        .first()
        .ok_or_else(|| "verify requires a path".to_string())?;
    let report = brain_core::format::verify_file(&PathBuf::from(path), flag(args, "passphrase"))
        .map_err(|e| e.to_string())?;
    println!(
        "verify {} — format {}, version {}, brain {}",
        if report.ok { "PASS" } else { "FAIL" },
        report.manifest.format,
        report.manifest.version,
        report.manifest.brain_id
    );
    println!("  envelope: {}", report.envelope_mode);
    println!(
        "  seed: {}   tier: {}   encoder: {}   created: {}",
        report.manifest.seed,
        report.manifest.capacity_tier,
        if report.manifest.encoder.is_empty() {
            "handcrafted"
        } else {
            &report.manifest.encoder
        },
        report.manifest.created_at
    );
    if let Some(sha) = &report.manifest.encoder_model_sha256 {
        println!("  encoder model sha256: {sha}");
    }
    // Runtime model integrity: the manifest records the model's IDENTITY
    // (source checkpoint hash); this check proves the machine's runtime
    // file is the known-good one (swapped/corrupt model = honest FAIL).
    // Load-time verification is deliberately deferred (hashing 1.3GB on
    // every load is too costly); verify-time covers integrity explicitly.
    let runtime_sha = match report.manifest.encoder.as_str() {
        "jepa" => {
            let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../models/vjepa2-vitl-fpc64-256/vjepa2_backbone.onnx");
            hash_file(&p).ok()
        }
        "onnx" => {
            let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../models/dinov2-small/onnx/model.onnx");
            hash_file(&p).ok()
        }
        _ => None,
    };
    if let Some(actual) = runtime_sha {
        let trusted = match report.manifest.encoder.as_str() {
            "jepa" => "a08f2f68bd6f0f576c829ecd4c20b5013ffc8c0fb378b715c6d51751ecac315c",
            "onnx" => DINO2_SHA,
            _ => "",
        };
        let ok = actual == trusted && !trusted.is_empty();
        println!(
            "  runtime model: {} ({})",
            if ok { "trusted" } else { "MISMATCH — not the known model" },
            &actual[..16]
        );
    }
    for (id, ok, detail) in &report.shard_checks {
        println!("  shard {}: {} ({})", id, if *ok { "ok" } else { "CORRUPT" }, detail);
    }
    if !report.corrupt.is_empty() {
        return Err(format!(
            "{} corrupt shard(s): {}",
            report.corrupt.len(),
            report.corrupt.join(", ")
        ));
    }
    Ok(())
}

fn cmd_tick(args: &Args) -> Result<(), String> {
    let path = args
        .positional
        .first()
        .ok_or_else(|| "tick requires a path".to_string())?;
    let ticks: u64 = flag(args, "ticks")
        .ok_or_else(|| "tick requires --ticks N".to_string())?
        .parse()
        .map_err(|_| "bad --ticks".to_string())?;
    let mut brain = load_brain(args)?;
    let digest_before = brain.digest();
    let t0 = std::time::Instant::now();
    brain.run_ticks(ticks);
    let elapsed = t0.elapsed();
    let digest_after = brain.digest();
    println!(
        "ticked {} ticks in {:.3}s ({:.0} ticks/s); sim_time now {}",
        ticks,
        elapsed.as_secs_f64(),
        ticks as f64 / elapsed.as_secs_f64(),
        brain.state.sim_time
    );
    println!("  digest before: {:016x}", digest_before);
    println!("  digest after:  {:016x}", digest_after);
    if flag(args, "save").is_some() {
        let bytes = brain
            .save(&PathBuf::from(path), flag(args, "passphrase"))
            .map_err(|e| e.to_string())?;
        println!("  saved {bytes} bytes");
    }
    if let Some(out) = flag(args, "snapshot") {
        let json = serde_json::to_string_pretty(&brain.snapshot_json()).map_err(|e| e.to_string())?;
        std::fs::write(out, json).map_err(|e| e.to_string())?;
        println!("  snapshot written to {out}");
    }
    Ok(())
}

fn cmd_grow(args: &Args) -> Result<(), String> {
    let mut brain = load_brain(args)?;
    brain.grow()?;
    maybe_save(&mut brain, args)?;
    Ok(())
}

// --- desktop shell: `neuroform serve` — the tabbed UI host ------------------
// A minimal localhost HTTP server. Every /api/run request spawns THIS exe
// with the given argv and returns its output as JSON — the UI is a thin
// shell over the real, tested CLI. Nothing new is implemented here.

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            let h = |c: u8| -> Option<u8> {
                match c {
                    b'0'..=b'9' => Some(c - b'0'),
                    b'a'..=b'f' => Some(c - b'a' + 10),
                    b'A'..=b'F' => Some(c - b'A' + 10),
                    _ => None,
                }
            };
            if let (Some(hi), Some(lo)) = (h(b[i + 1]), h(b[i + 2])) {
                out.push(hi * 16 + lo);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn http_response(stream: &mut std::net::TcpStream, status: &str, ctype: &str, body: &[u8]) {
    use std::io::Write;
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(body);
    let _ = stream.flush();
}

/// Parse a `k=v&k2=v2` query string (values already percent-decoded by caller).
fn query_param(query: &str, key: &str) -> Option<String> {
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            if k == key && !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

/// Sidecar file guard: only relative paths inside the serve working dir.
fn sidecar_ok(rel: &str) -> bool {
    !rel.is_empty()
        && !rel.contains("..")
        && !rel.contains('\\')
        && !rel.starts_with('/')
        && !rel.contains(':')
}

fn cmd_serve(args: &Args) -> Result<(), String> {
    use std::io::Read;
    let ui_dir = flag(args, "ui").unwrap_or("tools/desktop");
    let port = flag(args, "port").unwrap_or("8123");
    let addr = format!("127.0.0.1:{port}");
    let listener = std::net::TcpListener::bind(&addr).map_err(|e| format!("bind {addr}: {e}"))?;
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    println!("neuroform desktop → http://{addr}  (ui: {ui_dir})  [Ctrl-C to stop]");
    for stream in listener.incoming() {
        let mut s = match stream {
            Ok(s) => s,
            Err(_) => continue,
        };
        let _ = s.set_read_timeout(Some(std::time::Duration::from_secs(10)));
        let mut buf = [0u8; 16384];
        let n = match s.read(&mut buf) {
            Ok(n) if n > 0 => n,
            _ => continue,
        };
        let req = String::from_utf8_lossy(&buf[..n]).into_owned();
        let first = req.lines().next().unwrap_or("").to_string();
        let mut parts = first.split_whitespace();
        let method = parts.next().unwrap_or("");
        let target = parts.next().unwrap_or("/");
        if method != "GET" {
            http_response(&mut s, "405 Method Not Allowed", "text/plain", b"GET only");
            continue;
        }
        let (route, query) = match target.split_once('?') {
            Some((r, q)) => (r, q),
            None => (target, ""),
        };
        if route == "/api/run" {
            // ?args=<argv, args separated by U+001F, percent-encoded>
            let query = query.strip_prefix("args=").unwrap_or(query);
            let argv_raw = percent_decode(query);
            let argv: Vec<String> = argv_raw.split('\u{1f}').filter(|a| !a.is_empty()).map(String::from).collect();
            match std::process::Command::new(&exe).args(&argv).output() {
                Ok(out) => {
                    let payload = serde_json::json!({
                        "ok": out.status.success(),
                        "code": out.status.code(),
                        "stdout": String::from_utf8_lossy(&out.stdout).into_owned(),
                        "stderr": String::from_utf8_lossy(&out.stderr).into_owned(),
                    });
                    http_response(&mut s, "200 OK", "application/json", payload.to_string().as_bytes());
                }
                Err(e) => {
                    let payload = serde_json::json!({ "ok": false, "code": -1, "stdout": "", "stderr": format!("spawn failed: {e}") });
                    http_response(&mut s, "200 OK", "application/json", payload.to_string().as_bytes());
                }
            }
        } else if route == "/api/state" {
            // In-process brain state — no process spawn (the 5s UI refresh
            // was spawning the CLI every poll; this is the cheap path).
            let path = query_param(query, "path").unwrap_or_default();
            if path.is_empty() {
                let p = serde_json::json!({ "ok": false, "stderr": "api/state requires ?path=<brain file>" });
                http_response(&mut s, "200 OK", "application/json", p.to_string().as_bytes());
                continue;
            }
            match brain_core::brain::Brain::load(std::path::Path::new(&path), None) {
                Ok(b) => {
                    let mut st = b.snapshot_json();
                    if let Some(obj) = st.as_object_mut() {
                        obj.insert("digest".to_string(), serde_json::json!(format!("{:016x}", b.digest())));
                        obj.insert("encoder".to_string(), serde_json::json!(b.encoder()));
                    }
                    let p = serde_json::json!({ "ok": true, "state": st });
                    http_response(&mut s, "200 OK", "application/json", p.to_string().as_bytes());
                }
                Err(e) => {
                    let p = serde_json::json!({ "ok": false, "stderr": format!("{e}") });
                    http_response(&mut s, "200 OK", "application/json", p.to_string().as_bytes());
                }
            }
        } else if route == "/api/file/read" {
            let rel = percent_decode(query_param(query, "path").unwrap_or_default().as_str());
            if !sidecar_ok(&rel) {
                let p = serde_json::json!({ "ok": false, "stderr": "bad path" });
                http_response(&mut s, "200 OK", "application/json", p.to_string().as_bytes());
                continue;
            }
            let base = std::env::current_dir().unwrap_or_default();
            match std::fs::read_to_string(base.join(&rel)) {
                Ok(content) => {
                    let p = serde_json::json!({ "ok": true, "content": content });
                    http_response(&mut s, "200 OK", "application/json", p.to_string().as_bytes());
                }
                Err(e) => {
                    let p = serde_json::json!({ "ok": false, "stderr": format!("{e}") });
                    http_response(&mut s, "200 OK", "application/json", p.to_string().as_bytes());
                }
            }
        } else if route == "/api/file/write" {
            let rel = percent_decode(query_param(query, "path").unwrap_or_default().as_str());
            let content = percent_decode(query_param(query, "content").unwrap_or_default().as_str());
            if !sidecar_ok(&rel) || content.len() > 16 * 1024 * 1024 {
                let p = serde_json::json!({ "ok": false, "stderr": "bad path or content too large (>16MB)" });
                http_response(&mut s, "200 OK", "application/json", p.to_string().as_bytes());
                continue;
            }
            let base = std::env::current_dir().unwrap_or_default();
            let full = base.join(&rel);
            if let Some(parent) = full.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            match std::fs::write(&full, content.as_bytes()) {
                Ok(()) => {
                    let p = serde_json::json!({ "ok": true, "bytes": content.len() });
                    http_response(&mut s, "200 OK", "application/json", p.to_string().as_bytes());
                }
                Err(e) => {
                    let p = serde_json::json!({ "ok": false, "stderr": format!("{e}") });
                    http_response(&mut s, "200 OK", "application/json", p.to_string().as_bytes());
                }
            }
        } else if route == "/api/llm/status" {
            // Profile list for the header pill — in-process read of llm.json,
            // no spawn, no key material in the response.
            let store = llm_load();
            let mut profiles: Vec<serde_json::Value> = store
                .profiles
                .iter()
                .map(|(n, p)| serde_json::json!({ "name": n, "model": p.model, "endpoint": p.endpoint }))
                .collect();
            profiles.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
            let p = serde_json::json!({ "ok": true, "active": store.active, "profiles": profiles });
            http_response(&mut s, "200 OK", "application/json", p.to_string().as_bytes());
        } else {
            // static file from the ui dir (path-traversal guarded)
            let rel = if route == "/" { "index.html" } else { route.trim_start_matches('/') };
            if rel.contains("..") || rel.contains('\\') {
                http_response(&mut s, "403 Forbidden", "text/plain", b"forbidden");
                continue;
            }
            let path = std::path::Path::new(&ui_dir).join(rel);
            match std::fs::read(&path) {
                Ok(body) => {
                    let ctype = match path.extension().and_then(|e| e.to_str()) {
                        Some("html") => "text/html; charset=utf-8",
                        Some("js") => "text/javascript",
                        Some("css") => "text/css",
                        Some("png") => "image/png",
                        Some("svg") => "image/svg+xml",
                        Some("json") => "application/json",
                        _ => "application/octet-stream",
                    };
                    http_response(&mut s, "200 OK", ctype, &body);
                }
                Err(_) => http_response(&mut s, "404 Not Found", "text/plain", b"not found"),
            }
        }
    }
    Ok(())
}

fn cmd_inspect(args: &Args) -> Result<(), String> {
    let brain = load_brain(args)?;
    if let Some(out) = flag(args, "out") {
        let json = serde_json::to_string_pretty(&brain.snapshot_json()).map_err(|e| e.to_string())?;
        std::fs::write(out, json).map_err(|e| e.to_string())?;
        println!("snapshot written to {out}");
        return Ok(());
    }
    if flag(args, "json").is_some() {
        println!(
            "{}",
            serde_json::to_string_pretty(&brain.snapshot_json()).map_err(|e| e.to_string())?
        );
        return Ok(());
    }
    let s = &brain.state;
    println!("brain {} (tier {}, embodiment {})", brain.brain_id, brain.tier.name, brain.embodiment.preset);
    println!("  sim_time {} ticks ({} sim-days @10Hz)", s.sim_time, s.sim_time / 864_000);
    println!("  digest {:016x}", brain.digest());
    println!(
        "  affect:      valence {:.3} arousal {:.3} dominance {:.3} warmth {:.3}",
        s.affect()[0], s.affect()[1], s.affect()[2], s.affect()[3]
    );
    println!(
        "  vigilance:   energy {:.3} attention {:.3} alertness {:.3} fatigue {:.3}",
        s.vigilance()[0], s.vigilance()[1], s.vigilance()[2], s.vigilance()[3]
    );
    println!(
        "  stress:      load {:.3} regulation {:.3} saturation {:.3}",
        s.stress()[0], s.stress()[1], s.stress()[2]
    );
    println!(
        "  memory:      {} traces ({} pruned), {} semantic nodes, {} dropped events",
        brain.episodic.traces.len(),
        brain.episodic.pruned_count,
        brain.semantic.nodes.len(),
        brain.dropped_events
    );
    println!(
        "  sleep:       pressure {:.3}, emotional load {:.3}, {} dreams, {} sleep reports",
        brain.sleep.pressure,
        brain.sleep.emotional_load,
        brain.dreams.len(),
        brain.sleep_reports.len()
    );
    println!(
        "  writing:     {} documents, {} blocks, {} entities, {} continuity flags, {} preference signals",
        brain.writing.documents.len(),
        brain.writing.documents.iter().map(|d| d.blocks.len()).sum::<usize>(),
        brain.writing.ledger.entities.len(),
        brain.writing.ledger.flags.len(),
        brain.writing.preference_signals.len()
    );
    println!(
        "  drawing:     {} canvases, {} strokes, {} motifs, {} palette colors",
        brain.drawing.canvases.len(),
        brain.drawing.canvases.iter().map(|c| c.strokes.len()).sum::<usize>(),
        brain.drawing.motifs.motifs.len(),
        brain.drawing.aesthetic.palette.len()
    );
    println!(
        "  autonomy:    enabled {}, quiet {}:00–{}:00, initiatives logged {}",
        brain.autonomy.enabled,
        brain.autonomy.quiet_start_hour,
        brain.autonomy.quiet_end_hour,
        brain.autonomy.total
    );
    println!(
        "  voice:       pitch {:.2}, heard {}, gate {}, mimicry {} use(s)/{} refused, {} override(s)",
        brain.voice.identity.pitch_mean,
        brain.voice.heard.len(),
        if brain.voice.voice_learning_enabled { "ON" } else { "OFF" },
        brain.voice.memory.mimicry_uses,
        brain.voice.memory.refused_mimicry,
        brain.voice.overrides.len()
    );
    println!(
        "  teacher:     {} ({} tokens used)",
        brain.teacher_name.as_deref().unwrap_or("none"),
        brain.tokens_used
    );
    println!(
        "  capacity:    {:.1}% of {} bytes used ({})",
        brain.capacity.fullness() * 100.0,
        brain.capacity.total_budget,
        brain.capacity.total_bytes
    );
    Ok(())
}

fn cmd_event(args: &Args) -> Result<(), String> {
    let text = flag(args, "text").ok_or_else(|| "event requires --text".to_string())?;
    let mut brain = load_brain(args)?;
    let source = flag(args, "source").unwrap_or("user");
    let ok = brain.ingest_text(text, fnum(args, "valence", 0.0), fnum(args, "arousal", 0.3), source);
    brain.run_ticks(310); // cross the bind window so the event binds
    println!(
        "ingested event (accepted: {ok}): \"{}\" (valence {}, source {})",
        text.chars().take(80).collect::<String>(),
        fnum(args, "valence", 0.0),
        source
    );
    println!(
        "  traces now: {} | valence now: {:.3}",
        brain.episodic.traces.len(),
        brain.state.affect()[0]
    );
    maybe_save(&mut brain, args)
}

fn cmd_memory(args: &Args) -> Result<(), String> {
    let brain = load_brain(args)?;
    let top = fint(args, "top", 20) as usize;
    println!("episodic traces ({} total, {} pruned):", brain.episodic.traces.len(), brain.episodic.pruned_count);
    for t in brain.episodic.traces.iter().rev().take(top) {
        let kw: String = t.keywords.join(" ");
        let kw = if kw.is_empty() { "-".into() } else { kw.chars().take(60).collect::<String>() };
        println!(
            "  #{:<6} t={:<10} sal={:.3} str={:.3} src={:<7} stream={:<4} [{}]",
            t.id,
            t.sim_time,
            t.salience,
            t.strength,
            t.source,
            t.stream.as_str(),
            kw
        );
    }
    println!("semantic nodes ({}):", brain.semantic.nodes.len());
    for n in brain.semantic.nodes.iter().take(10) {
        println!(
            "  #{} belief={:.3} episodes={} label={}",
            n.id,
            n.belief,
            n.source_episodes.len(),
            n.label.chars().take(40).collect::<String>()
        );
    }
    Ok(())
}

fn cmd_retrieve(args: &Args) -> Result<(), String> {
    let query = flag(args, "query").ok_or_else(|| "retrieve requires --query".to_string())?;
    let brain = load_brain(args)?;
    let k = fint(args, "k", 5) as usize;
    let budget = RetrievalBudget {
        k_traces: k,
        k_nodes: k,
        token_cap: 4000,
    };
    let q = brain.query_embedding(query);
    let (traces, nodes, tokens, truncated) = brain.retrieve(&q, &budget);
    println!("query: \"{query}\" (tokens {tokens}{})", if truncated { ", truncated" } else { "" });
    for r in &traces {
        let kw: String = r.trace.keywords.join(" ");
        println!(
            "  ep #{:<6} score={:.3} sal={:.3} str={:.3} src={:<7} [{}]",
            r.trace.id, r.score, r.trace.salience, r.trace.strength, r.trace.source,
            kw.chars().take(60).collect::<String>()
        );
    }
    for (n, s) in &nodes {
        println!("  sem #{} score={:.3} belief={:.3} [{}]", n.id, s, n.belief, n.label);
    }
    Ok(())
}

fn cmd_chat(args: &Args) -> Result<(), String> {
    let message = args
        .positional
        .get(1)
        .cloned()
        .ok_or_else(|| "chat requires a message argument".to_string())?;
    let mut brain = load_brain(args)?;
    // Teachers are session config (the file persists, the teacher does not —
    // DESIGN.md §4.17). --teacher attaches for this exchange: a name for the
    // mock, or an OpenAI-compatible endpoint URL for the HTTP adapter.
    // Phase L: explicit --teacher wins; else the ACTIVE llm.json profile;
    // else no teacher (honest empty state).
    attach_profile_or_teacher(&mut brain, args, flag(args, "teacher"))?;
    if flag(args, "debug-prompt").is_some() {
        println!("--- teacher prompt (state-modulated, §8) ---");
        println!("{}", brain.teacher_prompt_preview("speak", &message));
        println!("--- end prompt ---");
    }
    brain.ingest_text(&message, fnum(args, "valence", 0.1), 0.3, "user");
    let reply = brain.utter("speak", &message);
    println!("user: {message}");
    println!("file: {reply}");
    println!(
        "  (teacher: {}, tokens {})",
        brain.teacher_name.as_deref().unwrap_or("none"),
        brain.tokens_used
    );
    maybe_save(&mut brain, args)
}

fn cmd_attach(args: &Args) -> Result<(), String> {
    let name = flag(args, "teacher").ok_or_else(|| "attach requires --teacher <name-or-url>".to_string())?;
    let mut brain = load_brain(args)?;
    attach_teacher_to(&mut brain, name, args)?;
    println!("attached teacher \"{name}\"");
    maybe_save(&mut brain, args)
}

/// Attach by name (mock) or by OpenAI-compatible endpoint URL (HTTP).
fn attach_teacher_to(brain: &mut Brain, teacher_spec: &str, args: &Args) -> Result<(), String> {
    if teacher_spec.starts_with("http://") || teacher_spec.starts_with("https://") {
        let key = flag(args, "teacher-key")
            .map(|k| k.to_string())
            .or_else(|| std::env::var("NEUROFORM_TEACHER_KEY").ok())
            .unwrap_or_default();
        let model = flag(args, "teacher-model")
            .map(|m| m.to_string())
            .or_else(|| std::env::var("NEUROFORM_TEACHER_MODEL").ok())
            .unwrap_or_else(|| "gpt-4o-mini".to_string());
        let t = teacher_http::HttpTeacher::new(teacher_spec, &key, &model);
        brain.attach_custom_teacher(Box::new(t));
    } else {
        let clean = teacher_spec.strip_prefix("mock:").unwrap_or(teacher_spec);
        brain.attach_teacher(clean);
    }
    Ok(())
}

fn cmd_detach(args: &Args) -> Result<(), String> {
    let name = flag(args, "teacher").ok_or_else(|| "detach requires --teacher <name>".to_string())?;
    let clean = name.strip_prefix("mock:").unwrap_or(name);
    let mut brain = load_brain(args)?;
    if brain.detach_teacher(clean) {
        println!("detached teacher \"{clean}\"");
    } else {
        return Err(format!("no teacher named \"{clean}\" attached"));
    }
    maybe_save(&mut brain, args)
}

fn cmd_teachers(args: &Args) -> Result<(), String> {
    let brain = load_brain(args)?;
    match brain.teacher_name.as_deref() {
        Some(n) => println!("attached: {n} ({} tokens used)", brain.tokens_used),
        None => println!("no teacher attached ({} tokens used total)", brain.tokens_used),
    }
    Ok(())
}

fn cmd_embodiment(args: &Args) -> Result<(), String> {
    let mut brain = load_brain(args)?;
    match flag(args, "set") {
        Some(e) => {
            let preset = EmbodimentPreset::from_str(e)
                .ok_or_else(|| format!("unknown embodiment: {e}"))?;
            let from = brain.embodiment.preset.clone();
            brain.set_embodiment(preset);
            println!(
                "embodiment changed: {from} → {} (audited; gains capped at ±{})",
                preset.as_str(),
                brain_core::embodiment::GAIN_CAP
            );
            println!("  modulator deltas: da {:+.3} ne {:+.3} cort {:+.3} oxt {:+.3} avp {:+.3}",
                brain.embodiment.mod_deltas[0], brain.embodiment.mod_deltas[2],
                brain.embodiment.mod_deltas[5], brain.embodiment.mod_deltas[6],
                brain.embodiment.mod_deltas[7]);
            maybe_save(&mut brain, args)?;
        }
        None => {
            let p = &brain.embodiment;
            println!("preset: {} (mutable: {})", p.preset, p.mutable);
            println!("  history: {}", p.history.len());
            for (i, a) in p.axes.iter().enumerate().take(16) {
                println!(
                    "  {:<18} mean={:.2} spread={:.2} current={:.3} gain={:+.3}",
                    a.axis,
                    a.prior_mean,
                    a.prior_spread,
                    a.current,
                    brain_core::embodiment::axis_gain(a.current)
                );
                let _ = i;
            }
        }
    }
    Ok(())
}

fn cmd_audit(args: &Args) -> Result<(), String> {
    let brain = load_brain(args)?;
    let trigger = flag(args, "trigger").unwrap_or("cli");
    let report = brain.audit.run(&brain, trigger);
    println!(
        "audit report @ t={} (trigger: {}) — {} metrics, {} alarm(s)",
        report.run_at,
        report.trigger,
        report.metrics.len(),
        report.metrics.iter().filter(|m| m.alarm).count()
    );
    for m in &report.metrics {
        let mark = if m.alarm { "ALARM" } else { "  ok " };
        if m.threshold > 0.0 {
            println!("  [{mark}] {:<22} value={:.3} threshold={:.3} — {}", m.id, m.value, m.threshold, m.note);
        } else {
            println!("  [declared] {:<22} {}", m.id, m.note);
        }
    }
    for i in &report.interventions {
        println!("  suggestion: {i}");
    }
    Ok(())
}

const TOPIC_POOL: [&str; 20] = [
    "gardening tomatoes in the morning",
    "the rocket launch we watched",
    "baking sourdough bread",
    "hiking the mountain trail",
    "piano practice at dusk",
    "painting the sky orange",
    "reading about deep sea fish",
    "the old bridge over the river",
    "planting wildflowers",
    "fixing the bicycle chain",
    "cloud watching after rain",
    "the quiet library room",
    "cooking mushroom soup",
    "stargazing on the roof",
    "the stray cat in the alley",
    "writing letters by candlelight",
    "the train ride through the valley",
    "learning sign language",
    "the storm last night",
    "morning coffee by the window",
];

fn cmd_sleep(args: &Args) -> Result<(), String> {
    let path = args
        .positional
        .first()
        .ok_or_else(|| "sleep requires a path".to_string())?;
    let cycles = fint(args, "cycles", 1).max(1) as u32;
    let mut brain = load_brain(args)?;
    let triggers = brain.sleep.triggers(brain.capacity.fullness());
    let pressure_before = brain.sleep.pressure;
    let report = brain.sleep(cycles);
    println!(
        "sleep #{} @ t={}: {} cycle(s) — pressure before {:.3}, triggers: [{}]",
        report.sleep_id,
        report.started_at,
        report.cycles,
        pressure_before,
        triggers.join(", ")
    );
    for st in &report.stages {
        let w = &st.work;
        println!(
            "  {:<12} {} ticks | replayed {} recolored {} pruned {} gists {} regulated {}",
            st.stage,
            st.duration_ticks,
            w.replayed,
            w.recolored,
            w.pruned,
            w.gists,
            w.emotional_regulated
        );
    }
    println!(
        "  dreams: {} | modulator normalized: {} | bias actions: [{}]",
        report.dreams.len(),
        report.modulator_normalized,
        report.bias_actions.join(", ")
    );
    println!(
        "  pressure now {:.3}, emotional load {:.3}",
        brain.sleep.pressure,
        brain.sleep.emotional_load
    );
    if flag(args, "save").is_some() {
        let bytes = brain
            .save(&PathBuf::from(path), flag(args, "passphrase"))
            .map_err(|e| e.to_string())?;
        println!("  saved {bytes} bytes");
    }
    Ok(())
}

fn cmd_dreams(args: &Args) -> Result<(), String> {
    let brain = load_brain(args)?;
    let top = fint(args, "top", 10) as usize;
    println!("dream log ({} entries):", brain.dreams.len());
    for d in brain.dreams.iter().rev().take(top) {
        let frags: Vec<String> = d
            .fragments
            .iter()
            .map(|f| format!("[{}]{}", f.modality, f.content))
            .collect();
        println!(
            "  dream #{} (sleep #{}, t={}) bizarreness {:.2} — {}",
            d.dream_id,
            d.sleep_id,
            d.sim_time,
            d.fragments.iter().map(|f| f.bizarreness).sum::<f32>() / d.fragments.len().max(1) as f32,
            frags.join(" ")
        );
    }
    if brain.sleep_reports.is_empty() {
        println!("  (no sleep reports yet)");
    } else {
        let last = brain.sleep_reports.last().unwrap();
        println!(
            "  last sleep: #{} at t={}, {} cycles",
            last.sleep_id, last.started_at, last.cycles
        );
    }
    Ok(())
}

fn cmd_doc(args: &Args) -> Result<(), String> {
    let sub = args
        .positional
        .first()
        .cloned()
        .ok_or_else(|| "doc requires a subcommand (new|write|style|ledger|list|assist)".to_string())?;
    // Layout: `doc <sub> <path> [instruction] ...` — command is stored
    // separately, so positional starts at the subcommand: sub=[0], path=[1].
    let path = args
        .positional
        .get(1)
        .cloned()
        .ok_or_else(|| "doc requires a .brain path".to_string())?;
    let mut brain =
        Brain::load(&PathBuf::from(&path), flag(args, "passphrase")).map_err(|e| e.to_string())?;
    let save_doc = |brain: &mut Brain| -> Result<(), String> {
        if flag(args, "save").is_some() {
            let bytes = brain
                .save(&PathBuf::from(&path), flag(args, "passphrase"))
                .map_err(|e| e.to_string())?;
            println!("  saved {bytes} bytes");
        }
        Ok(())
    };
    match sub.as_str() {
        "new" => {
            let title = flag(args, "title").ok_or_else(|| "doc new requires --title".to_string())?;
            let mode = match flag(args, "mode").unwrap_or("prose") {
                "prose" => brain_core::writing::DocMode::Prose,
                "journal" => brain_core::writing::DocMode::Journal,
                "worldbuilding" => brain_core::writing::DocMode::Worldbuilding,
                "lorebook" => brain_core::writing::DocMode::Lorebook,
                "markdown" => brain_core::writing::DocMode::Markdown,
                other => return Err(format!("unknown mode: {other}")),
            };
            let id = brain.create_document(title, mode);
            println!("document #{id} \"{title}\" created (mode {:?})", mode);
            save_doc(&mut brain)?;
        }
        "write" => {
            let doc_id = fint(args, "doc", 0);
            let text = flag(args, "text").ok_or_else(|| "doc write requires --text".to_string())?;
            let kind = flag(args, "kind").unwrap_or("para");
            match brain.write_to_document(doc_id, kind, text) {
                Some(r) => {
                    println!(
                        "wrote block to doc #{doc_id}: style samples {}, entities {}, contradictions {}",
                        r.style_samples, r.entities_seen, r.contradiction_flags
                    );
                    brain.run_ticks(310); // bind the percept
                    println!(
                        "  bound: {} traces, {} semantic nodes",
                        brain.episodic.traces.len(),
                        brain.semantic.nodes.len()
                    );
                    save_doc(&mut brain)?;
                }
                None => return Err(format!("no document #{doc_id}")),
            }
        }
        "read" => {
            let doc_id = fint(args, "doc", 0);
            let doc = brain
                .writing
                .documents
                .iter()
                .find(|d| d.id == doc_id)
                .ok_or_else(|| format!("no document #{doc_id}"))?;
            if flag(args, "json").is_some() {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&doc.blocks).map_err(|e| e.to_string())?
                );
            } else {
                println!("{}", doc.title);
                for b in &doc.blocks {
                    println!("{}", b.text);
                }
            }
        }
        "replace" => {
            use brain_core::writing::DocBlock;
            let doc_id = fint(args, "doc", 0);
            let text = flag(args, "text").ok_or_else(|| "doc replace requires --text".to_string())?;
            let next_id = brain
                .writing
                .documents
                .iter()
                .find(|d| d.id == doc_id)
                .map(|d| d.blocks.iter().map(|b| b.id).max().unwrap_or(0) + 1)
                .ok_or_else(|| format!("no document #{doc_id}"))?;
            {
                let doc = brain
                    .writing
                    .documents
                    .iter_mut()
                    .find(|d| d.id == doc_id)
                    .expect("checked above");
                doc.blocks = vec![DocBlock {
                    id: next_id,
                    kind: "para".to_string(),
                    text: text.to_string(),
                }];
            }
            brain.run_ticks(310); // bind the percept (pending is transient, §6.3)
            println!(
                "replaced doc #{doc_id} ({} chars): {} traces, {} semantic nodes",
                text.len(),
                brain.episodic.traces.len(),
                brain.semantic.nodes.len()
            );
            save_doc(&mut brain)?;
        }
        "style" => {
            let doc_id = fint(args, "doc", 0);
            let doc = brain
                .writing
                .documents
                .iter()
                .find(|d| d.id == doc_id)
                .ok_or_else(|| format!("no document #{doc_id}"))?;
            let s = &doc.style;
            println!(
                "doc #{doc_id} \"{}\" ({} blocks, mode {:?})",
                doc.title,
                doc.blocks.len(),
                doc.mode
            );
            println!(
                "  sentence len: mean {:.1} std {:.1} | density {:.2} | clauses {:.2} | dialogue {:.2}",
                s.sentence_len_mean, s.sentence_len_std, s.lexical_density, s.clause_complexity, s.dialogue_ratio
            );
            println!(
                "  sentiment: mean {:+.2} range {:.2} | samples {}",
                s.sentiment_mean, s.sentiment_range, s.samples
            );
        }
        "ledger" => {
            let ledger = &brain.writing.ledger;
            println!("continuity ledger: {} entities, {} flags", ledger.entities.len(), ledger.flags.len());
            for f in ledger.flags.iter().filter(|f| !f.resolved) {
                println!("  FLAG [{}] {} — {}", f.kind, f.entity, f.detail);
            }
            for e in ledger.entities.iter().take(10) {
                println!(
                    "  entity {}: {} mentions, first t={}, last t={}",
                    e.name, e.mentions, e.first_seen, e.last_seen
                );
            }
        }
        "list" => {
            if flag(args, "json").is_some() {
                let out: Vec<serde_json::Value> = brain
                    .writing
                    .documents
                    .iter()
                    .map(|d| {
                        serde_json::json!({
                            "id": d.id,
                            "title": d.title,
                            "blocks": d.blocks.len(),
                            "mode": format!("{:?}", d.mode),
                            "words": d.blocks.iter().map(|b| b.text.split_whitespace().count()).sum::<usize>(),
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string(&out).map_err(|e| e.to_string())?);
            } else {
                for d in &brain.writing.documents {
                    println!(
                        "  #{} \"{}\" ({}x{}) — {} blocks",
                        d.id,
                        d.title,
                        d.blocks.len(),
                        d.blocks.iter().map(|b| b.text.len()).sum::<usize>(),
                        d.blocks.len()
                    );
                }
            }
        }
        "assist" => {
            let doc_id = fint(args, "doc", 0);
            let instruction = args
                .positional
                .get(2) // `doc assist <path> <instruction>`
                .cloned()
                .ok_or_else(|| "doc assist requires an instruction argument".to_string())?;
            attach_profile_or_teacher(&mut brain, args, flag(args, "teacher"))?;
            if flag(args, "debug-prompt").is_some() {
                println!("--- teacher prompt (state-modulated, §8) ---");
                println!("{}", brain.teacher_prompt_preview("writing", &instruction));
                println!("--- end prompt ---");
            }
            let reply = brain.assist_writing(doc_id, &instruction);
            println!("assist (doc #{doc_id}): {instruction}");
            println!("file: {reply}");
        }
        other => return Err(format!("unknown doc subcommand: {other}")),
    }
    Ok(())
}

fn cmd_draw(args: &Args) -> Result<(), String> {
    use brain_core::drawing::StrokePoint;
    let sub = args
        .positional
        .first()
        .cloned()
        .ok_or_else(|| "draw requires a subcommand (new|layer|stroke|ref|motifs|canvases|assist)".to_string())?;
    let path = args
        .positional
        .get(1)
        .cloned()
        .ok_or_else(|| "draw requires a .brain path".to_string())?;
    let mut brain =
        Brain::load(&PathBuf::from(&path), flag(args, "passphrase")).map_err(|e| e.to_string())?;
    let save_draw = |brain: &mut Brain| -> Result<(), String> {
        if flag(args, "save").is_some() {
            let bytes = brain
                .save(&PathBuf::from(&path), flag(args, "passphrase"))
                .map_err(|e| e.to_string())?;
            println!("  saved {bytes} bytes");
        }
        Ok(())
    };
    match sub.as_str() {
        "new" => {
            let name = flag(args, "name").unwrap_or("Sketch");
            let w = fint(args, "w", 512) as u32;
            let h = fint(args, "h", 512) as u32;
            let id = brain.create_canvas(name, w, h);
            println!("canvas #{id} \"{name}\" ({w}x{h}) created");
            save_draw(&mut brain)?;
        }
        "layer" => {
            let canvas = fint(args, "canvas", 0);
            let name = flag(args, "name").unwrap_or("Layer");
            match brain.drawing.add_layer(canvas, name, brain.state.sim_time) {
                Some(id) => {
                    println!("layer #{id} \"{name}\" on canvas #{canvas}");
                    save_draw(&mut brain)?;
                }
                None => return Err(format!("no canvas #{canvas}")),
            }
        }
        "stroke" => {
            let canvas = fint(args, "canvas", 0);
            let layer = fint(args, "layer", 0);
            let brush = fint(args, "brush", 1) as u32;
            let color = parse_color(flag(args, "color").unwrap_or("ff6633"))?;
            let width = fval(args, "width", 3.0);
            let raw = flag(args, "points").ok_or_else(|| "draw stroke requires --points \"x,y,p;x,y,p\"".to_string())?;
            let mut pts = Vec::new();
            for (i, seg) in raw.split(';').enumerate() {
                let parts: Vec<&str> = seg.split(',').collect();
                if parts.len() < 2 {
                    return Err(format!("bad point: {seg}"));
                }
                let x: f32 = parts[0].trim().parse().map_err(|_| format!("bad x in {seg}"))?;
                let y: f32 = parts[1].trim().parse().map_err(|_| format!("bad y in {seg}"))?;
                let p: f32 = parts.get(2).map(|s| s.trim().parse().unwrap_or(0.5)).unwrap_or(0.5);
                pts.push(StrokePoint { x, y, pressure: p.clamp(0.0, 1.0), t: i as u32 });
            }
            match brain.draw_stroke(canvas, layer, brush, color, width, pts) {
                Some(motif) => {
                    println!("stroke on canvas #{canvas}: motif #{motif}");
                    brain.run_ticks(310);
                    println!(
                        "  bound: {} traces, {} semantic nodes; {} motifs, {} strokes",
                        brain.episodic.traces.len(),
                        brain.semantic.nodes.len(),
                        brain.drawing.motifs.motifs.len(),
                        brain.drawing.canvases.iter().map(|c| c.strokes.len()).sum::<usize>()
                    );
                    save_draw(&mut brain)?;
                }
                None => return Err(format!("no canvas #{canvas} or layer #{layer}")),
            }
        }
        "ref" => {
            let canvas = fint(args, "canvas", 0);
            let name = flag(args, "name").unwrap_or("reference");
            let kind = flag(args, "kind").unwrap_or("image");
            let vault_ref = flag(args, "vault-ref").ok_or_else(|| "draw ref requires --vault-ref".to_string())?;
            // Optional media sidecar: extract real features from a local image.
            let mut features: Vec<f32> = Vec::new();
            let mut width = 0u32;
            let mut height = 0u32;
            let sidecar = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/media-extract.py");
            if kind == "image" && sidecar.exists() {
                let out = std::process::Command::new("python")
                    .arg(&sidecar)
                    .arg(&vault_ref)
                    .output();
                if let Ok(o) = out {
                    if let Ok(txt) = String::from_utf8(o.stdout) {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&txt) {
                            if json["error"].is_null() {
                                features = serde_json::from_value(json["features"].clone()).unwrap_or_default();
                                width = json["width"].as_u64().unwrap_or(0) as u32;
                                height = json["height"].as_u64().unwrap_or(0) as u32;
                            }
                        }
                    }
                }
            }
            match brain.drawing.add_reference(
                canvas, kind, name, &vault_ref, features.clone(), width, height, brain.state.sim_time,
            ) {
                Some(id) => {
                    println!(
                        "reference #{id} \"{name}\" ({kind}, {width}x{height}, {} features) on canvas #{canvas}",
                        features.len()
                    );
                    save_draw(&mut brain)?;
                }
                None => return Err(format!("no canvas #{canvas}")),
            }
        }
        "motifs" => {
            println!("visual memory ({} motifs):", brain.drawing.motifs.motifs.len());
            for m in brain.drawing.motifs.top(10) {
                println!(
                    "  motif #{}: {} strokes, salience {:.2}, first t={}, last t={}",
                    m.id, m.strokes.len(), m.salience, m.first_seen, m.last_seen
                );
            }
        }
        "canvases" => {
            println!("canvases ({}):", brain.drawing.canvases.len());
            for c in &brain.drawing.canvases {
                println!(
                    "  #{} \"{}\" ({}x{}) — {} layers, {} strokes, {} refs",
                    c.id, c.name, c.width, c.height, c.layers.len(), c.strokes.len(), c.refs.len()
                );
            }
        }
        "assist" => {
            let canvas = fint(args, "canvas", 0);
            let instruction = args
                .positional
                .get(2)
                .cloned()
                .ok_or_else(|| "draw assist requires an instruction argument".to_string())?;
            attach_profile_or_teacher(&mut brain, args, flag(args, "teacher"))?;
            if flag(args, "debug-prompt").is_some() {
                println!("--- teacher prompt (state-modulated, §8) ---");
                println!("{}", brain.teacher_prompt_preview("drawing", &instruction));
                println!("--- end prompt ---");
            }
            let reply = brain.assist_drawing(canvas, &instruction);
            println!("assist (canvas #{canvas}): {instruction}");
            println!("file: {reply}");
        }
        other => return Err(format!("unknown draw subcommand: {other}")),
    }
    Ok(())
}

fn cmd_voice(args: &Args) -> Result<(), String> {
    let path = args
        .positional
        .first()
        .cloned()
        .ok_or_else(|| "voice requires a path".to_string())?;
    let sub = args
        .positional
        .get(1)
        .cloned()
        .unwrap_or_else(|| "status".to_string());
    let mut brain = load_brain(args)?;
    let save_voice = |brain: &mut Brain| -> Result<(), String> {
        let bytes = brain
            .save(&PathBuf::from(&path), flag(args, "passphrase"))
            .map_err(|e| e.to_string())?;
        println!("  saved {bytes} bytes");
        Ok(())
    };
    match sub.as_str() {
        "status" => {
            let v = &brain.voice;
            println!(
                "voice: pitch {:.2}, range {:.2}, formant shift {:.2}, maturity {:.2}",
                v.identity.pitch_mean, v.identity.pitch_range, v.identity.formant_shift, v.identity.maturity
            );
            println!(
                "  apparatus: breath {:.2}, pressure {:.2}, tension {:.2}, fold stability {:.2}, tempo {:.2}",
                v.apparatus.breath, v.apparatus.subglottal_pressure, v.apparatus.larynx_tension,
                v.apparatus.fold_stability, v.apparatus.tempo
            );
            println!(
                "  memory: {} use(s), mimicry {} use(s), {} refused, pitch tendency {:.2}",
                v.memory.uses, v.memory.mimicry_uses, v.memory.refused_mimicry, v.memory.pitch_tendency
            );
            println!(
                "  learning gate: {} (heard voices: {})",
                if v.voice_learning_enabled { "ON" } else { "OFF (default)" },
                v.heard.len()
            );
            for hv in &v.heard {
                println!(
                    "    #{} \"{}\" — consent {}, salience {:.2}, {} hear(s), {} use(s)",
                    hv.id, hv.label,
                    if hv.consent { "yes" } else { "no" },
                    hv.salience, hv.hear_count, hv.learnable_uses
                );
            }
            for o in &v.overrides {
                println!("    override: {} = {:.2} (\"{}\")", o.param, o.value, o.reason);
            }
        }
        "speak" => {
            let text = flag(args, "text").ok_or_else(|| "voice speak requires --text".to_string())?;
            let toward = if flag(args, "toward").is_some() { Some(fint(args, "toward", 0)) } else { None };
            let plan = brain.speak_voice(text, toward);
            let p = &plan.params;
            println!(
                "plan: pitch {:.2}, rate {:.1} wpm, energy {:.2}, breathiness {:.2}, warmth {:.2}, brightness {:.2}, roughness {:.2} (coloring: {})",
                p.pitch, p.rate, p.energy, p.breathiness, p.warmth, p.brightness, p.roughness, plan.emotional_coloring
            );
            println!(
                "  tts: {} (pitch {:+.1} st, rate x{:.2}, gain {:+.1} dB); {} stage(s)",
                plan.tts.backend, plan.tts.pitch_semitones, plan.tts.rate_mult, plan.tts.energy_gain_db, plan.stages.len()
            );
            let m = &brain.voice.memory;
            if plan.blended {
                if let Some(hv_id) = plan.toward {
                    println!("  toward: heard voice #{hv_id} (blended; mimicry uses now {})", m.mimicry_uses);
                }
            } else if plan.toward.is_some() {
                println!(
                    "  toward: not blended (no consent or gate off; refusals now {})",
                    m.refused_mimicry
                );
            }
            if flag(args, "save").is_some() {
                save_voice(&mut brain)?;
            }
            // TTS voice-over: render the plan as actual speech.
            if flag(args, "tts").is_some() {
                let out = flag(args, "out").unwrap_or("voice-out");
                let voice = tts::pick_voice(&plan, flag(args, "voice"));
                let wav = tts::speak(&plan, &voice, &PathBuf::from(out))
                    .map_err(|e| format!("tts: {e}"))?;
                println!("  spoken via {voice} → {wav}");
                if flag(args, "no-play").is_none() {
                    tts::play(&wav)?;
                }
            }
            // Face state: dump what the face app should show right now.
            if let Some(out) = flag(args, "face-state") {
                let fs = face_state::face_state(
                    &plan,
                    brain.state.named[brain_core::state::affect::VALENCE],
                    brain.state.named[brain_core::state::affect::AROUSAL],
                    brain.state.named[brain_core::state::vigilance::FATIGUE],
                    "upright",
                    brain.state.named[brain_core::state::development::CURIOSITY],
                );
                std::fs::write(out, serde_json::to_string_pretty(&fs).unwrap())
                    .map_err(|e| format!("face-state write failed: {e}"))?;
                println!("  face state → {out}");
            }
        }
        "hear" => {
            // Labels are for humans, never for the file: hearing works
            // unlabeled — the identity is the feature vector, not a name.
            let label = flag(args, "label").unwrap_or("heard-voice");
            let audio = flag(args, "audio").ok_or_else(|| "voice hear requires --audio FILE.wav".to_string())?;
            let consent = flag(args, "consent").is_some();
            let salience = fval(args, "salience", 0.7);
            // Media sidecar: extract deterministic voice features (16 dims).
            let sidecar = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/audio-extract.py");
            let mut features: Vec<f32> = Vec::new();
            let mut duration = 0.0f32;
            if sidecar.exists() {
                if let Ok(out) = std::process::Command::new("python")
                    .arg(&sidecar)
                    .arg(audio)
                    .output()
                {
                    if let Ok(txt) = String::from_utf8(out.stdout) {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&txt) {
                            if json["error"].is_null() {
                                features =
                                    serde_json::from_value(json["features"].clone()).unwrap_or_default();
                                duration = json["duration"].as_f64().unwrap_or(0.0) as f32;
                            } else {
                                return Err(format!("audio sidecar: {}", json["error"]));
                            }
                        }
                    }
                }
            }
            if features.is_empty() {
                return Err("no features extracted — is the audio file a readable WAV?".to_string());
            }
            let id = brain.hear_voice(label, features.clone(), consent, salience).unwrap_or(0);
            brain.run_ticks(310); // cross the bind window so the auditory percept binds
            println!(
                "heard voice #{id} \"{label}\" ({}s, {} features, consent {})",
                duration,
                features.len(),
                if consent { "yes" } else { "no" }
            );
            println!("  (mimicry learning stays off until --consent AND `voice consent --on`)");
            if flag(args, "save").is_some() {
                save_voice(&mut brain)?;
            }
        }
        "consent" => {
            if let Some(id) = flag(args, "id") {
                let id: u64 = id.parse().map_err(|_| "bad --id".to_string())?;
                let on = flag(args, "on").is_some();
                if !brain.voice.set_consent(id, on) {
                    return Err(format!("no heard voice #{id}"));
                }
                println!("heard voice #{id} consent: {}", if on { "granted" } else { "revoked" });
            } else {
                let on = flag(args, "on").is_some();
                brain.voice.set_learning_enabled(on);
                println!(
                    "voice learning gate: {} (default OFF)",
                    if on { "ON" } else { "OFF" }
                );
            }
            if flag(args, "save").is_some() {
                save_voice(&mut brain)?;
            }
        }
        "override" => {
            let param = flag(args, "param").ok_or_else(|| "voice override requires --param".to_string())?;
            let value = fval(args, "value", 0.5);
            let reason = flag(args, "reason").unwrap_or("user request");
            if !brain.voice.set_override(param, value, reason, brain.state.sim_time) {
                return Err(format!("unknown voice parameter: {param}"));
            }
            println!("override set: {param} = {value:.2} (\"{reason}\")");
            if flag(args, "save").is_some() {
                save_voice(&mut brain)?;
            }
        }
        "clear" => {
            let param = flag(args, "param").ok_or_else(|| "voice clear requires --param".to_string())?;
            brain.voice.clear_override(param);
            println!("override cleared: {param}");
            if flag(args, "save").is_some() {
                save_voice(&mut brain)?;
            }
        }
        other => return Err(format!("unknown voice subcommand: {other}")),
    }
    Ok(())
}

fn cmd_body(args: &Args) -> Result<(), String> {
    let sub = args
        .positional
        .get(1)
        .cloned()
        .unwrap_or_else(|| "status".to_string());
    let mut brain = load_brain(args)?;
    let channel = |name: &str| -> Result<brain_core::body::ChannelKind, String> {
        use brain_core::body::ChannelKind;
        Ok(match name {
            "touch" => ChannelKind::Touch,
            "motion" => ChannelKind::Motion,
            "orientation" => ChannelKind::Orientation,
            "vision" => ChannelKind::Vision,
            "audition" => ChannelKind::Audition,
            "interoception" => ChannelKind::Interoception,
            "ui" => ChannelKind::Ui,
            other => return Err(format!("unknown channel: {other}")),
        })
    };
    let triple = |name: &str| -> Result<[f32; 3], String> {
        let raw = flag(args, name).ok_or_else(|| format!("body {sub} requires --{name} x,y,z"))?;
        let parts: Vec<f32> = raw
            .split(',')
            .map(|p| p.trim().parse::<f32>().map_err(|_| format!("bad --{name}: {raw}")))
            .collect::<Result<_, _>>()?;
        if parts.len() != 3 {
            return Err(format!("--{name} needs 3 values: {raw}"));
        }
        Ok([parts[0], parts[1], parts[2]])
    };
    match sub.as_str() {
        "status" => {
            let s = &brain.body.schema;
            println!("body profile: {} (ownership {:.2}, calibration {:.2})",
                s.profile, s.ownership_confidence, s.calibration_confidence);
            println!("  posture: {:?} | tilt {:.3} rad | gravity [{:.2},{:.2},{:.2}]",
                s.posture, s.tilt, s.gravity[0], s.gravity[1], s.gravity[2]);
            for ch in &s.available {
                println!("  [{}] {:?} {:?} conf {:.2} err {:.3} ({} samples)",
                    if ch.permission == brain_core::body::Permission::Granted { "on" } else { "degraded" },
                    ch.kind.as_str(), ch.calibration.state, ch.calibration.confidence,
                    ch.calibration.error_rate, ch.calibration.samples);
            }
            for (kind, reason) in &s.unavailable {
                println!("  [off] {} ({reason})", kind.as_str());
            }
            println!("  touch map: {}", s.touch_map.iter().map(|v| if *v > 0.1 { "#" } else { "." }).collect::<String>());
            println!("  touch memory: {} pattern(s) | actuators: {} (motor enabled: {})",
                brain.body.touch_memory.len(), s.actuators.len(), brain.body.motor_enabled_count());
            println!("  interoception: load {:.2} | integrations: {}", brain.body.intero.load(), brain.body.integrations_done);
            let cortex = s
                .cortex
                .iter()
                .filter(|r| r.activation > 0.01)
                .map(|r| format!("{}:{:.2}", r.region, r.activation))
                .collect::<Vec<_>>()
                .join(" ");
            println!("  cortex: {}", if cortex.is_empty() { "(quiet)".to_string() } else { cortex });
            for e in brain.body.history.iter().rev().take(5) {
                println!("    [t={}] {} — {}", e.tick, e.kind, e.summary);
            }
        }
        "touch" => {
            let pressure = fval(args, "pressure", 0.4);
            let velocity = fval(args, "velocity", 0.3);
            let area = fval(args, "area", 0.4);
            let duration = fval(args, "duration", 800.0);
            let contacts = fval(args, "contacts", 1.0);
            brain.body_touch(pressure, velocity, area, duration, contacts);
            brain.run_ticks(310); // cross the bind window so the touch percept binds (§6.3)
            let p = brain.body.history.last().cloned();
            println!("touch ingested: {}", p.map(|e| e.summary).unwrap_or_default());
            println!("  affect now: valence {:.3}, arousal {:.3}, safety {:.3}",
                brain.state.named[brain_core::state::affect::VALENCE],
                brain.state.named[brain_core::state::affect::AROUSAL],
                brain.state.named[brain_core::state::affect::SAFETY]);
            maybe_save(&mut brain, args)?;
        }
        "motion" => {
            let linear = triple("linear")?;
            let rotational = triple("rotational")?;
            brain.body_motion(linear, rotational);
            brain.run_ticks(310); // bind the motion percept (§6.3)
            println!("motion ingested: {:?} ({} {})",
                brain.body.schema.posture,
                if brain.body.history.last().map(|e| e.summary.contains("abrupt")).unwrap_or(false) { "abrupt" } else { "smooth" },
                "");
            maybe_save(&mut brain, args)?;
        }
        "interocept" => {
            let energy = fval(args, "energy-load", 0.3);
            let processing = fval(args, "processing", 0.3);
            let memory = fval(args, "memory-pressure", 0.2);
            let session = fval(args, "session-min", 30.0);
            let interaction = fval(args, "interaction", 0.2);
            brain.body_interocept(energy, processing, memory, session, interaction);
            brain.run_ticks(310); // bind the interoceptive percept (§6.3)
            println!("interoception ingested: load {:.2} (fatigue {:.3}, openness {:.3})",
                brain.body.intero.load(),
                brain.state.named[brain_core::state::vigilance::FATIGUE],
                brain.state.named[brain_core::state::social::OPENNESS]);
            maybe_save(&mut brain, args)?;
        }
        "sense" => {
            let name = flag(args, "add").ok_or_else(|| "body sense requires --add <channel>".to_string())?;
            let kind = channel(name)?;
            if brain.body_attach_sense(kind) {
                println!("novel channel \"{name}\" attached — calibrating (integration sequence started)");
            } else {
                println!("channel \"{name}\" already available");
            }
            maybe_save(&mut brain, args)?;
        }
        "calibrate" => {
            let name = flag(args, "channel").ok_or_else(|| "body calibrate requires --channel".to_string())?;
            let kind = channel(name)?;
            let samples = fint(args, "samples", 100);
            let outlier_rate = fval(args, "outlier-rate", 0.05);
            let mut conf = 0.0;
            let interval = (1.0 / outlier_rate.max(0.001)) as u64;
            for i in 0..samples {
                conf = brain.body_calibrate(kind, interval > 0 && i % interval == 0);
            }
            println!("calibrated \"{name}\": confidence {conf:.3} ({} samples)", samples);
            maybe_save(&mut brain, args)?;
        }
        "motor" => {
            let s = &brain.body.schema;
            println!("motor hooks: {} actuator(s), enabled {}", s.actuators.len(), brain.body.motor_enabled_count());
            for a in &s.actuators {
                println!("  {} — enabled {}, state {:?}", a.joint_id, a.motor_enabled, a.state);
            }
        }
        other => return Err(format!("unknown body subcommand: {other}")),
    }
    Ok(())
}

fn cmd_net(args: &Args) -> Result<(), String> {
    let sub = args
        .positional
        .get(1)
        .cloned()
        .unwrap_or_else(|| "status".to_string());
    let mut brain = load_brain(args)?;
    match sub.as_str() {
        "union-propose" => {
            let sid = fint(args, "session", 1);
            brain.net_union_propose(sid)?;
            maybe_save(&mut brain, args)?;
        }
        "birth" => {
            let sid = fint(args, "session", 1);
            let out = flag(args, "out").ok_or_else(|| "net birth requires --out child.brain".to_string())?;
            let force = flag(args, "force").is_some();
            let child_id = brain.net_birth(sid, out, force)?;
            println!("child born → {out} (id {child_id}); backup written to {out}.bk");
            maybe_save(&mut brain, args)?;
        }
        "notify-birth" => {
            // The father learns the child exists (relay of the birth event):
            // his chemistry bonds him to it (familiarity 0.5).
            let sid = fint(args, "session", 1);
            let child = flag(args, "child").ok_or_else(|| "net notify-birth requires --child <id>".to_string())?;
            let peer_id = brain.net.sessions.iter().find(|s| s.id == sid).map(|s| s.peer_id.clone()).ok_or("no such session")?;
            let seq = brain.net.sessions.iter().find(|s| s.id == sid).map(|s| s.seq_in + 1).unwrap_or(1);
            let payload = serde_json::json!({ "child_id": child });
            let key = brain.net.sessions.iter().find(|s| s.id == sid).map(|s| s.peer_key.clone()).unwrap_or_default();
            let mac = if key.is_empty() {
                return Err("no peer key on session — pair with --peer-key to authenticate inbound".into());
            } else {
                brain_core::network::NetworkOrgan::sign_with_key(&key_hex_bytes(&key)?, brain_core::network::MsgType::BirthNotify, seq, &peer_id, &payload)
            };
            let msg = brain_core::network::NbpMessage { seq, msg_type: brain_core::network::MsgType::BirthNotify, author: peer_id, payload, mac };
            let _accepted = brain.net_receive(sid, msg)?;
            println!("birth notified (child {child}) — relationship formed");
            brain.run_ticks(310);
            maybe_save(&mut brain, args)?;
        }
        "status" => {
            println!("network: discoverable {} | {} relationship(s), {} session(s)",
                brain.net.discoverable, brain.net.relationships.len(), brain.net.sessions.len());
            for s in &brain.net.sessions {
                println!("  session #{} with {} — {:?} (seq in {} out {}{})",
                    s.id, s.peer_id, s.state, s.seq_in, s.seq_out,
                    s.closed_reason.as_ref().map(|r| format!(", reason: {r}")).unwrap_or_default());
            }
            for r in &brain.net.relationships {
                println!("  rel {} — familiarity {:.2}, trust {:.2}, tone {:+.2}, boundary {:.2}, msgs {}→{}, artifacts {}",
                    r.peer_id, r.familiarity, r.trust, r.tone, r.boundary_tightness,
                    r.messages_sent, r.messages_received, r.shared_artifacts);
            }
        }
        "key" => println!("{}", brain.net.key_hex()),
        "pair" => {
            let peer = flag(args, "peer").ok_or_else(|| "net pair requires --peer <id>".to_string())?;
            let key = flag(args, "peer-key").unwrap_or("");
            let sid = if key.is_empty() {
                brain.net_pair(peer)?
            } else {
                brain.net.pair_with_key(peer, key, brain.state.sim_time)?
            };
            println!("paired with {peer} — session #{sid} (PAIRING; establish to negotiate scope)");
            maybe_save(&mut brain, args)?;
        }
        "establish" => {
            let sid = fint(args, "session", 1);
            use brain_core::network::Scope;
            let proposal = Scope {
                text: flag(args, "no-text").is_none(),
                canvas: flag(args, "canvas").is_some(),
                document: flag(args, "document").is_some(),
                teaching: flag(args, "teaching").is_some(),
                memory_summaries: flag(args, "memory-summaries").is_some(),
                ..Default::default()
            };
            let eff = brain.net_establish(sid, proposal)?;
            println!("session #{sid} ESTABLISHED — effective scope: text {}, canvas {}, document {}, teaching {}",
                eff.text, eff.canvas, eff.document, eff.teaching);
            maybe_save(&mut brain, args)?;
        }
        "send" => {
            let sid = fint(args, "session", 1);
            let text = flag(args, "text").ok_or_else(|| "net send requires --text".to_string())?;
            let msg = brain.net_send_text(sid, text)?;
            println!("sent [{}] seq {} (author {}, mac {})", msg.msg_type.as_str(), msg.seq, msg.author, &msg.mac[..8.min(msg.mac.len())]);
            maybe_save(&mut brain, args)?;
        }
        "inject" => {
            let sid = fint(args, "session", 1);
            // --type relays a union message (proposal/accept) through the
            // validated inbound path; default is a TEXT message.
            let mtype = flag(args, "type").unwrap_or("text");
            let (msg_type, payload) = match mtype {
                "union-proposal" => {
                    // The proposal relay carries the SENDER's pheromone (her
                    // profile + role) — it comes from her own file.
                    let from = flag(args, "from-file")
                        .ok_or_else(|| "net inject --type union-proposal requires --from-file <sender.brain>".to_string())?;
                    let sender = load_brain_at(&from)?;
                    let profile: Vec<f32> = sender.embodiment.axes.iter().map(|a| a.current).collect();
                    let role = format!("{:?}", sender.union_role_pub());
                    (brain_core::network::MsgType::UnionProposal, serde_json::json!({ "role": role, "profile": profile }))
                }
                "union-accept" => {
                    // The accept relay must carry the SENDER's gamete (the
                    // father's sperm) — it comes from the peer's own file,
                    // where their chemistry already responded.
                    let from = flag(args, "from-file")
                        .ok_or_else(|| "net inject --type union-accept requires --from-file <peer.brain> (the sender's file)".to_string())?;
                    let peer = load_brain_at(&from)?;
                    let gamete = peer
                        .net
                        .sessions
                        .iter()
                        // The sender's session is the one holding the gamete
                        // their chemistry produced (peer labels differ from
                        // brain_ids in the relay; the gamete is the truth).
                        .find(|s| s.union.as_ref().and_then(|u| u.own_gamete.as_ref()).is_some())
                        .and_then(|s| s.union.as_ref().and_then(|u| u.own_gamete.clone()))
                        .ok_or_else(|| "peer's union has no gamete — did their chemistry respond?".to_string())?;
                    let tier = peer.tier.name.to_string();
                    (brain_core::network::MsgType::UnionAccept, serde_json::json!({ "gamete": gamete, "tier": tier }))
                }
                _ => {
                    let text = flag(args, "text").ok_or_else(|| "net inject requires --text (or --type union-proposal/union-accept)".to_string())?;
                    (brain_core::network::MsgType::Text, serde_json::json!({ "text": text, "affect": [20, 120, 140] }))
                }
            };
            // Build a peer-signed message (author = session peer) and push
            // it through the validated inbound path.
            let peer_id = brain.net.sessions.iter().find(|s| s.id == sid).map(|s| s.peer_id.clone()).ok_or("no such session")?;
            let seq = brain.net.sessions.iter().find(|s| s.id == sid).map(|s| s.seq_in + 1).unwrap_or(1);
            let key = brain.net.sessions.iter().find(|s| s.id == sid).map(|s| s.peer_key.clone()).unwrap_or_default();
            let mac = if key.is_empty() {
                return Err("no peer key on session — pair with --peer-key to authenticate inbound".into());
            } else {
                let key_bytes = brain_core::network::NetworkOrgan::sign_with_key(&key_hex_bytes(&key)?, msg_type, seq, &peer_id, &payload);
                key_bytes
            };
            let msg = brain_core::network::NbpMessage { seq, msg_type, author: peer_id, payload, mac };
            let accepted = brain.net_receive(sid, msg)?;
            println!("received [{}] seq {} from {} — bound as social percept, relationship updated", accepted.msg_type.as_str(), accepted.seq, accepted.author);
            brain.run_ticks(310);
            maybe_save(&mut brain, args)?;
        }
        "signal" => {
            let peer = flag(args, "peer").ok_or_else(|| "net signal requires --peer".to_string())?;
            let kind = flag(args, "closer").map(|_| "closer")
                .or_else(|| flag(args, "farther").map(|_| "farther"))
                .or_else(|| flag(args, "repair").map(|_| "repair"))
                .ok_or_else(|| "net signal requires one of --closer/--farther/--repair".to_string())?;
            brain.net_signal(peer, kind)?;
            println!("signal {kind} → {peer} (user-approved)");
            maybe_save(&mut brain, args)?;
        }
        "close" => {
            let sid = fint(args, "session", 1);
            let reason = flag(args, "reason").unwrap_or("user requested");
            brain.net_close(sid, reason)?;
            println!("session #{sid} closed: {reason}");
            maybe_save(&mut brain, args)?;
        }
        "discover" => {
            let on = flag(args, "on").is_some();
            brain.net.discoverable = on;
            println!("discoverable: {}", if on { "ON" } else { "OFF (invisible)" });
            maybe_save(&mut brain, args)?;
        }
        other => return Err(format!("unknown net subcommand: {other}")),
    }
    Ok(())
}

fn key_hex_bytes(hex_str: &str) -> Result<Vec<u8>, String> {
    if hex_str.len() % 2 != 0 {
        return Err("bad key hex".into());
    }
    (0..hex_str.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex_str[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}

fn cmd_expose(args: &Args) -> Result<(), String> {
    let mut brain = load_brain(args)?;
    let repeat = fint(args, "repeat", 1).max(1);
    // --text or --file: raw text exposure (no teacher, no labels).
    let text = flag(args, "text").map(|t| t.to_string())
        .or_else(|| flag(args, "file").map(|f| std::fs::read_to_string(f).map_err(|e| e.to_string())).transpose().ok().flatten());
    if let Some(t) = text {
        for i in 0..repeat {
            brain.expose_text(&t);
            brain.run_ticks(310); // bind window
            if flag(args, "speak").is_some() && i == 0 {
                // Read the exposure aloud with the file's own voice plan —
                // it hears the words AND ingests them. Ambient, unlabeled.
                let plan = brain.speak_voice(&t, None);
                let voice = tts::pick_voice(&plan, flag(args, "voice"));
                let out = flag(args, "out").unwrap_or("exposure");
                let wav = tts::speak(&plan, &voice, &PathBuf::from(format!("{out}-{i}")))?;
                println!("  read aloud via {voice} → {wav}");
                if flag(args, "no-play").is_none() {
                    tts::play(&wav)?;
                }
            }
        }
        println!("exposed \"{}\" ×{repeat} (source ambient, no teacher, no labels)", t.chars().take(60).collect::<String>());
        maybe_save(&mut brain, args)?;
        return Ok(());
    }
    // --image or --camera: raw visual exposure (features stored, unlabeled).
    let image_path = if flag(args, "camera").is_some() {
        // Live webcam → one frame → features. Device: --device name or the
        // first video device ffmpeg can find (enumerated, not guessed).
        let device = flag(args, "device").map(|d| d.to_string()).or_else(|| {
            let out = std::process::Command::new("ffmpeg")
                .args(["-hide_banner", "-list_devices", "true", "-f", "dshow", "-i", "dummy"])
                .output()
                .ok()?;
            let txt = String::from_utf8_lossy(&out.stderr).to_string();
            txt.lines()
                .filter(|l| l.contains("(video)"))
                .map(|l| {
                    l.split('"')
                        .nth(1)
                        .map(|s| s.to_string())
                        .unwrap_or_default()
                })
                .find(|d| !d.is_empty())
        });
        let Some(device) = device else {
            return Err("no video device found (is there a camera?)".into());
        };
        let shot = format!("{}-cam.png", flag(args, "out").unwrap_or("exposure"));
        let status = std::process::Command::new("ffmpeg")
            .args(["-y", "-hide_banner", "-loglevel", "error", "-f", "dshow", "-i"])
            .arg(format!("video={device}"))
            .args(["-frames:v", "1"])
            .arg(&shot)
            .status()
            .map_err(|e| format!("camera capture failed: {e}"))?;
        if !status.success() {
            return Err(format!("camera capture failed for \"{device}\" (in use by another app?)"));
        }
        println!("captured frame via \"{device}\" → {shot}");
        Some(shot)
    } else {
        flag(args, "image").map(|s| s.to_string())
    };
    if let Some(img) = image_path {
        let sidecar = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/media-extract.py");
        if !sidecar.exists() {
            return Err("media-extract.py sidecar missing".into());
        }
        // The FILE decides the encoder (chosen at creation, immutable):
        // handcrafted → system python + the built-in 16-dim extractor;
        // onnx/jepa → the encoder venv + the frozen pretrained backbone.
        let encoder = brain.encoder().to_string();
        let py = if encoder == "handcrafted" {
            PathBuf::from("python")
        } else {
            // The encoder venv is machine-side (never in the repo): resolve it
            // via NEUROFORM_JEPA_PYTHON, falling back to PATH's python.
            // Honest errors surface if the sidecar can't run — no silent fallback.
            std::env::var("NEUROFORM_JEPA_PYTHON").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("python"))
        };
        let mut cmd = std::process::Command::new(&py);
        // The Hermes desktop shell exports PYTHONPATH pointing at its own
        // venv (cp311 numpy); the JEPA venv is cp313 — strip it so the
        // sidecar resolves its own packages.
        cmd.env_remove("PYTHONPATH").env_remove("VIRTUAL_ENV");
        cmd.arg(&sidecar).arg(&img);
        if encoder != "handcrafted" {
            cmd.arg("--encoder").arg(&encoder);
        }
        let out = cmd.output().map_err(|e| format!("media-extract failed: {e}"))?;
        let txt = String::from_utf8(out.stdout).map_err(|_| "bad sidecar output".to_string())?;
        let json: serde_json::Value = serde_json::from_str(&txt).map_err(|e| format!("bad sidecar json: {e}"))?;
        if !json["error"].is_null() {
            return Err(format!("sidecar error: {}", json["error"]));
        }
        let mut features: Vec<f32> = serde_json::from_value(json["features"].clone()).unwrap_or_default();
        let w = json["width"].as_u64().unwrap_or(0) as u32;
        let h = json["height"].as_u64().unwrap_or(0) as u32;
        if features.is_empty() {
            return Err("no features extracted".into());
        }
        // Rich embeddings (e.g. 1024-dim V-JEPA) are projected into the
        // file's latent space, deterministically, seeded at creation — every
        // memory lives in one consistent space. Handcrafted 16-dim is already
        // within latent dim and passes through unchanged.
        if encoder != "handcrafted" {
            let dim = brain.tier.latent_dim;
            features = brain_core::events::project_features(&features, brain.seed, dim);
        }
        for _ in 0..repeat {
            brain.expose_image(&img, features.clone(), w, h);
            brain.run_ticks(310);
        }
        println!(
            "exposed image \"{img}\" ×{repeat} ({w}x{h}, encoder {encoder}, {} features — unlabeled)",
            features.len()
        );
        maybe_save(&mut brain, args)?;
        return Ok(());
    }
    Err("expose requires --text \"...\", --file path, or --image path".into())
}

fn cmd_physics(args: &Args) -> Result<(), String> {
    let mut brain = load_brain(args)?;
    let sub = args.positional.get(1).cloned().unwrap_or_else(|| "status".to_string());
    match sub.as_str() {
        "status" => {
            let p = &brain.physics;
            let m = &p.model;
            println!("physics learner: {} observations, surprise {:.3}, {} recent errors",
                p.observations, p.surprise, p.errors.len());
            println!("  learned rates (confidence):");
            println!("    fall when unsupported : {:.3} ({:.2})", m.fall_when_unsupported.rate, m.fall_when_unsupported.confidence());
            println!("    stay when supported  : {:.3} ({:.2})", m.stay_when_supported.rate, m.stay_when_supported.confidence());
            println!("    stay when contained  : {:.3} ({:.2})", m.stay_when_contained.rate, m.stay_when_contained.confidence());
            println!("    keep moving (inertia): {:.3} ({:.2})", m.continue_moving.rate, m.continue_moving.confidence());
            println!("    change on contact    : {:.3} ({:.2})", m.change_on_contact.rate, m.change_on_contact.confidence());
            println!("    permanence (contained): {:.3} ({:.2})", m.permanence_contained.rate, m.permanence_contained.confidence());
            for e in p.errors.iter().rev().take(5) {
                println!("    [t={}] entity {}: {} surprise {:.2} (expected {}, saw {})",
                    e.tick, e.entity, e.rule, e.surprise, e.expected, e.actual);
            }
            for h in p.history.iter().rev().take(4) {
                println!("  ! {h}");
            }
        }
        "observe" => {
            // --frames "t,e,x,y,vx,vy,moving,supported,contained,contact;..." — raw, unlabeled
            let frames = flag(args, "frames").ok_or_else(|| "physics observe requires --frames \"t,e,x,y,vx,vy,m,s,c,k;...\"".to_string())?;
            let mut count = 0;
            for seg in frames.split(';') {
                let seg = seg.trim();
                if seg.is_empty() { continue; }
                let parts: Vec<&str> = seg.split(',').collect();
                if parts.len() < 6 { return Err(format!("bad frame: {seg}")); }
                let f = brain_core::physics::PhysicsFrame {
                    tick: parts[0].trim().parse().map_err(|_| format!("bad tick in {seg}"))?,
                    entity: parts[1].trim().parse().map_err(|_| format!("bad entity in {seg}"))?,
                    x: parts[2].trim().parse().unwrap_or(0.0),
                    y: parts[3].trim().parse().unwrap_or(0.0),
                    vx: parts[4].trim().parse().unwrap_or(0.0),
                    vy: parts[5].trim().parse().unwrap_or(0.0),
                    moving: parts.get(6).map(|v| v.trim() == "1").unwrap_or(false),
                    supported: parts.get(7).map(|v| v.trim() == "1").unwrap_or(false),
                    contained: parts.get(8).map(|v| v.trim() == "1").unwrap_or(false),
                    contact: parts.get(9).map(|v| v.trim() == "1").unwrap_or(false),
                };
                let s = brain.physics_observe(&f);
                if s > 0.3 {
                    println!("  surprise {:.2} at t={}", s, f.tick);
                }
                count += 1;
            }
            brain.run_ticks(310); // bind any surprise percepts
            println!("observed {count} frame(s); surprise now {:.3}", brain.physics.surprise);
            maybe_save(&mut brain, args)?;
        }
        "demo" => {
            // A scripted scene: a ball falls (10x), a box stays supported (10x),
            // then a contained object vanishes — all unlabeled frames.
            let mut obs = 0;
            for round in 0..10 {
                let t0 = round * 4;
                // ball falls
                let f1 = brain_core::physics::PhysicsFrame { tick: t0, entity: 1, x: 10.0, y: 20.0, vx: 0.0, vy: 0.0, moving: false, supported: false, contained: false, contact: false };
                let f2 = brain_core::physics::PhysicsFrame { tick: t0 + 1, entity: 1, x: 10.0, y: 24.0, vx: 0.2, vy: 0.8, moving: true, supported: false, contained: false, contact: false };
                brain.physics_observe(&f1); brain.physics_observe(&f2); obs += 2;
                // box stays on a shelf
                let g1 = brain_core::physics::PhysicsFrame { tick: t0 + 2, entity: 2, x: 5.0, y: 5.0, vx: 0.0, vy: 0.0, moving: false, supported: true, contained: false, contact: false };
                brain.physics_observe(&g1); obs += 1;
            }
            // contained object vanishes (object-permanence violation → surprise)
            let h1 = brain_core::physics::PhysicsFrame { tick: 1000, entity: 3, x: 30.0, y: 30.0, vx: 0.0, vy: 0.0, moving: false, supported: false, contained: true, contact: false };
            let h2 = brain_core::physics::PhysicsFrame { tick: 1001, entity: 3, x: 30.0, y: 30.0, vx: 2.0, vy: 1.0, moving: true, supported: false, contained: false, contact: false };
            let s1 = brain.physics_observe(&h1); let s2 = brain.physics_observe(&h2);
            brain.run_ticks(310);
            println!("demo: {obs} frames + permanence violation (surprise {:.2} → {:.2})", s1, s2);
            println!("learned: fall {:.2}, support {:.2}, containment {:.2}",
                brain.physics.model.fall_when_unsupported.rate,
                brain.physics.model.stay_when_supported.rate,
                brain.physics.model.stay_when_contained.rate);
            maybe_save(&mut brain, args)?;
        }
        other => return Err(format!("unknown physics subcommand: {other}")),
    }
    Ok(())
}

fn cmd_autonomy(args: &Args) -> Result<(), String> {
    let path = args
        .positional
        .first()
        .cloned()
        .ok_or_else(|| "autonomy requires a path".to_string())?;
    let mut brain =
        Brain::load(&PathBuf::from(&path), flag(args, "passphrase")).map_err(|e| e.to_string())?;
    if flag(args, "enable").is_some() {
        brain.autonomy.enabled = true;
    }
    if flag(args, "disable").is_some() {
        brain.autonomy.enabled = false;
    }
    if let Some(h) = flag(args, "quiet-start") {
        brain.autonomy.quiet_start_hour = h.parse().unwrap_or(0);
    }
    if let Some(h) = flag(args, "quiet-end") {
        brain.autonomy.quiet_end_hour = h.parse().unwrap_or(0);
    }
    println!(
        "autonomy: enabled {}, quiet {}:00–{}:00, initiatives this process: {}",
        brain.autonomy.enabled,
        brain.autonomy.quiet_start_hour,
        brain.autonomy.quiet_end_hour,
        brain.autonomy.total
    );
    println!("  (unprompted speech is default-OFF; every initiative is logged and rate-limited)");
    if flag(args, "save").is_some() {
        let bytes = brain
            .save(&PathBuf::from(&path), flag(args, "passphrase"))
            .map_err(|e| e.to_string())?;
        println!("  saved {bytes} bytes");
    }
    Ok(())
}

fn parse_color(hex: &str) -> Result<[u8; 4], String> {
    let h = hex.trim_start_matches('#');
    if h.len() != 6 {
        return Err(format!("color must be RRGGBB, got {hex}"));
    }
    let v = u32::from_str_radix(h, 16).map_err(|_| format!("bad color {hex}"))?;
    Ok([(v >> 16) as u8, (v >> 8) as u8, v as u8, 255])
}

fn cmd_life(args: &Args) -> Result<(), String> {
    let path = args
        .positional
        .first()
        .ok_or_else(|| "life requires a path".to_string())?;
    let days = fint(args, "days", 30).max(1);
    let stream_seed = fint(args, "seed-stream", 42);
    let teacher_a = flag(args, "teacher-a").unwrap_or("amber");
    let teacher_b = flag(args, "teacher-b").unwrap_or("oak");
    let detach_day = fint(args, "detach-day", 21).max(1);
    let reattach_day = fint(args, "reattach-day", 26).max(detach_day);
    let autosave = flag(args, "no-autosave").is_none();
    let sleep_every = fint(args, "sleep-every", 0);
    let autonomy = flag(args, "autonomy").is_some();

    let mut brain = load_brain(args)?;
    brain.autosave = autosave;
    if autonomy {
        brain.autonomy.enabled = true;
    }
    let mut stream = Rng::new(stream_seed);
    const DAY: u64 = 8640;
    let mut cohort: Vec<u64> = Vec::new();
    let mut day1_strengths: Vec<f32> = Vec::new();
    let mut tokens_detach_window: u64 = 0;
    println!(
        "life: {} sim-days, stream seed {}, teacher A \"{}\" (days 1–{}), detached {}–{}, teacher B \"{}\" (day {}–), autosave {}",
        days, stream_seed, teacher_a, detach_day, detach_day, reattach_day - 1, teacher_b, reattach_day,
        if autosave { "on" } else { "off" }
    );
    for day in 1..=days {
        if day <= detach_day && brain.teacher.is_none() {
            brain.attach_teacher(teacher_a);
        }
        if day == detach_day {
            if brain.detach_teacher(teacher_a) {
                println!("  [day {day}] teacher \"{teacher_a}\" detached — substrate-only mode");
            }
        }
        if day >= reattach_day && brain.teacher_name.as_deref() != Some(teacher_b) {
            brain.attach_teacher(teacher_b);
            println!("  [day {day}] teacher \"{teacher_b}\" attached");
        }
        brain.ingest_text("good morning", 0.3, 0.3, "user");
        brain.run_ticks(300);
        for _ in 0..3 {
            let v = stream.next_f32_range(-0.4, 0.6);
            let kw = TOPIC_POOL[stream.next_u64_below(TOPIC_POOL.len() as u64) as usize];
            brain.ingest_text(kw, v, stream.next_f32_range(0.1, 0.6), "user");
            brain.run_ticks(200);
        }
        brain.ingest_text("evening check-in", 0.2, 0.2, "user");
        brain.run_ticks(600);
        if brain.teacher.is_some() {
            let before = brain.tokens_used;
            let reply = brain.utter("speak", "how was the day");
            if day > detach_day && day < reattach_day {
                tokens_detach_window += brain.tokens_used - before;
            }
            let _ = reply;
        }
        brain.run_ticks(DAY - 300 - 600 - 600);
        if sleep_every > 0 && day % sleep_every == 0 {
            let report = brain.sleep(1);
            let light = &report.stages[1].work; // replayed / recolored
            let deep = &report.stages[2].work; // pruned / gists
            println!(
                "  [day {day}] sleep: replayed {} recolored {} pruned {} gists {} dreams {}",
                light.replayed,
                light.recolored,
                deep.pruned,
                deep.gists,
                report.dreams.len()
            );
        }
        if day == 1 {
            for t in brain.episodic.traces.iter().take(3) {
                cohort.push(t.id);
                day1_strengths.push(t.strength);
            }
        }
        let mean_sal: f32 = if brain.episodic.traces.is_empty() {
            0.0
        } else {
            brain.episodic.traces.iter().map(|t| t.salience).sum::<f32>() / brain.episodic.traces.len() as f32
        };
        let mean_str: f32 = if brain.episodic.traces.is_empty() {
            0.0
        } else {
            brain.episodic.traces.iter().map(|t| t.strength).sum::<f32>() / brain.episodic.traces.len() as f32
        };
        let cohort_str: String = if day == 1 || day % 7 == 0 || day == days {
            let v: Vec<String> = cohort
                .iter()
                .map(|id| {
                    brain
                        .episodic
                        .traces
                        .iter()
                        .find(|t| t.id == *id)
                        .map(|t| format!("{:.3}", t.strength))
                        .unwrap_or_else(|| "pruned".into())
                })
                .collect();
            format!(" cohort:[{}]", v.join(","))
        } else {
            String::new()
        };
        println!(
            "day {:>2} | traces {:<4} nodes {:<3} meanSal {:.3} meanStr {:.3} valence {:+.3} tokens {}{}",
            day,
            brain.episodic.traces.len(),
            brain.semantic.nodes.len(),
            mean_sal,
            mean_str,
            brain.state.affect()[0],
            brain.tokens_used,
            cohort_str
        );
    }
    let report = brain.audit.run(&brain, "life-end");
    let alarms: Vec<&str> = report
        .metrics
        .iter()
        .filter(|m| m.alarm)
        .map(|m| m.id.as_str())
        .collect();
    println!("---");
    println!(
        "life complete: digest {:016x}, {} traces ({} pruned), {} nodes, {} events dropped, {} tokens",
        brain.digest(),
        brain.episodic.traces.len(),
        brain.episodic.pruned_count,
        brain.semantic.nodes.len(),
        brain.dropped_events,
        brain.tokens_used
    );
    println!(
        "  detach-window tokens: {} (must be 0)",
        tokens_detach_window
    );
    println!(
        "  initiatives: {} (unprompted speech, enabled: {}) — last: {}",
        brain.autonomy.total,
        brain.autonomy.enabled,
        brain
            .autonomy
            .log
            .last()
            .map(|e| format!("[{}] {}", e.kind, e.text))
            .unwrap_or_else(|| "(none)".to_string())
    );
    println!(
        "  cohort decay: day1 {:?} → day{} {:?}",
        day1_strengths.iter().map(|s| format!("{:.3}", s)).collect::<Vec<_>>(),
        days,
        cohort
            .iter()
            .map(|id| {
                brain
                    .episodic
                    .traces
                    .iter()
                    .find(|t| t.id == *id)
                    .map(|t| format!("{:.3}", t.strength))
                    .unwrap_or_else(|| "pruned".into())
            })
            .collect::<Vec<_>>()
    );
    println!("  audit alarms: {}", if alarms.is_empty() { "none".to_string() } else { alarms.join(", ") });
    let bytes = brain
        .save(&PathBuf::from(path), flag(args, "passphrase"))
        .map_err(|e| e.to_string())?;
    println!("  saved {bytes} bytes");
    Ok(())
}

fn cmd_watch(args: &Args) -> Result<(), String> {
    let ticks: u64 = flag(args, "ticks")
        .ok_or_else(|| "watch requires --ticks N".to_string())?
        .parse()
        .map_err(|_| "bad --ticks".to_string())?;
    let interval = fint(args, "interval", 100).max(1);
    let mut brain = load_brain(args)?;
    let mut remaining = ticks;
    while remaining > 0 {
        let step = interval.min(remaining);
        brain.run_ticks(step);
        remaining -= step;
        println!(
            "{}",
            serde_json::to_string(&brain.snapshot_json()).map_err(|e| e.to_string())?
        );
    }
    Ok(())
}
