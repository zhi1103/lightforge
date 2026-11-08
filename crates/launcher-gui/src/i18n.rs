//! Fluent resources and the launcher's small localization surface.

use std::cell::RefCell;
use std::collections::HashMap;

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use unic_langid::LanguageIdentifier;

const EN_US: &str = include_str!("../locales/en-US.ftl");
const ZH_CN: &str = include_str!("../locales/zh-CN.ftl");

pub(crate) const ENGLISH: &str = "en-US";
pub(crate) const CHINESE: &str = "zh-CN";

pub(crate) const MENU_SETTINGS: &str = "app-menu-settings";
pub(crate) const MENU_UPDATES: &str = "app-menu-updates";
pub(crate) const MENU_REPORT_BUG: &str = "app-menu-report-bug";
pub(crate) const MENU_FEEDBACK: &str = "app-menu-feedback";
pub(crate) const MENU_EXIT: &str = "app-menu-exit";
pub(crate) const COMMON_REMOVE: &str = "common-remove";
pub(crate) const SETTINGS_GENERAL: &str = "settings-nav-general";
pub(crate) const SETTINGS_SETUP: &str = "settings-nav-setup";
pub(crate) const SETTINGS_PROTON: &str = "settings-nav-proton";
pub(crate) const SETTINGS_BATTLENET: &str = "settings-nav-battlenet";
pub(crate) const SETTINGS_ADDONS: &str = "settings-nav-addons";
pub(crate) const SETTINGS_STORAGE: &str = "settings-nav-storage";
pub(crate) const SETTINGS_ABOUT: &str = "settings-nav-about";

/// A translated catalog title, keyed only by Blizzard's stable product code.
/// Unknown and custom products keep the label supplied by the manifest.
pub(crate) fn product_title(language: &str, code: &str, label: &str) -> String {
    if language != CHINESE {
        return label.to_owned();
    }
    let id = match code {
        "wow" | "wowt" | "wowxptr" | "wow_beta" => "product-title-wow-retail",
        "wow_classic" | "wow_classic_ptr" => "product-title-wow-classic",
        "wow_classic_era" | "wow_classic_era_ptr" => "product-title-wow-classic-era",
        "wow_classic_beta" => "product-title-wow-forever",
        "wowf" => "product-title-wow-forever",
        "wow_anniversary" => "product-title-wow-anniversary",
        "osi" => "product-title-diablo-ii",
        "d3" => "product-title-diablo-iii",
        "fenris" => "product-title-diablo-iv",
        "anbs" => "product-title-diablo-immortal",
        "hsb" | "hs_beta" => "product-title-hearthstone",
        "w1r" => "product-title-warcraft-i",
        "w2r" => "product-title-warcraft-ii",
        "w3" => "product-title-warcraft-iii",
        "s1" => "product-title-starcraft",
        "s2" => "product-title-starcraft-ii",
        "pro" => "product-title-overwatch",
        "hero" => "product-title-heroes",
        _ => return label.to_owned(),
    };
    text(language, id)
}

/// The icon tooltip may translate only the unmodified built-in product name.
pub(crate) fn product_title_if_default(language: &str, code: &str, label: &str, manifest_label: Option<&str>) -> String {
    if language != CHINESE || product_title(language, code, label) == label {
        return label.to_owned();
    }
    let built_in = match code {
        "wow" => &["World of Warcraft"][..],
        "wow_classic" => &["WoW Classic (progression)", "World of Warcraft Classic"][..],
        "wow_classic_era" => &["World of Warcraft Classic"][..],
        "wowf" => &["World of Warcraft: Forever"][..],
        "wowt" => &["WoW PTR"][..],
        _ => &[][..],
    };
    let default = manifest_label.is_some_and(|name| name == label) || built_in.contains(&label);
    if default { product_title(language, code, label) } else { label.to_owned() }
}

pub(crate) const GENERAL_LANGUAGE_TITLE: &str = "settings-general-language-title";
pub(crate) const GENERAL_LANGUAGE_ABOUT: &str = "settings-general-language-about";
pub(crate) const GENERAL_LANGUAGE_ENGLISH: &str = "settings-general-language-english";
pub(crate) const GENERAL_LANGUAGE_CHINESE: &str = "settings-general-language-chinese";
pub(crate) const GENERAL_MINIMIZE_TITLE: &str = "settings-general-minimize-title";
pub(crate) const GENERAL_MINIMIZE_ABOUT: &str = "settings-general-minimize-about";
pub(crate) const GENERAL_MINIMIZE_HINT: &str = "settings-general-minimize-hint";
pub(crate) const GENERAL_BATTLENET_MINIMIZED_TITLE: &str = "settings-general-battlenet-minimized-title";
pub(crate) const GENERAL_BATTLENET_MINIMIZED_ABOUT: &str = "settings-general-battlenet-minimized-about";
pub(crate) const GENERAL_BATTLENET_MINIMIZED_HINT: &str = "settings-general-battlenet-minimized-hint";
pub(crate) const GENERAL_BATTLENET_CLOSE_TITLE: &str = "settings-general-battlenet-close-title";
pub(crate) const GENERAL_BATTLENET_CLOSE_ABOUT: &str = "settings-general-battlenet-close-about";
pub(crate) const GENERAL_BATTLENET_CLOSE_HINT: &str = "settings-general-battlenet-close-hint";
pub(crate) const GENERAL_NOTES_TITLE: &str = "settings-general-notes-title";
pub(crate) const GENERAL_NOTES_ABOUT: &str = "settings-general-notes-about";
pub(crate) const GENERAL_UPDATES_TITLE: &str = "settings-general-updates-title";
pub(crate) const GENERAL_UPDATES_ABOUT: &str = "settings-general-updates-about";
pub(crate) const GENERAL_UPDATES_HINT: &str = "settings-general-updates-hint";
pub(crate) const GENERAL_INTERFACE_SIZE_TITLE: &str = "settings-general-interface-size-title";
pub(crate) const GENERAL_INTERFACE_SIZE_ABOUT: &str = "settings-general-interface-size-about";
pub(crate) const GENERAL_INTERFACE_SIZE_LIMITED: &str = "settings-general-interface-size-limited";
pub(crate) const GENERAL_INTERFACE_SIZE_HINT: &str = "settings-general-interface-size-hint";
pub(crate) const GENERAL_CONTROLLER_TITLE: &str = "settings-general-controller-title";
pub(crate) const GENERAL_CONTROLLER_ABOUT: &str = "settings-general-controller-about";
pub(crate) const GENERAL_CONTROLLER_HINT: &str = "settings-general-controller-hint";
pub(crate) const GENERAL_STEAM_TITLE: &str = "settings-general-steam-title";
pub(crate) const GENERAL_STEAM_ABOUT: &str = "settings-general-steam-about";
pub(crate) const GENERAL_STEAM_HINT: &str = "settings-general-steam-hint";
pub(crate) const GENERAL_STEAM_BUTTON: &str = "settings-general-steam-button";

pub(crate) const DOWNLOADS_GAME_LANGUAGE_TITLE: &str = "settings-downloads-game-language-title";
pub(crate) const DOWNLOADS_SECTION_TITLE: &str = "settings-downloads-section-title";
pub(crate) const DOWNLOADS_GAME_LANGUAGE_ABOUT: &str = "settings-downloads-game-language-about";
pub(crate) const DOWNLOADS_GAME_LANGUAGE_HINT: &str = "settings-downloads-game-language-hint";
pub(crate) const DOWNLOADS_SPEED_LIMIT_TITLE: &str = "settings-downloads-speed-limit-title";
pub(crate) const DOWNLOADS_SPEED_LIMIT_ABOUT: &str = "settings-downloads-speed-limit-about";
pub(crate) const DOWNLOADS_SPEED_LIMIT_UNLIMITED: &str = "settings-downloads-speed-limit-unlimited";
pub(crate) const DOWNLOADS_BATTLENET_LIMIT_TITLE: &str = "settings-downloads-battlenet-limit-title";
pub(crate) const DOWNLOADS_BATTLENET_LIMIT_ABOUT: &str = "settings-downloads-battlenet-limit-about";
pub(crate) const DOWNLOADS_BATTLENET_LIMIT_HINT: &str = "settings-downloads-battlenet-limit-hint";
pub(crate) const DOWNLOADS_BATTLENET_LIMIT_NONE: &str = "settings-downloads-battlenet-limit-none";
pub(crate) const DOWNLOADS_PAUSE_TITLE: &str = "settings-downloads-pause-title";
pub(crate) const DOWNLOADS_PAUSE_ABOUT: &str = "settings-downloads-pause-about";
pub(crate) const DOWNLOADS_PAUSE_HINT: &str = "settings-downloads-pause-hint";

#[cfg(test)]
const MENU_KEYS: [&str; 5] = [
    MENU_SETTINGS,
    MENU_UPDATES,
    MENU_REPORT_BUG,
    MENU_FEEDBACK,
    MENU_EXIT,
];

#[cfg(test)]
const GENERAL_KEYS: [&str; 29] = [
    GENERAL_LANGUAGE_TITLE, GENERAL_LANGUAGE_ABOUT, GENERAL_LANGUAGE_ENGLISH, GENERAL_LANGUAGE_CHINESE,
    GENERAL_MINIMIZE_TITLE, GENERAL_MINIMIZE_ABOUT, GENERAL_MINIMIZE_HINT,
    GENERAL_BATTLENET_MINIMIZED_TITLE, GENERAL_BATTLENET_MINIMIZED_ABOUT, GENERAL_BATTLENET_MINIMIZED_HINT,
    GENERAL_BATTLENET_CLOSE_TITLE, GENERAL_BATTLENET_CLOSE_ABOUT, GENERAL_BATTLENET_CLOSE_HINT,
    GENERAL_NOTES_TITLE, GENERAL_NOTES_ABOUT, GENERAL_UPDATES_TITLE, GENERAL_UPDATES_ABOUT,
    GENERAL_UPDATES_HINT, GENERAL_INTERFACE_SIZE_TITLE, GENERAL_INTERFACE_SIZE_ABOUT,
    GENERAL_INTERFACE_SIZE_LIMITED, GENERAL_INTERFACE_SIZE_HINT, GENERAL_CONTROLLER_TITLE,
    GENERAL_CONTROLLER_ABOUT, GENERAL_CONTROLLER_HINT, GENERAL_STEAM_TITLE, GENERAL_STEAM_ABOUT,
    GENERAL_STEAM_HINT, GENERAL_STEAM_BUTTON,
];

#[cfg(test)]
const DOWNLOADS_KEYS: [&str; 14] = [
    DOWNLOADS_SECTION_TITLE,
    DOWNLOADS_GAME_LANGUAGE_TITLE,
    DOWNLOADS_GAME_LANGUAGE_ABOUT,
    DOWNLOADS_GAME_LANGUAGE_HINT,
    DOWNLOADS_SPEED_LIMIT_TITLE,
    DOWNLOADS_SPEED_LIMIT_ABOUT,
    DOWNLOADS_SPEED_LIMIT_UNLIMITED,
    DOWNLOADS_BATTLENET_LIMIT_TITLE,
    DOWNLOADS_BATTLENET_LIMIT_ABOUT,
    DOWNLOADS_BATTLENET_LIMIT_HINT,
    DOWNLOADS_BATTLENET_LIMIT_NONE,
    DOWNLOADS_PAUSE_TITLE,
    DOWNLOADS_PAUSE_ABOUT,
    DOWNLOADS_PAUSE_HINT,
];

pub(crate) fn supported_language(language: &str) -> bool {
    matches!(language, ENGLISH | CHINESE)
}

/// Format a stable message id in the selected locale. Missing messages and
/// messages without values fall back to English, then to the id itself.
pub(crate) fn text(language: &str, id: &str) -> String {
    text_with_args(language, id, &[])
}

/// Mark a fixed message produced below the GUI layer for localization where
/// it is rendered. Dynamic system errors remain unmarked and unchanged.
pub(crate) fn error_marker(id: &str) -> String {
    format!("\u{1f}{id}\u{1f}")
}

pub(crate) fn error_text(language: &str, message: &str) -> String {
    if let Some(id) = message.strip_prefix('\u{1f}').and_then(|rest| rest.strip_suffix('\u{1f}')) {
        text(language, id)
    } else {
        message.to_owned()
    }
}

/// Translate the fixed shell of a prefix migration error at the GUI boundary;
/// paths, filesystem reasons, and operating-system details stay verbatim.
pub(crate) fn prefix_move_error(language: &str, error: &lightforge_core::prefix_move::PrefixMoveError) -> String {
    use lightforge_core::prefix_move::PrefixMoveError as E;
    match error {
        E::AlreadyThere(path) => text_with_args(language, "settings-storage-prefix-already-there", &[("path", &path.display().to_string())]),
        E::Away(path) => text_with_args(language, "settings-storage-prefix-missing", &[("path", &path.display().to_string())]),
        E::Busy(programs) => text_with_args(language, "settings-storage-prefix-busy", &[("programs", &programs.join(", "))]),
        E::Occupied(path) => text_with_args(language, "settings-storage-prefix-occupied", &[("path", &path.display().to_string())]),
        E::Overlap { target, prefix } => text_with_args(language, "settings-storage-prefix-overlap", &[("target", &target.display().to_string()), ("prefix", &prefix.display().to_string())]),
        E::LibraryInside(path) => text_with_args(language, "settings-storage-prefix-library-inside", &[("path", &path.display().to_string())]),
        E::Unfit { path, reason } => text_with_args(language, "settings-storage-prefix-unfit", &[("path", &path.display().to_string()), ("reason", reason)]),
        E::NoSpace { path, needed, free } => text_with_args(language, "settings-storage-prefix-no-space", &[("path", &path.display().to_string()), ("needed", &crate::format::size_of(*needed)), ("free", &crate::format::size_of(*free))]),
        E::Mismatch(path) => text_with_args(language, "settings-storage-prefix-mismatch", &[("path", &path.display().to_string())]),
        E::Io { path, source } => text_with_args(language, "settings-storage-prefix-io-error", &[("path", &path.display().to_string()), ("error", &source.to_string())]),
    }
}

/// Keep nested download and Proton details intact while translating the
/// fixed repair refusals and release-selection errors.
pub(crate) fn repair_error(language: &str, error: &lightforge_core::setup::SetupError) -> String {
    use lightforge_core::setup::SetupError as E;
    match error {
        E::NoSuchRelease(version) => text_with_args(language, "settings-setup-no-such-release", &[("version", version)]),
        E::NoStableRelease => crate::i18n::text(language, "settings-setup-no-stable-release"),
        E::BattleNetDeclined => crate::i18n::text(language, "settings-setup-battlenet-declined"),
        E::Busy(programs) => text_with_args(language, "settings-setup-busy", &[("programs", &programs.join(", "))]),
        other => other.to_string(),
    }
}

/// Format a Fluent message with string-valued named arguments.
pub(crate) fn text_with_args(language: &str, id: &str, args: &[(&str, &str)]) -> String {
    let selected_locale = match language {
        CHINESE => CHINESE,
        _ => ENGLISH,
    };
    format_cached_resources(selected_locale, id, args)
}

pub(crate) fn library_state_reason(
    language: &str,
    state: lightforge_core::library::State,
    label: &str,
) -> Option<String> {
    let id = match state {
        lightforge_core::library::State::Missing => "storage-library-not-mounted",
        lightforge_core::library::State::Foreign => "storage-library-foreign",
        lightforge_core::library::State::Ready | lightforge_core::library::State::New => return None,
    };
    Some(text_with_args(language, id, &[("label", label)]))
}

pub(crate) fn library_error(language: &str, error: &lightforge_core::library::LibraryError) -> String {
    use lightforge_core::library::LibraryError;
    match error {
        LibraryError::Offline { label } => text_with_args(language, "storage-drive-not-mounted", &[("drive", label)]),
        LibraryError::Foreign { label } => text_with_args(language, "storage-drive-foreign", &[("drive", label)]),
        LibraryError::NoLettersLeft => text(language, "storage-no-drive-letters-left"),
        // Keep the system error and path intact; only the shell is translated.
        LibraryError::Io { path, source } => text_with_args(
            language,
            "storage-drive-io-error",
            &[("path", &path.display().to_string()), ("error", &source.to_string())],
        ),
    }
}

pub(crate) fn setup_step_label(language: &str, step: lightforge_core::setup::Step) -> String {
    let id = match step {
        lightforge_core::setup::Step::Proton => "setup-step-proton",
        lightforge_core::setup::Step::Prefix => "setup-step-prefix",
        lightforge_core::setup::Step::BattleNet => "setup-step-battlenet",
    };
    text(language, id)
}

pub(crate) fn setup_step_detail(language: &str, step: lightforge_core::setup::Step) -> String {
    let id = match step {
        lightforge_core::setup::Step::Proton => "setup-detail-proton",
        lightforge_core::setup::Step::Prefix => "setup-detail-prefix",
        lightforge_core::setup::Step::BattleNet => "setup-detail-battlenet",
    };
    text(language, id)
}

pub(crate) fn doctor_status(language: &str, status: lightforge_core::doctor::Status) -> String {
    let id = match status {
        lightforge_core::doctor::Status::Ok => "settings-doctor-status-ok",
        lightforge_core::doctor::Status::Warn => "settings-doctor-status-warn",
        lightforge_core::doctor::Status::Fail => "settings-doctor-status-fail",
    };
    text(language, id)
}

pub(crate) fn doctor_name(language: &str, name: &str) -> String {
    if let Some(label) = name.strip_prefix("space on ") {
        return text_with_args(language, "settings-doctor-space-on", &[("label", label)]);
    }
    let id = match name {
        "open files" => "settings-doctor-open-files",
        "ntsync" => "settings-doctor-ntsync",
        "vulkan" => "settings-doctor-vulkan",
        "32-bit runtime" => "settings-doctor-runtime",
        "python3" => "settings-doctor-python",
        "32-bit gnutls" => "settings-doctor-gnutls",
        "proton" => "settings-doctor-proton",
        "prefix" => "settings-doctor-prefix",
        "nvidia driver" => "settings-doctor-nvidia-driver",
        "nvidia modeset" => "settings-doctor-nvidia-modeset",
        "nvidia 32-bit gl" => "settings-doctor-nvidia-gl",
        other => return other.to_owned(),
    };
    text(language, id)
}

pub(crate) fn doctor_detail(language: &str, check: &lightforge_core::doctor::Check, value: &str) -> String {
    let id = match value {
        "available" => Some("settings-doctor-available"),
        "not available, Proton uses fsync" => Some("settings-doctor-fsync"),
        "not needed on this processor" => Some("settings-doctor-not-needed"),
        "present" => Some("settings-doctor-present"),
        "no Vulkan driver found" => Some("settings-doctor-no-vulkan"),
        "no 32-bit loader" => Some("settings-doctor-no-loader"),
        "missing" => Some("settings-doctor-missing"),
        "unknown, ldconfig could not be read" => Some("settings-doctor-unknown-ldconfig"),
        "not set up yet" => Some("settings-doctor-not-set-up"),
        "on" => Some("settings-doctor-on"),
        "off" => Some("settings-doctor-off"),
        "nvidia_drm is not loaded" => Some("settings-doctor-nvidia-not-loaded"),
        _ => None,
    };
    if let Some(id) = id {
        return text(language, id);
    }
    if value == "could not read /proc/self/limits" {
        return text(language, "settings-doctor-cannot-read-limits");
    }
    if let Some(path) = value.strip_suffix(" could not be read") {
        return text_with_args(language, "settings-doctor-path-unreadable", &[("path", path)]);
    }
    if value == "no supported Proton build" {
        return text(language, "settings-doctor-no-proton");
    }
    if let Some(pin) = value.strip_prefix("pinned ").and_then(|rest| rest.strip_suffix(" is not installed")) {
        return text_with_args(language, "settings-doctor-pinned-missing", &[("version", pin)]);
    }
    if let Some((build, why)) = value.strip_prefix("built by ").and_then(|rest| rest.split_once(", game link ")) {
        return text_with_args(language, "settings-doctor-prefix-detail", &[("build", build), ("link", why)]);
    }
    if let Some(rest) = value.strip_prefix("soft ") {
        if let Some((soft, hard)) = rest.split_once(", hard ") {
            return text_with_args(language, "settings-doctor-open-limits", &[("soft", soft), ("hard", hard)]);
        }
    }
    if let Some(drivers) = value.strip_prefix("set by ").and_then(|rest| rest.split_once(": ")) {
        return text_with_args(language, "settings-doctor-vulkan-override", &[("variable", drivers.0), ("drivers", drivers.1)]);
    }
    if check.name.starts_with("space on ") {
        if let Some((amount, path)) = value.split_once(" at ") {
            if let Some((free, total)) = amount.split_once(" free of ") {
                return text_with_args(language, "settings-doctor-space", &[("free", free), ("total", total), ("path", path)]);
            }
        }
    }
    if let Some(drive) = value.strip_suffix(" is still not mounted") {
        return text_with_args(language, "settings-doctor-drive-unmounted", &[("drive", drive)]);
    }
    value.to_owned()
}

pub(crate) fn doctor_hint(language: &str, value: &str) -> String {
    if let Some(rest) = value.strip_prefix("esync needs a hard limit of ") {
        if let Some((limit, config)) = rest.split_once(": set DefaultLimitNOFILE in systemd's ") {
            return text_with_args(language, "settings-doctor-esync-hint", &[("limit", limit), ("config", config)]);
        }
    }
    if let Some(remedy) = value.strip_prefix("Battle.net is 32-bit, the games are not; ") {
        if let Some((distro, remedy)) = distro_remedy(remedy) {
            return text_with_args(language, "settings-doctor-runtime-hint-distro", &[("distro", distro), ("remedy", remedy)]);
        }
        return text_with_args(language, "settings-doctor-runtime-hint", &[("remedy", remedy)]);
    }
    if let Some(version) = value.strip_prefix("Lightforge is tested with ") {
        return text_with_args(language, "settings-doctor-tested-with", &[("version", version)]);
    }
    if let Some(remedy) = value.strip_prefix("Battle.net is 32-bit and needs the driver's 32-bit libraries") {
        if let Some((distro, remedy)) = remedy.strip_prefix("; ").and_then(distro_remedy) {
            return text_with_args(language, "settings-doctor-nvidia-gl-hint-distro", &[("distro", distro), ("remedy", remedy)]);
        }
        return text_with_args(language, "settings-doctor-nvidia-gl-hint", &[("remedy", remedy)]);
    }
    match value {
        "a patch can need 15 GiB of room" => text(language, "settings-doctor-patch-room"),
        "ntsync needs kernel 6.14 or newer with the ntsync module loaded" => text(language, "settings-doctor-ntsync-hint"),
        "install your GPU's Vulkan driver; DXVK and VKD3D cannot run without one" => text(language, "settings-doctor-vulkan-hint"),
        "Proton's launcher is a Python script; install your distribution's python3" => text(language, "settings-doctor-python-hint"),
        "Battle.net then runs inside the Steam Linux Runtime, which Steam has to have installed" => text(language, "settings-doctor-gnutls-hint"),
        "install it under Settings, Proton, or remove the pin" => text(language, "settings-doctor-proton-pin-hint"),
        "run setup from the app, or `lightforge setup`" => text(language, "settings-doctor-prefix-setup-hint"),
        "`lightforge prefix-link` repairs the link" => text(language, "settings-doctor-prefix-link-hint"),
        "Wayland needs 555 or newer for explicit sync; see TROUBLESHOOTING.md" => text(language, "settings-doctor-nvidia-wayland-hint"),
        "Wayland and gamescope need nvidia_drm.modeset=1 on the kernel command line" => text(language, "settings-doctor-nvidia-modeset-hint"),
        _ => value.to_owned(),
    }
}

pub(crate) fn maintenance_label(language: &str, what: &str) -> String {
    let id = match what {
        "the scan" => "maintenance-scan",
        "the repair" => "maintenance-repair",
        "freeing unused game data" => "maintenance-freeing-data",
        "the uninstall" => "maintenance-uninstall",
        "removing the cancelled download" => "maintenance-removing-download",
        "moving a game" => "maintenance-moving-game",
        "the download" => "maintenance-download",
        _ => return what.to_owned(),
    };
    text(language, id)
}

/// Translate the worker's stable English stage description where it reaches the GUI.
/// Values such as paths, versions, addon names, and provider reasons pass through unchanged.
pub(crate) fn transfer_stage(language: &str, stage: &str) -> String {
    let with = |id, args: &[(&str, &str)]| text_with_args(language, id, args);
    let count = |id, value: &str| with(id, &[("count", value)]);
    if stage == "queued" { return text(language, "download-queued"); }
    if stage == "preparing" { return text(language, "download-preparing"); }
    if stage == "waiting for Battle.net to close" { return crate::i18n::text(language, "transfer-waiting-for-battlenet"); }
    if let Some(value) = stage.strip_prefix("build ") { return with("transfer-build", &[("version", value)]); }
    if let Some(value) = stage.strip_prefix("encoding — ").and_then(|s| s.strip_suffix(" entries")) { return count("transfer-encoding", value); }
    if let Some(value) = stage.strip_prefix("manifests — ").and_then(|s| s.strip_suffix(" entries")) { return count("transfer-manifests", value); }
    if let Some(value) = stage.strip_prefix("checking ").and_then(|s| s.strip_suffix(" files against the store")) { return count("transfer-checking-files", value); }
    if stage == "no optional content offered" { return text(language, "transfer-no-optional-content"); }
    if let Some(value) = stage.strip_prefix("optional content available: ") { return with("transfer-optional-content", &[("content", value)]); }
    if let Some(value) = stage.strip_prefix("archive indices ") {
        if value.contains('/') { return with("transfer-archive-indices", &[("progress", value)]); }
    }
    if let Some(value) = stage.strip_prefix("indices ready — ") {
        if let Some((cached, fetched)) = value.split_once(" cached, ").and_then(|(a,b)| b.strip_suffix(" fetched").map(|b| (a,b))) {
            return with("transfer-indices-ready", &[("cached", cached), ("fetched", fetched)]);
        }
    }
    if let Some(value) = stage.strip_suffix(" loose files") { return count("transfer-loose-files", value); }
    if let Some(value) = stage.strip_prefix("located ") {
        if let Some((done, total)) = value.split_once('/') { return with("transfer-located", &[("done", done), ("total", total)]); }
    }
    if let Some(value) = stage.strip_prefix("planned ") {
        if let Some((requests, size)) = value.split_once(" requests, ").and_then(|(a,b)| b.strip_suffix(" GiB").map(|b| (a,b))) {
            return with("transfer-planned", &[("requests", requests), ("size", size)]);
        }
    }
    if let Some(path) = stage.strip_prefix("extracting ") { return with("transfer-extracting", &[("path", path)]); }
    if let Some(value) = stage.strip_prefix("extracted ").and_then(|s| s.strip_suffix(" install files")) { return count("transfer-extracted-files", value); }
    if let Some(version) = stage.strip_prefix("finished — ") { return with("transfer-finished", &[("version", version)]); }
    if let Some(version) = stage.strip_prefix("pre-downloaded ") { return with("transfer-pre-downloaded", &[("version", version)]); }
    if stage == "writing .product.db" { return text(language, "transfer-writing-product-db"); }
    if stage == "registered with Battle.net" { return text(language, "transfer-registered"); }
    if let Some(reasons) = stage.strip_prefix("incomplete: ") { return with("transfer-incomplete", &[("reasons", reasons)]); }
    if let Some(detail) = stage.strip_suffix(" would not take a file; added owner write") {
        return with("transfer-repaired-directory", &[("path", detail)]);
    }
    stage.to_owned()
}

/// Translate the addon worker's displayed progress line without changing addon identifiers.
pub(crate) fn addon_progress_line(language: &str, line: &str) -> String {
    let with = |id, args: &[(&str, &str)]| text_with_args(language, id, args);
    if let Some(count) = line.strip_prefix("resolving ").and_then(|s| s.strip_suffix(" addons…")) { return with("addon-progress-resolving", &[("count", count)]); }
    if let Some((ok, unavailable)) = line.split_once(" available, ").and_then(|(a,b)| b.strip_suffix(" not").map(|b| (a,b))) {
        return with("addon-progress-resolved", &[("available", ok), ("unavailable", unavailable)]);
    }
    if let Some(name) = line.strip_prefix("fetching ").and_then(|s| s.strip_suffix('…')) { return with("addon-progress-fetching", &[("name", name)]); }
    if let Some(name) = line.strip_prefix("installed ") { return with("addon-progress-installed", &[("name", name)]); }
    if let Some(name) = line.strip_suffix(" is up to date") { return with("addon-progress-up-to-date", &[("name", name)]); }
    if let Some(name) = line.strip_suffix(" unavailable") { return with("addon-progress-unavailable", &[("name", name)]); }
    line.to_owned()
}

pub(crate) fn transfer_error(language: &str, message: &str) -> String {
    if message.starts_with('\u{1f}') && message.ends_with('\u{1f}') {
        return error_text(language, message);
    }
    if let Some(game) = message.strip_prefix("close ").and_then(|s| s.strip_suffix(" first — the game files are in use")) {
        return text_with_args(language, "transfer-close-game-first", &[("game", game)]);
    }
    message.to_owned()
}

fn distro_remedy(value: &str) -> Option<(&str, &str)> {
    value.strip_prefix("on ")?.split_once(": ")
}

#[cfg(test)]
fn format_from_resources(
    selected: &str,
    selected_locale: &str,
    english: &str,
    id: &str,
    args: &[(&str, &str)],
) -> String {
    format_resource(selected, selected_locale, id, args).unwrap_or_else(|| {
        format_resource(english, ENGLISH, id, args).unwrap_or_else(|| id.to_owned())
    })
}

thread_local! {
    static BUNDLES: RefCell<HashMap<&'static str, FluentBundle<FluentResource>>> =
        RefCell::new(HashMap::new());

    #[cfg(test)]
    static BUNDLE_INIT_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

fn build_cached_bundle(locale: &'static str) -> FluentBundle<FluentResource> {
    let source = match locale {
        CHINESE => ZH_CN,
        ENGLISH => EN_US,
        _ => unreachable!("only built-in locales are cached"),
    };
    build_bundle(source, locale)
}

fn build_bundle(source: &str, locale: &str) -> FluentBundle<FluentResource> {
    let resource = match FluentResource::try_new(source.to_owned()) {
        Ok(resource) => resource,
        Err((resource, errors)) => {
            tracing::warn!(?errors, "Fluent resource contains parse errors");
            resource
        }
    };
    let locale: LanguageIdentifier = locale.parse().expect("valid built-in locale");
    let mut bundle = FluentBundle::new(vec![locale]);
    if let Err(error) = bundle.add_resource(resource) {
        tracing::warn!(?error, "could not add Fluent resource to bundle");
    }
    bundle
}

fn format_cached_resources(locale: &'static str, id: &str, args: &[(&str, &str)]) -> String {
    if let Some(formatted) = format_cached_resource(locale, id, args) {
        return formatted;
    }
    if locale != ENGLISH {
        if let Some(formatted) = format_cached_resource(ENGLISH, id, args) {
            return formatted;
        }
    }
    id.to_owned()
}

fn format_cached_resource(
    locale: &'static str,
    id: &str,
    args: &[(&str, &str)],
) -> Option<String> {
    BUNDLES.with(|bundles| {
        let mut bundles = bundles.borrow_mut();
        let bundle = match bundles.entry(locale) {
            std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                #[cfg(test)]
                BUNDLE_INIT_COUNT.with(|count| count.set(count.get() + 1));
                entry.insert(build_cached_bundle(locale))
            }
        };
        // No Fluent references leave the thread-local cache borrow.
        format_bundle(bundle, id, args)
    })
}

#[cfg(test)]
fn format_resource(source: &str, locale: &str, id: &str, args: &[(&str, &str)]) -> Option<String> {
    let mut bundle = build_bundle(source, locale);
    format_bundle(&mut bundle, id, args)
}

fn format_bundle(
    bundle: &mut FluentBundle<FluentResource>,
    id: &str,
    args: &[(&str, &str)],
) -> Option<String> {
    let message = bundle.get_message(id)?;
    let pattern = message.value()?;
    let mut fluent_args = FluentArgs::new();
    for (name, value) in args {
        let value = match (*name, value.parse::<f64>()) {
            ("count", Ok(number)) => FluentValue::from(number),
            _ => FluentValue::from(*value),
        };
        fluent_args.set(*name, value);
    }
    let mut errors = Vec::new();
    let formatted = bundle.format_pattern(pattern, Some(&fluent_args), &mut errors);
    if !errors.is_empty() {
        tracing::warn!(message_id = id, ?errors, "Fluent message formatting reported errors");
    }
    Some(formatted.into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_languages_are_explicit_and_unknown_values_use_english() {
        assert!(supported_language(ENGLISH));
        assert!(supported_language(CHINESE));
        assert!(!supported_language("fr-FR"));
        assert_eq!(text("fr-FR", MENU_SETTINGS), "Settings");
    }

    #[test]
    fn repeated_lookups_reuse_bundles_across_languages_and_fallback() {
        // Warm both locale bundles before observing the counter so parallel tests
        // cannot make this assertion depend on test ordering.
        assert_eq!(text(ENGLISH, MENU_SETTINGS), "Settings");
        assert_eq!(text(CHINESE, MENU_SETTINGS), "设置");
        let initialized = BUNDLE_INIT_COUNT.with(std::cell::Cell::get);

        for _ in 0..100 {
            assert_eq!(text(ENGLISH, MENU_SETTINGS), "Settings");
            assert_eq!(text(CHINESE, MENU_SETTINGS), "设置");
            assert_eq!(text("fr-FR", MENU_SETTINGS), "Settings");
            assert_eq!(
                text_with_args(CHINESE, "header-more-games", &[("count", "2")]),
                "还有 \u{2068}2\u{2069} 款游戏"
            );
        }
        assert_eq!(
            BUNDLE_INIT_COUNT.with(std::cell::Cell::get),
            initialized,
            "ordinary lookups must not reconstruct Fluent bundles"
        );

        let chinese_without_exit = ZH_CN.replace("app-menu-exit = 退出", "");
        assert_eq!(
            format_from_resources(&chinese_without_exit, CHINESE, EN_US, MENU_EXIT, &[]),
            "Exit",
            "missing Chinese messages must retain the English fallback"
        );
    }

    #[test]
    fn settings_navigation_is_localized_without_changing_tab_identity() {
        use crate::model::SettingsTab;

        assert_eq!(SettingsTab::General.label(CHINESE), "常规");
        assert!(!SettingsTab::General.label(CHINESE).contains("General"));
        assert_eq!(SettingsTab::General.label(ENGLISH), "General");
        assert_eq!(SettingsTab::Downloads.label(CHINESE), "下载");
        assert_eq!(SettingsTab::General.anchor(), "settings General");
    }

    #[test]
    fn visible_transfer_stages_localize_copy_and_preserve_values() {
        assert_eq!(transfer_stage(CHINESE, "queued"), "已排队…");
        assert_eq!(transfer_stage(CHINESE, "preparing"), "正在准备…");
        assert!(transfer_stage(CHINESE, "checking 12 files against the store").contains("12"));
        assert!(transfer_stage(CHINESE, "archive indices 2/7").contains("2/7"));
        assert!(transfer_stage(CHINESE, "extracting /Games/WoW/Data/file.idx").contains("/Games/WoW/Data/file.idx"));
        assert!(transfer_stage(CHINESE, "finished — 12.0.1.12345").contains("12.0.1.12345"));
        assert!(transfer_stage(CHINESE, "pre-downloaded 12.0.1.12345").contains("12.0.1.12345"));
        assert!(transfer_stage(CHINESE, "incomplete: CDN timed out").contains("CDN timed out"));
        let english = transfer_stage(ENGLISH, "encoding — 1 entries");
        let english = english.replace(['\u{2068}', '\u{2069}'], "");
        assert_eq!(english, "encoding — 1 entries");
    }

    #[test]
    fn addon_progress_localizes_states_without_changing_addon_names() {
        assert!(addon_progress_line(CHINESE, "resolving 3 addons…").contains("3"));
        assert!(addon_progress_line(CHINESE, "fetching BigWigs…").contains("BigWigs"));
        assert!(addon_progress_line(CHINESE, "BigWigs is up to date").contains("BigWigs"));
        assert!(addon_progress_line(CHINESE, "WeakAuras unavailable").contains("WeakAuras"));
    }

    #[test]
    fn fixed_network_and_transfer_errors_localize_but_keep_original_values() {
        let marker = error_marker("error-no-network-connection");
        assert_eq!(error_text(CHINESE, &marker), "无网络连接");
        assert_eq!(transfer_error(CHINESE, &error_marker("transfer-no-cdn-for-region")), "版本服务中没有此区域可用的 CDN");
        assert!(text_with_args(CHINESE, "error-could-not-start-updated-lightforge", &[("error", "permission denied")]).contains("permission denied"));
        assert!(transfer_error(CHINESE, "close World of Warcraft first — the game files are in use").contains("World of Warcraft"));
        assert_eq!(text(CHINESE, "settings-nav-general"), "常规");
        for key in [
            "upgrade-copy-cannot-update-itself",
            "transfer-thread-stopped-unexpectedly",
            "transfer-no-build-for-region",
            "transfer-no-cdn-for-region",
        ] {
            assert_ne!(text(CHINESE, key), text(ENGLISH, key));
        }
        assert!(text_with_args(CHINESE, "game-not-managed", &[("game", "WoW")]).contains("WoW"));
        assert!(text_with_args(CHINESE, "storage-drive-io-error", &[("path", "/mnt/games"), ("error", "Permission denied")]).contains("Permission denied"));
    }

    #[test]
    fn gui_localizes_core_library_reasons_and_setup_progress_copy() {
        use lightforge_core::{library::State, setup::Step};

        assert_eq!(
            library_state_reason(CHINESE, State::Missing, "游戏盘"),
            Some("\u{2068}游戏盘\u{2069} 未挂载".to_owned())
        );
        assert_eq!(
            library_state_reason(CHINESE, State::Foreign, "游戏盘"),
            Some("\u{2068}游戏盘\u{2069} 中有其他 Lightforge 库".to_owned())
        );
        assert_eq!(library_state_reason(CHINESE, State::Ready, "游戏盘"), None);
        assert_eq!(setup_step_label(CHINESE, Step::Prefix), "Wine 前缀");
        assert_eq!(setup_step_detail(CHINESE, Step::BattleNet), "正在将 Battle.net 安装到前缀中");
        assert_eq!(
            text_with_args(CHINESE, "setup-step-finished", &[("step", "Proton-GE"), ("note", "/games/wow")]),
            "\u{2068}Proton-GE\u{2069} — \u{2068}/games/wow\u{2069}"
        );
        assert_eq!(
            text_with_args(CHINESE, "header-more-games", &[("count", "1")]),
            "还有 \u{2068}1\u{2069} 款游戏"
        );
        assert_eq!(
            text_with_args(ENGLISH, "header-more-games", &[("count", "2")]),
            "\u{2068}2\u{2069} more \u{2068}games\u{2069}"
        );
        assert_eq!(text(CHINESE, "report-choose-screenshot"), "选择屏幕截图");
        assert_eq!(text(CHINESE, "report-image-filter"), "图片");
    }

    #[test]
    fn gui_localizes_doctor_copy_while_preserving_system_values() {
        use lightforge_core::doctor::{Check, Status};

        assert_eq!(doctor_name(CHINESE, "space on Games SSD"), "磁盘空间：\u{2068}Games SSD\u{2069}");

        let check = Check {
            name: "proton".to_owned(),
            status: Status::Fail,
            detail: "pinned GE-Proton9-25 is not installed".to_owned(),
            hint: None,
        };
        assert_eq!(doctor_detail(CHINESE, &check, &check.detail), "固定使用的 \u{2068}GE-Proton9-25\u{2069} 尚未安装");
        let runtime_hint = doctor_hint(CHINESE, "Battle.net is 32-bit, the games are not; on Fedora: dnf install glibc.i686");
        assert!(runtime_hint.contains("在 \u{2068}Fedora\u{2069} 上"));
        assert!(runtime_hint.contains("dnf install glibc.i686"));
        assert_eq!(doctor_detail(CHINESE, &check, "/run/user/1000/lightforge could not be read"), "无法读取 \u{2068}/run/user/1000/lightforge\u{2069}");
        assert_eq!(lightforge_core::doctor::render(&[check]), "fail  proton              pinned GE-Proton9-25 is not installed\n");
    }

    #[test]
    fn remaining_gui_state_and_report_messages_are_localized() {
        assert_eq!(text(CHINESE, crate::report::Category::Game.message_id()), "游戏");
        assert_eq!(text(CHINESE, "steam-shortcut-added"), "下次启动 Steam 时，Lightforge 会出现在你的 Steam 库中。");
        assert_eq!(text(CHINESE, "status-version-service-offline"), "版本服务无法连接 — 离线");
        assert_eq!(text(CHINESE, "error-only-game-left"), "这是仅剩的游戏，请先添加另一款游戏");
        assert_eq!(text_with_args(CHINESE, "backup-restored-account", &[("account", "Rhea"), ("when", "2026-11-03")]), "已从 \u{2068}2026-11-03\u{2069} 的备份还原账户 \u{2068}Rhea\u{2069}。");
        assert_eq!(text_with_args(CHINESE, "report-library-free", &[("free", "8 GiB"), ("total", "20 GiB")]), "剩余 \u{2068}8 GiB\u{2069} / \u{2068}20 GiB\u{2069}");
        assert_eq!(text_with_args(CHINESE, "launch-battlenet-install-prompt", &[("game", "World of Warcraft")]), "Battle.net 正在询问要将 \u{2068}World of Warcraft\u{2069} 安装到哪里");
        assert_eq!(text_with_args(CHINESE, "proton-removed", &[("version", "GE-Proton9-25")]), "已移除 \u{2068}GE-Proton9-25\u{2069}");
        assert_eq!(text(CHINESE, "wizard-choose-prefix-or-game-folder"), "选择 Wine 前缀或游戏文件夹");
    }

    #[test]
    fn fluent_resources_cover_the_menu_and_missing_chinese_entries_fall_back_to_english() {
        for id in MENU_KEYS {
            assert!(format_resource(EN_US, ENGLISH, id, &[]).is_some(), "English resource is missing {id}");
            assert!(format_resource(ZH_CN, CHINESE, id, &[]).is_some(), "Chinese resource is missing {id}");
        }
        assert_eq!(text(CHINESE, MENU_SETTINGS), "设置");
        let chinese_without_exit = ZH_CN.replace("app-menu-exit = 退出", "");
        assert_eq!(
            format_from_resources(&chinese_without_exit, CHINESE, EN_US, MENU_EXIT, &[]),
            "Exit"
        );
    }

    #[test]
    fn fluent_resources_cover_settings_general() {
        for id in GENERAL_KEYS {
            assert!(format_resource(EN_US, ENGLISH, id, &[]).is_some(), "English resource is missing {id}");
            assert!(format_resource(ZH_CN, CHINESE, id, &[]).is_some(), "Chinese resource is missing {id}");
        }
        assert_eq!(
            text_with_args(CHINESE, GENERAL_INTERFACE_SIZE_LIMITED, &[("percent", "125")]),
            "当前屏幕上的上限为 \u{2068}125\u{2069}%。"
        );
    }

    #[test]
    fn fluent_resources_cover_settings_downloads() {
        for id in DOWNLOADS_KEYS {
            assert!(format_resource(EN_US, ENGLISH, id, &[]).is_some(), "English resource is missing {id}");
            assert!(format_resource(ZH_CN, CHINESE, id, &[]).is_some(), "Chinese resource is missing {id}");
        }
        assert_eq!(text(CHINESE, DOWNLOADS_SPEED_LIMIT_UNLIMITED), "不限");
        assert_eq!(text(CHINESE, DOWNLOADS_BATTLENET_LIMIT_NONE), "默认");
    }

    #[test]
    fn formats_variables_with_fluent_interpolation() {
        let resource = "welcome = Welcome, { $name }!";
        assert_eq!(
            format_resource(resource, ENGLISH, "welcome", &[("name", "Ada")]),
            Some("Welcome, \u{2068}Ada\u{2069}!".to_owned())
        );
    }

    #[test]
    fn malformed_ftl_and_missing_variables_produce_diagnostics() {
        let malformed = "broken = {";
        let Err((_, parse_errors)) = FluentResource::try_new(malformed.to_owned()) else {
            panic!("malformed FTL should report parser diagnostics");
        };
        assert!(!parse_errors.is_empty());

        let resource = FluentResource::try_new("welcome = Welcome, { $name }!".to_owned())
            .expect("valid FTL");
        let mut bundle = FluentBundle::new(vec![ENGLISH.parse().expect("valid locale")]);
        bundle.add_resource(resource).expect("resource can be added");
        let message = bundle.get_message("welcome").expect("message exists");
        let mut errors = Vec::new();
        let output = bundle.format_pattern(
            message.value().expect("message has a value"),
            None,
            &mut errors,
        );
        assert_eq!(output, "Welcome, \u{2068}{$name}\u{2069}!");
        assert!(!errors.is_empty());
    }
}
