//! Udp Checker
//!

use super::checker::Checker;
use super::state::{Protocol, State};
use async_trait::async_trait;
use tokio::net::UdpSocket;
use tokio::time::Duration;

/// UdpChecker
pub struct UdpChecker {
    uuid: uuid::Uuid,
    host: String,
    port: u16,
    timeout: Duration,
    payload: Vec<u8>,
    expected_response: Vec<u8>,
    state: State,
}

/// UdpChecker
/// 
/// # Example
/// ```rust
/// use moon_healthcheck::udp::UdpChecker;
/// 
/// #[tokio::main]
/// async fn main() {
///     let mut checker = UdpChecker::new("localhost", 53, &vec![1, 2, 3], &vec![1, 2, 3], 1)
///         .with_timeout(tokio::time::Duration::from_secs(3)); // optional
/// }
/// ```
/// 
impl UdpChecker {
    /// Create a new UdpChecker
    /// 
    /// # Arguments
    /// * `host` - Hostname or IP address
    /// * `port` - Port number
    /// * `payload` - Payload to send
    /// * `expected_response` - Expected response
    /// * `max_consecutive_failures` - Maximum consecutive failures before marking the checker as unhealthy
    /// 
    /// # Example
    /// ```rust
    /// use moon_healthcheck::udp::UdpChecker;
    /// 
    /// #[tokio::main]
    /// async fn main() {
    ///    let mut checker = UdpChecker::new("localhost", 53, &vec![1, 2, 3], &vec![1, 2, 3], 1)
    ///         .with_timeout(tokio::time::Duration::from_secs(3)); // optional
    /// }
    /// 
    pub fn new<T>(
        host: T,
        port: u16,
        payload: &Vec<u8>,
        expected_response: &Vec<u8>,
        max_consecutive_failures: usize,
    ) -> UdpChecker
    where
        T: AsRef<str>,
    {
        UdpChecker {
            uuid: uuid::Uuid::new_v4(),
            host: host.as_ref().to_string().clone(),
            port,
            timeout: Duration::from_secs(5),
            payload: payload.clone(),
            expected_response: expected_response.clone(),
            state: State::new(Protocol::Udp, max_consecutive_failures),
        }
    }

    /// Set the timeout
    /// 
    /// 
    /// # Arguments
    /// * `timeout` - The sum of the connect, read and write timeouts
    /// 
    /// # Example
    /// ```rust
    /// use moon_healthcheck::udp::UdpChecker;
    /// 
    /// #[tokio::main]
    /// async fn main() {
    ///    let mut checker = UdpChecker::new("localhost", 53, &vec![1, 2, 3], &vec![1, 2, 3], 1)
    ///         .with_timeout(tokio::time::Duration::from_secs(3));
    /// }
    /// 
    pub fn with_timeout(mut self, timeout: Duration) -> UdpChecker {
        self.timeout = timeout;
        self
    }
}

#[async_trait]
impl Checker for UdpChecker {
    async fn check(&mut self) {
        let timeout_at = tokio::time::Instant::now() + self.timeout;

        let rc = tokio::time::timeout_at(timeout_at, UdpSocket::bind("localhost:0")).await;
        if rc.is_err() {
            self.state
                .report_failure(String::from("Failed to bind to localhost: timeout"));
            return;
        }

        let rc = rc.unwrap();
        if rc.is_err() {
            self.state.report_failure(String::from(format!(
                "Failed to bind to localhost: {}",
                rc.unwrap_err()
            )));
            return;
        }

        let socket = rc.unwrap();

        let rc = tokio::time::timeout_at(
            timeout_at,
            socket.connect(format!("{}:{}", self.host, self.port)),
        )
        .await;

        if rc.is_err() {
            self.state
                .report_failure(String::from("Failed to connect to host: timeout"));
            return;
        }

        let rc = rc.unwrap();
        if rc.is_err() {
            self.state.report_failure(String::from(format!(
                "Failed to connect to remote server: {}",
                rc.unwrap_err()
            )));
            return;
        }

        let payload_size = self.payload.len();
        let payload = self.payload.as_slice();
        let rc = tokio::time::timeout_at(timeout_at, socket.send(payload)).await;
        if rc.is_err() {
            self.state
                .report_failure(String::from("Failed to send payload: timeout"));
            return;
        }

        let rc = rc.unwrap();
        if rc.is_err() {
            self.state.report_failure(String::from(format!(
                "Failed to send payload: {}",
                rc.unwrap_err()
            )));
            return;
        }

        let mut buf = vec![0; payload_size];
        let rc = tokio::time::timeout_at(timeout_at, socket.recv(&mut buf)).await;
        if rc.is_err() {
            self.state
                .report_failure(String::from("Failed to receive response: timeout"));
            return;
        }

        let rc = rc.unwrap();
        if rc.is_err() {
            self.state.report_failure(String::from(format!(
                "Failed to receive response: {}",
                rc.unwrap_err()
            )));
            return;
        }

        if self.expected_response.as_slice() != buf {
            self.state.report_failure(String::from(
                "Received response does not match expected response",
            ));
            return;
        }

        self.state.report_success();
    }

    fn is_healthy(&self) -> bool {
        self.state.is_healthy()
    }

    fn get_uuid(&self) -> uuid::Uuid {
        self.uuid
    }

    fn get_protocol(&self) -> &Protocol {
        self.state.get_protocol()
    }

    fn get_last_check(&self) -> Option<std::time::Instant> {
        self.state.get_last_check()
    }

    fn get_last_error(&self) -> Option<String> {
        self.state.get_last_error()
    }

    fn set_health(&mut self, health: bool) {
        self.state.set_health(health);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ::utils;

    mod healthy {
        use super::*;

        #[tokio::test]
        async fn with_default() {
            let port = utils::test::rand_port();
            let payload: Vec<u8> = vec![0x01, 0x02, 0x03, 0x04];

            let udp_server = utils::test::udp_server_echo("localhost", port).await;

            let mut checker = UdpChecker::new("localhost", port, &payload, &payload, 1);

            for _ in 0..5 {
                checker.check().await;
                assert!(checker.is_healthy(), "checker should be healthy");
            }

            udp_server.abort();
        }

        #[tokio::test]
        async fn with_timeout() {
            let port = utils::test::rand_port();
            let payload: Vec<u8> = vec![0x01, 0x02, 0x03, 0x04];

            let udp_server = utils::test::udp_server_echo_delay("localhost", port).await;

            let mut checker = UdpChecker::new("localhost", port, &payload, &payload, 1)
                .with_timeout(tokio::time::Duration::from_secs(utils::test::TIMEOUT + 1));

            for _ in 0..5 {
                checker.check().await;
                assert!(checker.is_healthy(), "checker should be healthy");
            }

            udp_server.abort();
        }
    }

    mod unhealthy {
        use super::*;

        #[tokio::test]
        async fn with_default() {
            let port = utils::test::rand_port();
            let payload: Vec<u8> = vec![0x01, 0x02, 0x03, 0x04];

            let mut checker = UdpChecker::new("localhost", port, &payload, &payload, 1);

            checker.check().await;
            assert!(checker.is_healthy());

            for _ in 0..5 {
                checker.check().await;
                assert!(!checker.is_healthy(), "checker should be unhealthy");
            }
        }

        #[tokio::test]
        async fn with_timeout() {
            let port = utils::test::rand_port();
            let payload: Vec<u8> = vec![0x01, 0x02, 0x03, 0x04];

            let mut checker = UdpChecker::new("localhost", port, &payload, &payload, 1)
                .with_timeout(tokio::time::Duration::from_secs(utils::test::TIMEOUT - 1));

            checker.check().await;
            assert!(checker.is_healthy());

            for _ in 0..5 {
                checker.check().await;
                assert!(!checker.is_healthy(), "checker should be unhealthy");
            }
        }
    }
}
