fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("greet/v1/greet.proto")?;
    Ok(())
}