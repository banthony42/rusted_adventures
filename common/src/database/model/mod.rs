pub mod account;
pub mod bestiary;
pub mod character;
pub mod entity;
pub mod location;
pub mod monster;

pub trait EntityIdentifiable {
    fn get_id(&self) -> i32;

    fn get_name(&self) -> &String;

    fn identifier(&self) -> String {
        format!("{}.{}", self.get_id(), self.get_name())
    }
}
