#![allow(clippy::from_over_into)]
pub mod admin;
pub mod config;
pub mod error;
pub mod images;
pub mod open_api;
pub mod paste;
pub mod responses;
pub mod state;
pub mod tracing_setup;
pub mod user;
pub mod utils;

pub type Error = error::WebsiteError;
pub type Result<T> = std::result::Result<T, Error>;
pub type DatabaseConnection = sea_orm::DatabaseConnection;
