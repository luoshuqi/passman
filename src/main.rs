use std::future::Future;
use std::pin::Pin;
use std::process::ExitCode;
use std::sync::Arc;

use conerror::conerror;
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::header::CONTENT_TYPE;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use log::{error, info, LevelFilter};
use rustic_jsonrpc::{BoxError, Registry};
use serde_json::{to_string, Value};
use structopt::StructOpt;
use tokio::net::TcpListener;

use crate::db::setup_db;
use crate::encryption::{Aes256GcmEncryptor, EncryptionManager};
use crate::password::PasswordManager;
use crate::service::methods;
use crate::user::UserManager;

#[macro_use]
mod query;
mod db;
mod encryption;
mod error;
mod password;
mod service;
mod user;
mod util;

#[cfg(not(debug_assertions))]
#[derive(rust_embed::RustEmbed)]
#[folder = "html/dist/"]
struct Asset;

#[derive(StructOpt, Clone)]
struct Opt {
    #[structopt(long)]
    bind: String,

    #[structopt(long)]
    data_dir: String,

    #[structopt(long)]
    allow_create_user: bool,
}

#[tokio::main]
async fn main() -> ExitCode {
    init_logger();
    let opt = Opt::from_args();
    if let Err(err) = run(&opt).await {
        error!("{}", err);
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

#[conerror]
async fn run(opt: &Opt) -> conerror::Result<()> {
    let db = setup_db(&opt.data_dir).await?;
    let encryption = EncryptionManager::new(vec![Box::new(Aes256GcmEncryptor)]);

    let mut registry = Registry::new();
    registry.provide(db.clone());
    registry.provide(opt.clone());
    registry.provide(UserManager::new(db.clone(), encryption.clone()));
    registry.provide(PasswordManager::new(db, encryption));
    registry.register(methods());
    registry.post_call(log_error);

    let registry = Arc::new(registry);
    serve_http(&opt.bind, move |req| {
        let registry = registry.clone();
        async move {
            match (req.method(), req.uri().path()) {
                (&Method::POST, "/rpc") => handle_rpc(&registry, req).await,
                #[cfg(not(debug_assertions))]
                (&Method::GET, path) => handle_static(path).await,
                _ => Ok(not_found()),
            }
        }
    })
    .await?;
    Ok(())
}

#[conerror]
pub async fn serve_http<F, H>(addr: &str, handler: H) -> conerror::Result<()>
where
    F: Future<Output = hyper::Result<Response<Full<Bytes>>>> + Send,
    H: Fn(Request<Incoming>) -> F + Send + Clone + 'static,
{
    let listener = TcpListener::bind(addr).await?;
    let addr = listener.local_addr()?;
    info!("server started at {}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let handler = handler.clone();
        tokio::spawn(async move {
            if let Err(err) = http1_builder()
                .serve_connection(io, service_fn(handler))
                .await
            {
                error!("error serve connection: {}", err);
            }
        });
    }
}

fn http1_builder() -> http1::Builder {
    let mut builder = http1::Builder::new();
    builder.max_buf_size(1024 * 1024);
    builder
}

async fn handle_rpc(
    registry: &Registry,
    req: Request<Incoming>,
) -> hyper::Result<Response<Full<Bytes>>> {
    match registry
        .handle(&req.into_body().collect().await?.to_bytes())
        .await
    {
        Some(v) => {
            let response = to_string(&v).unwrap();
            let mut response = Response::new(Full::new(Bytes::from(response)));
            response
                .headers_mut()
                .insert(CONTENT_TYPE, "application/json".parse().unwrap());
            Ok(response)
        }
        None => Ok(Response::default()),
    }
}

#[cfg(not(debug_assertions))]
async fn handle_static(path: &str) -> hyper::Result<Response<Full<Bytes>>> {
    let mut path = path.trim_start_matches('/');
    if path == "" {
        path = "index.html"
    }
    match Asset::get(path) {
        Some(file) => {
            let mut response = match file.data {
                std::borrow::Cow::Borrowed(data) => Response::new(Full::new(Bytes::from(data))),
                std::borrow::Cow::Owned(data) => Response::new(Full::new(Bytes::from(data))),
            };
            response
                .headers_mut()
                .insert(CONTENT_TYPE, file.metadata.mimetype().parse().unwrap());
            Ok(response)
        }
        None => Ok(not_found()),
    }
}

fn log_error<'a>(
    req: &'a rustic_jsonrpc::Request<'a>,
    result: &'a Result<Value, BoxError>,
) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
    if let Err(e) = result {
        if rustic_jsonrpc::Error::cast(&**e).is_none() {
            error!("{}: {}", req.method, e);
        }
    }
    Box::pin(async {})
}

fn not_found() -> Response<Full<Bytes>> {
    let mut r = Response::default();
    *r.status_mut() = StatusCode::NOT_FOUND;
    r
}

#[cfg(not(debug_assertions))]
fn init_logger() {
    env_logger::builder().filter_level(LevelFilter::Info).init();
}

#[cfg(debug_assertions)]
fn init_logger() {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();
}
