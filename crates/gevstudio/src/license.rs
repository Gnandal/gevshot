//! Licence GEVSTUDIO : version gratuite par défaut, déblocage « Pro » via une
//! clé à paiement unique.
//!
//! Les licences sont **signées Ed25519**. La clé privée reste chez le vendeur
//! (elle sert à émettre les licences avec `examples/license_tool.rs`) ; seule
//! la clé publique est embarquée ici. Forger une licence valide exige la clé
//! privée — un simple format ne suffit plus.
//!
//! Format d'une licence : `base64url(nom) "." base64url(signature)`. La
//! signature couvre le nom du licencié, ce qui rend chaque licence nominative
//! et infalsifiable sans la clé privée.

use std::path::PathBuf;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

/// Mode « tout gratuit » : tant qu'il vaut `true`, toutes les fonctions Pro
/// sont débloquées pour tout le monde (aucun filigrane, aucun verrou). Le
/// code de licence reste en place ; passer à `false` réactive la vente.
pub const FREE_MODE: bool = true;

/// Clé publique de vérification (la clé privée correspondante émet les
/// licences hors ligne — voir `examples/license_tool.rs`).
const PUBLIC_KEY: [u8; 32] = [
    248, 171, 163, 85, 190, 183, 142, 220, 37, 244, 129, 223, 80, 123, 41, 136, 93, 110, 121, 232,
    217, 230, 167, 251, 220, 181, 108, 243, 114, 199, 172, 247,
];

#[derive(Default)]
pub struct License {
    licensee: Option<String>,
}

impl License {
    /// Charge l'état de licence depuis le disque (gratuit si absent/invalide).
    pub fn load() -> License {
        match config_path().and_then(|p| std::fs::read_to_string(p).ok()) {
            Some(content) => match verify(content.trim()) {
                Ok(name) => License {
                    licensee: Some(name),
                },
                Err(_) => License::default(),
            },
            None => License::default(),
        }
    }

    pub fn is_pro(&self) -> bool {
        FREE_MODE || self.licensee.is_some()
    }

    /// Nom du licencié (pour affichage « Sous licence de … »).
    pub fn licensee(&self) -> Option<&str> {
        self.licensee.as_deref()
    }

    /// Tente d'activer une licence. En cas de succès, la persiste.
    pub fn activate(&mut self, input: &str) -> Result<(), String> {
        let license = input.trim();
        let name = verify(license)?;
        let path = config_path().ok_or("Dossier de configuration introuvable.")?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("Écriture impossible : {e}"))?;
        }
        std::fs::write(&path, license).map_err(|e| format!("Écriture impossible : {e}"))?;
        self.licensee = Some(name);
        Ok(())
    }
}

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("GEVSTUDIO").join("license.key"))
}

/// Vérifie une licence avec la clé publique embarquée et renvoie le nom du
/// licencié si la signature est authentique.
fn verify(license: &str) -> Result<String, String> {
    let vk = VerifyingKey::from_bytes(&PUBLIC_KEY).map_err(|_| "Clé interne invalide.".to_string())?;
    verify_with(&vk, license)
}

fn verify_with(vk: &VerifyingKey, license: &str) -> Result<String, String> {
    let (payload_b64, sig_b64) = license
        .split_once('.')
        .ok_or("Format de licence invalide.")?;
    let payload = B64
        .decode(payload_b64.trim())
        .map_err(|_| "Licence illisible.")?;
    let sig_bytes: [u8; 64] = B64
        .decode(sig_b64.trim())
        .map_err(|_| "Licence illisible.")?
        .try_into()
        .map_err(|_| "Signature de taille invalide.")?;
    let signature = Signature::from_bytes(&sig_bytes);
    vk.verify(&payload, &signature)
        .map_err(|_| "Licence non authentique.")?;
    String::from_utf8(payload).map_err(|_| "Licence corrompue.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    /// Fabrique une licence signée par `sk` pour `name`.
    fn issue(sk: &SigningKey, name: &str) -> String {
        let sig = sk.sign(name.as_bytes());
        format!(
            "{}.{}",
            B64.encode(name.as_bytes()),
            B64.encode(sig.to_bytes())
        )
    }

    #[test]
    fn accepts_a_genuine_license() {
        let sk = SigningKey::generate(&mut rand::rngs::OsRng);
        let vk = sk.verifying_key();
        let license = issue(&sk, "Jean Dupont");
        assert_eq!(verify_with(&vk, &license).unwrap(), "Jean Dupont");
    }

    #[test]
    fn rejects_wrong_key() {
        // Signée par une autre clé que celle attendue.
        let attacker = SigningKey::generate(&mut rand::rngs::OsRng);
        let expected = SigningKey::generate(&mut rand::rngs::OsRng).verifying_key();
        let license = issue(&attacker, "Pirate");
        assert!(verify_with(&expected, &license).is_err());
    }

    #[test]
    fn rejects_tampered_name() {
        let sk = SigningKey::generate(&mut rand::rngs::OsRng);
        let vk = sk.verifying_key();
        let license = issue(&sk, "Client A");
        // Remplace le nom sans re-signer.
        let (_, sig) = license.split_once('.').unwrap();
        let forged = format!("{}.{}", B64.encode(b"Client B"), sig);
        assert!(verify_with(&vk, &forged).is_err());
    }

    #[test]
    fn rejects_garbage() {
        let vk = SigningKey::generate(&mut rand::rngs::OsRng).verifying_key();
        assert!(verify_with(&vk, "pas-une-licence").is_err());
        assert!(verify_with(&vk, "abc.def").is_err());
    }
}
