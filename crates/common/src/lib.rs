pub mod file_location;
pub mod file_utils;
pub mod paste;
pub mod response_type;
pub mod rules;
pub mod serde_chrono;
pub mod user_types;
pub mod visibility;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub use http;
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
