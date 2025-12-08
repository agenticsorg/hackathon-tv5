fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "grpc")]
    {
        tonic_build::configure()
            .build_server(true)
            .build_client(true)
            .compile(
                &["proto/sync.proto"],
                &["proto"],
            )?;
    }
    Ok(())
}
