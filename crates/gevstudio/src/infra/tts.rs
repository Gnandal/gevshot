//! Intégration TTS (Text-to-Speech) : trait abstrait et implémentation stub.
//! Le fournisseur TTS n'est pas figé (ElevenLabs, Azure, Google, OpenAI).
//! Les appels partent dans un thread pour rester cohérents avec GEVSHOT/infra/ai.rs.

use std::sync::mpsc::Sender;

use gev_core::document::{NarrationAudio, AudioFormat};

pub enum TtsResult {
    Audio(NarrationAudio),
    Error(String),
}

/// Trait abstrait pour les fournisseurs TTS.
pub trait TtsProvider: Send + Sync {
    /// Synthétise un texte en audio narration.
    fn synthesize(&self, text: &str, voice_id: &str) -> Result<NarrationAudio, String>;
}

/// Implémentation stub (v1 : pas d'appel réseau, retourne une erreur).
pub struct StubTts;

impl TtsProvider for StubTts {
    fn synthesize(&self, _text: &str, _voice_id: &str) -> Result<NarrationAudio, String> {
        Err("TTS not implemented in v1".to_string())
    }
}

/// Lance une synthèse TTS dans un thread, le résultat revient via un canal.
/// Suit le pattern GEVSHOT/infra/ai.rs (thread + mpsc).
pub fn spawn_synthesize(
    provider: std::sync::Arc<dyn TtsProvider>,
    text: String,
    voice_id: String,
    tx: Sender<TtsResult>,
) {
    std::thread::spawn(move || {
        let result = match provider.synthesize(&text, &voice_id) {
            Ok(audio) => TtsResult::Audio(audio),
            Err(e) => TtsResult::Error(e),
        };
        let _ = tx.send(result);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_tts_returns_error() {
        let tts = StubTts;
        let result = tts.synthesize("test", "default");
        assert!(result.is_err());
    }
}
