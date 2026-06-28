use serde_json::Value;

use super::proto::{
    AppearanceFlagAutomap, AppearanceFlagBank, AppearanceFlagChangedToExpire,
    AppearanceFlagClothes, AppearanceFlagCyclopedia, AppearanceFlagDefaultAction,
    AppearanceFlagHeight, AppearanceFlagHook, AppearanceFlagLenshelp, AppearanceFlagLight,
    AppearanceFlagMarket, AppearanceFlagShift, AppearanceFlagUpgradeClassification,
    AppearanceFlagWrite, AppearanceFlagWriteOnce, AppearanceFlags,
};

pub fn merge_flags(existing: Option<AppearanceFlags>, edit: &std::collections::HashMap<String, Value>) -> AppearanceFlags {
    let mut flags = existing.unwrap_or_default();
    for (key, value) in edit {
        apply_field(&mut flags, key, value);
    }
    flags
}

fn apply_field(flags: &mut AppearanceFlags, key: &str, value: &Value) {
    match key {
        "clip" => flags.clip = value.as_bool(),
        "bottom" => flags.bottom = value.as_bool(),
        "top" => flags.top = value.as_bool(),
        "container" => flags.container = value.as_bool(),
        "cumulative" => flags.cumulative = value.as_bool(),
        "usable" => flags.usable = value.as_bool(),
        "forceuse" => flags.forceuse = value.as_bool(),
        "multiuse" => flags.multiuse = value.as_bool(),
        "liquidpool" => flags.liquidpool = value.as_bool(),
        "unpass" => flags.unpass = value.as_bool(),
        "unmove" => flags.unmove = value.as_bool(),
        "unsight" => flags.unsight = value.as_bool(),
        "avoid" => flags.avoid = value.as_bool(),
        "no_movement_animation" => flags.no_movement_animation = value.as_bool(),
        "take" => flags.take = value.as_bool(),
        "liquidcontainer" => flags.liquidcontainer = value.as_bool(),
        "hang" => flags.hang = value.as_bool(),
        "rotate" => flags.rotate = value.as_bool(),
        "dont_hide" => flags.dont_hide = value.as_bool(),
        "translucent" => flags.translucent = value.as_bool(),
        "lying_object" => flags.lying_object = value.as_bool(),
        "animate_always" => flags.animate_always = value.as_bool(),
        "fullbank" => flags.fullbank = value.as_bool(),
        "ignore_look" => flags.ignore_look = value.as_bool(),
        "wrap" => flags.wrap = value.as_bool(),
        "unwrap" => flags.unwrap = value.as_bool(),
        "topeffect" => flags.topeffect = value.as_bool(),
        "corpse" => flags.corpse = value.as_bool(),
        "player_corpse" => flags.player_corpse = value.as_bool(),
        "ammo" => flags.ammo = value.as_bool(),
        "show_off_socket" => flags.show_off_socket = value.as_bool(),
        "reportable" => flags.reportable = value.as_bool(),
        "reverse_addons_east" => flags.reverse_addons_east = value.as_bool(),
        "reverse_addons_west" => flags.reverse_addons_west = value.as_bool(),
        "reverse_addons_south" => flags.reverse_addons_south = value.as_bool(),
        "reverse_addons_north" => flags.reverse_addons_north = value.as_bool(),
        "wearout" => flags.wearout = value.as_bool(),
        "clockexpire" => flags.clockexpire = value.as_bool(),
        "expire" => flags.expire = value.as_bool(),
        "expirestop" => flags.expirestop = value.as_bool(),
        "bank" => {
            if value.is_object() {
                flags.bank = Some(AppearanceFlagBank::default());
            }
        }
        "write" => {
            if value.is_object() {
                flags.r#write = Some(AppearanceFlagWrite::default());
            }
        }
        "write_once" => {
            if value.is_object() {
                flags.write_once = Some(AppearanceFlagWriteOnce::default());
            }
        }
        "hook" => {
            if value.is_object() {
                flags.hook = Some(AppearanceFlagHook::default());
            }
        }
        "light" => {
            if value.is_object() {
                flags.light = Some(AppearanceFlagLight::default());
            }
        }
        "shift" => {
            if value.is_object() {
                flags.shift = Some(AppearanceFlagShift::default());
            }
        }
        "height" => {
            if value.is_object() {
                flags.height = Some(AppearanceFlagHeight::default());
            }
        }
        "automap" => {
            if value.is_object() {
                flags.automap = Some(AppearanceFlagAutomap::default());
            }
        }
        "lenshelp" => {
            if value.is_object() {
                flags.lenshelp = Some(AppearanceFlagLenshelp::default());
            }
        }
        "clothes" => {
            if value.is_object() {
                flags.clothes = Some(AppearanceFlagClothes::default());
            }
        }
        "default_action" => {
            if value.is_object() {
                flags.default_action = Some(AppearanceFlagDefaultAction::default());
            }
        }
        "market" => {
            if value.is_object() {
                flags.market = Some(AppearanceFlagMarket::default());
            }
        }
        "changedtoexpire" => {
            if value.is_object() {
                flags.changedtoexpire = Some(AppearanceFlagChangedToExpire::default());
            }
        }
        "cyclopediaitem" => {
            if value.is_object() {
                flags.cyclopediaitem = Some(AppearanceFlagCyclopedia::default());
            }
        }
        "upgradeclassification" => {
            if value.is_object() {
                flags.upgradeclassification = Some(AppearanceFlagUpgradeClassification::default());
            }
        }
        _ => {}
    }
}
