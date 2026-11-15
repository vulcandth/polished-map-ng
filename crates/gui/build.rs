//! Package the selected multi-resolution Windows icon in all GUI launchers.
fn main() {
    println!("cargo:rerun-if-changed=app.rc");
    println!("cargo:rerun-if-changed=assets/application/app.ico");
    embed_resource::compile("app.rc", embed_resource::NONE)
        .manifest_required()
        .expect("compile the application icon resource");
}
