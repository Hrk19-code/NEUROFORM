//! HTTP teacher adapter — the shell's transport for the LLM boundary
//! (DESIGN.md §4.17, master prompt §8). The brain-core side decides *what*
//! the LLM sees (UtterancePacket + build_teacher_prompt); this module
//! decides *how* to reach it: an OpenAI-compatible /chat/completions POST.
//!
//! Usage: `neuroform attach <path> --teacher https://host/v1` with
//! `--teacher-key` or `NEUROFORM_TEACHER_KEY`. `chat --teacher <url>`
//! works per-exchange too.
//!
//! Failure is never silent: endpoint errors surface as "(teacher error: …)"
//! in the utterance, and the file keeps running on memory alone.

use brain_core::boundary::{build_teacher_prompt, Teacher, UtterancePacket};

pub struct HttpTeacher {
    endpoint: String, // base, e.g. https://host/v1 (we append /chat/completions)
    key: String,
    model: String,
}

impl HttpTeacher {
    pub fn new(endpoint: &str, key: &str, model: &str) -> Self {
        HttpTeacher {
            endpoint: endpoint.trim_end_matches('/').to_string(),
            key: key.to_string(),
            model: model.to_string(),
        }
    }
}

impl Teacher for HttpTeacher {
    fn name(&self) -> &str {
        "http"
    }

    fn utter(&mut self, packet: &UtterancePacket) -> Result<String, String> {
        let url = format!("{}/chat/completions", self.endpoint);
        let system = build_teacher_prompt(packet, &packet.attention_focus);
        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": packet.attention_focus },
            ],
            "temperature": 0.8,
            "max_tokens": 300,
        });
        let mut req = ureq::post(&url)
            .timeout(std::time::Duration::from_secs(60))
            .set("Content-Type", "application/json");
        if !self.key.is_empty() {
            req = req.set("Authorization", &format!("Bearer {}", self.key));
        }
        let resp = req
            .send_json(body)
            .map_err(|e| format!("endpoint error: {e}"))?;
        let json: serde_json::Value = resp
            .into_json()
            .map_err(|e| format!("bad response JSON: {e}"))?;
        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| "no choices[0].message.content in response".to_string())?
            .trim()
            .to_string();
        if content.is_empty() {
            return Err("empty completion".into());
        }
        Ok(content)
    }
}
