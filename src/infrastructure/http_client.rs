use crate::domain::entities::{Method as DomainMethod, Request, Response};
use crate::domain::value_objects::JsonBody;

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::header::{CONTENT_TYPE, HOST, HeaderValue};
use hyper::{Method, Request as HyperRequest, Response as HyperResponse, Uri};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;

/// HTTP client using Hyper without hyper-util
pub struct HyperHttpClient;

impl HyperHttpClient {
    pub fn new() -> Self {
        Self
    }

    async fn create_connection(&self, uri: &Uri) -> Result<Box<dyn Connection>> {
        let host = uri.host().ok_or_else(|| anyhow!("No host in URI"))?;
        let port = uri
            .port_u16()
            .unwrap_or(if uri.scheme_str() == Some("https") {
                443
            } else {
                80
            });
        let addr = format!("{}:{}", host, port);

        if uri.scheme_str() == Some("https") {
            let stream = TcpStream::connect(&addr)
                .await
                .map_err(|e| anyhow!("Failed to connect to {}: {}", addr, e))?;

            let connector = tokio_native_tls::native_tls::TlsConnector::new()
                .map_err(|e| anyhow!("Failed to create TLS connector: {}", e))?;
            let connector = tokio_native_tls::TlsConnector::from(connector);

            let tls_stream = connector
                .connect(host, stream)
                .await
                .map_err(|e| anyhow!("TLS handshake failed: {}", e))?;

            let io = TokioIoAdapter::new(tls_stream);
            let (sender, conn) = hyper::client::conn::http1::handshake(io)
                .await
                .map_err(|e| anyhow!("HTTP handshake failed: {}", e))?;

            tokio::task::spawn(async move {
                if let Err(err) = conn.await {
                    eprintln!("Connection failed: {:?}", err);
                }
            });

            Ok(Box::new(HttpsConnection { sender }))
        } else {
            let stream = TcpStream::connect(&addr)
                .await
                .map_err(|e| anyhow!("Failed to connect to {}: {}", addr, e))?;

            let io = TokioIoAdapter::new(stream);
            let (sender, conn) = hyper::client::conn::http1::handshake(io)
                .await
                .map_err(|e| anyhow!("HTTP handshake failed: {}", e))?;

            tokio::task::spawn(async move {
                if let Err(err) = conn.await {
                    eprintln!("Connection failed: {:?}", err);
                }
            });

            Ok(Box::new(HttpConnection { sender }))
        }
    }
}

// Simple adapter that implements hyper::rt traits for tokio IO types
struct TokioIoAdapter<T> {
    inner: T,
}

impl<T> TokioIoAdapter<T> {
    fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T> hyper::rt::Read for TokioIoAdapter<T>
where
    T: AsyncRead + Unpin,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut buf: hyper::rt::ReadBufCursor<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        let n = unsafe {
            let mut tbuf = ReadBuf::uninit(buf.as_mut());
            match AsyncRead::poll_read(Pin::new(&mut self.inner), cx, &mut tbuf) {
                Poll::Ready(Ok(())) => tbuf.filled().len(),
                other => return other,
            }
        };

        unsafe {
            buf.advance(n);
        }
        Poll::Ready(Ok(()))
    }
}

impl<T> hyper::rt::Write for TokioIoAdapter<T>
where
    T: AsyncWrite + Unpin,
{
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        AsyncWrite::poll_write(Pin::new(&mut self.inner), cx, buf)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        AsyncWrite::poll_flush(Pin::new(&mut self.inner), cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        AsyncWrite::poll_shutdown(Pin::new(&mut self.inner), cx)
    }
}

// Trait to abstract over HTTP and HTTPS connections
#[async_trait]
trait Connection: Send {
    async fn send_request(
        &mut self,
        req: HyperRequest<Full<Bytes>>,
    ) -> Result<HyperResponse<hyper::body::Incoming>>;
}

struct HttpConnection {
    sender: hyper::client::conn::http1::SendRequest<Full<Bytes>>,
}

#[async_trait]
impl Connection for HttpConnection {
    async fn send_request(
        &mut self,
        req: HyperRequest<Full<Bytes>>,
    ) -> Result<HyperResponse<hyper::body::Incoming>> {
        self.sender
            .send_request(req)
            .await
            .map_err(|e| anyhow!("Failed to send HTTP request: {}", e))
    }
}

struct HttpsConnection {
    sender: hyper::client::conn::http1::SendRequest<Full<Bytes>>,
}

#[async_trait]
impl Connection for HttpsConnection {
    async fn send_request(
        &mut self,
        req: HyperRequest<Full<Bytes>>,
    ) -> Result<HyperResponse<hyper::body::Incoming>> {
        self.sender
            .send_request(req)
            .await
            .map_err(|e| anyhow!("Failed to send HTTPS request: {}", e))
    }
}

#[async_trait]
impl crate::application::services::HttpClient for HyperHttpClient {
    async fn send(&self, request: Request) -> Result<Response> {
        let uri = request.url.0.clone();

        let mut conn = self.create_connection(&uri).await?;
        let hyper_request = RequestAdapter::to_hyper_request(request, &uri)?;
        let hyper_response = conn.send_request(hyper_request).await?;

        ResponseAdapter::to_domain_response(hyper_response).await
    }
}

// Adapter to convert domain Request to Hyper Request
struct RequestAdapter;

impl RequestAdapter {
    fn to_hyper_request(domain_request: Request, uri: &Uri) -> Result<HyperRequest<Full<Bytes>>> {
        let method = MethodAdapter::to_hyper_method(domain_request.method);
        let body = BodyAdapter::to_hyper_body(&domain_request.body);

        let mut builder = HyperRequest::builder().method(method).uri(uri);

        // Add HOST header as required by hyper
        if let Some(authority) = uri.authority() {
            builder = builder.header(HOST, authority.as_str());
        }

        builder = HeaderAdapter::add_json_content_type(builder, &domain_request.body);
        builder = HeaderAdapter::add_headers(builder, &domain_request.headers);

        builder
            .body(body)
            .map_err(|e| anyhow!("Failed to build HTTP request: {}", e))
    }
}

// Adapter to convert Hyper Response to domain Response
struct ResponseAdapter;

impl ResponseAdapter {
    async fn to_domain_response(
        hyper_response: HyperResponse<hyper::body::Incoming>,
    ) -> Result<Response> {
        let status = hyper_response.status();
        let body_bytes = hyper_response
            .into_body()
            .collect()
            .await
            .map_err(|e| anyhow!("Failed to read response body: {}", e))?
            .to_bytes();

        let body = String::from_utf8(body_bytes.to_vec())
            .map_err(|e| anyhow!("Invalid UTF-8 in response body: {}", e))?;

        Ok(Response { status, body })
    }
}

// Converts domain Method enum to hyper::Method
struct MethodAdapter;

impl MethodAdapter {
    fn to_hyper_method(domain_method: DomainMethod) -> Method {
        match domain_method {
            DomainMethod::Get => Method::GET,
            DomainMethod::Post => Method::POST,
            DomainMethod::Put => Method::PUT,
            DomainMethod::Delete => Method::DELETE,
            DomainMethod::Patch => Method::PATCH,
            DomainMethod::Head => Method::HEAD,
            DomainMethod::Options => Method::OPTIONS,
        }
    }
}

// Converts Option<JsonBody> to hyper body
struct BodyAdapter;

impl BodyAdapter {
    fn to_hyper_body(domain_body: &Option<JsonBody>) -> Full<Bytes> {
        match domain_body {
            Some(json_body) => Full::new(Bytes::from(json_body.0.clone())),
            None => Full::new(Bytes::new()),
        }
    }
}

// Handles header insertion
struct HeaderAdapter;

impl HeaderAdapter {
    fn add_json_content_type(
        builder: hyper::http::request::Builder,
        body: &Option<JsonBody>,
    ) -> hyper::http::request::Builder {
        if body.is_some() {
            builder.header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
        } else {
            builder
        }
    }

    fn add_headers(
        mut builder: hyper::http::request::Builder,
        headers: &[(String, String)],
    ) -> hyper::http::request::Builder {
        for (key, value) in headers {
            builder = builder.header(key.as_str(), value.as_str());
        }
        builder
    }
}
