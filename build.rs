const AUTH_SERVICE_PROTO: &str = "proto/auth_service.proto";
const AUTH_SERVICE_PROTO_DESCRIPTOR: &str = "proto/proto_descriptor.bin";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    rerun_proto()?;
    rerun_migrations()?;

    Ok(())
}

fn rerun_proto() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .file_descriptor_set_path(AUTH_SERVICE_PROTO_DESCRIPTOR)
        .compile_protos(&[AUTH_SERVICE_PROTO], &["proto/"])?;

    println!("cargo:rerun-if-changed={}", AUTH_SERVICE_PROTO);
    println!("cargo:rerun-if-changed=proto/");

    Ok(())
}

fn rerun_migrations() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=migrations");

    Ok(())
}
