use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::Path;

use serde::Deserialize;
use serenity::all::{ButtonStyle, ChannelId, CreateActionRow, CreateButton, CreateMessage, RoleId};

/// Maximum number of buttons Discord allows in a single message
/// (5 buttons per action row, 5 action rows).
const MAX_ROLES: usize = 25;
const BUTTONS_PER_ROW: usize = 5;
const MAX_CUSTOM_ID_LEN: usize = 100;
const MAX_LABEL_LEN: usize = 80;

/// A role entry as it is written in the YAML configuration file.
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
    #[serde(default)]
    panel_channel_ids: Vec<u64>,
    roles: Vec<RoleOption>,
}

/// Complete bot configuration loaded from a YAML file.
#[derive(Debug, Clone)]
pub struct Config {
    pub roles: Roles,
    panel_channel_ids: Vec<ChannelId>,
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
        let roles = Roles::new(file.roles).map_err(|error| -> Box<dyn Error> { error.into() })?;
        if file.panel_channel_ids.contains(&0) {
            return Err("panel_channel_ids must not contain 0".into());
        }
        Ok(Self {
            roles,
            panel_channel_ids: file
                .panel_channel_ids
                .into_iter()
                .map(ChannelId::new)
                .collect(),
        })
    }

    pub fn panel_channel_ids(&self) -> &[ChannelId] {
        &self.panel_channel_ids
    }
}

/// The list of self-assignable roles, loaded from a YAML configuration file.
#[derive(Debug, Clone)]
pub struct Roles {
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
                return Err(format!(
                    "role with custom_id `{}` has an empty label",
                    role.custom_id
                ));
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

            if role.custom_id.chars().count() > MAX_CUSTOM_ID_LEN {
                return Err(format!(
                    "role `{}` has a custom_id longer than {MAX_CUSTOM_ID_LEN} characters",
                    role.label
                ));
            }

            if !custom_ids.insert(role.custom_id.as_str()) {
                return Err(format!(
                    "duplicate custom_id `{}` in roles config",
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

    pub fn len(&self) -> usize {
        self.roles.len()
    }

    pub fn find(&self, custom_id: &str) -> Option<&RoleOption> {
        self.roles.iter().find(|role| role.custom_id == custom_id)
    }

    pub fn components(&self) -> Vec<CreateActionRow> {
        self.roles
            .chunks(BUTTONS_PER_ROW)
            .map(|roles| {
                CreateActionRow::Buttons(
                    roles
                        .iter()
                        .map(|role| {
                            CreateButton::new(role.custom_id.as_str())
                                .label(role.label.as_str())
                                .style(ButtonStyle::Secondary)
                        })
                        .collect(),
                )
            })
            .collect()
    }
}

pub async fn send_role_panel(
    http: &serenity::http::Http,
    channel_id: ChannelId,
    roles: &Roles,
) -> Result<(), Box<serenity::Error>> {
    let message = CreateMessage::new()
        .content("**Выберите себе роль:**")
        .components(roles.components());

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
roles:
  - label: Minecraft
    custom_id: role_minecraft
    role_id: 1073194781984952431
  - label: CS2
    custom_id: role_cs2
    role_id: 1072840854806470727
"#;

    #[test]
    fn parses_valid_config() {
        let roles = Config::from_yaml(VALID)
            .expect("config should be valid")
            .roles;

        assert_eq!(roles.len(), 2);
        assert_eq!(
            roles.find("role_cs2").map(|role| role.label.as_str()),
            Some("CS2")
        );
        assert!(roles.find("missing").is_none());
    }

    #[test]
    fn rejects_duplicate_custom_ids() {
        let yaml = r#"
roles:
  - label: First
    custom_id: same
    role_id: 1
  - label: Second
    custom_id: same
    role_id: 2
"#;

        assert!(Config::from_yaml(yaml).is_err());
    }

    #[test]
    fn rejects_empty_role_list() {
        assert!(Config::from_yaml("roles: []").is_err());
    }

    #[test]
    fn parses_multiple_panel_channels_and_allows_none() {
        let yaml = format!("panel_channel_ids: [12, 34]\n{VALID}");
        let config = Config::from_yaml(&yaml).expect("config should be valid");
        assert_eq!(config.panel_channel_ids().len(), 2);

        let config = Config::from_yaml(VALID).expect("channels default to empty");
        assert!(config.panel_channel_ids().is_empty());
    }
}
