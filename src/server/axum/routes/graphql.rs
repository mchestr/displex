use async_graphql::{
    http::{
        playground_source,
        GraphQLPlaygroundConfig,
        ALL_WEBSOCKET_PROTOCOLS,
    },
    Data,
};
use async_graphql_axum::{
    GraphQLProtocol,
    GraphQLRequest,
    GraphQLResponse,
    GraphQLWebSocket,
};
use axum::{
    extract::{
        State,
        WebSocketUpgrade,
    },
    response::{
        Html,
        IntoResponse,
        Response,
    },
    routing::get,
    Router,
};
use http::HeaderMap;
use serde::Deserialize;

use crate::server::axum::DisplexState;

pub struct Token(pub String);

async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(
        GraphQLPlaygroundConfig::new("/").subscription_endpoint("/ws"),
    ))
}

fn get_token_from_headers(headers: &HeaderMap) -> Option<Token> {
    headers
        .get("Token")
        .and_then(|value| value.to_str().map(|s| Token(s.to_string())).ok())
}

async fn graphql_handler(
    State(state): State<DisplexState>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut req = req.into_inner();
    if let Some(token) = get_token_from_headers(&headers) {
        req = req.data(token);
    }
    state.schema.execute(req).await.into()
}

async fn on_connection_init(value: serde_json::Value) -> async_graphql::Result<Data> {
    #[derive(Deserialize)]
    struct Payload {
        token: String,
    }

    // Coerce the connection params into our `Payload` struct so we can
    // validate the token exists in the headers.
    if let Ok(payload) = serde_json::from_value::<Payload>(value) {
        let mut data = Data::default();
        data.insert(Token(payload.token));
        Ok(data)
    } else {
        Err("Token is required".into())
    }
}

async fn graphql_ws_handler(
    State(state): State<DisplexState>,
    protocol: GraphQLProtocol,
    websocket: WebSocketUpgrade,
) -> Response {
    websocket
        .protocols(ALL_WEBSOCKET_PROTOCOLS)
        .on_upgrade(move |stream| {
            GraphQLWebSocket::new(stream, state.schema.clone(), protocol)
                .on_connection_init(on_connection_init)
                .serve()
        })
}

pub fn routes() -> Router<DisplexState> {
    Router::new()
        .route("/", get(graphql_playground).post(graphql_handler))
        .route("/ws", get(graphql_ws_handler))
}
