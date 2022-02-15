#![allow(non_snake_case)]

#[cfg(feature="tnt_impl")]
use super::tnt::*;

pub(crate) const TIMEOUT : std::time::Duration = std::time::Duration::from_secs(3);

#[async_trait::async_trait]
pub trait Users {
    async fn create_user(&self, id: String, name: String, email: String) -> Result</* user */ my_custom_package::User, anyhow::Error> { self.create_user__with_custom_timeout(id, name, email, TIMEOUT).await }
    #[cfg(not(feature="tnt_impl"))]
    async fn create_user__with_custom_timeout(&self, id: String, name: String, email: String, timeout: std::time::Duration) -> Result</* user */ my_custom_package::User, anyhow::Error>;
    #[cfg(feature="tnt_impl")]
    async fn create_user__with_custom_timeout(&self, id: String, name: String, email: String, timeout: std::time::Duration) -> Result</* user */ my_custom_package::User, anyhow::Error> {
        let (res, rx) = futures::channel::oneshot::channel();
        TarantoolMessage::Users(UsersEnum::CreateUser {id, name, email, res}).send_to_coio()?;
        tokio::time::timeout(timeout, rx).await??
    }
    async fn get_user_by_email(&self, email: String) -> Result</* user */ Option<my_custom_package::User>, anyhow::Error> { self.get_user_by_email__with_custom_timeout(email, TIMEOUT).await }
    #[cfg(not(feature="tnt_impl"))]
    async fn get_user_by_email__with_custom_timeout(&self, email: String, timeout: std::time::Duration) -> Result</* user */ Option<my_custom_package::User>, anyhow::Error>;
    #[cfg(feature="tnt_impl")]
    async fn get_user_by_email__with_custom_timeout(&self, email: String, timeout: std::time::Duration) -> Result</* user */ Option<my_custom_package::User>, anyhow::Error> {
        let (res, rx) = futures::channel::oneshot::channel();
        TarantoolMessage::Users(UsersEnum::GetUserByEmail {email, res}).send_to_coio()?;
        tokio::time::timeout(timeout, rx).await??
    }
    async fn create_space_and_fill_random(&self, space: String) -> Result<(), anyhow::Error> { self.create_space_and_fill_random__with_custom_timeout(space, TIMEOUT).await }
    #[cfg(not(feature="tnt_impl"))]
    async fn create_space_and_fill_random__with_custom_timeout(&self, space: String, timeout: std::time::Duration) -> Result<(), anyhow::Error>;
    #[cfg(feature="tnt_impl")]
    async fn create_space_and_fill_random__with_custom_timeout(&self, space: String, timeout: std::time::Duration) -> Result<(), anyhow::Error> {
        let (res, rx) = futures::channel::oneshot::channel();
        TarantoolMessage::Users(UsersEnum::CreateSpaceAndFillRandom {space, res}).send_to_coio()?;
        tokio::time::timeout(timeout, rx).await??
    }
    async fn get_value_from_space(&self, space: String, key: usize) -> Result</* row */ Option<my_custom_package::Row>, anyhow::Error> { self.get_value_from_space__with_custom_timeout(space, key, TIMEOUT).await }
    #[cfg(not(feature="tnt_impl"))]
    async fn get_value_from_space__with_custom_timeout(&self, space: String, key: usize, timeout: std::time::Duration) -> Result</* row */ Option<my_custom_package::Row>, anyhow::Error>;
    #[cfg(feature="tnt_impl")]
    async fn get_value_from_space__with_custom_timeout(&self, space: String, key: usize, timeout: std::time::Duration) -> Result</* row */ Option<my_custom_package::Row>, anyhow::Error> {
        let (res, rx) = futures::channel::oneshot::channel();
        TarantoolMessage::Users(UsersEnum::GetValueFromSpace {space, key, res}).send_to_coio()?;
        tokio::time::timeout(timeout, rx).await??
    }
    async fn empty(&self) -> Result<(), anyhow::Error> { self.empty__with_custom_timeout(TIMEOUT).await }
    #[cfg(not(feature="tnt_impl"))]
    async fn empty__with_custom_timeout(&self, timeout: std::time::Duration) -> Result<(), anyhow::Error>;
    #[cfg(feature="tnt_impl")]
    async fn empty__with_custom_timeout(&self, timeout: std::time::Duration) -> Result<(), anyhow::Error> {
        let (res, rx) = futures::channel::oneshot::channel();
        TarantoolMessage::Users(UsersEnum::Empty {res}).send_to_coio()?;
        tokio::time::timeout(timeout, rx).await??
    }
}

#[async_trait::async_trait]
pub trait Apps {
    async fn apps__create(&self, user_id: String, pub_key: String, device_name: String) -> Result<(), anyhow::Error> { self.apps__create__with_custom_timeout(user_id, pub_key, device_name, TIMEOUT).await }
    #[cfg(not(feature="tnt_impl"))]
    async fn apps__create__with_custom_timeout(&self, user_id: String, pub_key: String, device_name: String, timeout: std::time::Duration) -> Result<(), anyhow::Error>;
    #[cfg(feature="tnt_impl")]
    async fn apps__create__with_custom_timeout(&self, user_id: String, pub_key: String, device_name: String, timeout: std::time::Duration) -> Result<(), anyhow::Error> {
        let (res, rx) = futures::channel::oneshot::channel();
        TarantoolMessage::Apps(AppsEnum::AppsCreate {user_id, pub_key, device_name, res}).send_to_coio()?;
        tokio::time::timeout(timeout, rx).await??
    }
    async fn apps__remove(&self, user_id: String, pub_key: String) -> Result<(), anyhow::Error> { self.apps__remove__with_custom_timeout(user_id, pub_key, TIMEOUT).await }
    #[cfg(not(feature="tnt_impl"))]
    async fn apps__remove__with_custom_timeout(&self, user_id: String, pub_key: String, timeout: std::time::Duration) -> Result<(), anyhow::Error>;
    #[cfg(feature="tnt_impl")]
    async fn apps__remove__with_custom_timeout(&self, user_id: String, pub_key: String, timeout: std::time::Duration) -> Result<(), anyhow::Error> {
        let (res, rx) = futures::channel::oneshot::channel();
        TarantoolMessage::Apps(AppsEnum::AppsRemove {user_id, pub_key, res}).send_to_coio()?;
        tokio::time::timeout(timeout, rx).await??
    }
}

#[async_trait::async_trait]
pub trait TarantoolImpl: Users + Apps + Sync + Send {
    fn clone_box(&self) -> Box<dyn TarantoolImpl>;
}

#[macro_export]
macro_rules! tnt {
    ($l:tt) => {
        impl my_simple_rpc::impls::Users for $l {}
        impl my_simple_rpc::impls::Apps for $l {}
    }
}
#[macro_export]
macro_rules! tnt_full {
    ($l:tt) => {
        my_simple_rpc::tnt!($l);
        impl my_simple_rpc::impls::TarantoolImpl for $l {
            fn clone_box(&self) -> Box<dyn my_simple_rpc::impls::TarantoolImpl> { Box::new(self.clone()) }
        }
    }
}
