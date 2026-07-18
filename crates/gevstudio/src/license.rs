//! Licence GEVSTUDIO : version gratuite par défaut, déblocage « Pro » via une
//! clé à paiement unique. Validation hors-ligne (format + checksum) et
//! persistance dans le dossier de configuration de l'utilisateur.
//!
//! Format de clé : `GEVS-XXXX-XXXX-XXXX` — préfixe `GEVS` puis 12 caractères
//! d'un alphabet sans ambiguïté ; le dernier caractère est un checksum des 11
//! précédents. Ce n'est pas un verrou cryptographique (déblocage local), mais
//! ça évite qu'une chaîne quelconque active la version Pro.

use std::path::PathBuf;

/// Alphabet Crockford (sans I, L, O, U, 0, 1) — 32 symboles.
const ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
const PREFIX: &str = "GEVS";

#[derive(Default)]
pub struct License {
    key: Option<String>,
}

impl License {
    /// Charge l'état de licence depuis le disque (gratuit si absent/invalide).
    pub fn load() -> License {
        match config_path().and_then(|p| std::fs::read_to_string(p).ok()) {
            Some(content) => {
                let key = content.trim().to_string();
                if is_valid_key(&key) {
                    License { key: Some(key) }
                } else {
                    License::default()
                }
            }
            None => License::default(),
        }
    }

    pub fn is_pro(&self) -> bool {
        self.key.is_some()
    }

    /// Tente d'activer une clé. En cas de succès, la persiste.
    pub fn activate(&mut self, input: &str) -> Result<(), String> {
        let key = normalize(input);
        if !is_valid_key(&key) {
            return Err("Clé de licence invalide.".into());
        }
        let path = config_path().ok_or("Dossier de configuration introuvable.")?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("Écriture impossible : {e}"))?;
        }
        std::fs::write(&path, &key).map_err(|e| format!("Écriture impossible : {e}"))?;
        self.key = Some(key);
        Ok(())
    }
}

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("GEVSTUDIO").join("license.key"))
}

/// Normalise une saisie : majuscules, sans espaces ni tirets superflus, avec
/// des tirets tous les 4 caractères (GEVS-XXXX-XXXX-XXXX).
fn normalize(input: &str) -> String {
    let raw: String = input
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect();
    raw.as_bytes()
        .chunks(4)
        .map(|c| std::str::from_utf8(c).unwrap_or(""))
        .collect::<Vec<_>>()
        .join("-")
}

fn is_valid_key(key: &str) -> bool {
    let compact: String = key.chars().filter(|c| *c != '-').collect();
    let Some(payload) = compact.strip_prefix(PREFIX) else {
        return false;
    };
    if payload.len() != 12 || !payload.bytes().all(|b| ALPHABET.contains(&b)) {
        return false;
    }
    let bytes = payload.as_bytes();
    checksum(&bytes[..11]) == bytes[11]
}

/// Caractère de contrôle = ALPHABET[(somme des valeurs) % 32].
fn checksum(payload: &[u8]) -> u8 {
    let sum: usize = payload
        .iter()
        .map(|b| ALPHABET.iter().position(|a| a == b).unwrap_or(0))
        .sum();
    ALPHABET[sum % 32]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fabrique une clé valide à partir de 11 caractères de charge utile.
    fn make_key(payload11: &str) -> String {
        let bytes = payload11.as_bytes();
        assert_eq!(bytes.len(), 11);
        let check = checksum(bytes) as char;
        normalize(&format!("{PREFIX}{payload11}{check}"))
    }

    #[test]
    fn valid_key_round_trips() {
        let key = make_key("7K2P9QX4A2B");
        assert!(is_valid_key(&key), "clé générée : {key}");
    }

    #[test]
    fn rejects_garbage() {
        assert!(!is_valid_key("hello"));
        assert!(!is_valid_key("GEVS-0000-0000-0000")); // 0 hors alphabet
        // Bon format mais mauvais checksum.
        assert!(!is_valid_key("GEVS-AAAA-AAAA-AAAB"));
    }

    #[test]
    fn activation_normalizes_input() {
        let key = make_key("MNPQRSTVWXY");
        let compact = key.replace('-', "").to_lowercase();
        assert!(is_valid_key(&normalize(&compact)));
    }
}
