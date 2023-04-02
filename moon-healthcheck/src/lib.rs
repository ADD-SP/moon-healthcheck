//! # Moon Healthcheck
//! An async healthcheck library for Rust.
//! 
//! # Features
//! * TCP healthcheck
//! * HTTP healthcheck
//!     * HTTP GET, POST, PUT, DELETE, HEAD, PATCH
//!     * HTTP/1.1, HTTP/2
//!
//! # Examples
//! ## TCP healthcheck
//! ```rust
//! use moon_healthcheck::tcp::TcpChecker;
//! use moon_healthcheck::dispatcher::Dispatcher;
//! use std::time::Duration;
//! use tokio::time::sleep;
//! 
//! #[tokio::main]
//! async fn main() {
//!     let mut dispatcher: Dispatcher<TcpChecker> = Dispatcher::new();
//!     
//!     let tcp_checker = TcpChecker::new("really.really.not.exists.host", 80, 1);
//!     let uuid = dispatcher.schedule(tcp_checker, Duration::from_secs(1)).unwrap();
//!     
//!     sleep(Duration::from_secs(2)).await;
//!     assert_eq!(dispatcher.is_healthy(&uuid).await.unwrap(), true);
//! }
//! ```
//! 
//! ## HTTP healthcheck
//! ```rust
//! use moon_healthcheck::http::HttpChecker;
//! use moon_healthcheck::dispatcher::Dispatcher;
//! use std::time::Duration;
//! use tokio::time::sleep;
//! 
//! #[tokio::main]
//! async fn main() {
//!     let mut dispatcher: Dispatcher<HttpChecker> = Dispatcher::new();
//! 
//!     let http_checker = HttpChecker::new("https://really.really.not.exists.host", 1);
//!     let uuid = dispatcher.schedule(http_checker, Duration::from_secs(1)).unwrap();
//! 
//!     sleep(Duration::from_secs(2)).await;
//!     assert_eq!(dispatcher.is_healthy(&uuid).await.unwrap(), true);
//! }
//! ```
//! 

mod state;
mod checker;

pub mod tcp;
pub mod http;
pub mod dispatcher;
