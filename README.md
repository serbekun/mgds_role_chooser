# MGDS Role Chooser

English | [Русский](README_ru.md)

A Discord bot written in Rust that publishes a button panel for self-assigning and removing game roles on the Maxim Games Discord Server.

The panel is a message saying **"Choose your role:"** with buttons (5 per row). Clicking a button grants the role; clicking it again removes it. The response is ephemeral (visible only to the user who clicked), and roles survive bot restarts: the buttons rely on static `custom_id`s rather than in-memory message state.

## Features

- Button panel for granting and removing roles.
- Toggle a role by clicking the same button again.
- Ephemeral responses showing the result of the operation.
- Logging via `tracing`.
- The role list is defined in code and easy to extend.

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
| `MGDS_ROLE_CHOOSIER_TOKEN` | Yes | Discord bot token. Do not commit it to the repository. |
| `ROLE_PANEL_CHANNEL_ID` | No | ID of the channel where the panel is posted on startup. |

If `ROLE_PANEL_CHANNEL_ID` is not set, the bot simply starts and waits for clicks on an existing panel.

> **Important:** the panel is posted on **every** startup when `ROLE_PANEL_CHANNEL_ID` is set. To avoid duplicate messages, post the panel once and then remove the variable or leave it unset on subsequent runs.

## Running

```bash
export MGDS_ROLE_CHOOSIER_TOKEN="your_token"
export ROLE_PANEL_CHANNEL_ID="123456789012345678"

cargo run --release
```

Build without running:

```bash
cargo build --release
```

The resulting binary is placed at `target/release/mgds_role_choosier`.

## Adding and modifying roles

Roles are defined in `src/roles.rs` in the `ROLES` array:

```rust
RoleOption {
    label: "Minecraft",                              // button label
    custom_id: "role_minecraft",                     // unique button ID
    role_id: RoleId::new(1073194781984952431),       // role ID on the server
},
```

To add a role:

1. Add a new entry to `ROLES` (for example, `RoleOption { label: "Factorio", custom_id: "role_factorio", role_id: RoleId::new(...) }`).
2. Make sure the `custom_id` is unique and not used anywhere else.
3. Rebuild and restart the bot.
4. Post a new panel (set `ROLE_PANEL_CHANNEL_ID` for one run) — the old message will not update automatically.

Note: if you remove a role from `ROLES`, the button with its `custom_id` in the old message will stop being handled. If you change a `custom_id`, the old buttons stop working, so it is best to keep `custom_id`s stable.

## Project structure

```
src/
├── main.rs      # startup, config parsing, posting the panel on start
├── handler.rs   # handling button clicks and toggling roles
└── roles.rs     # role list and panel/button construction
```

## Logs

The bot logs to stdout at the `INFO` level. Discord API errors (missing permissions, failed role assignment, etc.) are logged via `tracing::error!`, and the user receives a clear message in the interaction response.
