//! Intégration de l'API Claude (Anthropic) : génération de légende et
//! annotation sur instruction. Les appels partent dans un thread et le
//! résultat revient à l'interface via un canal mpsc.

use std::sync::mpsc::Sender;
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use eframe::egui::{Color32, Rect, pos2, vec2};
use serde_json::json;

use crate::domain::annotation::{Annotation, FontChoice, Shape};

const API_URL: &str = "https://api.anthropic.com/v1/messages";
const API_VERSION: &str = "2023-06-01";
/// Long côté maximal envoyé au modèle (limite de résolution utile).
const MAX_EDGE: u32 = 1568;

pub enum AiResult {
    /// Textes d'étape par numéro de pastille.
    Legend(Vec<(u32, String)>),
    /// Annotations générées, prêtes à insérer.
    Annotate(Vec<Annotation>),
    Error(String),
}

/// Style courant de l'éditeur, appliqué aux annotations générées.
#[derive(Clone)]
pub struct AiStyle {
    pub color: Color32,
    pub stroke_width: f32,
    pub text_size: f32,
    pub font: FontChoice,
    pub bold: bool,
    pub badge_border: bool,
}

/// Capture le style courant de l'éditeur partagé.
pub fn style_from_editor(editor: &crate::domain::editor::Editor) -> AiStyle {
    AiStyle {
        color: editor.color(),
        stroke_width: editor.stroke_width(),
        text_size: editor.text_size(),
        font: editor.text_font(),
        bold: editor.text_bold(),
        badge_border: editor.badge_border(),
    }
}

/// Redimensionne si nécessaire, encode en PNG base64.
/// Retourne (base64, facteur image envoyée → image originale, largeur, hauteur).
fn prepare_image(img: &image::RgbaImage) -> Result<(String, f32, u32, u32), String> {
    let (w, h) = img.dimensions();
    let long = w.max(h);
    let (encoded, factor, sw, sh) = if long > MAX_EDGE {
        let s = MAX_EDGE as f32 / long as f32;
        let (nw, nh) = (
            ((w as f32 * s) as u32).max(1),
            ((h as f32 * s) as u32).max(1),
        );
        let resized = image::imageops::resize(img, nw, nh, image::imageops::FilterType::Triangle);
        (resized, 1.0 / s, nw, nh)
    } else {
        (img.clone(), 1.0, w, h)
    };
    let mut png = Vec::new();
    encoded
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .map_err(|e| format!("encodage PNG impossible : {e}"))?;
    Ok((BASE64.encode(&png), factor, sw, sh))
}

/// Appel bloquant à l'API Messages avec sortie structurée (JSON Schema) :
/// la réponse est garantie conforme au schéma.
fn request(
    api_key: &str,
    model: &str,
    image_b64: &str,
    prompt: &str,
    schema: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let body = json!({
        "model": model,
        "max_tokens": 8192,
        "messages": [{
            "role": "user",
            "content": [
                {
                    "type": "image",
                    "source": {"type": "base64", "media_type": "image/png", "data": image_b64}
                },
                {"type": "text", "text": prompt}
            ]
        }],
        "output_config": {"format": {"type": "json_schema", "schema": schema}}
    });

    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(180))
        .build();
    let response = agent
        .post(API_URL)
        .set("x-api-key", api_key)
        .set("anthropic-version", API_VERSION)
        .set("content-type", "application/json")
        .send_json(body);

    match response {
        Ok(resp) => {
            let v: serde_json::Value = resp
                .into_json()
                .map_err(|e| format!("réponse illisible : {e}"))?;
            if v["stop_reason"] == "refusal" {
                return Err("la requête a été refusée par le modèle".to_string());
            }
            if v["stop_reason"] == "max_tokens" {
                return Err("réponse tronquée (limite de tokens atteinte)".to_string());
            }
            let text = v["content"]
                .as_array()
                .and_then(|blocks| blocks.iter().find(|b| b["type"] == "text"))
                .and_then(|b| b["text"].as_str())
                .ok_or("réponse sans contenu texte")?;
            serde_json::from_str(text).map_err(|e| format!("JSON inattendu : {e}"))
        }
        Err(ureq::Error::Status(code, resp)) => {
            let v: serde_json::Value = resp.into_json().unwrap_or_default();
            let msg = v["error"]["message"].as_str().unwrap_or("").to_string();
            Err(match code {
                401 => "clé API invalide (Paramètres → IA)".to_string(),
                429 => "limite de débit atteinte, réessayez dans un instant".to_string(),
                _ => format!("erreur API {code} : {msg}"),
            })
        }
        Err(e) => Err(format!("connexion impossible : {e}")),
    }
}

/// Génère les textes d'étape de la légende à partir de l'image annotée.
pub fn spawn_legend(
    api_key: String,
    model: String,
    image: image::RgbaImage,
    numbers: Vec<u32>,
    tx: Sender<AiResult>,
) {
    std::thread::spawn(move || {
        let outcome = (|| -> Result<AiResult, String> {
            let (b64, _factor, w, h) = prepare_image(&image)?;
            let nums = numbers
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            let prompt = format!(
                "Cette capture d'écran ({w}×{h} px) d'une interface logicielle contient des \
                 pastilles numérotées ({nums}). Pour chaque numéro, rédige en français le texte \
                 de l'étape correspondante pour un guide utilisateur : une phrase courte à \
                 l'impératif décrivant l'action sur l'élément que la pastille désigne."
            );
            let schema = json!({
                "type": "object",
                "properties": {
                    "steps": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "number": {"type": "integer"},
                                "text": {"type": "string"}
                            },
                            "required": ["number", "text"],
                            "additionalProperties": false
                        }
                    }
                },
                "required": ["steps"],
                "additionalProperties": false
            });
            let v = request(&api_key, &model, &b64, &prompt, schema)?;
            let steps = v["steps"]
                .as_array()
                .ok_or("réponse inattendue")?
                .iter()
                .filter_map(|s| {
                    Some((
                        s["number"].as_u64()? as u32,
                        s["text"].as_str()?.to_string(),
                    ))
                })
                .collect();
            Ok(AiResult::Legend(steps))
        })();
        let _ = tx.send(outcome.unwrap_or_else(AiResult::Error));
    });
}

/// Génère des annotations à partir d'une instruction en langage naturel.
pub fn spawn_annotate(
    api_key: String,
    model: String,
    image: image::RgbaImage,
    instruction: String,
    style: AiStyle,
    tx: Sender<AiResult>,
) {
    std::thread::spawn(move || {
        let outcome = (|| -> Result<AiResult, String> {
            let (b64, factor, w, h) = prepare_image(&image)?;
            let prompt = format!(
                "Capture d'écran d'une interface logicielle, {w}×{h} pixels.\n\
                 Instruction : {instruction}\n\n\
                 Génère les annotations demandées, positionnées précisément sur les éléments \
                 d'interface concernés. Coordonnées en pixels de cette image. Types :\n\
                 - badge : pastille numérotée, x,y = centre ; label = texte d'étape court ;\n\
                 - arrow : flèche, x,y = point de départ (zone vide), x2,y2 = pointe posée \
                   sur l'élément cible ;\n\
                 - rect et ellipse : encadrement, x,y = coin haut-gauche, w,h = dimensions, \
                   englobant l'élément avec une petite marge ;\n\
                 - text : étiquette, x,y = coin haut-gauche (zone vide proche), text = contenu.\n\
                 N'annote que ce que demande l'instruction."
            );
            let schema = json!({
                "type": "object",
                "properties": {
                    "annotations": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "kind": {"type": "string", "enum": ["badge", "arrow", "rect", "ellipse", "text"]},
                                "x": {"type": "number"},
                                "y": {"type": "number"},
                                "x2": {"type": "number"},
                                "y2": {"type": "number"},
                                "w": {"type": "number"},
                                "h": {"type": "number"},
                                "text": {"type": "string"},
                                "label": {"type": "string"}
                            },
                            "required": ["kind", "x", "y"],
                            "additionalProperties": false
                        }
                    }
                },
                "required": ["annotations"],
                "additionalProperties": false
            });
            let v = request(&api_key, &model, &b64, &prompt, schema)?;
            let mut annotations = Vec::new();
            for item in v["annotations"].as_array().ok_or("réponse inattendue")? {
                let get = |k: &str| item[k].as_f64().map(|x| x as f32 * factor);
                let x = get("x").unwrap_or(0.0);
                let y = get("y").unwrap_or(0.0);
                let shape = match item["kind"].as_str().unwrap_or("") {
                    "badge" => Shape::Badge {
                        center: pos2(x, y),
                        number: 0, // renuméroté à l'insertion
                        label: item["label"].as_str().unwrap_or("").to_string(),
                        bordered: style.badge_border,
                    },
                    "arrow" => Shape::Arrow {
                        from: pos2(x, y),
                        to: pos2(get("x2").unwrap_or(x + 60.0), get("y2").unwrap_or(y + 60.0)),
                    },
                    "rect" => Shape::Rect {
                        rect: Rect::from_min_size(
                            pos2(x, y),
                            vec2(
                                get("w").unwrap_or(120.0).max(4.0),
                                get("h").unwrap_or(60.0).max(4.0),
                            ),
                        ),
                    },
                    "ellipse" => Shape::Ellipse {
                        rect: Rect::from_min_size(
                            pos2(x, y),
                            vec2(
                                get("w").unwrap_or(120.0).max(4.0),
                                get("h").unwrap_or(60.0).max(4.0),
                            ),
                        ),
                    },
                    "text" => Shape::Text {
                        pos: pos2(x, y),
                        text: item["text"].as_str().unwrap_or("…").to_string(),
                        size: style.text_size,
                        font: style.font,
                        bold: style.bold,
                    },
                    _ => continue,
                };
                annotations.push(Annotation {
                    shape,
                    color: style.color,
                    stroke_width: style.stroke_width,
                });
            }
            if annotations.is_empty() {
                return Err("aucune annotation générée".to_string());
            }
            Ok(AiResult::Annotate(annotations))
        })();
        let _ = tx.send(outcome.unwrap_or_else(AiResult::Error));
    });
}
