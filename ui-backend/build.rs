fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../proto/string_list.proto")?;
    tonic_build::compile_protos("../proto/map_data.proto")?;
    Ok(())
}
