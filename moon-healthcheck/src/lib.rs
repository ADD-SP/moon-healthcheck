//! # Moon Healthcheck
//! An async healthcheck library for Rust.
//! 
//! # Features
//! * TCP healthcheck
//! * HTTP healthcheck
//!     * HTTP GET, POST, PUT, DELETE, HEAD, PATCH
//!     * HTTP/1.1, HTTP/2
//! * UDP healthcheck
//! * Active (background) healthcheck
//! * Passive (report manually) healthcheck
//!
//! # Examples
//! ## TCP healthcheck
//! ```rust
//! use moon_healthcheck::tcp::TcpChecker;
//! use moon_healthcheck::dispatcher::Dispatcher;
//! use tokio::time::Duration;
//! use tokio::time::sleep;
//! 
//! #[tokio::main]
//! async fn main() {
//!     let mut dispatcher: Dispatcher<TcpChecker> = Dispatcher::new();
//!     
//!     let tcp_checker = TcpChecker::new("really.really.not.exists.host", 80, 1);
//!     let uuid = dispatcher.schedule(tcp_checker, Duration::from_secs(1)).unwrap();
//!     
//!     sleep(Duration::from_secs(3)).await;
//!     assert_eq!(dispatcher.is_healthy(&uuid).await.unwrap(), false);
//! }
//! ```
//! 
//! ## UDP healthcheck
//! ```rust
//! use moon_healthcheck::udp::UdpChecker;
//! use moon_healthcheck::dispatcher::Dispatcher;
//! use tokio::time::Duration;
//! use tokio::time::sleep;
//! 
//! #[tokio::main]
//! async fn main() {
//!     let mut dispatcher: Dispatcher<UdpChecker> = Dispatcher::new();
//! 
//!     let udp_checker = UdpChecker::new("really.really.not.exists.host", 53, &vec![1, 2, 3], &vec![1, 2, 3], 1);
//!     let uuid = dispatcher.schedule(udp_checker, Duration::from_secs(1)).unwrap();
//! 
//!     sleep(Duration::from_secs(3)).await;
//!     assert_eq!(dispatcher.is_healthy(&uuid).await.unwrap(), false);
//! }
//! ```
//! 
//! ## HTTP healthcheck
//! ```rust
//! use moon_healthcheck::http::HttpChecker;
//! use moon_healthcheck::dispatcher::Dispatcher;
//! use tokio::time::Duration;
//! use tokio::time::sleep;
//! 
//! #[tokio::main]
//! async fn main() {
//!     let mut dispatcher: Dispatcher<HttpChecker> = Dispatcher::new();
//! 
//!     let http_checker = HttpChecker::new("https://really.really.not.exists.host", 1);
//!     let uuid = dispatcher.schedule(http_checker, Duration::from_secs(1)).unwrap();
//! 
//!     sleep(Duration::from_secs(3)).await;
//!     assert_eq!(dispatcher.is_healthy(&uuid).await.unwrap(), false);
//! }
//! ```
//! 

mod state;
mod checker;

pub mod tcp;
pub mod udp;
pub mod http;
pub mod dispatcher;
