
mod method;
mod traits;

pub(crate) use traits::Traits;

impl Traits {
    pub(crate) fn new(traits_yaml: String) -> Result<Self, anyhow::Error> {
        let traits: traits::Traits = serde_yaml::from_str(&traits_yaml)?;
        traits.validate()
    }
    pub(crate) fn generate(&self, crate_name: &str, output_path: std::path::PathBuf) -> Result<(), anyhow::Error> {
        let rpc_path = {
            let mut path = output_path.clone();
            std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path)?;
            path
        };

        let mod_rs_path = {
            let mut path = rpc_path.clone();
            path.push("lib.rs");
            path
        };

        let traits_rs_path = {
            let mut path = rpc_path.clone();
            path.push("traits.rs");
            path
        };
        let mut traits_rs = format!("");
        traits_rs += "#![allow(non_snake_case)]\n";
        traits_rs += "\n";
        traits_rs += &self.code_trait();
        traits_rs += "\n";
        std::fs::write(traits_rs_path, traits_rs)?;

        let impls_rs_path = {
            let mut path = rpc_path.clone();
            path.push("impls.rs");
            path
        };
        let mut impls_rs = format!("");
        impls_rs += "#![allow(non_snake_case)]\n";
        impls_rs += "\n";
        impls_rs += "#[cfg(feature=\"tnt_impl\")]\n";
        impls_rs += "use super::tnt::*;\n";
        impls_rs += "\n";
        impls_rs += "pub(crate) const TIMEOUT : std::time::Duration = std::time::Duration::from_secs(3);\n";
        impls_rs += "\n";
        impls_rs += &self.code_trait_impl(crate_name);
        std::fs::write(impls_rs_path, impls_rs)?;

        let mut mod_rs = format!("");
        mod_rs += "\n";
        mod_rs += "pub mod impls;\n";
        mod_rs += "pub use impls::*;\n";
        mod_rs += "\n";
        mod_rs += "#[cfg(feature=\"tnt_impl\")]\n";
        mod_rs += "pub mod traits;\n";
        mod_rs += "#[cfg(feature=\"tnt_impl\")]\n";
        mod_rs += "pub use traits::*;\n";
        mod_rs += "#[cfg(feature=\"tnt_impl\")]\n";
        mod_rs += "mod channel;\n";
        mod_rs += "#[cfg(feature=\"tnt_impl\")]\n";
        mod_rs += "mod tnt;\n";
        mod_rs += "#[cfg(feature=\"tnt_impl\")]\n";
        mod_rs += "pub use tnt::start;\n";
        std::fs::write(mod_rs_path, mod_rs)?;

        let mut tnt_rs = format!("");
        tnt_rs += "\n";
        tnt_rs += "use super::channel;\n";
        tnt_rs += "\n";
        // tnt_rs += &self.code_call();
        tnt_rs += &self.code_start();
        tnt_rs += "\n";
        tnt_rs += &self.code_enum();
        tnt_rs += "\n";
        tnt_rs += &self.code_message();
        tnt_rs += "\n";
        let tnt_rs_path = {
            let mut path = rpc_path.clone();
            path.push("tnt.rs");
            path
        };
        std::fs::write(tnt_rs_path, tnt_rs)?;

        let channel_rs_path = {
            let mut path = rpc_path.clone();
            path.push("channel.rs");
            path
        };
        std::fs::write(channel_rs_path, include_str!("../sources/channel.rs"))?;

        Ok(())
    }
}