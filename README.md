# DisPlex

Discord & Plex (& more maybe one day) application.

## Features

### Discord Linked Role

You can use this bot to create a [Discord Linked Roles](https://support.discord.com/hc/en-us/articles/8063233404823-Connections-Linked-Roles-Community-Members) which verifies the user has access to your Plex instance.
It requires you to deploy the app behind proper HTTPS certificates, [cloudflared](https://github.com/cloudflare/cloudflared) can be used during developement to easily issue valid SSL certs in testing.

## Getting Started

1. [Create a Discord Application](https://discord.com/developers/applications)
1. Generate the configuration.
```bash
> cat .env
# Your FQDN
TAUTBOT_HOSTNAME="displex.example.com"
TAUTBOT_PLEX_CLIENT_ID="MyPlex"
# visit https://<plex>:32400/identity the value of 'machineIdentifier'
TAUTBOT_PLEX_SERVER_ID="<id>"
# Found after creating Discord Application
TAUTBOT_DISCORD_CLIENT_ID="<discord-client-id>"
TAUTBOT_DISCORD_CLIENT_SECRET="<discord-client-secret>"
# Can generate with openssl rand -hex 32
TAUTBOT_SESSION_SECRET_KEY="somevalue"
```
3. Populate settings in Discord Application
   1. `Linked Roles Verification URL` = `https://displex.example.com/discord/linked-role`
   1. `OAuth2 Redirects` = `https://displex.example.com/discord/callback`

4. Associate the App to a Roles Links in the Discord server.
