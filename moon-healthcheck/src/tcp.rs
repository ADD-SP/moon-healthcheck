//! TCP checker
//!

use super::checker::Checker;
use super::state::{Protocol, State};
use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::Duration;


/// TCP checker
pub struct TcpChecker {
    uuid: uuid::Uuid,
    host: String,
    port: u16,
    timeout: Duration,
    payload: Option<Vec<u8>>,
    expected_response: Option<Vec<u8>>,
    state: State,
}


/// TCP checker
/// 
/// # Examples
/// ```rust
/// use moon_healthcheck::tcp::TcpChecker;
/// 
/// #[tokio::main]
/// async fn main() {
///     let mut checker = TcpChecker::new("really.really.not.exists.echo.host", 80, 1)
///         .with_payload(vec![1, 2, 3])            // optional
///         .with_expected_response(vec![1, 2, 3]); // optional
/// }
/// 
impl TcpChecker {
    /// Create a new TcpChecker
    /// 
    /// # Arguments
    /// * `host` - The host to check
    /// * `port` - The port to check
    /// * `max_failures` - The maximum number of failures before the checker is considered unhealthy
    /// 
    pub fn new<T>(host: T, port: u16, max_failures: usize) -> TcpChecker
    where
        T: AsRef<str>,
    {
        TcpChecker {
            uuid: uuid::Uuid::new_v4(),
            host: host.as_ref().to_string().clone(),
            port,
            timeout: Duration::from_secs(5),
            payload: None,
            expected_response: None,
            state: State::new(Protocol::Tcp, max_failures),
        }
    }

    /// Set the timeout for checking
    /// 
    /// Timeout for the summary of:
    /// * DNS resolution
    /// * Connecting to the host
    /// * Sending the payload
    /// * Receiving the expected response
    /// 
    /// # Arguments
    /// * `timeout` - The timeout
    /// 
    /// # Default
    /// 5 seconds
    /// 
    /// # Example
    /// ```rust
    /// use moon_healthcheck::tcp::TcpChecker;
    /// 
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut checker = TcpChecker::new("really.really.not.exists.echo.host", 80, 1)
    ///         .with_timeout(tokio::time::Duration::from_secs(10));
    /// }
    /// 
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the payload to send to the host
    /// 
    /// # Arguments
    /// * `payload` - The payload to send
    pub fn with_payload<T>(mut self, payload: T) -> Self
    where
        T: AsRef<Vec<u8>>,
    {
        self.payload = Some(payload.as_ref().to_vec());
        self
    }

    /// Set the expected response from the host
    /// 
    /// # Arguments
    /// * `expected_response` - The expected response
    pub fn with_expected_response<T>(mut self, expected_response: T) -> Self
    where
        T: AsRef<Vec<u8>>,
    {
        self.expected_response = Some(expected_response.as_ref().to_vec());
        self
    }
}


#[async_trait]
impl Checker for TcpChecker {
    async fn check(&mut self) {
        let addr = format!("{}:{}", self.host, self.port);
        let timeout_at = tokio::time::Instant::now() + self.timeout;


        let stream = tokio::time::timeout_at(timeout_at, TcpStream::connect(addr.clone())).await;
        if stream.is_err() {
            self.state.report_failure(format!("Failed to connect to {}: timeout", addr.clone()));
            return;
        }

        let stream = stream.unwrap();
        if stream.is_err() {
            self.state.report_failure(format!("Failed to connect to {}: {}", addr.clone(), stream.unwrap_err()));
            return;
        }

        if self.payload.is_some() {
            let mut stream = stream.unwrap();
            let payload = self.payload.as_ref().unwrap();

            let rc = tokio::time::timeout_at(timeout_at, stream.write_all(payload)).await;

            if rc.is_err() {
                self.state.report_failure(format!("Failed to send payload to {}: timeout", addr.clone()));
                return;
            }

            let rc = rc.unwrap();
            if rc.is_err() {
                self.state.report_failure(format!("Failed to send payload to {}: {}", addr.clone(), rc.unwrap_err()));
                return;
            }
            
            let mut response = vec![0; payload.len()];
            let rc = tokio::time::timeout_at(timeout_at, stream.read_exact(&mut response)).await;
            if rc.is_err() {
                self.state.report_failure(format!("Failed to receive response from {}: timeout", addr.clone()));
                return;
            }

            let rc = rc.unwrap();
            if rc.is_err() {
                self.state.report_failure(format!("Failed to receive response from {}: {}", addr.clone(), rc.unwrap_err()));
                return;
            }

            match *(self.expected_response.as_ref().unwrap()) == response {
                true => self.state.report_success(),
                false => self.state.report_failure(format!("Expected response {:?} but got {:?}", self.expected_response, response)),
            }
        } else {
            self.state.report_success();
        }
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

    fn report_success(&mut self) {
        self.state.report_success()
    }

    fn report_failure(&mut self, error: String) {
        self.state.report_failure(error)
    }

    fn set_health(&mut self, health: bool) {
        self.state.set_health(health);
    }
}


#[cfg(test)]
mod test {
    use ::utils;
    use super::*;

    mod healthy {
        use super::*;

        #[tokio::test]
        async fn timeout() {
            let port = utils::test::rand_port();
            let server = utils::test::tcp_server_with_delay("localhost", port).await;
    
            let mut checker = TcpChecker::new("localhost", port, 1)
                .with_timeout(Duration::from_secs(utils::test::TIMEOUT + 10));
            

            for _ in 0..3 {
                checker.check().await;
                assert_eq!(checker.is_healthy(), true);
            }
    
            server.abort();
        }

        #[tokio::test]
        async fn without_payload() {
            let port = utils::test::rand_port();
            let server = utils::test::tcp_server("localhost", port).await;
    
            let mut checker = TcpChecker::new("localhost", port, 3);
            
            for _ in 0..5 {
                checker.check().await;
            }
            assert_eq!(checker.is_healthy(), true);
    
            server.abort();
        }

        #[tokio::test]
        async fn with_payload() {
            let port = utils::test::rand_port();
            let server = utils::test::tcp_server("localhost", port).await;

            let mut checker = TcpChecker::new("localhost", port, 3)
                .with_payload(vec![1, 2, 3])
                .with_expected_response(vec![1, 2, 3]);
            
            for _ in 0..5 {
                checker.check().await;
            }
            assert_eq!(checker.is_healthy(), true);

            server.abort();
        }

        #[tokio::test]
        async fn passive() {
            let port = utils::test::rand_port();

            let mut checker = TcpChecker::new("localhost", port, 1);
            
            checker.check().await;
            assert!(checker.is_healthy());

            checker.check().await;
            assert!(!checker.is_healthy());

            checker.report_success();
            assert!(checker.is_healthy());
        }
    }

    mod un_healthy {
        use super::*;

        #[tokio::test]
        async fn timeout() {
            let port = utils::test::rand_port();
            let server = utils::test::tcp_server_with_delay("localhost", port).await;
            let mut checker = TcpChecker::new("localhost", port, 1)
                .with_timeout(Duration::from_secs(utils::test::TIMEOUT - 1))
                .with_payload(vec![1, 2, 3])
                .with_expected_response(vec![1, 2, 3]);
            
            checker.check().await;
            assert_eq!(checker.is_healthy(), true);

            checker.check().await;
            assert_eq!(checker.is_healthy(), false);

            server.abort();
        }

        #[tokio::test]
        async fn without_payload() {
            let port = utils::test::rand_port();
            let mut checker = TcpChecker::new("localhost", port, 3);
            
            for _ in 0..3 {
                checker.check().await;
                assert_eq!(checker.is_healthy(), true);
            }
            
            checker.check().await;
            assert_eq!(checker.is_healthy(), false);
        }

        #[tokio::test]
        async fn with_payload() {
            let port = utils::test::rand_port();
            let mut checker = TcpChecker::new("localhost", port, 3)
                .with_payload(vec![1, 2, 3])
                .with_expected_response(vec![1, 2, 3]);
            
            for _ in 0..3 {
                checker.check().await;
                assert_eq!(checker.is_healthy(), true);
            }
            
            checker.check().await;
            assert_eq!(checker.is_healthy(), false);
        }

        #[tokio::test]
        async fn passive() {
            let port = utils::test::rand_port();

            let tcp_server = utils::test::tcp_server("localhost", port).await;

            let mut checker = TcpChecker::new("localhost", port, 1);

            for _ in 0..5 {
                checker.check().await;
                assert!(checker.is_healthy());
            }

            checker.report_failure(String::from("test"));
            assert!(checker.is_healthy());

            checker.report_failure(String::from("test"));
            assert!(!checker.is_healthy());

            tcp_server.abort();
        }
    }

    
}
