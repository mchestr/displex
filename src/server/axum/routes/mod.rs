use async_graphql::http::{
    playground_source,
    GraphQLPlaygroundConfig,
};
use async_graphql_axum::{
    GraphQLRequest,
    GraphQLResponse,
};
use axum::{
    http::HeaderMap,
    response::{
        Html,
        IntoResponse,
    },
    routing::get,
    Extension,
    Router,
};

use reqwest::{
    header::{
        AUTHORIZATION,
    },
};
use tower_cookies::{
    Cookies,
};


use crate::{
    discord_token::resolver::COOKIE_NAME,
    graphql::GraphqlSchema,
};

use super::DisplexState;

mod discord;
mod plex;

#[derive(Debug)]
pub struct GqlCtx {
    auth_token: Option<String>,
}

async fn graphql_handler(
    schema: Extension<GraphqlSchema>,
    cookies: Cookies,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut req = req.0;
    let mut ctx = GqlCtx { auth_token: None };
    if let Some(c) = cookies.get(COOKIE_NAME) {
        ctx.auth_token = Some(c.value().to_owned());
    } else if let Some(h) = headers.get(AUTHORIZATION) {
        ctx.auth_token = h.to_str().map(|e| e.replace("Bearer ", "")).ok();
    }
    req = req.data(ctx);
    schema.execute(req).await.into()
}

async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}

pub fn configure() -> Router<DisplexState> {
    Router::new()
        .route("/graphql", get(graphql_playground).post(graphql_handler))
        .merge(discord::routes())
        .merge(plex::routes())
}
