fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rerun-if-changed=windows/app.rc");
        println!("cargo:rerun-if-changed=windows/app-icon.ico");
        // rc.exe runs from the crate root and llvm-rc (cross-builds) from windows/, so the icon resolves through the include path.
        embed_resource::compile(
            "windows/app.rc",
            embed_resource::ParamsIncludeDirs([concat!(env!("CARGO_MANIFEST_DIR"), "/windows")]),
        )
        .manifest_required()
        .expect("failed to embed the Windows app icon");
    }
}
