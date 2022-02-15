#[macro_use] extern crate log;

static TOKIO_RUNTIME: once_cell::sync::Lazy<parking_lot::RwLock<tokio::runtime::Runtime>> = once_cell::sync::Lazy::new(|| {
    parking_lot::RwLock::new(tokio::runtime::Runtime::new().unwrap())
});

#[derive(Clone)]
pub struct DbImpl {}
impl my_simple_rpc::TarantoolDbImpl for DbImpl {
    fn clone_box(&self) -> Box<dyn my_simple_rpc::TarantoolDbImpl> { Box::new(self.clone()) }
}
impl my_simple_rpc::UsersTntImpl for DbImpl {
    fn create_user(&self, id: String, name: String, email: String) -> Result</* user */ my_custom_package::User, anyhow::Error> {
        todo!()
    }
    fn get_user_by_email(&self, email: String) -> Result</* user */ Option<my_custom_package::User>, anyhow::Error> {
        Ok(Some(
            my_custom_package::User { email,
                id: "user_id_0".to_string(),
                name: "UserId0".to_string(),
            }
        ))
    }
    fn create_space_and_fill_random(&self, space: String) -> Result<(), anyhow::Error> {
        let mut opts = tarantool::space::SpaceCreateOptions::default();
        opts.if_not_exists = true;
        opts.format = Some(vec![
            tarantool::space::SpaceFieldFormat{ name: "key".to_string(), field_type: tarantool::space::SpaceFieldType::Unsigned },
            tarantool::space::SpaceFieldFormat{ name: "value".to_string(), field_type: tarantool::space::SpaceFieldType::String }
        ]);
        let mut space = tarantool::space::Space::create(&space, &opts)?;
        let opts = tarantool::index::IndexOptions::default();
        space.create_index("primary", &opts);
        for i in 0..1000 {
            let row = my_custom_package::Row { key: i, value: format!("value_{i}") };
            space.insert(&row)?;
        }
        Ok(())
    }
    fn get_value_from_space(&self, space: String, key: usize) -> Result<Option<my_custom_package::Row>, anyhow::Error> {
        let mut space = tarantool::space::Space::find(&space).ok_or(anyhow::anyhow!("Can't find space {space}"))?;
        Ok(match space.get(&(key,))? {
            Some(tuple) => Some(tuple.into_struct::<my_custom_package::Row>()?),
            None => None,
        })
    }
}
impl my_simple_rpc::AppsTntImpl for DbImpl {
    fn apps__create(&self, user_id: String, pub_key: String, device_name: String) -> Result<(), anyhow::Error> {
        todo!()
    }
    fn apps__remove(&self, user_id: String, pub_key: String) -> Result<(), anyhow::Error> {
        todo!()
    }
}




#[no_mangle]
pub extern "C" fn start_service(_: tarantool::tuple::FunctionCtx, _: tarantool::tuple::FunctionArgs) -> std::os::raw::c_int {
    env_logger::builder().filter_level(log::LevelFilter::Trace).init();
    my_simple_rpc::start(Box::new(DbImpl {})).unwrap();
    TOKIO_RUNTIME.read().spawn(service_main());
    0
}

#[derive(Clone)]
struct DB{}
my_simple_rpc::tnt_full!(DB);

async fn service_main() -> Result<(), anyhow::Error> {
    use my_simple_rpc::Users;

    // call Tarantool methods from tokio threads
    let db = DB{};

    tokio::spawn({
        let db = db.clone();
        async move {
            let space = "test_space".to_string();
            db.create_space_and_fill_random(space.clone()).await.unwrap();
            loop {
                use rand::Rng;
                let key = rand::thread_rng().gen_range(0..1100_usize);
                let value = db.get_value_from_space(space.clone(), key).await.unwrap();
                info!("key: {key}, value: {:?}", value);
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    });

    loop {
        let res = db.get_user_by_email("chertovmv@gmail.com".to_string()).await;
        info!("res: {:?}", res);
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
    Ok(())
}
