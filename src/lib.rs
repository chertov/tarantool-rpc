#[macro_use] extern crate serde;

mod rpc;
mod cargo;

pub fn generate(traits_path: &str, output_path: &str, crate_name: &str, tarantool: Option<String>) -> Result<(), anyhow::Error> {
    let traits_yaml = {
        let data = std::fs::read(traits_path)?;
        String::from_utf8(data)?
    };
    let traits = rpc::Traits::new(traits_yaml)?;

    let mut output_path = std::path::PathBuf::from(output_path);
    output_path.push(crate_name);

    let _ = std::fs::remove_dir_all(&output_path);
    std::fs::create_dir_all(&output_path)?;
    cargo::generate(&output_path, crate_name, &traits, tarantool)?;
    output_path.push("src");

    std::fs::create_dir_all(&output_path)?;

    traits.generate(crate_name, output_path.to_path_buf())?;
    Ok(())
}
