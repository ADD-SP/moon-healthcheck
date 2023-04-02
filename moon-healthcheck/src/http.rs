//! HTTP checker
//!

use super::checker::Checker;
use super::state;
use std::collections::HashMap;
use tokio::time::{Duration, Instant};


#[derive(PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Patch,
}


pub struct HttpChecker {
    uuid: uuid::Uuid,

    timeout: Duration,

    method: HttpMethod,
    version: reqwest::Version,
    url: String,
    request_headers: HashMap<String, String>,
    request_body: Option<Vec<u8>>,

    expected_status: u16,
    expected_response_headers: HashMap<String, String>,
    expected_response_body: Option<Vec<u8>>,

    state: state::State,
}


/// HTTP checker
///
/// # Examples
/// ```rust
/// use moon_healthcheck::http::{HttpMethod, HttpChecker};
///
/// #[tokio::main]
/// async fn main() {
///     let mut http_checker = HttpChecker::new("https://really.really.not.exists.host/echo-body", 1)
///         .with_method(HttpMethod::Get)                               // optional
///         .with_version(reqwest::Version::HTTP_11)                    // optional
///         .with_header("Content-Type", "text/plain")                  // optional
///         .with_header("User-Agent", "Moon Healthcheck")              // optional
///         .with_body("Hello, world!".as_bytes().to_vec())             // optional
///         .with_expected_status(200)                                  // optional
///         .with_expected_header("Content-Type", "text/plain")         // optional
///         .with_expected_body("Hello, world!".as_bytes().to_vec());   // optional
/// }
///
impl HttpChecker {
    /// Create a new HttpChecker
    ///
    /// # Arguments
    /// * `url` - The URL to check
    /// * `max_consecutive_failures` - The maximum number of consecutive failures before the checker is considered unhealthy
    ///
    /// # Examples
    /// ```rust
    /// use moon_healthcheck::http::HttpChecker;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut http_checker = HttpChecker::new("https://really.really.not.exists.host/echo-body", 1);
    /// }
    ///
    pub fn new<T: AsRef<str>>(url: T, max_consecutive_failures: usize) -> HttpChecker {
        HttpChecker {
            uuid: uuid::Uuid::new_v4(),

            timeout: Duration::from_secs(5),

            method: HttpMethod::Get,
            version: reqwest::Version::HTTP_11,
            url: url.as_ref().to_string().clone(),
            request_headers: HashMap::new(),
            request_body: None,

            expected_status: 200,
            expected_response_headers: HashMap::new(),
            expected_response_body: None,

            state: state::State::new(state::Protocol::Http, max_consecutive_failures),
        }
    }

    /// Set the timeout
    /// 
    /// Timeout for the summary of:
    /// * DNS resolution
    /// * TCP processing
    /// * TLS handshake (only for HTTPS)
    /// * HTTP processing
    ///
    /// # Arguments
    /// * `timeout` - The timeout
    ///
    /// # Examples
    /// ```rust
    /// use moon_healthcheck::http::HttpChecker;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut http_checker = HttpChecker::new("https://really.really.not.exists.host/echo-body", 1)
    ///         .with_timeout(tokio::time::Duration::from_secs(10));  // optional
    /// }
    /// ```
    ///
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the HTTP method to use
    ///
    /// # Arguments
    /// * `method` - The HTTP method to use
    ///
    /// # Examples
    /// ```rust
    /// use moon_healthcheck::http::{HttpMethod, HttpChecker};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut http_checker = HttpChecker::new("https://really.really.not.exists.host/echo-body", 1)
    ///         .with_method(HttpMethod::Get);
    /// }
    ///
    pub fn with_method(mut self, method: HttpMethod) -> Self {
        self.method = method;
        self
    }

    /// Set the HTTP version to use
    ///
    /// # Arguments
    /// * `version` - The HTTP version to use
    ///
    /// # Examples
    /// ```rust
    /// use moon_healthcheck::http::HttpChecker;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut http_checker = HttpChecker::new("https://really.really.not.exists.host/echo-body", 1)
    ///         .with_version(reqwest::Version::HTTP_11);
    ///
    ///     let mut http_checker = HttpChecker::new("https://really.really.not.exists.host/echo-body", 1)
    ///         .with_version(reqwest::Version::HTTP_2);
    /// }
    ///
    pub fn with_version(mut self, version: reqwest::Version) -> Self {
        match version {
            reqwest::Version::HTTP_11 => {}
            reqwest::Version::HTTP_2 => {}
            _ => panic!("Unsupported HTTP version"),
        }

        self.version = version;
        self
    }

    /// Add a header to the request
    ///
    /// # Arguments
    /// * `key` - The header name
    /// * `value` - The header value
    ///
    /// # Examples
    /// ```rust
    /// use moon_healthcheck::http::HttpChecker;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut http_checker = HttpChecker::new("https://really.really.not.exists.host/echo-body", 1)
    ///         .with_header("Content-Type", "text/plain")
    ///         .with_header("User-Agent", "Moon Healthcheck");
    /// }
    ///
    pub fn with_header<T: AsRef<str>>(mut self, key: T, value: T) -> Self {
        let k = key.as_ref().to_string().clone();
        let v = value.as_ref().to_string().clone();
        self.request_headers.insert(k, v);
        self
    }

    /// Set the request body
    ///
    /// # Arguments
    /// * `body` - The request body
    ///
    /// # Examples
    /// ```rust
    /// use moon_healthcheck::http::HttpChecker;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut http_checker = HttpChecker::new("https://really.really.not.exists.host/echo-body", 1)
    ///         .with_body("Hello, world!".as_bytes().to_vec());
    /// }
    ///
    pub fn with_body<T: AsRef<Vec<u8>>>(mut self, body: T) -> Self {
        self.request_body = Some(body.as_ref().to_vec());
        self
    }

    /// Set the expected HTTP status code
    ///
    /// # Arguments
    /// * `status` - The expected HTTP status code
    ///
    /// # Examples
    /// ```rust
    /// use moon_healthcheck::http::HttpChecker;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut http_checker = HttpChecker::new("https://really.really.not.exists.host/echo-body", 1)
    ///         .with_expected_status(200);
    /// }
    pub fn with_expected_status(mut self, status: u16) -> Self {
        self.expected_status = status;
        self
    }

    /// Add an expected header
    ///
    /// # Arguments
    /// * `key` - The header name
    /// * `value` - The header value
    ///
    /// # Examples
    /// ```rust
    /// use moon_healthcheck::http::HttpChecker;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut http_checker = HttpChecker::new("https://really.really.not.exists.host/echo-body", 1)
    ///         .with_expected_header("Content-Type", "text/plain");
    /// }
    ///
    pub fn with_expected_header<T: AsRef<str>>(mut self, key: T, value: T) -> Self {
        let k = key.as_ref().to_string().clone();
        let v = value.as_ref().to_string().clone();
        self.expected_response_headers.insert(k, v);
        self
    }

    /// Set the expected response body
    ///
    /// # Arguments
    /// * `body` - The expected response body
    ///
    /// # Examples
    /// ```rust
    /// use moon_healthcheck::http::HttpChecker;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut http_checker = HttpChecker::new("https://really.really.not.exists.host/echo-body", 1)
    ///         .with_expected_body("Hello, world!".as_bytes().to_vec());
    /// }
    ///
    pub fn with_expected_body<T: AsRef<Vec<u8>>>(mut self, body: T) -> Self {
        self.expected_response_body = Some(body.as_ref().to_vec());
        self
    }
}


#[async_trait::async_trait]
impl Checker for HttpChecker {
    async fn check(&mut self) {
        let timeout_at = Instant::now() + self.timeout;

        let client = reqwest::ClientBuilder::new();

        let client = match self.version {
            reqwest::Version::HTTP_10 => client.http1_only(),
            reqwest::Version::HTTP_11 => client.http1_only(),
            reqwest::Version::HTTP_2 => client.http2_prior_knowledge(),
            _ => client,
        };

        let client = client.build().unwrap();

        let mut request = match self.method {
            HttpMethod::Get => client.get(&self.url),
            HttpMethod::Post => client.post(&self.url),
            HttpMethod::Head => client.head(&self.url),
            HttpMethod::Put => client.put(&self.url),
            HttpMethod::Delete => client.delete(&self.url),
            HttpMethod::Patch => client.patch(&self.url),
        };

        request = request.version(self.version);

        for (key, value) in self.request_headers.iter() {
            request = request.header(key, value);
        }

        match self.method {
            HttpMethod::Get => {}
            HttpMethod::Head => {}
            _ => {
                if self.request_body.is_none() {
                    self.state.report_failure("Request body is required");
                    return;
                }

                request = request.body(self.request_body.clone().unwrap());
            }
        }

        // let response = request.send().await;
        let rc = tokio::time::timeout_at(timeout_at, request.send()).await;
        if rc.is_err() {
            self.state.report_failure("Request timed out");
            return;
        }

        let response = rc.unwrap();
        if response.is_err() {
            self.state.report_failure("Failed to send request");
            return;
        }

        let response = response.unwrap();

        if response.status().as_u16() != self.expected_status {
            self.state.report_failure(
                format!(
                    "Expected status {}, got {}",
                    self.expected_status,
                    response.status().as_u16()
                )
                .as_str(),
            );
            return;
        }

        for (key, value) in self.expected_response_headers.iter() {
            let header = response.headers().get(key);
            if header.is_none() {
                self.state
                    .report_failure(format!("Expected header {} not found", key).as_str());
                return;
            }

            let header = header.unwrap().to_str();
            if header.is_err() {
                self.state
                    .report_failure(format!("Failed to parse header {}", key).as_str());
                return;
            }

            let header = header.unwrap();
            if header != value {
                self.state.report_failure(
                    format!("Expected header {} to be {}, got {}", key, value, header).as_str(),
                );
                return;
            }
        }

        if self.expected_response_body.is_none() {
            self.state.report_success();
            return;
        }

        let rc = tokio::time::timeout_at(timeout_at, response.text()).await;
        if rc.is_err() {
            self.state.report_failure("Request timed out");
            return;
        }

        let body = rc.unwrap();
        if body.is_err() {
            self.state.report_failure("Failed to parse response body");
            return;
        }

        let body = body.unwrap().as_bytes().to_vec();
        if body != self.expected_response_body.clone().unwrap() {
            self.state.report_failure(format!(
                "Expected body {}, got {}",
                String::from_utf8(self.expected_response_body.clone().unwrap()).unwrap(),
                String::from_utf8(body).unwrap()
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

    fn get_protocol(&self) -> &state::Protocol {
        self.state.get_protocol()
    }

    fn get_last_check(&self) -> Option<std::time::Instant> {
        self.state.get_last_check()
    }

    fn get_last_error(&self) -> Option<String> {
        self.state.get_last_error()
    }

    fn set_health(&mut self, health: bool) {
        self.state.set_health(health)
    }
}


#[cfg(test)]
mod test {
    use ::utils;
    use super::HttpChecker;
    use super::*;

    mod healthy {
        use super::*;

        macro_rules! build_healthy_tests {
            ($sub_test:ident, $http_server_fn:expr, $http_version:expr) => {
                mod $sub_test {
                    use super::*;

                    #[tokio::test]
                    async fn with_default() {
                        let port = utils::test::rand_port();
                        let server = $http_server_fn(
                            "localhost",
                            port,
                            utils::test::http_handler_always_200,
                        )
                        .await;

                        let url = format!("http://localhost:{}", port);
                        let mut checker = HttpChecker::new(url, 3).with_version($http_version);

                        for _ in 0..5 {
                            checker.check().await;
                            assert_eq!(
                                checker.is_healthy(),
                                true,
                                "{}",
                                checker.get_last_error().unwrap()
                            );
                        }

                        server.abort();
                    }

                    #[tokio::test]
                    async fn timeout() {
                        let port = utils::test::rand_port();
                        let server =
                            $http_server_fn("localhost", port, utils::test::http_handler_delay)
                                .await;

                        let url = format!("http://localhost:{}", port);
                        let mut checker = HttpChecker::new(url, 1)
                            .with_version($http_version)
                            .with_timeout(tokio::time::Duration::from_secs(
                                utils::test::TIMEOUT + 1,
                            ));

                        for _ in 0..3 {
                            checker.check().await;
                            assert_eq!(
                                checker.is_healthy(),
                                true,
                                "{}",
                                checker.get_last_error().unwrap()
                            );
                        }

                        server.abort();
                    }

                    #[tokio::test]
                    async fn with_header() {
                        let port = utils::test::rand_port();
                        let server = $http_server_fn(
                            "localhost",
                            port,
                            utils::test::http_handler_check_header,
                        )
                        .await;

                        let url = format!("http://localhost:{}", port);
                        let mut checker = HttpChecker::new(url, 3)
                            .with_method(HttpMethod::Get)
                            .with_version($http_version)
                            .with_header(utils::test::HEADER_NAME, utils::test::HEADER_VALUE);

                        for _ in 0..5 {
                            checker.check().await;
                            assert_eq!(
                                checker.is_healthy(),
                                true,
                                "{}",
                                checker.get_last_error().unwrap()
                            );
                        }

                        server.abort();
                    }

                    #[tokio::test]
                    async fn with_body() {
                        let port = utils::test::rand_port();
                        let server = $http_server_fn(
                            "localhost",
                            port,
                            utils::test::http_handler_check_body,
                        )
                        .await;

                        let url = format!("http://localhost:{}", port);
                        let mut checker = HttpChecker::new(url, 3)
                            .with_method(HttpMethod::Post)
                            .with_version($http_version)
                            .with_body(utils::test::BODY.to_vec());

                        for _ in 0..5 {
                            checker.check().await;
                            assert_eq!(
                                checker.is_healthy(),
                                true,
                                "{}",
                                checker.get_last_error().unwrap()
                            );
                        }

                        server.abort();
                    }

                    #[tokio::test]
                    async fn with_expected_status() {
                        let port = utils::test::rand_port();
                        let server = $http_server_fn(
                            "localhost",
                            port,
                            utils::test::http_handler_always_404,
                        )
                        .await;

                        let url = format!("http://localhost:{}", port);
                        let mut checker = HttpChecker::new(url, 3)
                            .with_method(HttpMethod::Get)
                            .with_version($http_version)
                            .with_expected_status(404);

                        for _ in 0..5 {
                            checker.check().await;
                            assert_eq!(
                                checker.is_healthy(),
                                true,
                                "{}",
                                checker.get_last_error().unwrap()
                            );
                        }

                        server.abort();
                    }

                    #[tokio::test]
                    async fn with_expected_header() {
                        let port = utils::test::rand_port();
                        let server = $http_server_fn(
                            "localhost",
                            port,
                            utils::test::http_handler_always_return_header,
                        )
                        .await;

                        let url = format!("http://localhost:{}", port);
                        let mut checker = HttpChecker::new(url, 3)
                            .with_method(HttpMethod::Get)
                            .with_version($http_version)
                            .with_expected_header(
                                utils::test::HEADER_NAME,
                                utils::test::HEADER_VALUE,
                            );

                        for _ in 0..5 {
                            checker.check().await;
                            assert_eq!(
                                checker.is_healthy(),
                                true,
                                "{}",
                                checker.get_last_error().unwrap()
                            );
                        }

                        server.abort();
                    }

                    #[tokio::test]
                    async fn with_expected_body() {
                        let port = utils::test::rand_port();
                        let server =
                            $http_server_fn("localhost", port, utils::test::http_handler_echo_body)
                                .await;

                        let url = format!("http://localhost:{}", port);
                        let mut checker = HttpChecker::new(url, 3)
                            .with_method(HttpMethod::Post)
                            .with_version($http_version)
                            .with_header("Content-Type", "text/plain")
                            .with_body(utils::test::BODY.to_vec())
                            .with_expected_body(utils::test::BODY.to_vec());

                        for _ in 0..5 {
                            checker.check().await;
                            assert_eq!(
                                checker.is_healthy(),
                                true,
                                "{}",
                                checker.get_last_error().unwrap()
                            );
                        }

                        server.abort();
                    }
                }
            };
        }

        build_healthy_tests!(http1, utils::test::http1_server, reqwest::Version::HTTP_11);
        build_healthy_tests!(http2, utils::test::http2_server, reqwest::Version::HTTP_2);
    }

    mod un_healthy {
        use super::*;

        macro_rules! build_un_healthy_tests {
            ($sub_test:ident, $http_server_fn:expr, $http_version:expr) => {
                mod $sub_test {
                    use super::*;

                    #[tokio::test]
                    async fn with_default() {
                        let port = utils::test::rand_port();
                        let server = $http_server_fn(
                            "localhost",
                            port,
                            utils::test::http_handler_always_404,
                        )
                        .await;

                        let url = format!("http://localhost:{}", port);
                        let mut checker = HttpChecker::new(url, 3).with_version($http_version);

                        for _ in 0..3 {
                            checker.check().await;
                            assert_eq!(checker.is_healthy(), true);
                        }

                        for _ in 0..3 {
                            checker.check().await;
                            assert_eq!(checker.is_healthy(), false);
                        }

                        server.abort();
                    }

                    #[tokio::test]
                    async fn timeout() {
                        let port = utils::test::rand_port();
                        let server =
                            $http_server_fn("localhost", port, utils::test::http_handler_delay)
                                .await;

                        let url = format!("http://localhost:{}", port);
                        let mut checker = HttpChecker::new(url, 1)
                            .with_version($http_version)
                            .with_timeout(tokio::time::Duration::from_secs(
                                utils::test::TIMEOUT - 1,
                            ));

                        checker.check().await;
                        assert_eq!(
                            checker.is_healthy(),
                            true,
                            "{}",
                            checker.get_last_error().unwrap()
                        );

                        for _ in 0..3 {
                            checker.check().await;
                            assert_eq!(checker.is_healthy(), false, "{}", "should be unhealthy");
                        }

                        server.abort();
                    }

                    #[tokio::test]
                    async fn with_header() {
                        let port = utils::test::rand_port();
                        let server = $http_server_fn(
                            "localhost",
                            port,
                            utils::test::http_handler_check_header,
                        )
                        .await;

                        let url = format!("http://localhost:{}", port);
                        let mut checker = HttpChecker::new(url, 3)
                            .with_method(HttpMethod::Get)
                            .with_version($http_version)
                            .with_header(utils::test::HEADER_NAME, "wrong value");

                        for _ in 0..3 {
                            checker.check().await;
                            assert_eq!(checker.is_healthy(), true);
                        }

                        for _ in 0..3 {
                            checker.check().await;
                            assert_eq!(checker.is_healthy(), false);
                        }

                        server.abort();
                    }

                    #[tokio::test]
                    async fn with_body() {
                        let port = utils::test::rand_port();
                        let server = $http_server_fn(
                            "localhost",
                            port,
                            utils::test::http_handler_check_body,
                        )
                        .await;

                        let url = format!("http://localhost:{}", port);
                        let mut checker = HttpChecker::new(url, 3)
                            .with_method(HttpMethod::Post)
                            .with_version($http_version)
                            .with_body("wrong body".as_bytes().to_vec());

                        for _ in 0..3 {
                            checker.check().await;
                            assert_eq!(checker.is_healthy(), true);
                        }

                        for _ in 0..3 {
                            checker.check().await;
                            assert_eq!(checker.is_healthy(), false);
                        }

                        server.abort();
                    }

                    #[tokio::test]
                    async fn with_expected_status() {
                        let port = utils::test::rand_port();
                        let server =
                            $http_server_fn("localhost", port, utils::test::http_handler_echo_body)
                                .await;

                        let url = format!("http://localhost:{}", port);
                        let mut checker = HttpChecker::new(url, 3)
                            .with_method(HttpMethod::Get)
                            .with_version($http_version)
                            .with_expected_status(404);

                        for _ in 0..3 {
                            checker.check().await;
                            assert_eq!(checker.is_healthy(), true);
                        }

                        for _ in 0..3 {
                            checker.check().await;
                            assert_eq!(checker.is_healthy(), false);
                        }

                        server.abort();
                    }

                    #[tokio::test]
                    async fn with_expected_header() {
                        let port = utils::test::rand_port();
                        let server = $http_server_fn(
                            "localhost",
                            port,
                            utils::test::http_handler_always_return_header,
                        )
                        .await;

                        let url = format!("http://localhost:{}", port);
                        let mut checker = HttpChecker::new(url, 3)
                            .with_method(HttpMethod::Get)
                            .with_version($http_version)
                            .with_expected_header(utils::test::HEADER_NAME, "wrong value");

                        for _ in 0..3 {
                            checker.check().await;
                            assert_eq!(checker.is_healthy(), true);
                        }

                        for _ in 0..3 {
                            checker.check().await;
                            assert_eq!(checker.is_healthy(), false);
                        }

                        server.abort();
                    }

                    #[tokio::test]
                    async fn with_expected_body() {
                        let port = utils::test::rand_port();
                        let server =
                            $http_server_fn("localhost", port, utils::test::http_handler_echo_body)
                                .await;

                        let url = format!("http://localhost:{}", port);
                        let mut checker = HttpChecker::new(url, 3)
                            .with_method(HttpMethod::Post)
                            .with_version($http_version)
                            .with_expected_body("wrong body".as_bytes().to_vec());

                        for _ in 0..3 {
                            checker.check().await;
                            assert_eq!(checker.is_healthy(), true);
                        }

                        for _ in 0..3 {
                            checker.check().await;
                            assert_eq!(checker.is_healthy(), false);
                        }

                        server.abort();
                    }
                }
            };
        }

        build_un_healthy_tests!(http1, utils::test::http1_server, reqwest::Version::HTTP_11);
        build_un_healthy_tests!(http2, utils::test::http2_server, reqwest::Version::HTTP_2);
    }
}
