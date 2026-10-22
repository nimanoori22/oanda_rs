use super::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

fn client(server: &MockServer, rate: usize, retries: usize) -> OandaClient {
    let mut client =
        OandaClient::new(Some("test-account"), "test-token", 10, 2, rate, retries).unwrap();
    client.base_url = server.uri();
    client
}

#[derive(Clone)]
struct RetryOnce(Arc<AtomicUsize>);

impl Respond for RetryOnce {
    fn respond(&self, _: &Request) -> ResponseTemplate {
        if self.0.fetch_add(1, Ordering::SeqCst) == 0 {
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "1")
                .set_body_json(serde_json::json!({"errorMessage": "slow down"}))
        } else {
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true}))
        }
    }
}

#[tokio::test]
async fn retries_429_and_honours_retry_after() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/retry"))
        .respond_with(RetryOnce(Arc::new(AtomicUsize::new(0))))
        .expect(2)
        .mount(&server)
        .await;
    let mut client = client(&server, 100, 1);
    let start = Instant::now();
    assert_eq!(client.get("/retry").await.unwrap()["ok"], true);
    assert!(start.elapsed() >= Duration::from_secs(1));
}

#[tokio::test]
async fn rejects_http_errors_even_without_error_message() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(403).set_body_json(serde_json::json!({"reason": "forbidden"})),
        )
        .expect(1)
        .mount(&server)
        .await;
    assert!(client(&server, 100, 2).get("/denied").await.is_err());
}

#[tokio::test]
async fn patch_sends_authentication_and_json() {
    let server = MockServer::start().await;
    let body = serde_json::json!({"alias": "test"});
    Mock::given(method("PATCH"))
        .and(path("/configuration"))
        .and(header("authorization", "Bearer test-token"))
        .and(header("content-type", "application/json"))
        .and(body_json(body.clone()))
        .respond_with(ResponseTemplate::new(200).set_body_json(body.clone()))
        .expect(1)
        .mount(&server)
        .await;
    assert_eq!(
        client(&server, 100, 0)
            .patch("/configuration", &body)
            .await
            .unwrap(),
        body
    );
}

#[tokio::test]
async fn clones_share_request_rate_limit() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .expect(5)
        .mount(&server)
        .await;
    let mut first = client(&server, 2, 0);
    first.get("/warm-up").await.unwrap();
    // Prime pacing after the initial connection wait has finished.
    first.get("/prime").await.unwrap();
    let mut second = first.clone();
    let start = Instant::now();
    let (a, b) = tokio::join!(first.get("/first"), second.get("/second"));
    a.unwrap();
    b.unwrap();
    second.get("/third").await.unwrap();
    assert!(start.elapsed() >= Duration::from_millis(1400));
}

#[tokio::test]
async fn exhausted_retries_preserve_oanda_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(503)
                .insert_header("Retry-After", "0")
                .set_body_json(serde_json::json!({"errorMessage": "unavailable"})),
        )
        .expect(2)
        .mount(&server)
        .await;
    let error = client(&server, 100, 1)
        .get("/unavailable")
        .await
        .unwrap_err();
    assert!(error.to_string().contains("unavailable"), "{error}");
}

#[tokio::test]
async fn validates_limits_and_clones_account_id() {
    assert!(OandaClient::new(None, "test-token", 1, 0, 100, 0).is_err());
    assert!(OandaClient::new(None, "test-token", 1, 1, 0, 0).is_err());
    assert!(OandaClient::new(None, "test-token", 1, 1, 101, 0).is_err());
    assert!(OandaClient::new(None, "test-token", 1, 1, 100, usize::MAX).is_err());
    let client = OandaClient::new(Some("test-account"), "test-token", 1, 1, 100, 0).unwrap();
    assert_eq!(client.clone().get_account_id(), client.get_account_id());
}

#[tokio::test]
async fn selects_practice_and_live_rest_urls() {
    let practice = OandaClient::new(None, "test-token", 1, 1, 100, 0).unwrap();
    let live =
        OandaClient::new_with_environment(None, "test-token", 1, 1, 100, 0, Environment::Live)
            .unwrap();
    assert_eq!(practice.base_url, "https://api-fxpractice.oanda.com");
    assert_eq!(live.base_url, "https://api-fxtrade.oanda.com");
}

#[tokio::test]
async fn zero_retries_sends_only_once() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(429)
                .set_body_json(serde_json::json!({"errorMessage": "rate limited"})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let error = client(&server, 100, 0).get("/limited").await.unwrap_err();
    assert!(error.to_string().contains("rate limited"));
}

async fn tcp_server(
    close: bool,
) -> (
    String,
    tokio::sync::mpsc::UnboundedReceiver<(usize, Instant)>,
    tokio::task::JoinHandle<()>,
) {
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let task = tokio::spawn(async move {
        let mut connection_id = 0;
        loop {
            let (mut stream, _) = listener.accept().await.unwrap();
            connection_id += 1;
            let id = connection_id;
            let tx = tx.clone();
            tokio::spawn(async move {
                loop {
                    let mut headers = Vec::new();
                    while !headers.ends_with(b"\r\n\r\n") {
                        let Ok(byte) = stream.read_u8().await else {
                            return;
                        };
                        headers.push(byte);
                    }
                    if tx.send((id, Instant::now())).is_err() {
                        return;
                    }
                    let connection = if close { "close" } else { "keep-alive" };
                    let response = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: {connection}\r\n\r\n{{}}");
                    if stream.write_all(response.as_bytes()).await.is_err() || close {
                        return;
                    }
                }
            });
        }
    });
    (url, rx, task)
}

#[tokio::test]
async fn clones_reuse_a_persistent_connection() {
    let (url, mut received, server) = tcp_server(false).await;
    let mut first = OandaClient::new(None, "test-token", 1, 2, 100, 0).unwrap();
    first.base_url = url;
    let mut second = first.clone();
    tokio::time::timeout(Duration::from_secs(10), async {
        first.get("/first").await.unwrap();
        second.get("/second").await.unwrap();
    })
    .await
    .unwrap();
    assert_eq!(
        received.recv().await.unwrap().0,
        received.recv().await.unwrap().0
    );
    server.abort();
}

#[tokio::test]
async fn separate_clients_share_connection_start_limit() {
    let (url, mut received, server) = tcp_server(true).await;
    let mut first = OandaClient::new(None, "test-token", 1, 1, 100, 0).unwrap();
    let mut second = OandaClient::new(None, "test-token", 1, 1, 100, 0).unwrap();
    first.base_url = url.clone();
    second.base_url = url;
    tokio::time::timeout(Duration::from_secs(10), async {
        first.get("/first").await.unwrap();
        second.get("/second").await.unwrap();
    })
    .await
    .unwrap();
    let (first_id, first_time) = received.recv().await.unwrap();
    let (second_id, second_time) = received.recv().await.unwrap();
    assert_ne!(first_id, second_id);
    assert!(second_time.duration_since(first_time) >= Duration::from_millis(475));
    server.abort();
}

#[tokio::test]
async fn explicit_proxy_routes_get_and_patch_across_clones() {
    let proxy = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/via-proxy"))
        .and(header("authorization", "Bearer test-token"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"proxied": true})),
        )
        .expect(1)
        .mount(&proxy)
        .await;
    let body = serde_json::json!({"alias": "proxied"});
    Mock::given(method("PATCH"))
        .and(path("/via-proxy"))
        .and(body_json(body.clone()))
        .and(header("authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body.clone()))
        .expect(1)
        .mount(&proxy)
        .await;
    let mut first = OandaClient::new_with_config(
        None,
        "test-token",
        ClientConfig {
            proxy: Some(proxy.uri()),
            retry_attempts: 0,
            ..ClientConfig::default()
        },
    )
    .unwrap();
    // This hostname is deliberately unresolvable; only the proxy can answer.
    first.base_url = "http://oanda.invalid".into();
    let mut second = first.clone();
    assert_eq!(first.get("/via-proxy").await.unwrap()["proxied"], true);
    assert_eq!(second.patch("/via-proxy", &body).await.unwrap(), body);
}

#[tokio::test]
async fn invalid_proxy_is_rejected_at_construction() {
    assert!(OandaClient::new_with_config(
        None,
        "test-token",
        ClientConfig {
            proxy: Some("://invalid".into()),
            ..ClientConfig::default()
        }
    )
    .is_err());
}
