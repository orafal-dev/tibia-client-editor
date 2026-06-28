export type AppearanceBooleanFlagKey =
  | "clip"
  | "bottom"
  | "top"
  | "container"
  | "cumulative"
  | "usable"
  | "forceuse"
  | "multiuse"
  | "liquidpool"
  | "unpass"
  | "unmove"
  | "unsight"
  | "avoid"
  | "no_movement_animation"
  | "take"
  | "liquidcontainer"
  | "hang"
  | "rotate"
  | "dont_hide"
  | "translucent"
  | "lying_object"
  | "animate_always"
  | "fullbank"
  | "ignore_look"
  | "wrap"
  | "unwrap"
  | "topeffect"
  | "corpse"
  | "player_corpse"
  | "ammo"
  | "show_off_socket"
  | "reportable"
  | "reverse_addons_east"
  | "reverse_addons_west"
  | "reverse_addons_south"
  | "reverse_addons_north"
  | "wearout"
  | "clockexpire"
  | "expire"
  | "expirestop";

export type AppearanceObjectFlagKey =
  | "bank"
  | "write"
  | "write_once"
  | "hook"
  | "light"
  | "shift"
  | "height"
  | "automap"
  | "lenshelp"
  | "clothes"
  | "default_action"
  | "market"
  | "changedtoexpire"
  | "cyclopediaitem"
  | "upgradeclassification";

export type AppearanceFlagGroup = {
  id: string;
  title: string;
  description: string;
  booleanFlags: AppearanceBooleanFlagKey[];
  objectFlags?: AppearanceObjectFlagKey[];
};

export const APPEARANCE_BOOLEAN_FLAG_META: Record<
  AppearanceBooleanFlagKey,
  { label: string; description: string }
> = {
  usable: { label: "Usable", description: "Can be used from inventory or map." },
  multiuse: { label: "Multi-use", description: "Supports use-with on multiple targets." },
  forceuse: { label: "Force use", description: "Forces a use action." },
  wrap: { label: "Wrap", description: "Can be wrapped for house decoration." },
  unwrap: { label: "Unwrap", description: "Can be unwrapped after wrapping." },
  unmove: { label: "Immovable", description: "Set to Off to allow moving the item." },
  unpass: { label: "Unpassable", description: "Blocks movement through the tile." },
  unsight: { label: "Blocks sight", description: "Blocks line of sight." },
  avoid: { label: "Avoid", description: "Creatures try to avoid this tile." },
  take: { label: "Take", description: "Can be picked up." },
  container: { label: "Container", description: "Acts as a container." },
  cumulative: { label: "Cumulative", description: "Stacks cumulatively." },
  liquidcontainer: { label: "Liquid container", description: "Holds liquids." },
  liquidpool: { label: "Liquid pool", description: "Creates a liquid pool." },
  clip: { label: "Clip", description: "Uses clip rendering." },
  bottom: { label: "Bottom layer", description: "Drawn on the bottom layer." },
  top: { label: "Top layer", description: "Drawn on the top layer." },
  hang: { label: "Hang", description: "Can hang on walls." },
  rotate: { label: "Rotate", description: "Can be rotated." },
  dont_hide: { label: "Don't hide", description: "Stays visible when obscured." },
  translucent: { label: "Translucent", description: "Rendered as translucent." },
  lying_object: { label: "Lying object", description: "Lies flat on the ground." },
  animate_always: { label: "Animate always", description: "Always animated." },
  fullbank: { label: "Full bank", description: "Uses full bank slot behavior." },
  ignore_look: { label: "Ignore look", description: "Look action is ignored." },
  topeffect: { label: "Top effect", description: "Shows a top effect." },
  corpse: { label: "Corpse", description: "Corpse item behavior." },
  player_corpse: { label: "Player corpse", description: "Player corpse behavior." },
  ammo: { label: "Ammo", description: "Ammunition item." },
  show_off_socket: { label: "Show off socket", description: "Shows off-hand socket." },
  reportable: { label: "Reportable", description: "Can be reported." },
  reverse_addons_east: { label: "Reverse addon E", description: "Reverse east addon." },
  reverse_addons_west: { label: "Reverse addon W", description: "Reverse west addon." },
  reverse_addons_south: { label: "Reverse addon S", description: "Reverse south addon." },
  reverse_addons_north: { label: "Reverse addon N", description: "Reverse north addon." },
  wearout: { label: "Wear out", description: "Item wears out over time." },
  clockexpire: { label: "Clock expire", description: "Expires on a clock timer." },
  expire: { label: "Expire", description: "Item expires." },
  expirestop: { label: "Expire stop", description: "Expiration can stop." },
  no_movement_animation: {
    label: "No move animation",
    description: "Disables movement animation.",
  },
};

export const APPEARANCE_OBJECT_FLAG_META: Record<
  AppearanceObjectFlagKey,
  { label: string; description: string }
> = {
  cyclopediaitem: { label: "Cyclopedia item", description: "Shows in the cyclopedia." },
  bank: { label: "Bank", description: "Bank-related metadata block." },
  write: { label: "Write", description: "Writable item metadata." },
  write_once: { label: "Write once", description: "Single-write metadata." },
  hook: { label: "Hook", description: "Hook metadata block." },
  light: { label: "Light", description: "Light-emitting metadata." },
  shift: { label: "Shift", description: "Shift metadata block." },
  height: { label: "Height", description: "Height metadata block." },
  automap: { label: "Automap", description: "Automap color metadata." },
  lenshelp: { label: "Lens help", description: "Lens help metadata." },
  clothes: { label: "Clothes", description: "Equipment slot metadata." },
  default_action: { label: "Default action", description: "Default action metadata." },
  market: { label: "Market", description: "Market metadata block." },
  changedtoexpire: { label: "Changed to expire", description: "Expire transition metadata." },
  upgradeclassification: {
    label: "Upgrade classification",
    description: "Upgrade tier metadata.",
  },
};

export const APPEARANCE_FLAG_GROUPS: AppearanceFlagGroup[] = [
  {
    id: "common",
    title: "Common edits",
    description: "Typical OT server tweaks from config.toml examples.",
    booleanFlags: ["usable", "multiuse", "forceuse", "wrap", "unwrap", "unmove", "take"],
    objectFlags: ["cyclopediaitem"],
  },
  {
    id: "movement",
    title: "Movement & blocking",
    description: "Passability and movement rules.",
    booleanFlags: ["unpass", "unsight", "avoid", "no_movement_animation", "rotate"],
  },
  {
    id: "container",
    title: "Containers & liquids",
    description: "Storage and fluid behavior.",
    booleanFlags: ["container", "cumulative", "liquidcontainer", "liquidpool", "fullbank"],
  },
  {
    id: "rendering",
    title: "Rendering",
    description: "Visual layering and effects.",
    booleanFlags: [
      "clip",
      "bottom",
      "top",
      "hang",
      "dont_hide",
      "translucent",
      "lying_object",
      "animate_always",
      "topeffect",
    ],
  },
  {
    id: "equipment",
    title: "Equipment & corpses",
    description: "Wearables, ammo, and corpse flags.",
    booleanFlags: ["corpse", "player_corpse", "ammo", "show_off_socket", "ignore_look", "reportable"],
    objectFlags: ["clothes", "market", "upgradeclassification"],
  },
  {
    id: "decay",
    title: "Decay & timers",
    description: "Expiration-related flags.",
    booleanFlags: ["wearout", "clockexpire", "expire", "expirestop"],
    objectFlags: ["changedtoexpire"],
  },
  {
    id: "addons",
    title: "Addons",
    description: "Reverse addon directions.",
    booleanFlags: [
      "reverse_addons_east",
      "reverse_addons_west",
      "reverse_addons_south",
      "reverse_addons_north",
    ],
  },
  {
    id: "metadata",
    title: "Extra metadata blocks",
    description: "Object blocks written as empty `{}` in config.toml.",
    booleanFlags: [],
    objectFlags: [
      "bank",
      "write",
      "write_once",
      "hook",
      "light",
      "shift",
      "height",
      "automap",
      "lenshelp",
      "default_action",
    ],
  },
];

export type FlagTriState = "unset" | "true" | "false";

export const getBooleanFlagState = (
  flags: Partial<Record<AppearanceBooleanFlagKey, boolean>>,
  key: AppearanceBooleanFlagKey,
): FlagTriState => {
  if (!(key in flags)) return "unset";
  return flags[key] ? "true" : "false";
};

export const setBooleanFlagState = (
  flags: Partial<Record<AppearanceBooleanFlagKey, boolean>>,
  key: AppearanceBooleanFlagKey,
  state: FlagTriState,
): Partial<Record<AppearanceBooleanFlagKey, boolean>> => {
  const next = { ...flags };
  if (state === "unset") {
    delete next[key];
    return next;
  }
  next[key] = state === "true";
  return next;
};

export const countActiveFlags = (entry: {
  booleanFlags: Partial<Record<AppearanceBooleanFlagKey, boolean>>;
  objectFlags: Partial<Record<AppearanceObjectFlagKey, true>>;
}): number =>
  Object.keys(entry.booleanFlags).length + Object.keys(entry.objectFlags).length;
