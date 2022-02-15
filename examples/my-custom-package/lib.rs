
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
}
impl tarantool::tuple::AsTuple for User{}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Row {
    pub key: usize,
    pub value: String,
}

impl tarantool::tuple::AsTuple for Row{}