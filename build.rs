fn main() {
    println!("cargo:rerun-if-changed=assets/profile.png");
    println!("cargo:rustc-check-cfg=cfg(bundled_portrait)");
    if std::path::Path::new("assets/profile.png").is_file() {
        println!("cargo:rustc-cfg=bundled_portrait");
    }
}
