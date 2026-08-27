use std::sync::{Arc, Mutex};

use jayjay_network::{Header, Method, Request, Response};

use super::{HttpTransport, NetworkClient, NetworkError};

#[derive(Debug)]
struct MockTransport {
    request: Mutex<Option<Request>>,
    response: Response,
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
impl HttpTransport for MockTransport {
    async fn fetch(&self, request: Request) -> Result<Response, NetworkError> {
        *self.request.lock().unwrap() = Some(request);
        Ok(self.response.clone())
    }
}

fn make_client(status: u16, body: &[u8]) -> (Arc<NetworkClient>, Arc<MockTransport>) {
    let transport = Arc::new(MockTransport {
        request: Mutex::new(None),
        response: Response {
            status,
            headers: vec![],
            body: body.to_vec(),
        },
    });
    (
        NetworkClient::new(Arc::clone(&transport) as Arc<dyn HttpTransport>),
        transport,
    )
}

#[test]
fn host_request_carries_auth_and_cap() {
    let (client, transport) = make_client(200, b"hello");
    let body = pollster::block_on(client.get_bytes(
        "https://example.test/data".into(),
        3,
        Some("Bearer token".into()),
    ))
    .unwrap();

    assert_eq!(body, b"hel");
    assert_eq!(
        *transport.request.lock().unwrap(),
        Some(Request {
            url: "https://example.test/data".into(),
            method: Method::Get,
            headers: vec![Header::new("Authorization", "Bearer token")],
            body: None,
            max_response_bytes: 3,
        })
    );
}

#[test]
fn get_text_classifies_http_and_utf8_failures() {
    let (client, _) = make_client(404, b"missing");
    assert!(matches!(
        pollster::block_on(client.get_text("https://example.test/missing".into(), None)),
        Err(NetworkError::NotFound)
    ));

    let (client, _) = make_client(200, &[0xff]);
    assert!(matches!(
        pollster::block_on(client.get_text("https://example.test/data".into(), None)),
        Err(NetworkError::Transport { .. })
    ));
}

#[test]
fn fetch_forwards_method_headers_and_body() {
    let (client, transport) = make_client(201, b"created");
    let request = Request::new(
        "https://example.test/data",
        Method::Post,
        vec![Header::new("Content-Type", "application/json")],
        Some(br#"{"name":"JayJay"}"#.to_vec()),
        4,
    );
    let response = pollster::block_on(client.fetch(request.clone())).unwrap();

    assert_eq!(response.status, 201);
    assert_eq!(response.body, b"crea");
    assert_eq!(*transport.request.lock().unwrap(), Some(request));
}
