//! Dispatcher is a struct that schedules and manages the health checkers.

use super::checker::Checker;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use uuid::Uuid;

/// Dispatcher is a struct that schedules and manages the health checkers.
pub struct Dispatcher<T: Checker + 'static> {
    /// uuid2coroutine is a map from the UUID of a checker to the coroutine that runs the checker.
    uuid2coroutine: std::collections::HashMap<Uuid, Arc<JoinHandle<()>>>,

    /// uuid2checker is a map from the UUID of a checker to the checker itself.
    uuid2checker: std::collections::HashMap<Uuid, Arc<Mutex<T>>>,
}

impl<T: Checker + 'static> Dispatcher<T> {
    /// Create a new dispatcher
    /// # Example
    /// ```rust
    /// use moon_healthcheck::dispatcher::Dispatcher;
    /// use moon_healthcheck::tcp::TcpChecker;
    /// let mut dispatcher: Dispatcher<TcpChecker> = Dispatcher::new();
    /// ```
    pub fn new() -> Dispatcher<T> {
        Dispatcher {
            uuid2coroutine: std::collections::HashMap::new(),
            uuid2checker: std::collections::HashMap::new(),
        }
    }

    /// Schedule a checker to run periodically
    /// 
    /// # Arguments
    /// * `checker` - The checker to schedule
    /// * `interval` - The interval at which to run the checker
    /// 
    /// # Returns
    /// * `Ok(Uuid)` - The UUID of the checker
    /// * `Err(String)` - The error message
    /// 
    /// # Example
    /// ```rust
    /// use moon_healthcheck::dispatcher::Dispatcher;
    /// use moon_healthcheck::tcp::TcpChecker;
    /// use std::time::Duration;
    /// 
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut dispatcher: Dispatcher<TcpChecker> = Dispatcher::new();
    ///     let tcp_checker = TcpChecker::new("localhost", 80, 1);
    ///     let uuid = dispatcher.schedule(tcp_checker, Duration::from_secs(1)).unwrap();
    /// }
    /// ```
    pub fn schedule(&mut self, checker: T, interval: std::time::Duration) -> Result<Uuid, String> {
        if self.uuid2coroutine.contains_key(&checker.get_uuid()) {
            return Err(format!("UUID {} already exists", checker.get_uuid()));
        }

        let uuid = checker.get_uuid();
        let checker = Arc::new(Mutex::new(checker));
        let checker_clone = checker.clone();

        let coroutine = tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                checker_clone.lock().await.check().await;
            }
        });

        let coroutine: Arc<tokio::task::JoinHandle<()>> = Arc::new(coroutine);

        self.uuid2coroutine.insert(uuid, coroutine.clone());
        self.uuid2checker.insert(uuid, checker.clone());

        Ok(uuid)
    }

    /// Remove a checker from the dispatcher
    /// 
    /// # Arguments
    /// * `uuid` - The UUID of the checker to remove
    /// 
    /// # Example
    /// ```rust
    /// use moon_healthcheck::dispatcher::Dispatcher;
    /// use moon_healthcheck::tcp::TcpChecker;
    /// use std::time::Duration;
    /// 
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut dispatcher: Dispatcher<TcpChecker> = Dispatcher::new();
    ///     let tcp_checker = TcpChecker::new("localhost", 80, 1);
    ///     let uuid = dispatcher.schedule(tcp_checker, Duration::from_secs(1)).unwrap();
    ///     dispatcher.remove(&uuid);
    /// }
    /// ```
    pub fn remove(&mut self, uuid: &Uuid) {
        if let Some(coroutine) = self.uuid2coroutine.get(&uuid) {
            coroutine.abort();
            self.uuid2coroutine.remove(&uuid);
        }
    }

    /// Get the health of a checker
    /// 
    /// # Arguments
    /// * `uuid` - The UUID of the checker
    ///
    /// # Example
    /// ```rust
    /// use moon_healthcheck::dispatcher::Dispatcher;
    /// use moon_healthcheck::tcp::TcpChecker;
    /// use std::time::Duration;
    /// use tokio::time::sleep;
    /// 
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut dispatcher: Dispatcher<TcpChecker> = Dispatcher::new();
    ///     let tcp_checker = TcpChecker::new("really.really.not.exists.host", 39498, 1);
    ///     let uuid = dispatcher.schedule(tcp_checker, Duration::from_secs(1)).unwrap();
    ///     sleep(Duration::from_secs(3)).await;
    ///     assert_eq!(dispatcher.is_healthy(&uuid).await.unwrap(), false);
    /// }
    /// ```
    pub async fn is_healthy(&self, uuid: &Uuid) -> Option<bool> {
        if let Some(checker) = self.uuid2checker.get(uuid) {
            Some(checker.lock().await.is_healthy())
        } else {
            None
        }
    }

    /// Set the health of a checker
    /// 
    /// # Arguments
    /// * `uuid` - The UUID of the checker
    /// * `health` - The health to set
    /// 
    /// # Example
    /// ```rust
    /// use moon_healthcheck::dispatcher::Dispatcher;
    /// use moon_healthcheck::tcp::TcpChecker;
    /// use std::time::Duration;
    /// use tokio::time::sleep;
    /// 
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut dispatcher: Dispatcher<TcpChecker> = Dispatcher::new();
    ///     let tcp_checker = TcpChecker::new("really.really.not.exists.host", 80, 1);
    ///     let uuid = dispatcher.schedule(tcp_checker, Duration::from_secs(9999)).unwrap();
    ///     dispatcher.set_health(&uuid, false).await;
    ///     assert_eq!(dispatcher.is_healthy(&uuid).await.unwrap(), false, "checker should be unhealthy");
    ///     dispatcher.set_health(&uuid, true).await;
    ///     assert_eq!(dispatcher.is_healthy(&uuid).await.unwrap(), true, "checker should be healthy");
    /// }
    /// ```
    pub async fn set_health(&self, uuid: &Uuid, health: bool) {
        if let Some(checker) = self.uuid2checker.get(uuid) {
            checker.lock().await.set_health(health);
        }
    }

    /// Get the last error of a checker
    /// 
    /// # Arguments
    /// * `uuid` - The UUID of the checker
    /// 
    /// # Example
    /// ```rust
    /// use moon_healthcheck::dispatcher::Dispatcher;
    /// use moon_healthcheck::tcp::TcpChecker;
    /// use std::time::Duration;
    /// use tokio::time::sleep;
    /// 
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut dispatcher: Dispatcher<TcpChecker> = Dispatcher::new();
    ///     let tcp_checker = TcpChecker::new("really.really.not.exists.host", 80, 1);
    ///     let uuid = dispatcher.schedule(tcp_checker, Duration::from_secs(1)).unwrap();
    ///     sleep(Duration::from_secs(2)).await;
    ///     assert!(dispatcher.get_last_error(&uuid).await.is_some());
    /// }
    pub async fn get_last_error(&self, uuid: &Uuid) -> Option<String> {
        if let Some(checker) = self.uuid2checker.get(uuid) {
            checker.lock().await.get_last_error()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use crate::http::HttpChecker;
    use crate::tcp::TcpChecker;

    use ::utils;
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;
    use reqwest::Version;

    #[tokio::test]
    async fn tcp() {
        let mut dispatcher: Dispatcher<TcpChecker> = Dispatcher::new();

        let port = utils::test::rand_port();
        let tcp_server = utils::test::tcp_server("localhost", port).await;
        let tcp_checker = TcpChecker::new("localhost", port, 1);

        let uuid = dispatcher
            .schedule(tcp_checker, Duration::from_secs(1))
            .unwrap();

        // expect the checker to be healthy
        for _ in 0..2 {
            assert_eq!(dispatcher.is_healthy(&uuid).await.unwrap(), true);
            sleep(Duration::from_secs(1)).await;
        }

        // stop the server
        tcp_server.abort();
        // wait for the checker to detect the unhealthy state
        sleep(Duration::from_secs(2)).await;
        assert_eq!(dispatcher.is_healthy(&uuid).await.unwrap(), false);

        // start the server again
        let tcp_server = utils::test::tcp_server("localhost", port).await;
        // wait for the checker to detect the healthy state
        sleep(Duration::from_secs(2)).await;
        assert_eq!(dispatcher.is_healthy(&uuid).await.unwrap(), true);

        tcp_server.abort();
    }

    #[tokio::test]
    async fn http1() {
        let mut dispatcher: Dispatcher<HttpChecker> = Dispatcher::new();

        let port = utils::test::rand_port();
        let http_server =
            utils::test::http1_server("localhost", port, utils::test::http_handler_echo_body)
                .await;
        let http_checker = HttpChecker::new(format!("http://localhost:{}", port), 1)
            .with_version(Version::HTTP_11);

        let uuid = dispatcher
            .schedule(http_checker, Duration::from_secs(1))
            .unwrap();

        // expect the checker to be healthy
        for _ in 0..2 {
            assert_eq!(dispatcher.is_healthy(&uuid).await.unwrap(), true);
            sleep(Duration::from_secs(1)).await;
        }

        // stop the server
        http_server.abort();
        // wait for the checker to detect the unhealthy state
        sleep(Duration::from_secs(2)).await;
        assert_eq!(dispatcher.is_healthy(&uuid).await.unwrap(), false);

        // start the server again
        let http_server =
            utils::test::http1_server("localhost", port, utils::test::http_handler_echo_body)
                .await;
        // wait for the checker to detect the healthy state
        sleep(Duration::from_secs(2)).await;
        assert_eq!(dispatcher.is_healthy(&uuid).await.unwrap(), true);

        http_server.abort();
    }

    #[tokio::test]
    async fn http2() {
        let mut dispatcher: Dispatcher<HttpChecker> = Dispatcher::new();

        let port = utils::test::rand_port();
        let http_server =
            utils::test::http2_server("localhost", port, utils::test::http_handler_echo_body)
                .await;
        let http_checker = HttpChecker::new(format!("http://localhost:{}", port), 1)
            .with_version(Version::HTTP_2);

        let uuid = dispatcher
            .schedule(http_checker, Duration::from_secs(1))
            .unwrap();

        // expect the checker to be healthy
        for _ in 0..2 {
            assert_eq!(dispatcher.is_healthy(&uuid).await.unwrap(), true);
            sleep(Duration::from_secs(1)).await;
        }

        // stop the server
        http_server.abort();
        // wait for the checker to detect the unhealthy state
        sleep(Duration::from_secs(2)).await;
        assert_eq!(dispatcher.is_healthy(&uuid).await.unwrap(), false);

        // start the server again
        let http_server =
            utils::test::http2_server("localhost", port, utils::test::http_handler_echo_body)
                .await;
        // wait for the checker to detect the healthy state
        sleep(Duration::from_secs(2)).await;
        assert_eq!(dispatcher.is_healthy(&uuid).await.unwrap(), true);

        http_server.abort();
    }

    #[tokio::test]
    async fn set_health() {
        let mut dispatcher: Dispatcher<TcpChecker> = Dispatcher::new();

        let port = utils::test::rand_port();
        let tcp_checker = TcpChecker::new("localhost", port, 1);

        let uuid = dispatcher
            .schedule(tcp_checker, Duration::from_secs(9999))
            .unwrap();

        dispatcher.set_health(&uuid, false).await;
    }
}
