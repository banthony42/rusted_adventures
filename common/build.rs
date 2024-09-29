use std::{fs::create_dir, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from("./GRPC_codegen");
    let _ = create_dir(out_dir.clone());

    tonic_build::configure()
        .out_dir(out_dir)
        .compile_protos(&["../proto/authentication.proto"], &[".."])?;
    Ok(())
}
