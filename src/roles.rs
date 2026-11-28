use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::Path;

use serde::Deserialize;
use serenity::all::{ButtonStyle, ChannelId, CreateActionRow, CreateButton, CreateMessage, RoleId};

/// Discord allows at most five rows with five buttons each in one message.
const MAX_ROLES: usize = 25;
const BUTTONS_PER_ROW: usize = 5;
const MAX_CUSTOM_ID_LEN: usize = 100;
const MAX_LABEL_LEN: usize = 80;
const BUTTON_ID_PREFIX: &str = "panel:";

#[derive(Debug, Clone, Deserialize)]
pub struct RoleOption {
    pub label: String,
    pub custom_id: String,
    pub role_id: u64,
}

impl RoleOption {
    pub fn role_id(&self) -> RoleId {
        RoleId::new(self.role_id)
    }
}

#[derive(Debug, Deserialize)]
struct ConfigFile {
    panels: Vec<PanelFile>,
}

#[derive(Debug, Deserialize)]
struct PanelFile {
    id: String,
    #[serde(default)]
    channel_ids: Vec<u64>,
    roles: Vec<RoleOption>,
}

#[derive(Debug, Clone)]
pub struct Panel {
    pub id: String,
    channel_ids: Vec<ChannelId>,
    roles: Roles,
}

impl Panel {
    #[allow(dead_code)]
    pub fn roles_len(&self) -> usize {
        self.roles.len()
    }

    pub fn channel_ids(&self) -> &[ChannelId] {
        &self.channel_ids
    }

    pub fn components(&self) -> Vec<CreateActionRow> {
        self.roles
            .roles
            .chunks(BUTTONS_PER_ROW)
            .map(|roles| {
                CreateActionRow::Buttons(
                    roles
                        .iter()
                        .map(|role| {
                            CreateButton::new(button_custom_id(&self.id, &role.custom_id))
                                .label(role.label.as_str())
                                .style(ButtonStyle::Secondary)
                        })
                        .collect(),
                )
            })
            .collect()
    }
}

/// Complete bot configuration loaded from YAML.
#[derive(Debug, Clone)]
pub struct Config {
    panels: Vec<Panel>,
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .map_err(|error| format!("failed to read config {}: {error}", path.display()))?;
        Self::from_yaml(&content)
            .map_err(|error| format!("invalid config {}: {error}", path.display()).into())
    }

    pub fn from_yaml(yaml: &str) -> Result<Self, Box<dyn Error>> {
        let file: ConfigFile = serde_yaml::from_str(yaml)?;
        if file.panels.is_empty() {
            return Err("at least one panel must be configured".into());
        }

        let mut panel_ids = HashSet::<String>::with_capacity(file.panels.len());
        let mut panels = Vec::with_capacity(file.panels.len());
        for panel in file.panels {
            if panel.id.is_empty()
                || !panel
                    .id
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
            {
                return Err(format!(
                    "panel id `{}` must contain only ASCII letters, digits, `_` or `-`",
                    panel.id
                )
                .into());
            }
            if !panel_ids.insert(panel.id.clone()) {
                return Err(format!("duplicate panel id `{}`", panel.id).into());
            }

            if panel.channel_ids.contains(&0) {
                return Err(format!("panel `{}` has a channel ID of 0", panel.id).into());
            }
            let mut channel_ids = HashSet::with_capacity(panel.channel_ids.len());
            for channel_id in &panel.channel_ids {
                if !channel_ids.insert(channel_id) {
                    return Err(format!(
                        "panel `{}` has duplicate channel ID `{channel_id}`",
                        panel.id
                    )
                    .into());
                }
            }

            let roles = Roles::new(panel.roles).map_err(|error| -> Box<dyn Error> {
                format!("panel `{}`: {error}", panel.id).into()
            })?;
            for role in &roles.roles {
                if button_custom_id(&panel.id, &role.custom_id).chars().count() > MAX_CUSTOM_ID_LEN
                {
                    return Err(format!(
                        "button ID for panel `{}` and role `{}` exceeds {MAX_CUSTOM_ID_LEN} characters",
                        panel.id, role.custom_id
                    )
                    .into());
                }
            }

            panels.push(Panel {
                id: panel.id,
                channel_ids: panel.channel_ids.into_iter().map(ChannelId::new).collect(),
                roles,
            });
        }

        Ok(Self { panels })
    }

    pub fn panels(&self) -> &[Panel] {
        &self.panels
    }

    /// Resolves the namespaced button ID emitted by a configured panel.
    pub fn find_button_role(&self, custom_id: &str) -> Option<&RoleOption> {
        let namespaced_id = custom_id.strip_prefix(BUTTON_ID_PREFIX)?;
        let (panel_id, role_custom_id) = namespaced_id.split_once(':')?;
        let panel = self.panels.iter().find(|panel| panel.id == panel_id)?;
        panel.roles.find(role_custom_id)
    }
}

fn button_custom_id(panel_id: &str, role_custom_id: &str) -> String {
    format!("{BUTTON_ID_PREFIX}{panel_id}:{role_custom_id}")
}

#[derive(Debug, Clone)]
struct Roles {
    roles: Vec<RoleOption>,
}

impl Roles {
    fn new(roles: Vec<RoleOption>) -> Result<Self, String> {
        if roles.is_empty() {
            return Err("at least one role must be configured".to_string());
        }
        if roles.len() > MAX_ROLES {
            return Err(format!(
                "too many roles: {} configured, at most {MAX_ROLES} are supported",
                roles.len()
            ));
        }

        let mut custom_ids = HashSet::with_capacity(roles.len());
        for role in &roles {
            if role.label.trim().is_empty() {
                return Err(format!("role `{}` has an empty label", role.custom_id));
            }
            if role.label.chars().count() > MAX_LABEL_LEN {
                return Err(format!(
                    "role `{}` has a label longer than {MAX_LABEL_LEN} characters",
                    role.custom_id
                ));
            }
            if role.custom_id.trim().is_empty() {
                return Err(format!("role `{}` has an empty custom_id", role.label));
            }
            if !custom_ids.insert(role.custom_id.as_str()) {
                return Err(format!(
                    "duplicate custom_id `{}` within panel",
                    role.custom_id
                ));
            }
            if role.role_id == 0 {
                return Err(format!(
                    "role `{}` has an invalid role_id (0)",
                    role.custom_id
                ));
            }
        }
        Ok(Self { roles })
    }

    fn len(&self) -> usize {
        self.roles.len()
    }

    fn find(&self, custom_id: &str) -> Option<&RoleOption> {
        self.roles.iter().find(|role| role.custom_id == custom_id)
    }
}

pub async fn send_role_panel(
    http: &serenity::http::Http,
    channel_id: ChannelId,
    panel: &Panel,
) -> Result<(), Box<serenity::Error>> {
    let message = CreateMessage::new()
        .content("**Выберите себе роль:**")
        .components(panel.components());
    channel_id
        .send_message(http, message)
        .await
        .map_err(Box::new)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = r#"
panels:
  - id: games
    channel_ids: [12, 34]
    roles:
      - label: Minecraft
        custom_id: role_game
        role_id: 123
      - label: CS2
        custom_id: role_cs2
        role_id: 456
  - id: interests
    channel_ids: [56]
    roles:
      - label: Gaming
        custom_id: role_game
        role_id: 789
"#;

    #[test]
    fn parses_panels_and_allows_duplicate_custom_ids_across_panels() {
        let config = Config::from_yaml(VALID).expect("config should be valid");
        assert_eq!(config.panels().len(), 2);
        assert_eq!(config.panels()[0].channel_ids().len(), 2);
        assert_eq!(config.panels()[0].roles_len(), 2);
        assert_eq!(
            config
                .find_button_role("panel:games:role_game")
                .unwrap()
                .role_id,
            123
        );
        assert_eq!(
            config
                .find_button_role("panel:interests:role_game")
                .unwrap()
                .role_id,
            789
        );
    }

    #[test]
    fn rejects_duplicate_panel_ids() {
        let yaml = VALID.replace("id: interests", "id: games");
        assert!(Config::from_yaml(&yaml).is_err());
    }

    #[test]
    fn rejects_empty_panels_and_invalid_role_lists() {
        assert!(Config::from_yaml("panels: []").is_err());
        assert!(Config::from_yaml("panels: [{ id: empty, roles: [] }]").is_err());
    }

    #[test]
    fn rejects_duplicate_role_ids_within_panel_and_invalid_channels() {
        let duplicate_roles = VALID.replace("custom_id: role_cs2", "custom_id: role_game");
        assert!(Config::from_yaml(&duplicate_roles).is_err());

        let duplicate_channels = VALID.replace("channel_ids: [12, 34]", "channel_ids: [12, 12]");
        assert!(Config::from_yaml(&duplicate_channels).is_err());

        let zero_channel = VALID.replace("channel_ids: [12, 34]", "channel_ids: [0]");
        assert!(Config::from_yaml(&zero_channel).is_err());
    }

    #[test]
    fn rejects_invalid_or_overlong_generated_button_ids() {
        let invalid_id = VALID.replace("id: games", "id: bad:id");
        assert!(Config::from_yaml(&invalid_id).is_err());

        let long_custom_id = "x".repeat(95);
        let yaml = VALID.replace("role_game", &long_custom_id);
        assert!(Config::from_yaml(&yaml).is_err());
    }
}
