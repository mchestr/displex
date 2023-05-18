// @generated automatically by Diesel CLI.

diesel::table! {
    discord_tokens (id) {
        id -> Varchar,
        username -> Varchar,
        access_token -> Varchar,
        token_type -> Varchar,
        refresh_token -> Varchar,
        scopes -> Varchar,
    }
}

diesel::table! {
    plex_tokens (id) {
        id -> Varchar,
        username -> Varchar,
        access_token -> Varchar,
    }
}

diesel::allow_tables_to_appear_in_same_query!(discord_tokens, plex_tokens,);
