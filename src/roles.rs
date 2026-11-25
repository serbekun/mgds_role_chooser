use serenity::all::{ButtonStyle, ChannelId, CreateActionRow, CreateButton, CreateMessage, RoleId};

pub struct RoleOption {
    pub label: &'static str,
    pub custom_id: &'static str,
    pub role_id: RoleId,
}

pub const ROLES: &[RoleOption] = &[
    RoleOption {
        label: "Minecraft",
        custom_id: "role_minecraft",
        role_id: RoleId::new(1073194781984952431),
    },
    RoleOption {
        label: "Repo",
        custom_id: "role_repo",
        role_id: RoleId::new(1394293055636373574),
    },
    RoleOption {
        label: "CS2",
        custom_id: "role_cs2",
        role_id: RoleId::new(1072840854806470727),
    },
    RoleOption {
        label: "Roblox",
        custom_id: "role_roblox",
        role_id: RoleId::new(1227606279959220314),
    },
    RoleOption {
        label: "Lethal Company",
        custom_id: "role_lethal_company",
        role_id: RoleId::new(1215252748984123442),
    },
    RoleOption {
        label: "Among Us",
        custom_id: "role_among_us",
        role_id: RoleId::new(1227907907996221515),
    },
    RoleOption {
        label: "BackShot",
        custom_id: "role_back_shot",
        role_id: RoleId::new(1404722267585380362),
    },
    RoleOption {
        label: "OpenTTD",
        custom_id: "role_open_ttd",
        role_id: RoleId::new(1328704474067239002),
    },
    RoleOption {
        label: "Terraria",
        custom_id: "role_terraria",
        role_id: RoleId::new(1335572344642736162),
    },
    RoleOption {
        label: "World of Tanks",
        custom_id: "role_world_of_tanks",
        role_id: RoleId::new(1407717531598454875),
    },
];

pub fn get_role_components() -> Vec<CreateActionRow> {
    ROLES
        .chunks(5)
        .map(|roles| {
            CreateActionRow::Buttons(
                roles
                    .iter()
                    .map(|role| {
                        CreateButton::new(role.custom_id)
                            .label(role.label)
                            .style(ButtonStyle::Secondary)
                    })
                    .collect(),
            )
        })
        .collect()
}

pub async fn send_role_panel(
    http: &serenity::http::Http,
    channel_id: ChannelId,
) -> serenity::Result<()> {
    let message = CreateMessage::new()
        .content("**Выберите себе роль:**")
        .components(get_role_components());

    channel_id.send_message(http, message).await?;

    Ok(())
}
