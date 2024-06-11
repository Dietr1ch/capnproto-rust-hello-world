fn main() -> Result<(), Box<dyn std::error::Error>> {
    capnpc::CompilerCommand::new()
        .file("./schema/hello_world.capnp")
        .run()?;
    Ok(())
}
