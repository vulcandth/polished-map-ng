//! Package the pinned upstream multi-resolution Windows icon in both launchers.
fn main() {
    println!("cargo:rerun-if-changed=app.rc");
    println!("cargo:rerun-if-changed=assets/artwork/app.ico");
    embed_resource::compile("app.rc", embed_resource::NONE)
        .manifest_required()
        .expect("compile the original application icon resource");
}
