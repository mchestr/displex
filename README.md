<div align="center">

# DisPlex

A Discord, Plex, & Tautulli Bot that displays live data in Discord + allows users to verify their identity with your Plex server.

[![Release](https://img.shields.io/github/v/release/mchestr/displex?color=blue&include_prereleases&label=version&style=flat-square)](https://github.com/mchestr/displex/releases)
[![Licence](https://img.shields.io/github/license/mchestr/displex?style=flat-square&color=blue)](https://opensource.org/licenses/MIT)

<img src="https://raw.githubusercontent.com/mchestr/displex/assets/images/displex.jpg" width="50%" height="50%" alt="logo">

</div>

# Features

DisPlex uses the Tautulli API to pull information from Tautulli and display them in a Discord channel, including:

### Overview:

- Status of Plex server
- Number of current streams
- Number of transcoding streams
- Total bandwidth
- Library item counts
- [Discord Linked Roles](https://support.discord.com/hc/en-us/articles/8063233404823-Connections-Linked-Roles-Community-Members)

#### Channel Stats

<img src="https://raw.githubusercontent.com/mchestr/displex/assets/images/stats.png" width="25%" height="25%" alt="logo">

#### User Metadata

<img src="https://raw.githubusercontent.com/mchestr/displex/assets/images/meta.png" width="25%" height="25%" alt="logo">

# Installation and setup

## Requirements

- A Plex Media Server
- Tautulli (formerly known as PlexPy)
- A Discord server
- Postgres
- Valid SSL Cert
- Docker
- [A Discord bot token](https://www.digitaltrends.com/gaming/how-to-make-a-discord-bot/)
  - Permissions required:
    - Manage Channels
    - View Channels
    - Send Messages
  - **Shortcut**: Use the following link to invite your bot to your server with the above permissions:
    https://discord.com/oauth2/authorize?client_id=YOUR_APPLICATION_ID&scope=bot&permissions=2064

DisPlex runs as a Docker container. The Dockerfile is included in this repository, or can be pulled
from [GitHub Packages](https://github.com/mchestr/displex/pkgs/container/displex).

### Environment Variables

| Name                                           | Description                                                                                             | Required | Default/Values |
| ---------------------------------------------- | ------------------------------------------------------------------------------------------------------- | -------- | -------------- |
| DISPLEX_HOSTNAME                               | Hostname of application. Used to generate the redirect URLs for OAuth2.                                 | yes      |                |
| DISPLEX_APPLICATION_NAME                       | Name of application. Will be displayed on Plex Sign-in mostly.                                          | yes      |                |
| DISPLEX_HTTP_HOST                              | Host to bind HTTP server.                                                                               | no       | 127.0.0.1      |
| DISPLEX_HTTP_PORT                              | Port to bind HTTP server                                                                                | no       | 8080           |
| DISPLEX_SESSION_SECRET_KEY                     | Session secret value for encryption. Mostly used in OAuth2 flow to store state between requests         | yes      |                |
| DISPLEX_DATABASE_URL                           | PostgresQL database url. For example postgres://displex:password@localhost/displex                      | yes      |                |
| DISPLEX_ACCEPT_INVALID_CERTS                   | Control whether reqwest will validate SSL certs. Useful for MITM proxy development.                     | no       | false          |
| DISPLEX_PLEX_SERVER_ID                         | Plex Server ID. When a user attempts to link the role, it will check if they have access to the server. | yes      |                |
| DISPLEX_DISCORD_CLIENT_ID                      | Discord Application Client ID.                                                                          | yes      |                |
| DISPLEX_DISCORD_CLIENT_SECRET                  | Discord Application Client Secret.                                                                      | yes      |                |
| DISPLEX_DISCORD_BOT_TOKEN                      | Discord Application Bot Token. Only used at the moment to register the application metadata.            | yes      |                |
| DISPLEX_DISCORD_SERVER_ID                      | Discord Server ID, used for the redirect back to Discord after authorization flow.                      | yes      |                |
| DISPLEX_TAUTULLI_API_KEY                       | Tautulli API key.                                                                                       | yes      |                |
| DISPLEX_TAUTULLI_URL                           | URL to Tautulli server. For example https://localhost:8181                                              | yes      |                |
| DISPLEX_DISCORD_BOT_STAT_CATEGORY_NAME         | Name of the category in Discord for stats, if omitted no channels are created                           | no       |                |
| DISPLEX_DISCORD_BOT_STAT_STATUS_NAME           | Name of the stat status channel, if omitted no channel is created                                       | no       |                |
| DISPLEX_DISCORD_BOT_STAT_STREAM_NAME           | Name of the stat stream channel, if omitted no channel is created                                       | no       |                |
| DISPLEX_DISCORD_BOT_STAT_TRANSCODE_NAME        | Name of the stat transcode channel, if omitted no channel is created                                    | no       |                |
| DISPLEX_DISCORD_BOT_STAT_TOTAL_BANDWIDTH_NAME  | Name of the stat total bandwidth channel, if omitted no channel is created                              | no       |                |
| DISPLEX_DISCORD_BOT_STAT_LOCAL_BANDWIDTH_NAME  | Name of the stat local bandwidth channel, if omitted no channel is created                              | no       |                |
| DISPLEX_DISCORD_BOT_STAT_REMOTE_BANDWIDTH_NAME | Name of the stat remote bandwidth channel, if omitted no channel is created                             | no       |                |
| DISPLEX_DISCORD_BOT_LIB_CATEGORY_NAME          | Name of the library category in Discord, if omitted no channels are created                             | no       |                |
| DISPLEX_DISCORD_BOT_LIB_MOVIES_NAME            | Name of the library movies channel, if omitted no channel is created                                    | no       |                |
| DISPLEX_DISCORD_BOT_LIB_TV_SHOWS_NAME          | Name of the library tv shows channel, if omitted no channel is created                                  | no       |                |
| DISPLEX_DISCORD_BOT_LIB_TV_EPISODES_NAME       | Name of the library tv episodes channel, if omitted no channel is created                               | no       |                |
| DISPLEX_DISCORD_BOT_ROLE_NAME                  | Name of the Discord role for the Bot                                                                    | no       | Bot            |
| DISPLEX_DISCORD_BOT_LIB_TV_EPISODES_NAME       | Name of the Discord role for subscribers                                                                | no       | Subscriber     |
| DISPLEX_DISCORD_BOT_STATUS                     | Name of the watching activity for the bot                                                               | no       | DisPlex        |
| DISPLEX_DISCORD_STAT_UPDATE_INTERVAL           | How often to update Discord channels                                                                    | no       | 60s            |
| DISPLEX_DISCORD_USER_UPDATE_INTERVAL           | How often to update Discord users metadata                                                              | no       | 1h             |
| DISPLEX_DISCORD_BOT                            | Can be used to disable the bot which refreshes Discord channels                                         | no       | Serenity/None  |
| DISPLEX_HTTP_SERVER                            | Can be used to disable the http server used for linking roles Discord                                   | no       | Axum/None      |

# Development

This bot is still a work in progress. If you have any ideas for improving or adding to Displex, please open an issue
or a pull request.
