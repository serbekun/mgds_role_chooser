# MGDS Role Chooser

English | [Русский](README_ru.md)

A Discord bot written in Rust that publishes button panels for self-assigning and removing roles on the Maxim Games Discord Server.

Each panel is a message saying **"Choose your role:"** with buttons (5 per row). Panels can have different roles, and one panel can be posted to several channels. Clicking a button grants its role; clicking it again removes it. The response is visible only to the user who clicked.

## Features

- Button panel for granting and removing roles.
- Toggle a role by clicking the same button again.
- Ephemeral responses showing the result of the operation.
- Logging via `tracing`.
- Roles and panel channels are configured together in `config.yaml`, so they can be changed without rebuilding the bot.

## Requirements

- Rust 1.85+ (the project uses edition 2024).
- A Discord application with a bot and its token.

## Discord setup

1. Create an application in the [Discord Developer Portal](https://discord.com/developers/applications).
2. Get the token on the **Bot** tab.
3. Privileged intents (Server Members, Message Content) are not required — the bot only uses `GUILDS`.
4. Invite the bot to your server with the `bot` scope and the following permissions:
   - **Manage Roles** — to grant and remove roles;
   - **View Channels** and **Send Messages** — to post the panel in the target channel.
5. The bot's highest role must be **above** every role it manages. Otherwise Discord will reject the role assignment.

## Configuration

The bot is configured through environment variables:

| Variable | Required | Description |
| --- | --- | --- |
| `MGDS_ROLE_CHOOSER_TOKEN` | Yes | Discord bot token. Do not commit it to the repository. |
| `MGDS_ROLE_CHOOSER_CONFIG_PATH` | No | Path to the YAML roles and channels configuration. Defaults to `config.yaml`. |

If a panel's `channel_ids` is empty, it is not posted. If `MGDS_ROLE_CHOOSER_CONFIG_PATH` is not set, the bot reads `config.yaml` from the current working directory.

> **Important:** panels are posted to every listed channel on **every** startup. To avoid duplicates, remove the channel IDs after posting. If a send fails, posting stops and the bot continues starting. After switching to this format, delete old messages and post the panels again.

## Running

```bash
export MGDS_ROLE_CHOOSER_TOKEN="your_token"
# Optional; defaults to ./config.yaml
export MGDS_ROLE_CHOOSER_CONFIG_PATH="config.yaml"

cargo run --release
```

Build without running:

```bash
cargo build --release
```

The resulting binary is placed at `target/release/mgds_role_choosier`.

## Adding and modifying roles

Panels, channels, and roles are defined in `config.yaml`:

```yaml
panels:
  - id: games                         # stable, unique panel ID
    channel_ids:                      # this panel is posted to each channel
      - 123456789012345678
      - 234567890123456789
    roles:
      - label: Minecraft
        custom_id: role_minecraft     # unique within this panel
        role_id: 1073194781984952431
  - id: interests
    channel_ids: []
    roles:
      - label: Gaming
        custom_id: role_minecraft     # duplicates across panels are allowed
        role_id: 1072840854806470727
```

To add a role:

1. Add a panel to `panels` with a stable unique `id`, its own `channel_ids`, and `roles`.
2. Add the IDs of the channels where that panel should be posted. A panel can target multiple channels.
3. Role `custom_id`s must be unique within their panel; duplicates across panels are allowed.
4. Restart the bot — no rebuild is required. Existing messages are not updated automatically.

The bot validates the configuration on startup and refuses to start if:

- the panel list is empty, a panel ID is duplicated, or a panel ID contains invalid characters;
- a panel has no roles or more than 25 roles (Discord's limit for one message);
- a `custom_id` is empty or duplicated within its panel;
- a generated button ID is longer than 100 characters;
- a channel ID is `0` or duplicated within one panel;
- a `label` is empty or longer than 80 characters;
- a `role_id` is `0`.

Keep each panel's `id` unchanged so its buttons continue working after restarts. If you remove a panel or change its `id` or a role `custom_id`, its old buttons stop working; post the panel again.

## Project structure

```
src/
├── main.rs      # startup, config parsing, posting the panel on start
├── handler.rs   # handling button clicks and toggling roles
└── roles.rs     # loading/validating roles and panel/button construction
config.yaml      # roles and panel channel configuration
```

## Logs

The bot logs to stdout at the `INFO` level. Discord API errors (missing permissions, failed role assignment, etc.) are logged via `tracing::error!`, and the user receives a clear message in the interaction response.
