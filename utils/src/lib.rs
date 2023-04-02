pub mod test {
    use std::convert::Infallible;

    use futures::Future;
    use hyper::server::conn::Http;
    use hyper::service::service_fn;
    use hyper::{StatusCode, Body};
    use hyper::{Request, Response};
    use rand::Rng;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tokio::task::JoinHandle;

    pub const HEADER_NAME: &str = "X-Test-Header";
    pub const HEADER_VALUE: &str = "test-value";
    pub const BODY: [u8; 4] = [1, 2, 3, 4];
    pub const TIMEOUT: u64 = 3;

    #[derive(Clone)]
    struct TokioExecutor;

    impl<F> hyper::rt::Executor<F> for TokioExecutor
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        fn execute(&self, future: F) {
            tokio::spawn(future);
        }
    }

    pub async fn http_handler_echo_body(
        request: Request<Body>,
    ) -> Result<Response<Body>, Infallible> {
        Ok(Response::new(request.into_body()))
    }

    pub async fn http_handler_delay(
        _request: Request<Body>,
    ) -> Result<Response<Body>, Infallible> {
        tokio::time::sleep(tokio::time::Duration::from_secs(TIMEOUT)).await;
        Ok(Response::builder()
            .status(StatusCode::OK)
            .body(Body::empty())
            .unwrap())
    }

    pub async fn http_handler_always_200 (
        _request: Request<Body>,
    ) -> Result<Response<Body>, Infallible> {
        Ok(Response::builder()
            .status(StatusCode::OK)
            .body(Body::empty())
            .unwrap())
    }

    pub async fn http_handler_always_404(
        _request: Request<Body>,
    ) -> Result<Response<Body>, Infallible> {
        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap())
    }

    pub async fn http_handler_always_return_header(
        _request: Request<Body>,
    ) -> Result<Response<Body>, Infallible> {
        Ok(Response::builder()
            .header(HEADER_NAME, HEADER_VALUE)
            .body(Body::empty())
            .unwrap())
    }

    pub async fn http_handler_check_header(
        request: Request<Body>,
    ) -> Result<Response<Body>, Infallible> {
        let mut response = Response::new(Body::empty());

        if request
            .headers()
            .get(HEADER_NAME)
            .map(|v| v.to_str().unwrap_or(""))
            .unwrap_or("")
            == HEADER_VALUE
        {
            *response.status_mut() = StatusCode::OK;
        } else {
            *response.status_mut() = StatusCode::BAD_REQUEST;
        }

        Ok(response)
    }

    pub async fn http_handler_check_body(
        request: Request<Body>,
    ) -> Result<Response<Body>, Infallible> {
        let mut response = Response::new(Body::empty());

        let body = hyper::body::to_bytes(request.into_body()).await.unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();

        if body.as_bytes().to_vec() == BODY {
            *response.status_mut() = StatusCode::OK;
        } else {
            *response.status_mut() = StatusCode::BAD_REQUEST;
        }

        Ok(response)
    }

    pub fn rand_port() -> u16 {
        let mut rng = rand::thread_rng();
        rng.gen_range(20000..60000)
    }

    pub async fn tcp_server(host: &str, port: u16) -> JoinHandle<()> {
        let addr = format!("{}:{}", host, port);
        let listener = TcpListener::bind(addr).await.unwrap();

        tokio::spawn(async move {
            loop {
                let (mut socket, _) = listener.accept().await.unwrap();
                tokio::spawn(async move {
                    let mut buf = [0; 1024];
                    let _ = socket.read(&mut buf).await;
                    let _ = socket.write(&buf).await;
                });
            }
        })
    }

    pub async fn tcp_server_with_delay(host: &str, port: u16) -> JoinHandle<()> {
        let addr = format!("{}:{}", host, port);
        let listener = TcpListener::bind(addr).await.unwrap();

        tokio::spawn(async move {
            loop {
                let (mut socket, _) = listener.accept().await.unwrap();
                tokio::time::sleep(std::time::Duration::from_secs(TIMEOUT)).await;
                tokio::spawn(async move {
                    let mut buf = [0; 1024];
                    let _ = socket.read(&mut buf).await;
                    let _ = socket.write(&buf).await;
                });
            }
        })
    }

    pub async fn http1_server<F, S>(host: &str, port: u16, f: F) -> JoinHandle<()>
    where
        F: FnMut(Request<Body>) -> S + Send + std::marker::Copy + 'static,
        S: Future<Output = Result<Response<Body>, Infallible>> + Send + Sync + 'static,
    {
        let addr = format!("{}:{}", host, port);
        let listener = TcpListener::bind(addr).await.unwrap();

        tokio::spawn(async move {
            loop {
                let (tcp_stream, _) = listener.accept().await.unwrap();

                tokio::spawn(async move {
                    if let Err(err) = Http::new()
                        .http1_only(true)
                        .serve_connection(tcp_stream, service_fn(f))
                        .await
                    {
                        panic!("Error serving connection: {:?}", err);
                    }
                });
            }
        })
    }

    pub async fn http2_server<F, S>(host: &str, port: u16, f: F) -> JoinHandle<()>
    where
        F: FnMut(Request<Body>) -> S + Send + std::marker::Copy + 'static,
        S: Future<Output = Result<Response<Body>, Infallible>> + Send + Sync + 'static,
    {
        let addr = format!("{}:{}", host, port);
        let listener = TcpListener::bind(addr).await.unwrap();

        tokio::spawn(async move {
            loop {
                let (tcp_stream, _) = listener.accept().await.unwrap();

                tokio::spawn(async move {
                    if let Err(err) = Http::new()
                        .http2_only(true)
                        .serve_connection(tcp_stream, service_fn(f))
                        .await
                    {
                        panic!("Error serving connection: {:?}", err);
                    }
                });
            }
        })
    }
}
