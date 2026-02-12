fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        // .build_server(true) // Default is true, but good to be explicit
        .compile_protos(
            &["proto/sentinel_service.proto"], // Ruta a tu archivo .proto
            &["proto"],              // Directorio de inclusión
        )?;
    Ok(())
}
