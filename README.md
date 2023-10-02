<div align="center">

# DisPlex

A Discord, Tautulli, and Overseerr program backed by Rust.

[![Release](https://img.shields.io/github/v/release/mchestr/displex?color=blue&include_prereleases&label=version&style=flat-square)](https://github.com/mchestr/displex/releases)
[![Licence](https://img.shields.io/github/license/mchestr/displex?style=flat-square&color=blue)](https://opensource.org/licenses/MIT)

<img src="https://raw.githubusercontent.com/mchestr/displex/assets/images/displex.jpg" width="50%" height="50%" alt="logo">

</div>

# Features

DisPlex contains a few components, all packaged within a single Dockerfile. 

```
Usage: displex [OPTIONS] <COMMAND>

Commands:
  bot               
  channel-refresh   
  clean-tokens      
  metadata          
  requests-upgrade  
  server            
  user-refresh      
  help              Print this message or the help of the given subcommand(s)
```

## Subcommand: bot

Runs a Discord bot which sits in your Discord server and responds to `~ping` commands.

## Subcommand: channel-refresh  

Script which will update your Discord server channels with the realtime stats of current streams.

<img src="https://raw.githubusercontent.com/mchestr/displex/assets/images/stats.png" width="25%" height="25%" alt="logo">

## Subcommand: clean-tokens

Script which will clean up any expired Discord tokens.

## Subcommand: metadata

Script to set the Application metadata on Discord. Only needs to be called once.

<img src="https://raw.githubusercontent.com/mchestr/displex/assets/images/meta.png" width="25%" height="25%" alt="logo">

## Subcommand: requests-upgrade

Script which will set user request limits in Overseerr based on user watch hours. Tiers can be configured via the Config file.

## Subcommand: server

Runs a webserver which will guide users through the Discord Linked Role OAuth2 flow.

1. Redirect user to sign in on Discord and authorize the Application.
2. Redirect user to Plex and have user sign in.
3. Validate Plex user has access to your Plex instance, and grant user Linked Role in Discord.

## Subcommand: user-refresh

Script to set users metadata on Discord and how many hours they have streamed. Uses Tautulli for the data.

# Installation and setup

## Requirements

- A Plex Media Server
- Tautulli
- Overseerr
- A Discord server
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

### Testing OAuth Flow Locally

You can use CloudFlare tunnels or similar to test the OAuth2 flow as it requires valid certs.

```
cloudflared tunnel --name displex.example.com --hostname displex.example.com --url 'http://localhost:8080' -f
```

Then visit `https://displex.example.com/auth/discord/linked-role`.