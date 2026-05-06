fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("failed to get vendored protoc");
    // SAFETY: build script process-local env assignment for prost build.
    unsafe { std::env::set_var("PROTOC", protoc); }
    println!("cargo:rerun-if-changed=proto/blog.proto");
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(&["proto/blog.proto"], &["proto"])
        .expect("failed to compile blog proto");
}
