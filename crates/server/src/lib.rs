#[cfg(feature = "server")]
pub mod config;

#[cfg(feature = "server")]
pub mod db;

pub mod api;

#[cfg(feature = "server")]
pub mod rest;

#[cfg(feature = "server")]
pub mod openapi;

#[cfg(feature = "server")]
pub mod error_convert;

#[cfg(feature = "server")]
pub mod telemetry;

#[cfg(feature = "server")]
pub mod health;

#[cfg(feature = "server")]
pub mod auth;

#[cfg(feature = "server")]
pub mod s3;

#[cfg(feature = "server")]
pub mod storage;

#[cfg(feature = "server")]
pub mod mailgun;

#[cfg(feature = "server")]
pub mod stripe;

#[cfg(feature = "server")]
pub mod twilio;

// Lexodus domain modules
#[cfg(feature = "server")]
pub mod repo;

#[cfg(feature = "server")]
pub mod nef_delivery;

#[cfg(feature = "server")]
pub mod tenant;

#[cfg(feature = "server")]
pub mod rate_limit;

#[cfg(feature = "server")]
pub mod typst;

#[cfg(feature = "server")]
pub mod search;
