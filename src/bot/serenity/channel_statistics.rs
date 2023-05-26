use std::{
    sync::Arc,
    time::Duration,
};

use anyhow::Result;
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
use tokio::{
    select,
    time,
};

use crate::{
    config::UpdateChannelConfig,
    tautulli::{
        client::TautulliClient,
        models::{
            GetActivity,
            GetLibrary,
        },
    },
};

#[derive(Clone, Debug)]
struct CreateChannelConfig {
    name_prefix: String,
    position: Option<u8>,

    permissions: Vec<Map<String, Value>>,

    type_: ChannelType,
    parent_channel: Option<u64>,

    server_id: u64,
}

#[derive(Clone, Debug)]
struct ChannelData {
    prefix: String,
    channel: GuildChannel,
}

#[derive(Clone, Debug, Default)]
struct StatCategoryChannels {
    status: Option<ChannelData>,
    streams: Option<ChannelData>,
    transcodes: Option<ChannelData>,
    total_bandwidth: Option<ChannelData>,
    local_bandwidth: Option<ChannelData>,
    remote_bandwidth: Option<ChannelData>,
}

#[derive(Clone, Debug, Default)]
struct LibraryStatCategoryChannels {
    movies: Option<ChannelData>,
    tv_shows: Option<ChannelData>,
    tv_episodes: Option<ChannelData>,
}

#[derive(Clone)]
pub struct ChannelStatisticArgs {
    pub interval_seconds: Duration,
    pub cache_and_http_client: Arc<CacheAndHttp>,
    pub tautulli_client: TautulliClient,
    pub config: UpdateChannelConfig,
    pub server_id: u64,
}

pub async fn setup(
    kill: tokio::sync::broadcast::Receiver<()>,
    args: ChannelStatisticArgs,
) -> Result<()> {
    tracing::info!(
        "refreshing channel statistics every {}s",
        args.interval_seconds.as_secs()
    );
    let client = args.cache_and_http_client.http.clone();

    let roles = client
        .get_guild_roles(args.server_id)
        .await
        .expect("failed to list discord roles");
    let bot_role = roles
        .iter()
        .find(|&r| r.name.eq(&args.config.bot_role_name))
        .ok_or_else(|| anyhow::anyhow!("unable to find bot role"))?;
    let sub_role = roles
        .iter()
        .find(|&r| r.name.eq(&args.config.subscriber_role_name))
        .ok_or_else(|| anyhow::anyhow!("unable to find subscriber role"))?;

    let everyone_role = roles
        .iter()
        .find(|&r| r.name.eq("@everyone"))
        .ok_or_else(|| anyhow::anyhow!("unable to find @everyone role"))?;

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

    tokio::spawn(periodic_refresh(
        kill,
        args.interval_seconds,
        client.clone(),
        args.tautulli_client.clone(),
        args.config.clone(),
        permissions,
        args.server_id,
    ));
    Ok(())
}

async fn periodic_refresh(
    mut kill: tokio::sync::broadcast::Receiver<()>,
    interval: std::time::Duration,
    client: Arc<Http>,
    tautulli_client: TautulliClient,
    config: UpdateChannelConfig,
    permissions: Vec<Map<String, Value>>,
    server_id: u64,
) {
    let mut interval = time::interval(interval);
    loop {
        select! {
            _ = interval.tick() => {
                match client.get_channels(server_id).await {
                    Ok(channels) => {
                        match generate_stats_categories(&client, &config, &channels, &permissions, server_id).await {
                            Ok(categories) => match update_stats(&client, &tautulli_client, &categories).await {
                                Ok(_) => (),
                                Err(why) => tracing::error!("failed to update stats: {why}"),
                            },
                            Err(why) => tracing::error!("failed to generate stat channels: {why}"),
                        };
                        match generate_library_categories(&client, &config, &channels, &permissions, server_id).await {
                            Ok(categories) => match update_library_stats(&client, &tautulli_client, &categories).await {
                                Ok(_) => (),
                                Err(why) => tracing::error!("failed to update library stats: {why}"),
                            },
                            Err(why) => tracing::error!("failed to generate library channels: {why}"),
                        };
                    },
                    Err(why) => tracing::error!("failed to get Discord channels: {why}"),
                }
            }
            _ = kill.recv() => {
                tracing::info!("shutting down periodic job...");
                return;
            },
        }
    }
}

async fn get_or_create_stat_category(
    client: &Arc<Http>,
    channels: &[GuildChannel],
    create: CreateChannelConfig,
) -> Result<ChannelData> {
    match channels
        .iter()
        .find(|c| c.name.starts_with(&create.name_prefix))
    {
        Some(channel) => {
            tracing::debug!("found channel: {}", channel.name);
            Ok(ChannelData {
                prefix: create.name_prefix,
                channel: channel.to_owned(),
            })
        }
        None => {
            let prefix = String::from(&create.name_prefix);
            tracing::info!("creating channel: {}", prefix);
            let channel = create_category(client, create).await?;
            Ok(ChannelData { prefix, channel })
        }
    }
}

async fn create_category(client: &Arc<Http>, config: CreateChannelConfig) -> Result<GuildChannel> {
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

    Ok(client
        .create_channel(config.server_id, &create_channel_map, None)
        .await?)
}

async fn update_channel_name(
    client: &Arc<Http>,
    channel: &GuildChannel,
    new_name: &str,
) -> Result<()> {
    if !channel.name.eq(&new_name) {
        tracing::info!("updating channel name {new_name}");
        let mut map = JsonMap::new();
        map.insert("name".into(), new_name.into());
        client.edit_channel(channel.id.0, &map, None).await?;
    } else {
        tracing::debug!("channel name is the same, skipping...");
    }
    Ok(())
}

async fn update_stats(
    client: &Arc<Http>,
    tautulli_client: &TautulliClient,
    channels: &StatCategoryChannels,
) -> Result<()> {
    if let Some(channel) = &channels.status {
        channel_update_stats_status(client, tautulli_client, channel).await?;
    }
    let activity = tautulli_client.get_activity().await?;
    if let Some(channel) = &channels.streams {
        channel_update_stats_streams(client, &activity, channel).await?;
    }
    if let Some(channel) = &channels.transcodes {
        channel_update_stats_transcodes(client, &activity, channel).await?;
    }
    if let Some(channel) = &channels.total_bandwidth {
        channel_update_stats_bandwidth(client, channel, activity.total_bandwidth).await?;
    }
    if let Some(channel) = &channels.local_bandwidth {
        channel_update_stats_bandwidth(client, channel, activity.lan_bandwidth).await?;
    }
    if let Some(channel) = &channels.remote_bandwidth {
        channel_update_stats_bandwidth(client, channel, activity.wan_bandwidth).await?;
    }
    Ok(())
}

async fn channel_update_stats_status(
    client: &Arc<Http>,
    tautulli_client: &TautulliClient,
    channel: &ChannelData,
) -> Result<()> {
    let server_status = match tautulli_client.server_status().await {
        Ok(result) => match result.connected {
            true => "🟢",
            false => "🔴",
        },
        Err(why) => {
            tracing::error!("failed to fetch server status: {why}");
            "🟡"
        }
    };

    let new_name = format!("{} ({server_status})", channel.prefix);
    update_channel_name(client, &channel.channel, &new_name).await?;
    Ok(())
}

async fn channel_update_stats_streams(
    client: &Arc<Http>,
    data: &GetActivity,
    channel: &ChannelData,
) -> Result<()> {
    let new_name = format!("{}: {}", channel.prefix, data.stream_count);
    update_channel_name(client, &channel.channel, &new_name).await?;
    Ok(())
}

async fn channel_update_stats_transcodes(
    client: &Arc<Http>,
    data: &GetActivity,
    channel: &ChannelData,
) -> Result<()> {
    let new_name = format!("{}: {}", channel.prefix, data.stream_count_transcode);
    update_channel_name(client, &channel.channel, &new_name).await?;
    Ok(())
}

async fn channel_update_stats_bandwidth(
    client: &Arc<Http>,
    channel: &ChannelData,
    bandwidth: u32,
) -> Result<()> {
    let new_name = match bandwidth {
        n if n >= 1048576 => format!("{}: 🔥", channel.prefix),
        n if n >= 1024 && n < 1048576 => {
            format!("{}: {:.1} Mbps", channel.prefix, n as f32 / 1024.0)
        }
        n if n > 0 && n < 1024 => format!("{}: {n:.1} Kbps", channel.prefix),
        _ => format!("{}: -", channel.prefix),
    };
    update_channel_name(client, &channel.channel, &new_name).await?;
    Ok(())
}

async fn update_library_stats(
    client: &Arc<Http>,
    tautulli_client: &TautulliClient,
    channels: &LibraryStatCategoryChannels,
) -> Result<()> {
    let stats = tautulli_client.get_libraries().await?;
    let movies = stats.iter().find(|s| s.section_name.eq("Movies"));
    let tv = stats.iter().find(|s| s.section_name.eq("TV Shows"));

    match movies {
        Some(data) => {
            if let Some(channel) = &channels.movies {
                update_movies(client, data, channel).await?;
            }
        }
        None => tracing::error!("failed to find library 'Movies'"),
    }

    match tv {
        Some(data) => {
            if let Some(channel) = &channels.tv_shows {
                update_tv_shows(client, data, channel).await?;
            }
            if let Some(channel) = &channels.tv_episodes {
                update_tv_episodes(client, data, channel).await?;
            }
        }
        None => tracing::error!("failed to find library 'TV Shows'"),
    }
    Ok(())
}

async fn update_movies(client: &Arc<Http>, data: &GetLibrary, channel: &ChannelData) -> Result<()> {
    let new_name = format!("{}: {}", channel.prefix, data.count);
    update_channel_name(client, &channel.channel, &new_name).await?;
    Ok(())
}

async fn update_tv_shows(
    client: &Arc<Http>,
    data: &GetLibrary,
    channel: &ChannelData,
) -> Result<()> {
    let new_name = format!("{}: {}", channel.prefix, data.count);
    update_channel_name(client, &channel.channel, &new_name).await?;
    Ok(())
}

async fn update_tv_episodes(
    client: &Arc<Http>,
    data: &GetLibrary,
    channel: &ChannelData,
) -> Result<()> {
    let new_name = {
        if let Some(episodes) = &data.child_count {
            format!("{}: {episodes}", channel.prefix)
        } else {
            format!("{}: N/A", channel.prefix)
        }
    };
    update_channel_name(client, &channel.channel, &new_name).await?;
    Ok(())
}

async fn generate_stats_categories(
    client: &Arc<Http>,
    update_config: &UpdateChannelConfig,
    channels: &[GuildChannel],
    permissions: &[Map<String, Value>],
    server_id: u64,
) -> Result<StatCategoryChannels> {
    {
        let mut stat_channels = StatCategoryChannels {
            ..Default::default()
        };
        match &update_config.stats_category {
            Some(config) => {
                let category = get_or_create_stat_category(
                    client,
                    channels,
                    CreateChannelConfig {
                        name_prefix: String::from(&config.stat_category_name),
                        position: Some(5),
                        type_: ChannelType::Category,
                        permissions: permissions.to_owned(),
                        parent_channel: None,
                        server_id,
                    },
                )
                .await?;
                let category_id = category.channel.id.0;
                stat_channels.status = Some(category);

                if let Some(name) = &config.stream_name {
                    stat_channels.streams = Some(
                        get_or_create_stat_category(
                            client,
                            channels,
                            CreateChannelConfig {
                                name_prefix: String::from(name),
                                position: Some(0),
                                permissions: permissions.to_owned(),
                                type_: ChannelType::Voice,
                                parent_channel: Some(category_id),
                                server_id,
                            },
                        )
                        .await?,
                    );
                };

                if let Some(name) = &config.transcode_name {
                    stat_channels.transcodes = Some(
                        get_or_create_stat_category(
                            client,
                            channels,
                            CreateChannelConfig {
                                name_prefix: String::from(name),
                                position: Some(1),
                                permissions: permissions.to_owned(),
                                type_: ChannelType::Voice,
                                parent_channel: Some(category_id),
                                server_id,
                            },
                        )
                        .await?,
                    );
                };

                if let Some(name) = &config.bandwidth_total_name {
                    stat_channels.total_bandwidth = Some(
                        get_or_create_stat_category(
                            client,
                            channels,
                            CreateChannelConfig {
                                name_prefix: String::from(name),
                                position: Some(2),
                                permissions: permissions.to_owned(),
                                type_: ChannelType::Voice,
                                parent_channel: Some(category_id),
                                server_id,
                            },
                        )
                        .await?,
                    );
                };

                if let Some(name) = &config.bandwidth_local_name {
                    stat_channels.local_bandwidth = Some(
                        get_or_create_stat_category(
                            client,
                            channels,
                            CreateChannelConfig {
                                name_prefix: String::from(name),
                                position: Some(3),
                                permissions: permissions.to_owned(),
                                type_: ChannelType::Voice,
                                parent_channel: Some(category_id),
                                server_id,
                            },
                        )
                        .await?,
                    );
                };

                if let Some(name) = &config.bandwidth_remote_name {
                    stat_channels.remote_bandwidth = Some(
                        get_or_create_stat_category(
                            client,
                            channels,
                            CreateChannelConfig {
                                name_prefix: String::from(name),
                                position: Some(4),
                                permissions: permissions.to_owned(),
                                type_: ChannelType::Voice,
                                parent_channel: Some(category_id),
                                server_id,
                            },
                        )
                        .await?,
                    );
                };

                Ok(stat_channels)
            }
            None => Ok(stat_channels),
        }
    }
}

async fn generate_library_categories(
    client: &Arc<Http>,
    update_config: &UpdateChannelConfig,
    channels: &[GuildChannel],
    permissions: &[Map<String, Value>],
    server_id: u64,
) -> Result<LibraryStatCategoryChannels> {
    {
        let mut lib_channels = LibraryStatCategoryChannels {
            ..Default::default()
        };
        match &update_config.library_category {
            Some(config) => {
                let category = get_or_create_stat_category(
                    client,
                    channels,
                    CreateChannelConfig {
                        name_prefix: String::from(&config.lib_category_name),
                        position: Some(5),
                        type_: ChannelType::Category,
                        permissions: permissions.to_owned(),
                        parent_channel: None,
                        server_id,
                    },
                )
                .await?;
                let category_id = category.channel.id.0;

                if let Some(name) = &config.movies_name {
                    lib_channels.movies = Some(
                        get_or_create_stat_category(
                            client,
                            channels,
                            CreateChannelConfig {
                                name_prefix: String::from(name),
                                position: Some(0),
                                permissions: permissions.to_owned(),
                                type_: ChannelType::Voice,
                                parent_channel: Some(category_id),
                                server_id,
                            },
                        )
                        .await?,
                    );
                }

                if let Some(name) = &config.tv_shows_name {
                    lib_channels.tv_shows = Some(
                        get_or_create_stat_category(
                            client,
                            channels,
                            CreateChannelConfig {
                                name_prefix: String::from(name),
                                position: Some(1),
                                permissions: permissions.to_owned(),
                                type_: ChannelType::Voice,
                                parent_channel: Some(category_id),
                                server_id,
                            },
                        )
                        .await?,
                    );
                }

                if let Some(name) = &config.tv_episodes_name {
                    lib_channels.tv_episodes = Some(
                        get_or_create_stat_category(
                            client,
                            channels,
                            CreateChannelConfig {
                                name_prefix: String::from(name),
                                position: Some(2),
                                permissions: permissions.to_owned(),
                                type_: ChannelType::Voice,
                                parent_channel: Some(category_id),
                                server_id,
                            },
                        )
                        .await?,
                    );
                }

                Ok(lib_channels)
            }
            None => Ok(lib_channels),
        }
    }
}
