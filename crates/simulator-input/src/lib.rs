pub mod event;
pub mod handler;

pub use event::InputEvent;
pub use handler::InputHandler;

#[cfg(test)]
mod handler_test;
