use anyhow::{Context, Result};
use axum::http::header::CONTENT_TYPE;
use axum::response::Html;
use axum::routing::get;
use axum::Router;
use clap::Parser;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpListener;
use tracing::{error, info};
use tracing_subscriber::filter::LevelFilter;
use wtransport::endpoint::IncomingSession;
use wtransport::tls::{Sha256Digest, Sha256DigestFmt};
use wtransport::{Endpoint, Identity, ServerConfig};

#[derive(Parser, Debug)]
#[command(name = "webtransport-server")]
#[command(about = "Browser-testable WebTransport-over-HTTP/3 server")]
struct Args {
    /// WebTransport listen address
    #[arg(short, long, default_value = "127.0.0.1:4433")]
    listen: SocketAddr,

    /// HTTP address for the browser test page
    #[arg(long, default_value = "127.0.0.1:8080")]
    http: SocketAddr,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    init_logging(args.verbose);

    let identity = Identity::self_signed(["localhost", "127.0.0.1", "::1"])?;
    let cert_digest = identity.certificate_chain().as_slice()[0].hash();

    let webtransport = WebTransportServer::new(args.listen, identity)?;
    let http = HttpServer::new(args.http, cert_digest, webtransport.local_addr()).await?;

    info!("Open http://{} in Chrome or Edge", http.local_addr());
    info!(
        "WebTransport endpoint: https://{}",
        webtransport.local_addr()
    );

    tokio::select! {
        result = webtransport.serve() => {
            error!("WebTransport server stopped: {:?}", result);
        }
        result = http.serve() => {
            error!("HTTP test page stopped: {:?}", result);
        }
    }

    Ok(())
}

struct WebTransportServer {
    endpoint: Endpoint<wtransport::endpoint::endpoint_side::Server>,
}

impl WebTransportServer {
    fn new(listen: SocketAddr, identity: Identity) -> Result<Self> {
        let config = ServerConfig::builder()
            .with_bind_address(listen)
            .with_identity(identity)
            .keep_alive_interval(Some(Duration::from_secs(3)))
            .build();

        let endpoint = Endpoint::server(config)?;
        Ok(Self { endpoint })
    }

    fn local_addr(&self) -> SocketAddr {
        self.endpoint
            .local_addr()
            .expect("endpoint has local address")
    }

    async fn serve(self) -> Result<()> {
        info!("WebTransport server listening on {}", self.local_addr());

        for id in 0.. {
            let incoming_session = self.endpoint.accept().await;
            tokio::spawn(async move {
                if let Err(err) = handle_session(incoming_session).await {
                    error!(connection_id = id, "session failed: {err:?}");
                }
            });
        }

        Ok(())
    }
}

async fn handle_session(incoming_session: IncomingSession) -> Result<()> {
    let session_request = incoming_session.await?;
    info!(
        "New session request: authority='{}' path='{}'",
        session_request.authority(),
        session_request.path()
    );

    let connection = session_request.accept().await?;
    info!("Session accepted");

    let mut buffer = vec![0; 64 * 1024].into_boxed_slice();

    loop {
        tokio::select! {
            stream = connection.accept_bi() => {
                let mut stream = stream?;
                let Some(bytes_read) = stream.1.read(&mut buffer).await? else {
                    continue;
                };
                let message = String::from_utf8_lossy(&buffer[..bytes_read]);
                info!("Received bidirectional stream: {message}");
                let response = format!("Echo: {message}");
                stream.0.write_all(response.as_bytes()).await?;
                stream.0.finish().await?;
            }
            stream = connection.accept_uni() => {
                let mut stream = stream?;
                let Some(bytes_read) = stream.read(&mut buffer).await? else {
                    continue;
                };
                let message = String::from_utf8_lossy(&buffer[..bytes_read]);
                info!("Received unidirectional stream: {message}");

                let response = format!("Server received uni stream: {message}");
                let mut response_stream = connection.open_uni().await?.await?;
                response_stream.write_all(response.as_bytes()).await?;
                response_stream.finish().await?;
            }
            datagram = connection.receive_datagram() => {
                let datagram = datagram?;
                let message = String::from_utf8_lossy(&datagram);
                info!("Received datagram: {message}");

                let response = if message.trim() == "ping" {
                    "pong".to_string()
                } else {
                    format!("Echo: {message}")
                };
                connection.send_datagram(response.as_bytes())?;
            }
        }
    }
}

struct HttpServer {
    local_addr: SocketAddr,
    router: Router,
}

impl HttpServer {
    async fn new(
        listen: SocketAddr,
        cert_digest: Sha256Digest,
        webtransport_addr: SocketAddr,
    ) -> Result<Self> {
        let digest = cert_digest.fmt(Sha256DigestFmt::BytesArray);
        let wt_url = format!("https://localhost:{}/", webtransport_addr.port());

        let index_html = include_str!("static/index.html").replace("${WEBTRANSPORT_URL}", &wt_url);
        let client_js = include_str!("static/client.js")
            .replace("${CERT_DIGEST}", &digest)
            .replace("${WEBTRANSPORT_URL}", &wt_url);
        let style_css = include_str!("static/style.css").to_string();

        let router = Router::new()
            .route(
                "/",
                get({
                    let index_html = index_html.clone();
                    move || async move { Html(index_html) }
                }),
            )
            .route(
                "/client.js",
                get({
                    let client_js = client_js.clone();
                    move || async move { ([(CONTENT_TYPE, "application/javascript")], client_js) }
                }),
            )
            .route(
                "/style.css",
                get({
                    let style_css = style_css.clone();
                    move || async move { ([(CONTENT_TYPE, "text/css")], style_css) }
                }),
            );

        Ok(Self {
            local_addr: listen,
            router,
        })
    }

    fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    async fn serve(self) -> Result<()> {
        let listener = TcpListener::bind(self.local_addr)
            .await
            .with_context(|| format!("failed to bind HTTP listener on {}", self.local_addr))?;
        let local_addr = listener.local_addr()?;
        info!("HTTP test page listening on http://{local_addr}");
        axum::serve(listener, self.router)
            .await
            .context("HTTP server error")?;
        Ok(())
    }
}

fn init_logging(verbose: bool) {
    let default_level = if verbose {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    };

    tracing_subscriber::fmt()
        .with_target(true)
        .with_level(true)
        .with_max_level(default_level)
        .init();
}
