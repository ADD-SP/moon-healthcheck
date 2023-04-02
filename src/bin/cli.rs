use moon_healthcheck::{tcp, http};
use moon_healthcheck::dispatcher::Dispatcher;
use std::time::Duration;
use tokio::time::sleep;
use clap::{arg, Command};

#[tokio::main]
async fn main() {
    let matches = Command::new("hc")
        .about("A simple healthcheck tool")
        .author("ADD-SP <add_sp@outlook.com>")
        .arg(
            arg!(--protocol <protocol> "The protocol to use for the healthcheck")
                .required(true)
                .value_parser(["tcp", "http"])
        )
        .arg(
            arg!(--host <host> "The host to check")
                .required_if_eq("protocol", "tcp")
        )
        .arg(
            arg!(--port <port> "The port to check")
                .required_if_eq("protocol", "tcp")
        )
        .arg(
            arg!(--url <url> "The url to check")
                .required_if_eq("protocol", "http")
        )
        .arg(
            arg!(--interval <interval> "The interval to check")
                .required(false)
                .default_value("1")
                .value_parser(clap::value_parser!(u64))
        )
        .arg(
            arg!(--retries <max_retries> "The max retries to check")
                .required(false)
                .default_value("3")
                .value_parser(clap::value_parser!(usize))
        )
        .get_matches();

    let protocol = matches.get_one::<String>("protocol").unwrap();
    let interval = matches.get_one::<u64>("interval").unwrap();
    let max_retries = matches.get_one::<usize>("retries").unwrap();

    match protocol.as_str() {
        "tcp" => {
            let host = matches.get_one::<String>("host").unwrap();
            let port = matches.get_one::<u16>("port").unwrap();

            let mut dispatcher: Dispatcher<tcp::TcpChecker> = Dispatcher::new();
            let tcp_checker = tcp::TcpChecker::new(host, *port, *max_retries);
            let uuid = dispatcher
                .schedule(tcp_checker, Duration::from_secs(*interval))
                .unwrap();
            loop {
                sleep(std::time::Duration::from_secs(*interval)).await;

                match dispatcher.is_healthy(&uuid).await.unwrap() {
                    true => {
                        println!("{}:{} \t ✅", host, port);
                    }
                    false => {
                        let mut err = dispatcher.get_last_error(&uuid).await.unwrap();
                        err.truncate(32);
                        println!("{}:{} \t ❌\t {}", host, port, err);
                    }
                }
            }
        }
        "http" => {
            let url = matches.get_one::<String>("url").unwrap();

            let mut dispatcher: Dispatcher<http::HttpChecker> = Dispatcher::new();
            let http_checker = http::HttpChecker::new(url, *max_retries);
            let uuid = dispatcher
                .schedule(http_checker, Duration::from_secs(*interval))
                .unwrap();
            loop {
                sleep(std::time::Duration::from_secs(*interval)).await;

                match dispatcher.is_healthy(&uuid).await.unwrap() {
                    true => {
                        println!("{} \t ✅", url);
                    }
                    false => {
                        let mut err = dispatcher.get_last_error(&uuid).await.unwrap();
                        err.truncate(32);
                        println!("{} \t ❌\t {}", url, err);
                    }
                }
            }
        }
        _ => {
            panic!("Unknown protocol: {}", protocol);
        }
    }
}