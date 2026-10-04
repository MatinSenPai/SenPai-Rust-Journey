// Compiles proto/notes.proto into Rust types + a server/client trait pair,
// generated into OUT_DIR and pulled in via `tonic::include_proto!("notes")`
// in src/lib.rs. See README.md for why `PROTOC` is pointed at a
// cargo-fetched vendored binary instead of relying on a system install.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protoc_path = protoc_bin_vendored::protoc_bin_path()?;
    // SAFETY (not `unsafe` Rust, just a build-script caveat): build scripts
    // are single-threaded at this point, so mutating process env here is
    // fine — this only affects the `protoc-build`/`prost-build` subprocess
    // this same build.rs invokes next, not the compiled crate's runtime.
    std::env::set_var("PROTOC", protoc_path);
    tonic_build::compile_protos("proto/notes.proto")?;
    Ok(())
}
