#[macro_use] extern crate log;

static TOKIO_RUNTIME: once_cell::sync::Lazy<parking_lot::RwLock<tokio::runtime::Runtime>> = once_cell::sync::Lazy::new(|| {
    debug!("tokio runtime created in thread_id {:?}", std::thread::current().id());
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
        if key == 0 {
            debug!("tnt thread_id {:?}", std::thread::current().id());
        }
        let mut space = tarantool::space::Space::find(&space).ok_or(anyhow::anyhow!("Can't find space {space}"))?;
        Ok(match space.get(&(key,))? {
            Some(tuple) => Some(tuple.into_struct::<my_custom_package::Row>()?),
            None => None,
        })
    }
    fn empty(&self) -> Result<(), anyhow::Error> { Ok(()) }
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

async fn test(space: &str, test_id: usize, empty: bool) -> f64 {
    use my_simple_rpc::Users;

    let db = DB{};
    let mut counter = 0;
    let now = std::time::Instant::now();
    let thread_id = std::thread::current().id();
    let mut threads = std::collections::BTreeSet::new();

    loop {
        if counter == 1_000_000 { break }
        if counter % 10_123 == 0 { threads.insert(std::thread::current().id().as_u64()); };
        if empty {
            db.empty().await.unwrap();
        } else {
            let value = db.get_value_from_space(space.to_string(), 100).await.unwrap();
            if value.is_none() { break }
        }
        counter += 1;
    }
    let seconds = now.elapsed().as_millis() as f64 / 1000.0;
    let rps = counter as f64 / seconds;
    debug!("[test_{test_id}] rps: {rps}, calls: {counter}, elapsed: {seconds}s, threads: {:?}", threads);
    rps
}

async fn service_main() -> Result<(), anyhow::Error> {
    use my_simple_rpc::Users;

    // call Tarantool methods from tokio threads
    let db = DB{};

    let space = "test_space".to_string();
    db.create_space_and_fill_random(space.clone()).await.unwrap();

    // show tatantool thread id
    db.get_value_from_space(space.to_string(), 0).await.unwrap();

    // benchmark with empty function
    let tasks : Vec<_> = (0..10).map(|i| test(&space, i, true)).collect();
    let rps = futures::future::join_all(tasks).await;
    let rps = rps.iter().sum::<f64>();
    debug!("empty function rps: {rps}");

    // benchmark with get_value_from_space function
    let tasks : Vec<_> = (0..10).map(|i| test(&space, i, false)).collect();
    let rps = futures::future::join_all(tasks).await;
    let rps = rps.iter().sum::<f64>();
    debug!("non empty rps: {rps}");

    loop {
        let res = db.get_user_by_email("chertovmv@gmail.com".to_string()).await;
        info!("res: {:?}", res);
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
    Ok(())
}
