#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
    #[error("Out of stock '{0}'")]
    OutOfStock(String),
}
