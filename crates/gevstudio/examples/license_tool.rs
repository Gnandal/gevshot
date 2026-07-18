//! Outil hors-ligne du vendeur pour GEVSTUDIO Pro.
//!
//! - `keygen` : génère une paire de clés Ed25519. La clé PRIVÉE reste secrète
//!   (elle sert à émettre les licences) ; la clé PUBLIQUE est embarquée dans
//!   l'application (`src/license.rs`, constante `PUBLIC_KEY`).
//! - `sign <clé_privée_b64> <nom_client>` : émet une licence signée à donner
//!   au client. Elle ne peut être forgée sans la clé privée.
//!
//! Exemples :
//!   cargo run --example license_tool -- keygen
//!   cargo run --example license_tool -- sign <PRIV_B64> "Jean Dupont"

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64;
use ed25519_dalek::{Signer, SigningKey};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("keygen") => {
            let sk = SigningKey::generate(&mut rand::rngs::OsRng);
            let vk = sk.verifying_key();
            println!("Clé PRIVÉE (à garder secrète) :");
            println!("  {}", B64.encode(sk.to_bytes()));
            println!();
            println!("Clé PUBLIQUE (à coller dans src/license.rs) :");
            println!("const PUBLIC_KEY: [u8; 32] = {:?};", vk.to_bytes());
        }
        Some("sign") => {
            let Some(priv_b64) = args.get(1) else {
                return usage();
            };
            let Some(name) = args.get(2) else {
                return usage();
            };
            let bytes = match B64.decode(priv_b64) {
                Ok(b) if b.len() == 32 => b,
                _ => {
                    eprintln!("Clé privée invalide (base64url de 32 octets attendu).");
                    return;
                }
            };
            let arr: [u8; 32] = bytes.try_into().unwrap();
            let sk = SigningKey::from_bytes(&arr);
            let sig = sk.sign(name.as_bytes());
            // Licence = base64url(nom) "." base64url(signature).
            let license = format!(
                "{}.{}",
                B64.encode(name.as_bytes()),
                B64.encode(sig.to_bytes())
            );
            println!("{license}");
        }
        _ => usage(),
    }
}

fn usage() {
    eprintln!("usage :");
    eprintln!("  license_tool keygen");
    eprintln!("  license_tool sign <clé_privée_b64> <nom_client>");
}
