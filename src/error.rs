// TODO make custom error for parsing env vars
pub type Error = Box<dyn std::error::Error>;

pub type Result<T> = core::result::Result<T, Error>;
