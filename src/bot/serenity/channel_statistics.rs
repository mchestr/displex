use std::sync::Arc;

use serde_json::{
    Map,
    Value,
};
use serenity::{
    http::Http,
    json::JsonMap,
    model::{
        prelude::{
            ChannelType,
            GuildChannel,
        },
        Permissions,
    },
    CacheAndHttp,
};
use tokio_schedule::{
    every,
    Job,
};

use crate::{tautulli::{
    client::TautulliClient,
    models::{
        GetActivity,
        GetLibrary,
    },
}, config::UpdateChannelConfig};

#[derive(Clone, Debug)]
struct CreateChannelConfig {
    name_prefix: String,
    position: Option<u8>,

    permissions: Vec<Map<String, Value>>,

    type_: ChannelType,
    parent_channel: Option<u64>,

    server_id: u64,
}

#[derive(Clone, Debug, Default)]
struct StatCategoryChannels {
    status: Option<GuildChannel>,
    streams: Option<GuildChannel>,
    transcodes: Option<GuildChannel>,
    bandwidth: Option<GuildChannel>,
}

#[derive(Clone, Debug, Default)]
struct LibraryStatCategoryChannels {
    movies: Option<GuildChannel>,
    tv_shows: Option<GuildChannel>,
    tv_episodes: Option<GuildChannel>,
}

pub async fn setup(
    interval_seconds: u32,
    cache_and_http_client: Arc<CacheAndHttp>,
    tautulli_client: TautulliClient,
    config: UpdateChannelConfig,
) -> anyhow::Result<()> {
    tracing::info!("refreshing channel statistics every {}s", interval_seconds);
    let client = cache_and_http_client.http.clone();

    let roles = client.get_guild_roles(config.discord_server_id).await.unwrap();
    let bot_role = roles
        .iter()
        .find(|&r| r.name.eq(&config.bot_role_name))
        .ok_or_else(|| anyhow::anyhow!("unable to find bot role"))?;
    let sub_role = roles
        .iter()
        .find(|&r| r.name.eq(&config.subscriber_role_name))
        .ok_or_else(|| anyhow::anyhow!("unable to find subscriber role"))?;

    let everyone_role = roles
        .iter()
        .find(|&r| r.name.eq("@everyone"))
        .ok_or_else(|| anyhow::anyhow!("unable to find @everyone role"))?;

    let channels = client.get_channels(config.discord_server_id).await.unwrap();
    let everyone_perms = Permissions::VIEW_CHANNEL | Permissions::CONNECT;
    let sub_perms = Permissions::VIEW_CHANNEL;
    let bot_perms = Permissions::VIEW_CHANNEL
        | Permissions::MANAGE_CHANNELS
        | Permissions::SEND_MESSAGES
        | Permissions::CONNECT;

    let mut bot_permissions = JsonMap::new();
    bot_permissions.insert("id".into(), bot_role.id.0.into());
    bot_permissions.insert("type".into(), 0.into());
    bot_permissions.insert("allow".into(), bot_perms.bits().into());

    let mut sub_permissions = JsonMap::new();
    sub_permissions.insert("id".into(), sub_role.id.0.into());
    sub_permissions.insert("type".into(), 0.into());
    sub_permissions.insert("allow".into(), sub_perms.bits().into());

    let mut everyone_permissions = JsonMap::new();
    everyone_permissions.insert("id".into(), everyone_role.id.0.into());
    everyone_permissions.insert("type".into(), 0.into());
    everyone_permissions.insert("deny".into(), everyone_perms.bits().into());

    let permissions = vec![bot_permissions, sub_permissions, everyone_permissions];

    let stats_category_channels = generate_stats_categories(
        &client,
        &config,
        channels.clone(),
        permissions.clone(),
    )
    .await;

    let lib_category_channels =
        generate_library_categories(&client, &config, channels, permissions.clone()).await;

    update_library_stats(&client, &tautulli_client, &lib_category_channels).await;
    update_stats(&client, &tautulli_client, &stats_category_channels).await;
    tokio::spawn(every(interval_seconds).seconds().perform(move || {
        let http_client = cache_and_http_client.http.clone();
        let tautulli_client = tautulli_client.clone();
        let config = config.clone();
        let permissions = permissions.clone();
        async move {
            let channels = http_client.get_channels(config.discord_server_id).await.unwrap();
            let stats_category_channels = generate_stats_categories(
                &http_client,
                &config,
                channels.clone(),
                permissions.clone(),
            )
            .await;
            update_stats(&http_client, &tautulli_client, &stats_category_channels).await;

            let lib_category_channels =
                generate_library_categories(&http_client, &config, channels, permissions).await;
            update_library_stats(&http_client, &tautulli_client, &lib_category_channels).await;
        }
    }));
    Ok(())
}

async fn get_or_create_stat_category(
    client: &Arc<Http>,
    channels: Vec<GuildChannel>,
    create: CreateChannelConfig,
) -> GuildChannel {
    match channels
        .into_iter()
        .find(|c| c.name.starts_with(&create.name_prefix))
    {
        Some(channel) => {
            tracing::info!("found channel: {}", channel.name);
            channel
        }
        None => {
            tracing::info!("creating channel: {}", create.name_prefix);
            create_category(client, create).await
        }
    }
}

async fn create_category(client: &Arc<Http>, config: CreateChannelConfig) -> GuildChannel {
    let mut create_channel_map = JsonMap::new();
    create_channel_map.insert("name".into(), config.name_prefix.as_str().into());
    create_channel_map.insert("type".into(), config.type_.num().into());
    create_channel_map.insert("permission_overwrites".into(), config.permissions.into());

    if let Some(position) = config.position {
        create_channel_map.insert("position".into(), position.into());
    }

    if let Some(parent_channel) = config.parent_channel {
        create_channel_map.insert("parent_id".into(), parent_channel.into());
    }

    client
        .create_channel(config.server_id, &create_channel_map, None)
        .await
        .unwrap()
}

#[tracing::instrument(skip(client, channel), fields(channel.name = channel.name))]
async fn update_channel_name(client: &Arc<Http>, channel: &GuildChannel, new_name: &str) {
    if !channel.name.eq(&new_name) {
        let mut map = JsonMap::new();
        map.insert("name".into(), new_name.into());
        client.edit_channel(channel.id.0, &map, None).await.unwrap();
    } else {
        tracing::info!("channel name '{new_name}' is the same, skipping...");
    }
}

async fn update_stats(
    client: &Arc<Http>,
    tautulli_client: &TautulliClient,
    channels: &StatCategoryChannels,
) {
    if let Some(channel) = &channels.status {
        channel_update_stats_status(client, tautulli_client, channel).await;
    }
    let activity = tautulli_client.get_activity().await.unwrap();
    if let Some(channel) = &channels.streams {
        channel_update_stats_streams(client, &activity, channel).await;
    }
    if let Some(channel) = &channels.transcodes {
        channel_update_stats_transcodes(client, &activity, channel).await;
    }
    if let Some(channel) = &channels.bandwidth {
        channel_update_stats_bandwidth(client, &activity, channel).await;
    }
}

async fn channel_update_stats_status(
    client: &Arc<Http>,
    tautulli_client: &TautulliClient,
    channel: &GuildChannel,
) {
    let server_status = tautulli_client.server_status().await.unwrap();
    let name_split: Vec<&str> = channel.name.split(':').collect();
    let prefix = name_split[0];

    let new_name = format!(
        "{prefix}: {}",
        match server_status.connected {
            true => "Online 🟢",
            false => "Offline 🔴",
        }
    );
    update_channel_name(client, channel, &new_name).await;
}

#[tracing::instrument(skip(client, channel))]
async fn channel_update_stats_streams(
    client: &Arc<Http>,
    data: &GetActivity,
    channel: &GuildChannel,
) {
    let name_split: Vec<&str> = channel.name.split(':').collect();
    let prefix = name_split[0];

    let new_name = format!("{prefix}: {}", data.stream_count,);
    update_channel_name(client, channel, &new_name).await;
}

#[tracing::instrument(skip(client, channel))]
async fn channel_update_stats_transcodes(
    client: &Arc<Http>,
    data: &GetActivity,
    channel: &GuildChannel,
) {
    let name_split: Vec<&str> = channel.name.split(':').collect();
    let prefix = name_split[0];

    let new_name = format!("{prefix}: {}", data.stream_count_transcode,);
    update_channel_name(client, channel, &new_name).await;
}

async fn channel_update_stats_bandwidth(
    client: &Arc<Http>,
    data: &GetActivity,
    channel: &GuildChannel,
) {
    let name_split: Vec<&str> = channel.name.split(':').collect();
    let prefix = name_split[0];

    let new_name = {
        if data.total_bandwidth > 1024 {
            let n = data.total_bandwidth as f32 / 1024.0;
            format!("{prefix}: {n:.1} Mbps")
        } else {
            let n = data.total_bandwidth as f32;
            format!("{prefix}: {n:.1} Kbps")
        }
    };
    update_channel_name(client, channel, &new_name).await;
}

async fn update_library_stats(
    client: &Arc<Http>,
    tautulli_client: &TautulliClient,
    channels: &LibraryStatCategoryChannels,
) {
    let stats = tautulli_client.get_libraries().await.unwrap();
    let movies = stats.iter().find(|s| s.section_name.eq("Movies"));
    let tv = stats.iter().find(|s| s.section_name.eq("TV Shows"));

    if let Some(data) = movies {
        if let Some(channel) = &channels.movies {
            update_movies(client, data, channel).await;
        }
    } else {
        tracing::error!("failed to find library '{}'", "Movies");
    }

    if let Some(data) = tv {
        if let Some(channel) = &channels.tv_shows {
            update_tv_shows(client, data, channel).await;
        }
        if let Some(channel) = &channels.tv_episodes {
            update_tv_episodes(client, data, channel).await;
        }
    } else {
        tracing::error!("failed to find library '{}'", "TV Shows");
    }
}

async fn update_movies(client: &Arc<Http>, data: &GetLibrary, channel: &GuildChannel) {
    let name_split: Vec<&str> = channel.name.split(':').collect();
    let prefix = name_split[0];
    let new_name = format!("{}: {}", prefix, data.count);

    update_channel_name(client, channel, &new_name).await;
}

async fn update_tv_shows(client: &Arc<Http>, data: &GetLibrary, channel: &GuildChannel) {
    let name_split: Vec<&str> = channel.name.split(':').collect();
    let prefix = name_split[0];

    let new_name = format!("{}: {}", prefix, data.count);
    update_channel_name(client, channel, &new_name).await;
}

async fn update_tv_episodes(client: &Arc<Http>, data: &GetLibrary, channel: &GuildChannel) {
    let name_split: Vec<&str> = channel.name.split(':').collect();
    let prefix = name_split[0];

    let new_name = {
        if let Some(episodes) = &data.child_count {
            format!("{prefix}: {episodes}")
        } else {
            String::from(&channel.name)
        }
    };
    update_channel_name(client, channel, &new_name).await;
}

async fn generate_stats_categories(
    client: &Arc<Http>,
    update_config: &UpdateChannelConfig,
    channels: Vec<GuildChannel>,
    permissions: Vec<Map<String, Value>>,
) -> StatCategoryChannels {
    {
        let mut stat_channels = StatCategoryChannels {
            ..Default::default()
        };
        match &update_config.stats_category {
            Some(config) => {
                let category = get_or_create_stat_category(
                    client,
                    channels.clone(),
                    CreateChannelConfig {
                        name_prefix: String::from(&config.stat_category_name),
                        position: Some(5),
                        type_: ChannelType::Category,
                        permissions: permissions.clone(),
                        parent_channel: None,
                        server_id: update_config.discord_server_id,
                    },
                )
                .await;

                if let Some(name) = &config.status_name {
                    stat_channels.status = Some(
                        get_or_create_stat_category(
                            client,
                            channels.clone(),
                            CreateChannelConfig {
                                name_prefix: String::from(name),
                                position: Some(0),
                                permissions: permissions.clone(),
                                type_: ChannelType::Voice,
                                parent_channel: Some(category.id.0),
                                server_id: update_config.discord_server_id,
                            },
                        )
                        .await,
                    );
                };

                if let Some(name) = &config.stream_name {
                    stat_channels.streams = Some(
                        get_or_create_stat_category(
                            client,
                            channels.clone(),
                            CreateChannelConfig {
                                name_prefix: String::from(name),
                                position: Some(0),
                                permissions: permissions.clone(),
                                type_: ChannelType::Voice,
                                parent_channel: Some(category.id.0),
                                server_id: update_config.discord_server_id,
                            },
                        )
                        .await,
                    );
                };

                if let Some(name) = &config.transcode_name {
                    stat_channels.transcodes = Some(
                        get_or_create_stat_category(
                            client,
                            channels.clone(),
                            CreateChannelConfig {
                                name_prefix: String::from(name),
                                position: Some(0),
                                permissions: permissions.clone(),
                                type_: ChannelType::Voice,
                                parent_channel: Some(category.id.0),
                                server_id: update_config.discord_server_id,
                            },
                        )
                        .await,
                    );
                };

                if let Some(name) = &config.bandwidth_name {
                    stat_channels.bandwidth = Some(
                        get_or_create_stat_category(
                            client,
                            channels.clone(),
                            CreateChannelConfig {
                                name_prefix: String::from(name),
                                position: Some(0),
                                permissions: permissions.clone(),
                                type_: ChannelType::Voice,
                                parent_channel: Some(category.id.0),
                                server_id: update_config.discord_server_id,
                            },
                        )
                        .await,
                    );
                };

                stat_channels
            }
            None => stat_channels,
        }
    }
}

async fn generate_library_categories(
    client: &Arc<Http>,
    update_config: &UpdateChannelConfig,
    channels: Vec<GuildChannel>,
    permissions: Vec<Map<String, Value>>,
) -> LibraryStatCategoryChannels {
    {
        let mut lib_channels = LibraryStatCategoryChannels {
            ..Default::default()
        };
        match &update_config.library_category {
            Some(config) => {
                let category = get_or_create_stat_category(
                    client,
                    channels.clone(),
                    CreateChannelConfig {
                        name_prefix: String::from(&config.lib_category_name),
                        position: Some(5),
                        type_: ChannelType::Category,
                        permissions: permissions.clone(),
                        parent_channel: None,
                        server_id: update_config.discord_server_id,
                    },
                )
                .await;

                if let Some(name) = &config.movies_name {
                    lib_channels.movies = Some(
                        get_or_create_stat_category(
                            client,
                            channels.clone(),
                            CreateChannelConfig {
                                name_prefix: String::from(name),
                                position: Some(0),
                                permissions: permissions.clone(),
                                type_: ChannelType::Voice,
                                parent_channel: Some(category.id.0),
                                server_id: update_config.discord_server_id,
                            },
                        )
                        .await,
                    );
                }

                if let Some(name) = &config.tv_shows_name {
                    lib_channels.tv_shows = Some(
                        get_or_create_stat_category(
                            client,
                            channels.clone(),
                            CreateChannelConfig {
                                name_prefix: String::from(name),
                                position: Some(0),
                                permissions: permissions.clone(),
                                type_: ChannelType::Voice,
                                parent_channel: Some(category.id.0),
                                server_id: update_config.discord_server_id,
                            },
                        )
                        .await,
                    );
                }

                if let Some(name) = &config.tv_episodes_name {
                    lib_channels.tv_episodes = Some(
                        get_or_create_stat_category(
                            client,
                            channels.clone(),
                            CreateChannelConfig {
                                name_prefix: String::from(name),
                                position: Some(0),
                                permissions: permissions.clone(),
                                type_: ChannelType::Voice,
                                parent_channel: Some(category.id.0),
                                server_id: update_config.discord_server_id,
                            },
                        )
                        .await,
                    );
                }

                lib_channels
            }
            None => lib_channels,
        }
    }
}
