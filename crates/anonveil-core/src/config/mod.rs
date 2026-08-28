//! `config.toml` schema and parsing.

pub mod schema;

pub use schema::{
    AnonveilConfig, Ipv6Setting, LoggingConfig, MacConfig, NetworkConfig, TuiConfig, TuiTheme,
};
