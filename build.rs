fn main() -> Result<(), std::io::Error> {
    tonic_build::configure().compile_protos(
        &[
            "proto/commands.proto",
            "proto/heartbeat.proto",
        ],
        &["proto"],
    )
}
