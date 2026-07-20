#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! GEVSTUDIO — assemble des captures annotées `.gevd` en documents.
//!
//! Deux modes :
//! - sans argument : lance l'interface (timeline des étapes, aperçu, export) ;
//! - `build <doc.gevd> [-o <sortie.pdf>]` : génère un PDF en ligne de commande.

use std::path::PathBuf;
use std::process::ExitCode;

use gev_core::document;

mod app;
mod app_impl;
mod domain;
mod infra;
mod license;
mod ui;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        // Mode CLI explicite.
        Some("build") => cli(build(&args[1..])),
        Some("-h") | Some("--help") => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        // Sans argument (ou un chemin) : interface graphique. Un `.gevd`
        // passé en argument est ouvert au démarrage.
        None => app_impl::launch(None),
        Some(_) => app_impl::launch(Some(PathBuf::from(&args[0]))),
    }
}

fn cli(result: Result<(), String>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("erreur : {e}");
            ExitCode::FAILURE
        }
    }
}

const USAGE: &str = "Usage :\n  gevstudio                        lance l'interface\n  gevstudio <doc.gevd>             ouvre un document dans l'interface\n  gevstudio build <doc.gevd> [-o <sortie.pdf>]   génère un PDF sans interface";

struct BuildArgs {
    input: PathBuf,
    output: PathBuf,
}

fn parse_build(args: &[String]) -> Result<BuildArgs, String> {
    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-o" | "--output" => {
                let v = it.next().ok_or("-o attend un chemin")?;
                output = Some(PathBuf::from(v));
            }
            _ if input.is_none() => input = Some(PathBuf::from(arg)),
            _ => return Err(format!("argument inattendu « {arg} »")),
        }
    }
    let input = input.ok_or_else(|| format!("fichier .gevd manquant\n\n{USAGE}"))?;
    let output = output.unwrap_or_else(|| input.with_extension("pdf"));
    Ok(BuildArgs { input, output })
}

fn build(args: &[String]) -> Result<(), String> {
    let BuildArgs { input, output } = parse_build(args)?;
    let doc = document::load(&input)?;
    if doc.steps.is_empty() {
        return Err("le document ne contient aucune étape".into());
    }
    // Le format de sortie suit l'extension : .pptx → PowerPoint, .gif → GIF
    // animé, sinon PDF.
    let ext = output
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    let pro = license::License::load().is_pro();
    // PPTX et GIF sont réservés à la version Pro ; le PDF gratuit est filigrané.
    let pro_only = |kind: &str| {
        if pro {
            Ok(())
        } else {
            Err(format!(
                "L'export {kind} est réservé à la version Pro. Activez une licence dans l'application."
            ))
        }
    };
    let (bytes, kind) = match ext.as_str() {
        "pptx" => {
            pro_only("PPTX")?;
            (infra::export::build_pptx(&doc)?, "PPTX")
        }
        "gif" => {
            pro_only("GIF")?;
            (infra::export::build_gif(&doc)?, "GIF")
        }
        "mp4" => {
            pro_only("MP4")?;
            (infra::export::build_mp4(&doc)?, "MP4")
        }
        _ => (infra::export::build_pdf(&doc, !pro)?, "PDF"),
    };
    std::fs::write(&output, &bytes)
        .map_err(|e| format!("écriture du fichier impossible : {e}"))?;
    let title = if doc.title.is_empty() {
        "(sans titre)"
    } else {
        &doc.title
    };
    println!(
        "« {title} » — {} étape(s) → {} [{kind}] ({} Ko)",
        doc.steps.len(),
        output.display(),
        bytes.len() / 1024
    );
    Ok(())
}
