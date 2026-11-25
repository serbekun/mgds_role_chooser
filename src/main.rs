use std::vec;

use serenity::all::CreateButton;
use tracing::info;
use tracing_subscriber;

use crate::Roles::Minecraft;

enum Roles {
    Minecraft,
    Repo,
    CS2,
    Roblox,
    LethalCompany,
    AmongUs,
    BackShot,
    OpenTTD,
    Terraria,
    WorldOfTanks,
}

impl Roles {
    fn get_id(role: &Roles) -> String {
        match &role {
            Roles::Minecraft => return "1073194781984952431".to_string(),
            Roles::Repo => return "1394293055636373574".to_string(),
            Roles::CS2 => return "1072840854806470727".to_string(),
            Roles::Roblox => return "1227606279959220314".to_string(),
            Roles::LethalCompany => return "1215252748984123442".to_string(),
            Roles::AmongUs => return "1227907907996221515".to_string(),
            Roles::BackShot => return "1404722267585380362".to_string(),
            Roles::OpenTTD => return "1328704474067239002".to_string(),
            Roles::Terraria => "1335572344642736162".to_string(),
            Roles::WorldOfTanks => "1407717531598454875".to_string(),
            _ => "".to_string(),
        }
    }
}

#[tokio::main]
async fn main() {
    // initial global logger.
    tracing_subscriber::fmt::init();

    info!("mgds role chooser");
}

fn get_all_roles_buttons() -> Vec<CreateButton> {
    let minecraft = CreateButton::new("role_minecraft");
    let repo = CreateButton::new("role_repo");
    let cs2 = CreateButton::new("role_cs2");
    let roblox = CreateButton::new("role_roblox");
    let lethal_company = CreateButton::new("role_lethal_company");
    let among_us = CreateButton::new("role_among_us");
    let back_shot = CreateButton::new("role_back_shot");
    let open_ttd = CreateButton::new("role_open_ttd");
    let terraria = CreateButton::new("role_terraria");
    let world_of_tanks = CreateButton::new("role_world_of_tanks");
    vec![
        minecraft,
        repo,
        cs2,
        roblox,
        lethal_company,
        among_us,
        back_shot,
        open_ttd,
        terraria,
        world_of_tanks,
    ]
}
