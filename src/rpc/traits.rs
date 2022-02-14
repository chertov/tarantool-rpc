use convert_case::{Case, Casing};

use super::method::Method;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trait {
    #[serde(skip)]
    pub name: String,

    pub methods: linked_hash_map::LinkedHashMap<String, Method>
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Traits {
    tarantool: Option<String>,
    dependencies: linked_hash_map::LinkedHashMap<String, String>,
    pub traits: linked_hash_map::LinkedHashMap<String, Trait>
}

impl Trait {
    pub fn args_name(&self) -> String { format!("{}Args", self.name.clone().to_case(Case::UpperCamel)) }
    pub fn rets_name(&self) -> String { format!("{}Rets", self.name.clone().to_case(Case::UpperCamel)) }
    pub fn enum_name(&self) -> String { format!("{}Enum", self.name.clone().to_case(Case::UpperCamel)) }
    pub fn trait_name(&self) -> String { format!("{}", self.name.clone().to_case(Case::UpperCamel)) }
    pub fn impl_name(&self) -> String { format!("{}TntImpl", self.name.clone().to_case(Case::UpperCamel)) }
    pub fn instance_name(&self) -> String { format!("{}", self.name.clone().to_case(Case::ScreamingSnake)) }

    fn validate(mut self, name: String) -> Result<Self, anyhow::Error> {
        self.name = name;

        let mut methods = linked_hash_map::LinkedHashMap::new();
        for (name, mut method) in self.methods.clone() {
            methods.insert(name.clone(), method.validate(name)?);
        }
        self.methods = methods;

        Ok(self)
    }

    pub fn code_enum(&self) -> String {
        let mut src = format!("");

        src += "#[derive(Debug)]\n";
        src += &format!("pub(crate) enum {}Enum {{\n", self.name);
        for (_, method) in &self.methods {
            src += &format!("    {}\n", method.code_enum());
        }
        src += "}\n";

        src += &format!("impl {} {{\n", self.enum_name());
        src += &format!("    pub(crate) fn call(self) -> Result<(), anyhow::Error> {{\n");
        src += &format!("        match self {{\n");
        for (_, method) in &self.methods {
            let mut args = vec![];
            for (_, arg) in &method.args {
                args.push(format!("{}", arg.name))
            }
            // Self::EmailGetPin { email, res } => { res.send({ UsersTntImpl::email_get_pin(&*instance(), email) }).map_err(|err| anyhow::anyhow!("Can't send res.send")) },
            // src += &format!("            Self::{} {{ {} }} => {{ res.send({{ let i: Box<dyn {}> = instance(); {} }}).map_err(|err| anyhow::anyhow!(\"Can't send res.send\")) }},\n", method.enum_name(), args.join(", "), self.impl_name(), func_call);

            let func_call = format!("{}::{}(&*instance(), {})", self.impl_name(), method.name, args.join(", "));
            args.push("res".to_string());
            src += &format!("            Self::{} {{ {} }} => {{\n", method.enum_name(), args.join(", "));
            src += &format!("                let catch_res = std::panic::catch_unwind(|| {{ {} }});\n", func_call);
            src += &format!("                let call_res = match catch_res {{\n");
            src += &format!("                    Ok(call_res) => call_res,\n");
            src += &format!("                    Err(err) => Err(anyhow::anyhow!(\"panic err: {{:?}}\", err)),\n");
            src += &format!("                }};\n");
            src += &format!("                res.send(call_res).map_err(|send_value| anyhow::anyhow!(\"Can't send res.send\"))\n");
            src += &format!("            }},\n");
            // src += &format!("            Self::{} {{ {} }} => {{ res.send({{ {} }}).map_err(|err| anyhow::anyhow!(\"Can't send res.send\")) }},\n", method.enum_name(), args.join(", "), func_call);

            // let func_call = format!("std::panic::catch_unwind(|| {{ {}::{}(&*instance(), {}) }});", self.impl_name(), method.name, args.join(", "));
            // args.push("res".to_string());
            // src += &format!("            Self::{} {{ {} }} => {{ res.send({{ {} }}).map_err(|err| anyhow::anyhow!(\"Can't send res.send\")) }},\n", method.enum_name(), args.join(", "), func_call);
        }
        src += &format!("        }}\n");
        src += &format!("    }}\n");
        src += &format!("}}\n");

        src
    }
}

impl Traits {
    pub(crate) fn dependencies(&self) -> linked_hash_map::LinkedHashMap<String, String> {
        self.dependencies.clone()
    }
    pub(crate) fn tarantool(&self) -> Option<String> {
        self.tarantool.clone()
    }
    pub(crate) fn validate(mut self) -> Result<Self, anyhow::Error> {
        let mut traits = linked_hash_map::LinkedHashMap::new();
        for (name, mut trait_) in self.traits.clone() {
            traits.insert(name.clone(), trait_.validate(name)?);
        }
        self.traits = traits;

        Ok(self)
    }

    pub(crate) fn code_enum(&self) -> String {
        let mut src = format!("");
        for (_, trait_) in &self.traits {
            src += &format!("{}\n", trait_.code_enum());
        }
        src
    }

    pub(crate) fn code_trait_impl(&self, crate_name: &str) -> String {
        let crate_name = crate_name.to_string();
        let crate_name = crate_name.replace("-", "_");
        let crate_name = crate_name.trim();

        let mut src = format!("");
        let mut traits = vec![];
        for (_, trait_) in &self.traits {
            traits.push(trait_.trait_name());
            src += &format!("#[async_trait::async_trait]\n");
            src += &format!("pub trait {} {{\n", trait_.trait_name());
            for (_, method) in &trait_.methods {
                src += &format!("{}\n", method.code_impl(&trait_, true));
            }
            src += &format!("}}\n");
            src += &format!("\n");
        }
        src += &format!("#[async_trait::async_trait]\n");
        src += &format!("pub trait TarantoolImpl: {} + Sync + Send {{\n", traits.join(" + "));
        src += &format!("    fn clone_box(&self) -> Box<dyn TarantoolImpl>;\n");
        src += &format!("}}\n");

        src += &format!("\n");
        src += &format!("#[macro_export]\n");
        src += &format!("macro_rules! tnt {{\n");
        src += &format!("    ($l:tt) => {{\n");
        for (_, trait_) in &self.traits {
            src += &format!("        impl {crate_name}::impls::{} for $l {{}};\n", trait_.trait_name());
        }
        src += &format!("    }}\n");
        src += &format!("}}\n");
        src += &format!("#[macro_export]\n");
        src += &format!("macro_rules! tnt_full {{\n");
        src += &format!("    ($l:tt) => {{\n");
        src += &format!("        {crate_name}::tnt!($l);\n");
        src += &format!("        impl {crate_name}::impls::TarantoolImpl for $l {{\n");
        src += &format!("            fn clone_box(&self) -> Box<dyn {crate_name}::impls::TarantoolImpl> {{ Box::new(self.clone()) }}\n");
        src += &format!("        }}\n");
        src += &format!("    }}\n");
        src += &format!("}}\n");

        // src += &format!("\n");
        // src += &format!("\n");
        // src += &format!("pub struct Tarantool {{}}\n");
        // src += &format!("\n");
        // for (_, trait_) in &self.traits {
        //     src += &format!("#[async_trait::async_trait]\n");
        //     src += &format!("impl {} for Tarantool {{}}\n", trait_.trait_name());
        //     src += &format!("\n");
        // }
        // src += &format!("impl TarantoolImpl for Tarantool {{}}\n");
        src
    }

    pub(crate) fn code_call(&self) -> String {
        let mut src = format!("");
        for (_, trait_) in &self.traits {
            src += &format!("pub struct {} {{}}\n", trait_.trait_name());
            src += &format!("impl {} {{\n", trait_.trait_name());
            for (_, method) in &trait_.methods {
                src += &format!("{}\n", method.code_impl(&trait_, false));
            }
            src += &format!("}}\n");
        }
        src
    }

    pub(crate) fn code_trait(&self) -> String {
        let mut src = format!("");
        for (_, trait_) in &self.traits {
            // src += &format!("#[async_trait::async_trait]\n");
            src += &format!("pub trait {} {{\n", trait_.impl_name());
            for (_, method) in &trait_.methods {
                src += &format!("    {}\n", method.define());
            }
            // src += &format!("\n");
            // src += &format!("    fn clone(&self) -> Box<dyn {}> where Self: Clone + 'static {{ Box::new(Clone::clone(self)) }}\n", trait_.trait_name());
            src += &format!("}}\n");
            // src += &format!("\n");
            // src += &format!("static {}: once_cell::sync::OnceCell<parking_lot::RwLock<Box<dyn {} + Send + Sync>>> = once_cell::sync::OnceCell::new();\n", trait_.instance_name(), trait_.trait_name());
            // src += &format!("impl dyn {} {{\n", trait_.trait_name());
            // src += &format!("    pub fn set(this: Box<dyn {}>) {{ todo!() }}\n", trait_.trait_name());
            // src += &format!("    fn instance() -> Box<dyn {}> {{ todo!() }}\n", trait_.trait_name());
            // src += &format!("}}\n");
        }
        let mut traits = vec![];
        for (_, trait_) in &self.traits {
            traits.push(trait_.impl_name());
        }
        src += &format!("pub trait TarantoolDbImpl: {} + Sync + Send {{\n", traits.join(" + "));
        src += &format!("    fn clone_box(&self) -> Box<dyn TarantoolDbImpl>;\n");
        // src += &format!("    // fn clone(&self) -> Box<dyn TarantoolDbImpl> where Self: Clone + 'static {{ Box::new(Clone::clone(self)) }}\n");
        // src += &format!("    // fn instance() -> Box<dyn TarantoolDbImpl> where Self: Sized {{ Box::new(IMPL.get().unwrap().clone()) }}\n");
        src += &format!("}}\n");
        src
    }

    pub(crate) fn code_start(&self) -> String {
        let mut src = format!("");
        for (_, trait_) in &self.traits {
            src += &format!("use super::traits::{};\n", trait_.impl_name());
        }
        src += &format!("use super::traits::TarantoolDbImpl;\n");
        src += &format!("static IMPL: once_cell::sync::OnceCell<parking_lot::RwLock<Box<dyn TarantoolDbImpl>>> = once_cell::sync::OnceCell::new();\n");
        src += &format!("fn instance() -> Box<dyn TarantoolDbImpl> {{ IMPL.get().unwrap().read().clone_box() }}\n");
        src += &format!("pub fn start(this: Box<dyn TarantoolDbImpl>) -> Result<(), anyhow::Error> {{\n");
        src += &format!("    IMPL.get_or_init(|| parking_lot::RwLock::new(this.clone_box()));\n");
        src += &format!("    {{ let mut w = IMPL.get().unwrap().write(); *w = this; }}\n");
        src += &format!("    let (tx, rx) = channel::channel::<TarantoolMessage>()?;\n");
        src += &format!("    if COIO_RPC_TX.get().is_some() {{ return Ok(()) }};\n");
        src += &format!("    COIO_RPC_TX.get_or_init(|| parking_lot::RwLock::new(None));\n");
        src += &format!("    COIO_RPC_TX.get().unwrap().write().replace(tx);\n");
        src += &format!("    let mut fiber = tarantool::fiber::Fiber::new(\"rpc_gen\", &mut |mut rx: Box<channel::TNTReceiver<TarantoolMessage>>| {{\n");
        src += &format!("        log::debug!(\"TNT GEN RPC message started...\");\n");
        src += &format!("        while let Ok(Some(message)) = rx.recv() {{\n");
        src += &format!("            if let Err(err) = message.call() {{\n");
        src += &format!("                log::error!(\"TNT RPC message call error: {{}}\", err);\n");
        src += &format!("            }};\n");
        src += &format!("        }}\n");
        src += &format!("        0\n");
        src += &format!("    }});\n");
        src += &format!("    fiber.start(rx);\n");
        src += &format!("    Ok(())\n");
        src += &format!("}}\n");
        src += &format!("\n");
        src += &format!("\n");
        src
    }


    pub(crate) fn code_message(&self) -> String {
        let mut src = format!("");

        src += &format!("type MethodSender = channel::TNTSender<TarantoolMessage>;\n");
        src += &format!("static COIO_RPC_TX: once_cell::sync::OnceCell<parking_lot::RwLock<Option<MethodSender>>> = once_cell::sync::OnceCell::new();\n");
        // src += &format!("lazy_static::lazy_static! {{ static ref COIO_RPC_TX: std::sync::Arc<parking_lot::RwLock<Option<MethodSender>>> = std::sync::Arc::new(parking_lot::RwLock::new(None)); }}\n");
        src += &format!("\n");
        src += &format!("#[derive(Debug)]\n");
        src += &format!("pub(crate) enum TarantoolMessage {{\n");
        for (_, trait_) in &self.traits {
            src += &format!("    {}({}),\n", trait_.trait_name(), trait_.enum_name());
        }
        src += &format!("}}\n");
        src += &format!("impl TarantoolMessage {{\n");
        src += &format!("    pub(crate) fn call(self) -> Result<(), anyhow::Error> {{\n");
        src += &format!("        match self {{\n");
        for (_, trait_) in &self.traits {
            src += &format!("            Self::{}(msg) => msg.call(),\n", trait_.trait_name());
        }
        src += &format!("        }}\n");
        src += &format!("    }}\n");
        src += &format!("    pub(crate) fn send_to_coio(self) -> Result<(), anyhow::Error> {{\n");
        src += &format!("        let mut tx = COIO_RPC_TX.get().clone().ok_or(anyhow::anyhow!(\"COIO_RPC_TX is not init\"))?\n");
        src += &format!("            .read().clone().ok_or(anyhow::anyhow!(\"COIO_RPC_TX is None\"))?;\n");
        src += &format!("        // let mut tx = COIO_RPC_TX.read().clone().ok_or(anyhow::anyhow!(\"coio RPC tx is None\"))?;\n");
        src += &format!("        tx.send(self).map_err(|err| anyhow::anyhow!(err))?;\n");
        src += &format!("        Ok(())\n");
        src += &format!("    }}\n");
        src += &format!("}}\n");
        src += &format!("\n");
        src
    }
}
