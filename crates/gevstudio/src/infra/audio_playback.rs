//! Lecture audio synchrone (v1 : stub, implémentation rodio en v2).
//!
//! Sera utilisée par le mode Présentation pour synchroniser narration et durée.
//! L'implémentation rodio sera ajoutée une fois le reste du pipeline opérationnel.

/// Joue des octets MP3/WAV de façon synchrone (bloquante).
/// Version v1 : stub qui simule la lecture (pour dev/test sans audio system).
pub fn play_audio(bytes: &[u8]) -> Result<(), String> {
    if bytes.is_empty() {
        return Err("audio: empty buffer".to_string());
    }

    // v1 : simplement valider que c'est du contenu audio (au moins quelques octets).
    // v2 : implémentation rodio pour lecture réelle.
    eprintln!("mock audio playback: {} bytes", bytes.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn play_audio_stub_accepts_data() {
        let result = play_audio(b"mock audio data");
        assert!(result.is_ok());
    }

    #[test]
    fn play_audio_rejects_empty() {
        let result = play_audio(b"");
        assert!(result.is_err());
    }
}
