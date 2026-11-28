# MGDS Role Chooser

English | [Русский](README_ru.md)

A Discord bot written in Rust that publishes a button panel for self-assigning and removing game roles on the Maxim Games Discord Server.

The panel is a message saying **"Choose your role:"** with buttons (5 per row). Clicking a button grants the role; clicking it again removes it. The response is ephemeral (visible only to the user who clicked), and roles survive bot restarts: the buttons rely on static `custom_id`s rather than in-memory message state.

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

If `panel_channel_ids` is empty or omitted, the bot starts and waits for clicks on existing panels. If `MGDS_ROLE_CHOOSER_CONFIG_PATH` is not set, the bot reads `config.yaml` from the current working directory.

> **Important:** panels are posted to every listed channel on **every** startup. To avoid duplicates, clear `panel_channel_ids` after posting. If a send fails, posting stops at that channel, but the bot still starts.

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

Roles and panel channels are defined in `config.yaml`:

```yaml
panel_channel_ids:
  - 123456789012345678
  - 234567890123456789

roles:
  - label: Minecraft                  # button label
    custom_id: role_minecraft         # unique button ID
    role_id: 1073194781984952431      # role ID on the server
```

To add a role:

1. Add a new entry to the `roles` list in `config.yaml` (for example, `label: Factorio`, `custom_id: role_factorio`, `role_id: <id>`).
2. Make sure the `custom_id` is unique and not used anywhere else.
3. Restart the bot — no rebuild is required.
4. Add the channel IDs to `panel_channel_ids` and restart the bot to post new panels. Existing messages are not updated automatically.

The bot validates the configuration on startup and refuses to start if:

- the role list is empty or contains more than 25 roles (Discord's limit for one message);
- a `custom_id` is empty or duplicated;
- a `label` is empty or longer than 80 characters;
- a `role_id` is `0`.

Note: if you remove a role from `config.yaml`, the button with its `custom_id` in an old message will stop being handled. If you change a `custom_id`, old buttons stop working, so it is best to keep `custom_id`s stable.

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
