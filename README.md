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

# Development

This bot is still a work in progress. If you have any ideas for improving or adding to Displex, please open an issue
or a pull request.
