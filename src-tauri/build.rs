fn main() {
    std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path().unwrap());

    prost_build::Config::new()
        .compile_protos(&["proto/appearances.proto"], &["proto/"])
        .expect("failed to compile protobuf definitions");

    tauri_build::build();
}
