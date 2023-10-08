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
    http::HeaderMap,
    response::{
        Html,
        IntoResponse,
    },
    routing::get,
    Extension,
    Router,
};

use prometheus_client::encoding::text::encode;
use reqwest::header::{
    self,
    AUTHORIZATION,
};
use tower_cookies::Cookies;

use crate::{
    config::AppConfig,
    graphql::GraphqlSchema,
    services::discord_token::resolver::COOKIE_NAME,
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

async fn metrics(State(state): State<DisplexState>) -> impl IntoResponse {
    let mut body = String::new();
    encode(&mut body, &state.registry).unwrap();
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "application/openmetrics-text; version=1.0.0; charset=utf-8"
            .parse()
            .unwrap(),
    );
    (headers, body)
}

pub fn configure(config: &AppConfig) -> Router<DisplexState> {
    let router = Router::new()
        .merge(discord::routes())
        .merge(plex::routes())
        .route("/metrics", get(metrics));
    if config.api.enabled {
        tracing::info!("GraphQL API is enabled");
        router.route("/graphql", get(graphql_playground).post(graphql_handler))
    } else {
        router
    }
}
