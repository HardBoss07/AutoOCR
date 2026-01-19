use std::fs;
use std::path::Path;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    // This finds the /target/release or /target/debug folder
    let target_dir = Path::new(&out_dir).join("../../../");

    let assets_src = Path::new("assets");
    
    // Copy tessdata
    let tess_src = assets_src.join("tessdata");
    let tess_dst = target_dir.join("tessdata");
    if tess_src.exists() {
        copy_dir(&tess_src, &tess_dst).ok();
    }

    // Copy favicon.ico
    let icon_src = assets_src.join("favicon.ico");
    let icon_dst = target_dir.join("favicon.ico");
    if icon_src.exists() {
        fs::copy(icon_src, icon_dst).ok();
    }
    
    println!("cargo:rerun-if-changed=assets/");
}

fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        fs::copy(entry.path(), dst.join(entry.file_name()))?;
    }
    Ok(())
}