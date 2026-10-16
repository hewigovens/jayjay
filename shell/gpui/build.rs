fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rerun-if-changed=windows/app.rc");
        println!("cargo:rerun-if-changed=windows/app-icon.ico");
        embed_resource::compile("windows/app.rc", embed_resource::NONE)
            .manifest_required()
            .expect("failed to embed the Windows app icon");
    }
}
