fn main() {
    prost_build::Config::new()
        .type_attribute(".", "#[derive(serde_derive::Serialize)]")
        .type_attribute(".", "#[derive(serde_derive::Deserialize)]")
        .compile_protos(&["../protos/core.proto"], &["../protos/"])
        .unwrap();
}
