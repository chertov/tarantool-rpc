
use super::channel;

use super::traits::UsersTntImpl;
use super::traits::AppsTntImpl;
use super::traits::TarantoolDbImpl;
static IMPL: once_cell::sync::OnceCell<parking_lot::RwLock<Box<dyn TarantoolDbImpl>>> = once_cell::sync::OnceCell::new();
fn instance() -> Box<dyn TarantoolDbImpl> { IMPL.get().unwrap().read().clone_box() }
pub fn start(this: Box<dyn TarantoolDbImpl>) -> Result<(), anyhow::Error> {
    IMPL.get_or_init(|| parking_lot::RwLock::new(this.clone_box()));
    { let mut w = IMPL.get().unwrap().write(); *w = this; }
    let (tx, rx) = channel::channel::<TarantoolMessage>()?;
    if COIO_RPC_TX.get().is_some() { return Ok(()) };
    COIO_RPC_TX.get_or_init(|| parking_lot::RwLock::new(None));
    COIO_RPC_TX.get().unwrap().write().replace(tx);
    let mut fiber = tarantool::fiber::Fiber::new("rpc_gen", &mut |mut rx: Box<channel::TNTReceiver<TarantoolMessage>>| {
        log::debug!("TNT GEN RPC message started...");
        while let Ok(Some(message)) = rx.recv() {
            if let Err(err) = message.call() {
                log::error!("TNT RPC message call error: {}", err);
            };
        }
        0
    });
    fiber.start(rx);
    Ok(())
}



#[derive(Debug)]
pub(crate) enum UsersEnum {
    CreateUser {
        id: String,
        name: String,
        email: String,
        res: futures::channel::oneshot::Sender<Result</* user */ my_custom_package::User, anyhow::Error>>
    },
    GetUserByEmail {
        email: String,
        res: futures::channel::oneshot::Sender<Result</* user */ Option<my_custom_package::User>, anyhow::Error>>
    },
    CreateSpaceAndFillRandom {
        space: String,
        res: futures::channel::oneshot::Sender<Result<(), anyhow::Error>>
    },
    GetValueFromSpace {
        space: String,
        key: usize,
        res: futures::channel::oneshot::Sender<Result</* row */ Option<my_custom_package::Row>, anyhow::Error>>
    },
}
impl UsersEnum {
    pub(crate) fn call(self) -> Result<(), anyhow::Error> {
        match self {
            Self::CreateUser { id, name, email, res } => {
                let catch_res = std::panic::catch_unwind(|| { UsersTntImpl::create_user(&*instance(), id, name, email) });
                let call_res = match catch_res {
                    Ok(call_res) => call_res,
                    Err(err) => Err(anyhow::anyhow!("panic err: {:?}", err)),
                };
                res.send(call_res).map_err(|send_value| anyhow::anyhow!("Can't send res.send"))
            },
            Self::GetUserByEmail { email, res } => {
                let catch_res = std::panic::catch_unwind(|| { UsersTntImpl::get_user_by_email(&*instance(), email) });
                let call_res = match catch_res {
                    Ok(call_res) => call_res,
                    Err(err) => Err(anyhow::anyhow!("panic err: {:?}", err)),
                };
                res.send(call_res).map_err(|send_value| anyhow::anyhow!("Can't send res.send"))
            },
            Self::CreateSpaceAndFillRandom { space, res } => {
                let catch_res = std::panic::catch_unwind(|| { UsersTntImpl::create_space_and_fill_random(&*instance(), space) });
                let call_res = match catch_res {
                    Ok(call_res) => call_res,
                    Err(err) => Err(anyhow::anyhow!("panic err: {:?}", err)),
                };
                res.send(call_res).map_err(|send_value| anyhow::anyhow!("Can't send res.send"))
            },
            Self::GetValueFromSpace { space, key, res } => {
                let catch_res = std::panic::catch_unwind(|| { UsersTntImpl::get_value_from_space(&*instance(), space, key) });
                let call_res = match catch_res {
                    Ok(call_res) => call_res,
                    Err(err) => Err(anyhow::anyhow!("panic err: {:?}", err)),
                };
                res.send(call_res).map_err(|send_value| anyhow::anyhow!("Can't send res.send"))
            },
        }
    }
}

#[derive(Debug)]
pub(crate) enum AppsEnum {
    AppsCreate {
        user_id: String,
        pub_key: String,
        device_name: String,
        res: futures::channel::oneshot::Sender<Result<(), anyhow::Error>>
    },
    AppsRemove {
        user_id: String,
        pub_key: String,
        res: futures::channel::oneshot::Sender<Result<(), anyhow::Error>>
    },
}
impl AppsEnum {
    pub(crate) fn call(self) -> Result<(), anyhow::Error> {
        match self {
            Self::AppsCreate { user_id, pub_key, device_name, res } => {
                let catch_res = std::panic::catch_unwind(|| { AppsTntImpl::apps__create(&*instance(), user_id, pub_key, device_name) });
                let call_res = match catch_res {
                    Ok(call_res) => call_res,
                    Err(err) => Err(anyhow::anyhow!("panic err: {:?}", err)),
                };
                res.send(call_res).map_err(|send_value| anyhow::anyhow!("Can't send res.send"))
            },
            Self::AppsRemove { user_id, pub_key, res } => {
                let catch_res = std::panic::catch_unwind(|| { AppsTntImpl::apps__remove(&*instance(), user_id, pub_key) });
                let call_res = match catch_res {
                    Ok(call_res) => call_res,
                    Err(err) => Err(anyhow::anyhow!("panic err: {:?}", err)),
                };
                res.send(call_res).map_err(|send_value| anyhow::anyhow!("Can't send res.send"))
            },
        }
    }
}


type MethodSender = channel::TNTSender<TarantoolMessage>;
static COIO_RPC_TX: once_cell::sync::OnceCell<parking_lot::RwLock<Option<MethodSender>>> = once_cell::sync::OnceCell::new();

#[derive(Debug)]
pub(crate) enum TarantoolMessage {
    Users(UsersEnum),
    Apps(AppsEnum),
}
impl TarantoolMessage {
    pub(crate) fn call(self) -> Result<(), anyhow::Error> {
        match self {
            Self::Users(msg) => msg.call(),
            Self::Apps(msg) => msg.call(),
        }
    }
    pub(crate) fn send_to_coio(self) -> Result<(), anyhow::Error> {
        let mut tx = COIO_RPC_TX.get().clone().ok_or(anyhow::anyhow!("COIO_RPC_TX is not init"))?
            .read().clone().ok_or(anyhow::anyhow!("COIO_RPC_TX is None"))?;
        // let mut tx = COIO_RPC_TX.read().clone().ok_or(anyhow::anyhow!("coio RPC tx is None"))?;
        tx.send(self).map_err(|err| anyhow::anyhow!(err))?;
        Ok(())
    }
}


