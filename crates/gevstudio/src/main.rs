//! GEVSTUDIO — assemble des captures annotées `.gevd` en documents.
//! Étape 1 : CLI. `gevstudio build <doc.gevd> -o <sortie.pdf>`.

use std::path::PathBuf;
use std::process::ExitCode;

use gev_core::document;

mod pdf;

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("erreur : {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let cmd = args.first().map(String::as_str);
    match cmd {
        Some("build") => build(&args[1..]),
        Some(other) => Err(format!("commande inconnue « {other} »\n\n{USAGE}")),
        None => Err(format!("commande manquante\n\n{USAGE}")),
    }
}

const USAGE: &str = "Usage :\n  gevstudio build <doc.gevd> [-o <sortie.pdf>]";

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
    let bytes = pdf::build(&doc)?;
    std::fs::write(&output, &bytes).map_err(|e| format!("écriture du PDF impossible : {e}"))?;
    let title = if doc.title.is_empty() {
        "(sans titre)"
    } else {
        &doc.title
    };
    println!(
        "« {title} » — {} étape(s) → {} ({} Ko)",
        doc.steps.len(),
        output.display(),
        bytes.len() / 1024
    );
    Ok(())
}
