const AUTH_SERVICE_PROTO: &str = "proto/auth_service.proto";
const AUTH_SERVICE_PROTO_DESCRIPTOR: &str = "proto/proto_descriptor.bin";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .file_descriptor_set_path(AUTH_SERVICE_PROTO_DESCRIPTOR)
        .compile_protos(&[AUTH_SERVICE_PROTO], &["proto/"])?;
    Ok(())
}
