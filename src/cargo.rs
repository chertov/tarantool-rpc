pub(crate) fn generate(output_path: &std::path::Path, crate_name: &str, traits: &super::rpc::Traits, tarantool: Option<String>) -> Result<(), anyhow::Error> {
    let mut cargo_path = output_path.to_path_buf();
    cargo_path.push("Cargo.toml");

    let mut src = format!("");
    src += &format!("[package]\n");
    src += &format!("name = \"{}\"\n", crate_name);
    src += &format!("version = \"0.1.0\"\n");
    src += &format!("authors = [\"Codegen TNT-RPC\"]\n");
    src += &format!("edition = \"2021\"\n");
    src += &format!("\n");
    src += &format!("[lib]\n");
    src += &format!("crate-type = [\"cdylib\", \"staticlib\", \"rlib\"]\n");
    src += &format!("\n");
    src += &format!("[features]\n");
    src += &format!("tnt_impl = [\"log\", \"parking_lot\", \"once_cell\", \"byteorder\", \"os_pipe\", \"futures\", \"tokio\", \"tarantool\"]\n");
    src += &format!("\n");
    src += &format!("[dependencies]\n");
    src += &format!("anyhow = \"1\"\n");
    src += &format!("async-trait = \"0.1\"\n");
    src += &format!("\n");
    src += &format!("log = {{ version = \"0.4\", optional = true }}\n");
    src += &format!("parking_lot = {{ version = \"0.11\", optional = true }}\n");
    src += &format!("once_cell = {{ version = \"1.8\", optional = true }}\n");
    src += &format!("byteorder = {{ version = \"1\", optional = true }}\n");
    src += &format!("os_pipe = {{ version = \"0.9\", optional = true }}\n");
    src += &format!("futures = {{ version = \"0.3\", optional = true }}\n");
    src += &format!("tokio = {{ version = \"1\", features = [\"time\"], optional = true }}\n");
    src += &format!("\n");

    src += &match tarantool {
        Some(tarantool) =>
            format!("# override default tarantool package with value from tarantool_rpc::generate() function\n") +
                &format!("tarantool = {tarantool}\n"),
        None => match traits.tarantool() {
            Some(tarantool) =>
                format!("# override default tarantool package with value from YAML trait description\n") +
                    &format!("tarantool = {tarantool}\n"),
            None => format!("tarantool = {{ version = \"*\", optional = true }}\n")
        }
    };
    src += &format!("\n");

    let mut dependencies = traits.dependencies();
    if !dependencies.is_empty() {
        src += &format!("################################################\n");
        src += &format!("### dependencies from YAML trait description ###\n");
        src += &format!("################################################\n");
    }
    for (name, value) in dependencies {
        src += &format!("{name} = {value}\n");
    }

    std::fs::write(cargo_path, src)?;
    Ok(())
}
