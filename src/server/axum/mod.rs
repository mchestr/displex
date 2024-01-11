use axum::{
    extract::Request,
    Extension,
    Router,
};

use hyper::body::Incoming;
use hyper_util::rt::TokioIo;
use tokio::{
    net::TcpListener,
    sync::{
        broadcast::Receiver,
        watch,
    },
};
use tower::Service;
use tower_cookies::CookieManagerLayer;
use tower_http::{
    catch_panic::CatchPanicLayer,
    cors::CorsLayer,
    trace::TraceLayer,
};

use crate::{
    config::AppConfig,
    graphql::GraphqlSchema,
    services::AppServices,
};

mod errors;
mod routes;

pub const DISCORD_CODE: &str = "code";
pub const DISCORD_STATE: &str = "state";

#[derive(Clone)]
pub struct DisplexState {
    pub config: AppConfig,
    pub services: AppServices,
}

pub async fn run(
    mut kill: Receiver<()>,
    config: AppConfig,
    services: &AppServices,
    schema: &GraphqlSchema,
) {
    let cors = CorsLayer::new()
        .allow_methods([http::Method::GET, http::Method::POST])
        .allow_headers([http::header::ACCEPT, http::header::CONTENT_TYPE])
        .allow_origin(
            config
                .web
                .cors_origins
                .iter()
                .map(|f| f.parse().unwrap())
                .collect::<Vec<_>>(),
        )
        .allow_credentials(true);

    let addr = format!("{}:{}", &config.http.host, &config.http.port);
    let app = Router::new()
        .merge(routes::configure(&config))
        .with_state(DisplexState {
            config,
            services: services.clone(),
        })
        .layer(CookieManagerLayer::new())
        .layer(Extension(schema.clone()))
        .layer(TraceLayer::new_for_http())
        .layer(CatchPanicLayer::new())
        .layer(CookieManagerLayer::new())
        .layer(cors);

    tracing::info!("starting server on {}", &addr);
    let listener = TcpListener::bind(&addr).await.unwrap();

    // Create a watch channel to track tasks that are handling connections and wait for them to
    // complete.
    let (close_tx, close_rx) = watch::channel(());

    // Continuously accept new connections.
    loop {
        let (socket, remote_addr) = tokio::select! {
            // Either accept a new connection...
            result = listener.accept() => {
                result.unwrap()
            }
            // ...or wait to receive a shutdown signal and stop the accept loop.
            _ = kill.recv() => {
                tracing::debug!("signal received, not accepting new connections");
                break;
            }
        };

        tracing::debug!("connection {remote_addr} accepted");

        // We don't need to call `poll_ready` because `Router` is always ready.
        let tower_service = app.clone();

        // Clone the watch receiver and move it into the task.
        let close_rx = close_rx.clone();
        let mut kill2 = kill.resubscribe();

        // Spawn a task to handle the connection. That way we can serve multiple connections
        // concurrently.
        tokio::spawn(async move {
            // Hyper has its own `AsyncRead` and `AsyncWrite` traits and doesn't use tokio.
            // `TokioIo` converts between them.
            let socket = TokioIo::new(socket);

            // Hyper also has its own `Service` trait and doesn't use tower. We can use
            // `hyper::service::service_fn` to create a hyper `Service` that calls our app through
            // `tower::Service::call`.
            let hyper_service = hyper::service::service_fn(move |request: Request<Incoming>| {
                // We have to clone `tower_service` because hyper's `Service` uses `&self` whereas
                // tower's `Service` requires `&mut self`.
                //
                // We don't need to call `poll_ready` since `Router` is always ready.
                tower_service.clone().call(request)
            });

            // `hyper_util::server::conn::auto::Builder` supports both http1 and http2 but doesn't
            // support graceful so we have to use hyper directly and unfortunately pick between
            // http1 and http2.
            let conn = hyper::server::conn::http1::Builder::new()
                .serve_connection(socket, hyper_service)
                // `with_upgrades` is required for websockets.
                .with_upgrades();

            // `graceful_shutdown` requires a pinned connection.
            let mut conn = std::pin::pin!(conn);

            loop {
                tokio::select! {
                    // Poll the connection. This completes when the client has closed the
                    // connection, graceful shutdown has completed, or we encounter a TCP error.
                    result = conn.as_mut() => {
                        if let Err(err) = result {
                            tracing::debug!("failed to serve connection: {err:#}");
                        }
                        break;
                    }
                    // Start graceful shutdown when we receive a shutdown signal.
                    //
                    // We use a loop to continue polling the connection to allow requests to finish
                    // after starting graceful shutdown. Our `Router` has `TimeoutLayer` so
                    // requests will finish after at most 10 seconds.
                    _ = kill2.recv() => {
                        tracing::debug!("signal received, starting graceful shutdown");
                        conn.as_mut().graceful_shutdown();
                    }
                }
            }

            tracing::debug!("connection {remote_addr} closed");

            // Drop the watch receiver to signal to `main` that this task is done.
            drop(close_rx);
        });
    }

    // We only care about the watch receivers that were moved into the tasks so close the residual
    // receiver.
    drop(close_rx);

    // Close the listener to stop accepting new connections.
    drop(listener);

    // Wait for all tasks to complete.
    tracing::debug!("waiting for {} tasks to finish", close_tx.receiver_count());
    close_tx.closed().await;
}
