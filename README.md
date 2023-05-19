# DisPlex

Discord & Plex & Tautulli (& more maybe one day) application.

## Features

### Discord Linked Role

You can use this bot to create a [Discord Linked Roles](https://support.discord.com/hc/en-us/articles/8063233404823-Connections-Linked-Roles-Community-Members) which verifies the user has access to your Plex instance.
It requires you to deploy the app behind proper HTTPS certificates, [cloudflared](https://github.com/cloudflare/cloudflared) can be used during developement to easily issue valid SSL certs in testing.

## Getting Started

1. [Create a Discord Application](https://discord.com/developers/applications)
1. Generate the configuration.
```bash
> cat .env
# General App Settings
DISPLEX_HOSTNAME="displex.example.com"
DISPLEX_APPLICATION_NAME="MyPlex"
# Can generate with `openssl rand -hex 32`
DISPLEX_SESSION_SECRET_KEY="secret"
DISPLEX_ACCEPT_INVALID_CERTS=false  # optional

# visit https://<plex>:32400/identity the value of 'machineIdentifier'
DISPLEX_PLEX_SERVER_ID="plex-server-id"

# Discord Application Settings
DISPLEX_DISCORD_CLIENT_ID="clientid"
DISPLEX_DISCORD_CLIENT_SECRET="clientsecret"
DISPLEX_DISCORD_BOT_TOKEN="bottoken"

DISPLEX_TAUTULLI_API_KEY="apikey"
DISPLEX_TAUTULLI_URL="https://tautulli.example.com"
```
3. Populate settings in Discord Application
   1. `Linked Roles Verification URL` = `https://displex.example.com/discord/linked-role`
   1. `OAuth2 Redirects` = `https://displex.example.com/discord/callback`

4. Associate the App to a Roles Links in the Discord server.

## Development

Checkout [docker-compose.yaml](./docker-compose.yaml) for a sample of how you can easily setup a dev environment. [Cloudflare Tunnel](https://developers.cloudflare.com/cloudflare-one/connections/connect-apps/install-and-setup/tunnel-guide/) (or similiar) can be used to easily put your development instance behind HTTPS with valid certificates which are needed for the OAuth2 flow with Discord.

# For Development if on MacOS you have to run cloudflared via CLI since docker host networking doesn't work.
1. Setup Cloudflare tunnel
2. Setup diesel cli
# Can just launch mitm/postgres as its easier to `cargo run`
3. `docker-compose up mitm postgres -d`
4. `diesel migrations run`
5. `cargo run`

By default `HTTPS_PROXY` will make `reqwest` route all traffic through the MITM proxy running at `http://localhost:8081`, which is useful for development.
