use convert_case::{Case, Casing};

use super::traits::Trait;

pub enum Type {

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Param {
    pub name: String,

    #[serde(skip)]
    pub type_: String,
    #[serde(rename = "type")]
    pub type_str: String,

    #[serde(default = "Param::default_optional")]
    pub optional: bool,
}
impl Param {
    fn default_optional() -> bool { false }
    fn default_type() -> bool { false }
}
impl Param {
    pub fn validate(mut self) -> Result<Self, anyhow::Error> {
        Ok(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Method {
    #[serde(skip)]
    pub name: String,

    #[serde(default = "Method::default_args")]
    #[serde(rename = "args")]
    args_v: Vec<Param>,
    #[serde(skip)]
    pub args: linked_hash_map::LinkedHashMap<String, Param>,

    #[serde(default = "Method::default_rets")]
    #[serde(rename = "rets")]
    rets_v: Vec<Param>,
    #[serde(skip)]
    pub rets: linked_hash_map::LinkedHashMap<String, Param>,
}
impl Method {
    fn default_args() -> Vec<Param> { vec![] }
    fn default_rets() -> Vec<Param> { vec![] }

    pub fn enum_name(&self) -> String { format!("{}", self.name.clone().to_case(Case::UpperCamel)) }
}

impl Method {
    pub fn validate(mut self, name: String) -> Result<Self, anyhow::Error> {
        self.name = name;

        let mut args = linked_hash_map::LinkedHashMap::new();
        for arg in self.args_v.clone() {
            args.insert(arg.name.clone(), arg.validate()?);
        }
        self.args = args;

        let mut rets = linked_hash_map::LinkedHashMap::new();
        for ret in self.rets_v.clone() {
            rets.insert(ret.name.clone(), ret.validate()?);
        }
        self.rets = rets;

        // let mut spaces = linked_hash_map::LinkedHashMap::new();
        // for (name, mut space) in self.spaces.clone() {
        //     spaces.insert(name.clone(), space.validate(name)?);
        // }
        // self.spaces = spaces;

        Ok(self)
    }

    pub fn define(&self) -> String {
        let mut src = format!("");

        let mut args = vec!["&self".to_string()];
        for (name, arg) in &self.args {
            args.push(format!("{}: {}", name, arg.type_str))
        }


        let mut rets = vec![];
        for (name, ret) in &self.rets {
            rets.push(format!("/* {} */ {}", name, ret.type_str))
        }

        let rets = match rets.len() {
            0 => "()".to_string(),
            1 => rets.join(", "),
            _ => format!("({})", rets.join(", "))
        };

        src += &format!("fn {}({}) -> Result<{}, anyhow::Error>;", self.name, args.join(", "), rets);
        src
    }


    pub fn code_impl(&self, trait_: &Trait, is_trait_impl: bool) -> String {
        let mut src = format!("");

        let pub_str = match is_trait_impl { true => "", false => "pub " };

        let mut args_list = vec![];
        for (name, arg) in &self.args {
            args_list.push(format!("{}", name))
        }

        let mut args = vec!["&self".to_string()];
        for (name, arg) in &self.args {
            args.push(format!("{}: {}", name, arg.type_str))
        }
        let mut rets = vec![];
        for (name, ret) in &self.rets {
            rets.push(format!("/* {} */ {}", name, ret.type_str))
        }
        let rets = match rets.len() {
            0 => "()".to_string(),
            1 => rets.join(", "),
            _ => format!("({})", rets.join(", "))
        };
        {
            let mut args_list = args_list.clone();
            args_list.push("TIMEOUT".to_string());
            src += &format!("    {}async fn {}({}) -> Result<{}, anyhow::Error> {{ self.{}__with_custom_timeout({}).await }}\n",
                            pub_str, self.name, args.join(", "), rets, self.name, args_list.join(", "));
        }
        args_list.push("res".to_string());

        args.push(format!("timeout: std::time::Duration"));
        let def = format!("    {}async fn {}__with_custom_timeout({}) -> Result<{}, anyhow::Error>", pub_str, self.name, args.join(", "), rets);
        if is_trait_impl {
            src += &format!("    #[cfg(not(feature=\"tnt_impl\"))]\n");
            src += &format!("{};\n", def);
            src += &format!("    #[cfg(feature=\"tnt_impl\")]\n");
        }
        src += &format!("{} {{\n", def);
        src += &format!("        let (res, rx) = futures::channel::oneshot::channel();\n");
        src += &format!("        TarantoolMessage::{}({}::{} {{{}}}).send_to_coio()?;\n", trait_.trait_name(), trait_.enum_name(), self.enum_name(), args_list.join(", "));
        src += &format!("        tokio::time::timeout(timeout, rx).await??\n");
        src += &format!("    }}");
        src
    }
    pub fn args_list(&self, with_type: bool, append: Option<String>) -> String {
        let mut args = vec![];
        for (name, arg) in &self.args {
            if with_type {
                args.push(format!("{}: {}", name, arg.type_str))
            } else {
                args.push(format!("{}", name))
            }
        }
        if let Some(append) = append {
            args.push(append);
        }
        format!("{}", args.join(", "))
    }
    pub fn rets_list(&self, err_type: &str) -> String {
        let mut src = format!("");
        let mut rets = vec![];
        for (name, ret) in &self.rets {
            rets.push(format!("/*{}*/ {}", name, ret.type_str))
        }
        let rets = match rets.len() {
            1 => format!("{}", rets.join(", ")),
            _ => format!("({})", rets.join(", ")),
        };
        src += &format!("Result<{}, {}>", rets, err_type);
        src
    }
    pub fn code_enum_args_list(&self, with_type: bool, append: Option<String>) -> String {
        let mut src = format!("");
        let mut args = vec![];
        for (name, arg) in &self.args {
            if with_type {
                args.push(format!("{}: {}", name, arg.type_str))
            } else {
                args.push(format!("{}", name))
            }
        }
        if let Some(append) = append {
            args.push(append);
        }
        let args = match args.len() {
            0 => format!(""),
            _ => format!(" {{ {} }}", args.join(", ")),
        };
        src += &format!("{}{}", self.enum_name(), args);
        src
    }
    pub fn code_enum_rets(&self) -> String {
        let mut src = format!("");
        let mut rets = vec![];
        for (name, ret) in &self.rets {
            rets.push(format!("/*{}*/ {}", name, ret.type_str))
        }
        let rets = match rets.len() {
            1 => format!("{}", rets.join(", ")),
            _ => format!("({})", rets.join(", ")),
        };
        src += &format!("{}(Result<{}, String>),", self.enum_name(), rets);
        src
    }
    pub fn code_enum(&self) -> String {
        let mut src = format!("");

        let mut args = vec![];
        for (name, arg) in &self.args {
            args.push(format!("{}: {}", name, arg.type_str))
        }

        let mut rets = vec![];
        for (name, ret) in &self.rets {
            rets.push(format!("/* {} */ {}", name, ret.type_str))
        }
        let rets = match rets.len() {
            0 => "()".to_string(),
            1 => rets.join(", "),
            _ => format!("({})", rets.join(", "))
        };

        args.push(format!("res: futures::channel::oneshot::Sender<Result<{}, anyhow::Error>>", rets));

        src += &format!("{} {{\n        {}\n    }},", self.enum_name(), args.join(",\n        "));
        src
    }
}