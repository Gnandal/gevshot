//! Génère l'icône .ico (multi-tailles) et l'embarque dans l'exe Windows.

include!("src/appicon.rs");

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/appicon.rs");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("gevshot.ico");
    write_ico(&out);
    let mut res = winresource::WindowsResource::new();
    res.set_icon(out.to_str().expect("chemin .ico"));
    res.set("ProductName", "GEVSHOT");
    res.set("FileDescription", "GEVSHOT — capture d'écran annotée");
    res.compile().expect("intégration de l'icône dans l'exe");
}

fn write_ico(path: &std::path::Path) {
    use image::codecs::ico::{IcoEncoder, IcoFrame};
    let sizes = [16u32, 24, 32, 48, 64, 128, 256];
    let frames: Vec<IcoFrame> = sizes
        .iter()
        .map(|&s| {
            let rgba = icon_rgba(s);
            IcoFrame::as_png(&rgba, s, s, image::ExtendedColorType::Rgba8)
                .expect("encodage d'une taille d'icône")
        })
        .collect();
    let file = std::fs::File::create(path).expect("création du .ico");
    IcoEncoder::new(file)
        .encode_images(&frames)
        .expect("écriture du .ico");
}
