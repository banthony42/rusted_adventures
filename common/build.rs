use std::{fs::create_dir, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from("./GRPC_codegen");
    let _ = create_dir(out_dir.clone());

    let proto_file_root_folder = ["../proto"];
    let proto_files = ["authentication.proto", "chat/chat.proto"];

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("rpg_services_descriptor.bin"))
        // .protoc_arg("--experimental_allow_proto3_optional")
        .out_dir(out_dir)
        .compile_protos(&proto_files, &proto_file_root_folder)?;
    Ok(())
}
