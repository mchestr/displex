use async_graphql::{
    dataloader::DataLoader,
    scalar,
    Context,
    EmptySubscription,
    MergedObject,
    Object,
    Schema,
    SimpleObject,
};
use sea_orm::DatabaseConnection;
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    config::AppConfig,
    server::cookies::{
        CookieData,
        Role,
    },
    services::{
        discord_token::resolver::{
            DiscordTokensMutation,
            DiscordTokensQuery,
        },
        discord_user::resolver::{
            DiscordUsersMutation,
            DiscordUsersQuery,
        },
        plex_token::resolver::{
            PlexTokensMutation,
            PlexTokensQuery,
        },
        plex_user::resolver::{
            PlexUsersMutation,
            PlexUsersQuery,
        },
        tautulli::resolver::TautulliQuery,
        AppServices,
    },
    AUTHOR,
    REPOSITORY_LINK,
    VERSION,
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Identifier(i32);

impl From<Identifier> for i32 {
    fn from(value: Identifier) -> Self {
        value.0
    }
}

impl From<i32> for Identifier {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

scalar!(Identifier);

#[derive(SimpleObject)]
pub struct CoreDetails {
    version: String,
    author_name: String,
    repository_link: String,
}

#[derive(SimpleObject, Default)]
pub struct UserDetails {
    discord_user_id: Option<String>,
    plex_user_id: Option<i64>,
    role: Role,
}

#[derive(Debug, SimpleObject)]
pub struct IdObject {
    pub id: Identifier,
}

#[derive(Default)]
struct CoreQuery;

#[Object]
impl CoreQuery {
    /// Get some primary information about the service
    async fn core_details(&self, _gql_ctx: &Context<'_>) -> CoreDetails {
        CoreDetails {
            version: VERSION.to_owned(),
            author_name: AUTHOR.to_owned(),
            repository_link: REPOSITORY_LINK.to_owned(),
        }
    }

    async fn whoami(&self, gql_ctx: &Context<'_>) -> UserDetails {
        match gql_ctx.data::<CookieData>() {
            Ok(cookie) => UserDetails {
                discord_user_id: cookie.discord_user.clone(),
                plex_user_id: cookie.plex_user,
                role: cookie.role,
            },
            Err(_) => UserDetails {
                ..Default::default()
            },
        }
    }
}

#[derive(MergedObject, Default)]
pub struct QueryRoot(
    CoreQuery,
    DiscordTokensQuery,
    DiscordUsersQuery,
    PlexTokensQuery,
    PlexUsersQuery,
    TautulliQuery,
);

#[derive(MergedObject, Default)]
pub struct MutationRoot(
    DiscordTokensMutation,
    DiscordUsersMutation,
    PlexTokensMutation,
    PlexUsersMutation,
);

pub type GraphqlSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub struct OrmDataloader {
    pub db: DatabaseConnection,
}

pub async fn get_schema(
    app_services: &AppServices,
    db: DatabaseConnection,
    config: &AppConfig,
) -> GraphqlSchema {
    let orm_dataloader: DataLoader<OrmDataloader> =
        DataLoader::new(OrmDataloader { db: db.clone() }, tokio::spawn);

    Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        EmptySubscription,
    )
    .data(config.to_owned())
    .data(db)
    .data(orm_dataloader)
    .data(app_services.discord_users_service.clone())
    .data(app_services.discord_tokens_service.clone())
    .data(app_services.plex_users_service.clone())
    .data(app_services.plex_tokens_service.clone())
    .data(app_services.tautulli_service.clone())
    .finish()
}
