extern crate nom;
extern crate tdmath;

mod obj;
pub mod model;

pub use self::model::{Model, Vertex};
pub use self::obj::parse_obj_file;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
