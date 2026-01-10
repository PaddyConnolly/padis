pub mod frame;
pub use frame::Frame;

pub type Error = Box<dyn std::error::Error>;
