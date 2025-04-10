use async_graphql::http::{
    playground_source,
    GraphQLPlaygroundConfig,
};
use async_graphql_axum::{
    GraphQLRequest,
    GraphQLResponse,
};
use axum::{
    extract::State,
    response::{
        Html,
        IntoResponse,
    },
    routing::get,
    Router,
};

use tower_cookies::Cookies;

use crate::{
    errors::DisplexError,
    server::{
        axum::DisplexState,
        cookies::get_cookie_data,
    },
};

async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(
        GraphQLPlaygroundConfig::new("/").subscription_endpoint("/ws"),
    ))
}

async fn graphql_handler(
    State(state): State<DisplexState>,
    cookies: Cookies,
    req: GraphQLRequest,
) -> Result<GraphQLResponse, DisplexError> {
    let cookie = get_cookie_data(&state.config.session.secret_key, &cookies).unwrap_or_default();
    let mut req = req.into_inner();
    req = req.data(cookie);
    Ok(state.schema.execute(req).await.into())
}

pub fn routes() -> Router<DisplexState> {
    Router::new().route("/", get(graphql_playground).post(graphql_handler))
}
