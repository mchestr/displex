// @generated automatically by Diesel CLI.

diesel::table! {
    discord_tokens (access_token) {
        access_token -> Varchar,
        refresh_token -> Varchar,
        scopes -> Varchar,
        expires_at -> Timestamptz,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        discord_user_id -> Varchar,
    }
}

diesel::table! {
    discord_users (id) {
        id -> Varchar,
        username -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    plex_tokens (access_token) {
        access_token -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        plex_user_id -> Varchar,
    }
}

diesel::table! {
    plex_users (id) {
        id -> Varchar,
        username -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        discord_user_id -> Varchar,
    }
}

diesel::joinable!(plex_tokens -> plex_users (plex_user_id));

diesel::allow_tables_to_appear_in_same_query!(
    discord_tokens,
    discord_users,
    plex_tokens,
    plex_users,
);
