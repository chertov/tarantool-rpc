#![allow(non_snake_case)]

pub trait UsersTntImpl {
    fn create_user(&self, id: String, name: String, email: String) -> Result</* user */ my_custom_package::User, anyhow::Error>;
    fn get_user_by_email(&self, email: String) -> Result</* user */ Option<my_custom_package::User>, anyhow::Error>;
    fn create_space_and_fill_random(&self, space: String) -> Result<(), anyhow::Error>;
    fn get_value_from_space(&self, space: String, key: usize) -> Result</* row */ Option<my_custom_package::Row>, anyhow::Error>;
}
pub trait AppsTntImpl {
    fn apps__create(&self, user_id: String, pub_key: String, device_name: String) -> Result<(), anyhow::Error>;
    fn apps__remove(&self, user_id: String, pub_key: String) -> Result<(), anyhow::Error>;
}
pub trait TarantoolDbImpl: UsersTntImpl + AppsTntImpl + Sync + Send {
    fn clone_box(&self) -> Box<dyn TarantoolDbImpl>;
}

