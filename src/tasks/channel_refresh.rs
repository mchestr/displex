use anyhow::Result;
use serde_json::{
    Map,
    Value,
};
use serenity::{
    json::JsonMap,
    model::{
        prelude::{
            ChannelType,
            GuildChannel,
        },
        Permissions,
    },
};

use crate::{
    config::{
        AppConfig,
        StatUpdateConfig,
    },
    services::{
        discord::DiscordService,
        tautulli::{
            models::{
                GetActivity,
                GetLibrary,
            },
            TautulliService,
        },
        AppServices,
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

pub async fn run(config: &AppConfig, services: &AppServices) -> Result<()> {
    let client = &services.discord_service;

    let roles = client
        .get_guild_roles(config.discord.server_id)
        .await
        .expect("failed to list discord roles");
    let bot_role = roles
        .iter()
        .find(|&r| r.name.eq(&config.discord_bot.stat_update.bot_role_name))
        .ok_or_else(|| anyhow::anyhow!("unable to find bot role"))?;
    let sub_role = roles
        .iter()
        .find(|&r| {
            r.name
                .eq(&config.discord_bot.stat_update.subscriber_role_name)
        })
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
    bot_permissions.insert("id".into(), bot_role.id.get().into());
    bot_permissions.insert("type".into(), 0.into());
    bot_permissions.insert("allow".into(), bot_perms.bits().into());

    let mut sub_permissions = JsonMap::new();
    sub_permissions.insert("id".into(), sub_role.id.get().into());
    sub_permissions.insert("type".into(), 0.into());
    sub_permissions.insert("allow".into(), sub_perms.bits().into());

    let mut everyone_permissions = JsonMap::new();
    everyone_permissions.insert("id".into(), everyone_role.id.get().into());
    everyone_permissions.insert("type".into(), 0.into());
    everyone_permissions.insert("deny".into(), everyone_perms.bits().into());

    let permissions = vec![bot_permissions, sub_permissions, everyone_permissions];

    let config = config.clone();
    let tautulli_svc = services.tautulli_service.clone();
    let discord_svc = services.discord_service.clone();
    refresh(config, discord_svc, tautulli_svc, permissions).await?;
    Ok(())
}

async fn refresh(
    config: AppConfig,
    discord_svc: DiscordService,
    tautulli_svc: TautulliService,
    permissions: Vec<Map<String, Value>>,
) -> Result<()> {
    let server_id = config.discord.server_id;
    let config = &config.discord_bot.stat_update;
    let channels = discord_svc.get_channels(server_id).await?;
    let categories =
        generate_stats_categories(&discord_svc, config, &channels, &permissions, server_id).await?;
    update_stats(&discord_svc, &tautulli_svc, &categories).await?;

    let categories =
        generate_library_categories(&discord_svc, config, &channels, &permissions, server_id)
            .await?;
    update_library_stats(&discord_svc, &tautulli_svc, &categories).await?;
    Ok(())
}

async fn get_or_create_stat_category(
    client: &DiscordService,
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

async fn create_category(
    client: &DiscordService,
    config: CreateChannelConfig,
) -> Result<GuildChannel> {
    let mut create_channel_map = JsonMap::new();
    let channel_type: u8 = config.type_.into();
    create_channel_map.insert("name".into(), config.name_prefix.as_str().into());
    create_channel_map.insert("type".into(), channel_type.into());
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
}

async fn update_channel_name(
    client: &DiscordService,
    channel: &GuildChannel,
    new_name: &str,
) -> Result<()> {
    if !channel.name.eq(&new_name) {
        tracing::info!("updating channel name {new_name}");
        let mut map = JsonMap::new();
        map.insert("name".into(), new_name.into());
        client.edit_channel(channel.id.get(), &map, None).await?;
    } else {
        tracing::debug!("channel name is the same, skipping...");
    }
    Ok(())
}

async fn update_stats(
    client: &DiscordService,
    tautulli_client: &TautulliService,
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
    client: &DiscordService,
    tautulli_client: &TautulliService,
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
    client: &DiscordService,
    data: &GetActivity,
    channel: &ChannelData,
) -> Result<()> {
    let new_name = format!("{}: {}", channel.prefix, data.stream_count);
    update_channel_name(client, &channel.channel, &new_name).await?;
    Ok(())
}

async fn channel_update_stats_transcodes(
    client: &DiscordService,
    data: &GetActivity,
    channel: &ChannelData,
) -> Result<()> {
    let new_name = format!("{}: {}", channel.prefix, data.stream_count_transcode);
    update_channel_name(client, &channel.channel, &new_name).await?;
    Ok(())
}

async fn channel_update_stats_bandwidth(
    client: &DiscordService,
    channel: &ChannelData,
    bandwidth: u32,
) -> Result<()> {
    let new_name = match bandwidth {
        n if n >= 1048576 => format!("{}: 🔥", channel.prefix),
        n if (1024..1048576).contains(&n) => {
            format!("{}: {:.1} Mbps", channel.prefix, n as f32 / 1024.0)
        }
        n if n > 0 && n < 1024 => format!("{}: {n:.1} Kbps", channel.prefix),
        _ => format!("{}: -", channel.prefix),
    };
    update_channel_name(client, &channel.channel, &new_name).await?;
    Ok(())
}

async fn update_library_stats(
    client: &DiscordService,
    tautulli_client: &TautulliService,
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

async fn update_movies(
    client: &DiscordService,
    data: &GetLibrary,
    channel: &ChannelData,
) -> Result<()> {
    let new_name = format!("{}: {}", channel.prefix, data.count);
    update_channel_name(client, &channel.channel, &new_name).await?;
    Ok(())
}

async fn update_tv_shows(
    client: &DiscordService,
    data: &GetLibrary,
    channel: &ChannelData,
) -> Result<()> {
    let new_name = format!("{}: {}", channel.prefix, data.count);
    update_channel_name(client, &channel.channel, &new_name).await?;
    Ok(())
}

async fn update_tv_episodes(
    client: &DiscordService,
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
    client: &DiscordService,
    update_config: &StatUpdateConfig,
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
                        name_prefix: String::from(&config.name),
                        position: Some(5),
                        type_: ChannelType::Category,
                        permissions: permissions.to_owned(),
                        parent_channel: None,
                        server_id,
                    },
                )
                .await?;
                let category_id = category.channel.id.get();
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
    client: &DiscordService,
    update_config: &StatUpdateConfig,
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
                        name_prefix: String::from(&config.name),
                        position: Some(5),
                        type_: ChannelType::Category,
                        permissions: permissions.to_owned(),
                        parent_channel: None,
                        server_id,
                    },
                )
                .await?;
                let category_id = category.channel.id.get();

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
