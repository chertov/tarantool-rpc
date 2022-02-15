fn main() -> Result<(), anyhow::Error> {
    let cargo_path = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut output_path =  cargo_path.clone();
    output_path += "/../";
    let mut traits_path =  output_path.clone();
    traits_path += "traits.yaml";

    let tarantool = "{ version = \"0.5.1\", optional = true }".to_string();
    // let tarantool = "{ git = \"https://github.com/chertov/tarantool-module.git\", branch = \"dev_old\", features=[\"schema\"], optional = true }".to_string();

    tarantool_rpc::generate(&traits_path, &output_path, "my-simple-rpc", Some(tarantool))?;

    println!("cargo:rustc-link-arg=-Wl,-undefined,dynamic_lookup");
    Ok(())
}
