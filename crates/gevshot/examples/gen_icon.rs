// Génère les PNG de l'icône pour le packaging Linux (.desktop) et
// macOS (.icns) : `cargo run --example gen_icon -- <dossier> <tailles…>`.
// Même source dessinée par code que l'exe Windows (via build.rs).

include!("../src/appicon.rs");

fn main() {
    let mut args = std::env::args().skip(1);
    let dir = std::path::PathBuf::from(
        args.next()
            .expect("usage : gen_icon <dossier> <tailles…>"),
    );
    std::fs::create_dir_all(&dir).expect("création du dossier de sortie");
    for arg in args {
        let size: u32 = arg.parse().expect("taille entière en pixels");
        let path = dir.join(format!("gevshot-{size}.png"));
        image::save_buffer(
            &path,
            &icon_rgba(size),
            size,
            size,
            image::ExtendedColorType::Rgba8,
        )
        .expect("écriture du PNG");
        println!("{}", path.display());
    }
}
