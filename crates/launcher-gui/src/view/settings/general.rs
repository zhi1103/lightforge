//! General, Addons, Battle.net, Downloads and About.

use crate::*;
use iced::widget::column;

#[derive(Clone, PartialEq)]
struct LanguageChoice {
    code: &'static str,
    label: String,
}

impl std::fmt::Display for LanguageChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

impl Lightforge {
    /// The Addons section: what every addon follows unless it says otherwise.
    pub(crate) fn addons_section(&self) -> Element<'_, Message> {
        let language = &self.config.interface_language;
        let choices: Vec<DisplayChoice<ChannelChoice>> = Channel::ALL
            .map(ChannelChoice)
            .map(|choice| DisplayChoice(choice, choice.label(language)))
            .to_vec();

        setting_group([
            setting(
                i18n::text(language, "settings-addons-channel-title"),
                i18n::text(language, "settings-addons-channel-about"),
                self.hint(
                    i18n::text(language, "settings-addons-channel-hint"),
                ),
                self.picker(
                    "addon channel".to_owned(),
                    choices,
                    Some(DisplayChoice(ChannelChoice(self.config.addon_channel), ChannelChoice(self.config.addon_channel).label(language))),
                    "",
                    |choice| Message::AddonChannelDefault(choice.0),
                    // The Proton pickers' size, so every dropdown on the page
                    // is one width and one text size.
                    Length::Fixed(260.0),
                    theme::BODY,
                ),
            ),
            setting(
                i18n::text(language, "settings-addons-auto-title"),
                i18n::text(language, "settings-addons-auto-about"),
                self.hint(
                    i18n::text(language, "settings-addons-auto-hint"),
                ),
                switch("addon auto update", self.config.addon_auto_update, Message::AddonAutoUpdateDefault),
            ),
        ])
    }

    /// The interface size, for a desktop window only: fullscreen has nothing
    /// to grow, and gamescope scales by the screen's height.
    fn interface_size_row(&self) -> Option<Element<'_, Message>> {
        if self.console() || fit::owned() {
            return None;
        }
        let chosen = self.config.interface_size;
        #[allow(clippy::cast_possible_truncation, reason = "0.5 to 2, see fit::desktop_scale")]
        let shown = (self.ui_scale * 100.0).round() as u16;
        let language = &self.config.interface_language;
        let about = match shown < chosen {
            true => i18n::text_with_args(
                language,
                i18n::GENERAL_INTERFACE_SIZE_LIMITED,
                &[("percent", &shown.to_string())],
            ),
            false => i18n::text(language, i18n::GENERAL_INTERFACE_SIZE_ABOUT),
        };
        let choices: Vec<InterfaceSizeChoice> =
            fit::INTERFACE_SIZES.into_iter().map(InterfaceSizeChoice).collect();
        Some(setting(
            i18n::text(language, i18n::GENERAL_INTERFACE_SIZE_TITLE),
            about,
            self.hint(
                i18n::text(language, i18n::GENERAL_INTERFACE_SIZE_HINT),
            ),
            self.picker(
                "interface size".to_owned(),
                choices,
                Some(InterfaceSizeChoice(chosen)),
                "",
                Message::InterfaceSize,
                Length::Fixed(260.0),
                theme::BODY,
            ),
        ))
    }

    /// What the launcher does around a launch: the settings that are about
    /// the app itself, not about a product, a transfer or a prefix.
    pub(crate) fn general_section(&self) -> Element<'_, Message> {
        let language = &self.config.interface_language;
        let language_choices = vec![
            LanguageChoice {
                code: i18n::ENGLISH,
                label: i18n::text(language, i18n::GENERAL_LANGUAGE_ENGLISH),
            },
            LanguageChoice {
                code: i18n::CHINESE,
                label: i18n::text(language, i18n::GENERAL_LANGUAGE_CHINESE),
            },
        ];
        let shortcut: Option<Element<'_, Message>> = self.steam_shortcut.as_ref().map(|outcome| match outcome {
            Ok(done) => muted(done.clone(), theme::LABEL).into(),
            Err(e) => text(e.clone()).size(theme::LABEL).color(theme::ERROR).into(),
        });

        column![
            setting_group([
                setting(
                    i18n::text(language, i18n::GENERAL_LANGUAGE_TITLE),
                    i18n::text(language, i18n::GENERAL_LANGUAGE_ABOUT),
                    None,
                    self.picker(
                        "interface language".to_owned(),
                        language_choices.clone(),
                        Some(
                            language_choices
                                .iter()
                                .find(|choice| choice.code == self.config.interface_language)
                                .cloned()
                                .unwrap_or_else(|| language_choices[0].clone()),
                        ),
                        "",
                        |choice| Message::InterfaceLanguage(choice.code.to_owned()),
                        Length::Fixed(260.0),
                        theme::BODY,
                    ),
                ),
                setting(
                    i18n::text(language, i18n::GENERAL_MINIMIZE_TITLE),
                    i18n::text(language, i18n::GENERAL_MINIMIZE_ABOUT),
                    self.hint(
                        i18n::text(language, i18n::GENERAL_MINIMIZE_HINT),
                    ),
                    switch("minimize on launch", self.config.minimize_on_launch, Message::MinimizeOnLaunch),
                ),
                setting(
                    i18n::text(language, i18n::GENERAL_BATTLENET_MINIMIZED_TITLE),
                    i18n::text(language, i18n::GENERAL_BATTLENET_MINIMIZED_ABOUT),
                    self.hint(
                        i18n::text(language, i18n::GENERAL_BATTLENET_MINIMIZED_HINT),
                    ),
                    switch("battle.net minimized", self.config.battle_net_minimized, Message::BattleNetMinimized),
                ),
                setting(
                    i18n::text(language, i18n::GENERAL_BATTLENET_CLOSE_TITLE),
                    i18n::text(language, i18n::GENERAL_BATTLENET_CLOSE_ABOUT),
                    self.hint(
                        i18n::text(language, i18n::GENERAL_BATTLENET_CLOSE_HINT),
                    ),
                    switch("close battle.net on play", self.config.close_battle_net_on_play, Message::CloseBattleNetOnPlay),
                ),
            ]),
            setting_group([
                setting(
                    i18n::text(language, i18n::GENERAL_NOTES_TITLE),
                    i18n::text(language, i18n::GENERAL_NOTES_ABOUT),
                    None,
                    switch("release notes", !self.config.patch_notes_muted, Message::ShowReleaseNotes),
                ),
                setting(
                    i18n::text(language, i18n::GENERAL_UPDATES_TITLE),
                    i18n::text(language, i18n::GENERAL_UPDATES_ABOUT),
                    self.hint(
                        i18n::text(language, i18n::GENERAL_UPDATES_HINT),
                    ),
                    switch("update check", self.config.check_for_updates, Message::CheckLauncherUpdates),
                ),
            ]),
            setting_group(self.interface_size_row().into_iter().chain([
                setting(
                    i18n::text(language, i18n::GENERAL_CONTROLLER_TITLE),
                    i18n::text(language, i18n::GENERAL_CONTROLLER_ABOUT),
                    self.hint(i18n::text(language, i18n::GENERAL_CONTROLLER_HINT)),
                    switch("controller look", self.console(), Message::ControllerLook),
                ),
                setting(
                    i18n::text(language, i18n::GENERAL_STEAM_TITLE),
                    i18n::text(language, i18n::GENERAL_STEAM_ABOUT),
                    self.hint(i18n::text(language, i18n::GENERAL_STEAM_HINT)),
                    secondary(&i18n::text(language, i18n::GENERAL_STEAM_BUTTON), Message::AddToSteam).enabled(true),
                ),
            ])),
        ]
        .extend(shortcut)
        .spacing(theme::MD)
        .into()
    }

    /// How Battle.net and every game it launches are started.
    ///
    /// One profile, not one per game: Lightforge only sets the environment of
    /// a client it starts cold, and a running client takes launches over its
    /// own IPC queue, so two games from one session cannot differ (see
    /// `Config::battle_net_tweaks`).
    pub(crate) fn battle_net_section(&self) -> Element<'_, Message> {
        let language = &self.config.interface_language;
        let tweaks = self.tweaks_of(Scope::BattleNet);
        let tools = &self.tools;
        let scope = Scope::BattleNet;

        let rows = vec![
            setting(
                i18n::text(language, "settings-battlenet-cpu-title"),
                match tools.game_performance {
                    Some(_) => i18n::text(language, "settings-battlenet-cpu-about"),
                    None => i18n::text(language, "settings-battlenet-cpu-missing"),
                },
                None,
                self.tool_switch(
                    "bnet game performance",
                    tools.game_performance.is_some(),
                    tweaks.wants_game_performance() && tools.game_performance.is_some(),
                    scope,
                    Tweak::GamePerformance,
                ),
            ),
            setting(
                i18n::text(language, "settings-battlenet-mangohud-title"),
                match tools.mangohud {
                    Some(_) => i18n::text(language, "settings-battlenet-mangohud-about"),
                    None => i18n::text(language, "settings-battlenet-mangohud-missing"),
                },
                None,
                self.tool_switch(
                    "bnet mangohud",
                    tools.mangohud.is_some(),
                    tweaks.mangohud && tools.mangohud.is_some(),
                    scope,
                    Tweak::MangoHud,
                ),
            ),
            setting(
                i18n::text(language, "settings-battlenet-wayland-title"),
                super::super::game_settings::wayland_about(tools, language),
                self.hint(
                    i18n::text(language, "settings-battlenet-wayland-hint"),
                ),
                self.tool_switch(
                    "bnet wayland",
                    tools.wayland,
                    tweaks.wayland && tools.wayland,
                    scope,
                    Tweak::Wayland,
                ),
            ),
        ];

        column![muted(
            i18n::text(language, "settings-battlenet-intro"),
            theme::LABEL,
        )]
        .extend(
            self.hint(
                i18n::text(language, "settings-battlenet-hint"),
            )
            .map(Element::from),
        )
        .push(setting_group(rows))
        // The only way to tune a hand-off game beyond the switches above: it
        // has no client settings for Lightforge to write, so anything else
        // has to arrive as an environment variable.
        .push(self.environment_group(scope))
        .spacing(theme::SM)
        .into()
    }

    /// Speed caps, ours and Battle.net's, and pausing while playing.
    pub(crate) fn downloads_section(&self) -> Element<'_, Message> {
        let language = &self.config.interface_language;
        let speed = |input: &str, placeholder: &str, on_input: fn(String) -> Message| -> Element<'_, Message> {
            row![
                clickable(
                    format!("speed {placeholder}"),
                    text_input(placeholder, input)
                        .on_input(on_input)
                        .style(theme::field)
                        .size(theme::BODY)
                        .padding(theme::SM)
                        .width(Length::Fixed(110.0)),
                ),
                muted("MB/s".to_owned(), theme::LABEL),
            ]
            .spacing(theme::SM)
            .align_y(Alignment::Center)
            .into()
        };

        let locales: Vec<LocaleChoice> = lightforge_core::tact::install::LOCALES
            .iter()
            .map(|(code, _)| LocaleChoice(code.to_string()))
            .collect();

        setting_group([
            setting(
                i18n::text(language, i18n::DOWNLOADS_GAME_LANGUAGE_TITLE),
                i18n::text(language, i18n::DOWNLOADS_GAME_LANGUAGE_ABOUT),
                self.hint(i18n::text(language, i18n::DOWNLOADS_GAME_LANGUAGE_HINT)),
                self.picker(
                    "game language".to_owned(),
                    locales,
                    Some(LocaleChoice(lightforge_core::tact::install::current_locale(&self.config.locale).to_owned())),
                    "",
                    |choice| Message::GameLanguage(choice.0),
                    Length::Fixed(260.0),
                    theme::BODY,
                ),
            ),
            setting(
                i18n::text(language, i18n::DOWNLOADS_SPEED_LIMIT_TITLE),
                i18n::text(language, i18n::DOWNLOADS_SPEED_LIMIT_ABOUT),
                None,
                speed(
                    &self.limit_input,
                    &i18n::text(language, i18n::DOWNLOADS_SPEED_LIMIT_UNLIMITED),
                    Message::DownloadLimitChanged,
                ),
            ),
            setting(
                i18n::text(language, i18n::DOWNLOADS_BATTLENET_LIMIT_TITLE),
                i18n::text(language, i18n::DOWNLOADS_BATTLENET_LIMIT_ABOUT),
                self.hint(
                    i18n::text(language, i18n::DOWNLOADS_BATTLENET_LIMIT_HINT),
                ),
                speed(
                    &self.bnet_limit_input,
                    &i18n::text(language, i18n::DOWNLOADS_BATTLENET_LIMIT_NONE),
                    Message::BattleNetLimitChanged,
                ),
            ),
            setting(
                i18n::text(language, i18n::DOWNLOADS_PAUSE_TITLE),
                i18n::text(language, i18n::DOWNLOADS_PAUSE_ABOUT),
                self.hint(i18n::text(language, i18n::DOWNLOADS_PAUSE_HINT)),
                switch(
                    "pause while playing",
                    self.config.pause_downloads_while_playing,
                    Message::PauseWhilePlaying,
                ),
            ),
        ])
    }

    /// The version, where its configuration lives, and the system checks.
    pub(crate) fn about_section(&self) -> Element<'_, Message> {
        let language = &self.config.interface_language;
        let (status, asking) = match (&self.launcher_check, &self.newer_release) {
            (LauncherCheck::Asking, _) => (i18n::text(language, "settings-about-checking-status"), true),
            (LauncherCheck::Failed(why), _) => (
                format!("{}: {}", i18n::text(language, "settings-about-check-failed"), i18n::error_text(language, why)),
                false,
            ),
            (_, Some(newer)) => (
                i18n::text_with_args(language, "settings-about-version-available", &[("version", &newer.latest)]),
                false,
            ),
            (LauncherCheck::Newest, None) => (i18n::text(language, "settings-about-current-version"), false),
            (LauncherCheck::Idle, None) => (String::new(), false),
        };
        let controls = row![text(env!("CARGO_PKG_VERSION")).size(theme::BODY).font(theme::MONO)]
            .push(self.newer_release.as_ref().map(|newer| {
                let label = match self.origin.route(&newer.latest) {
                    lightforge_core::upgrade::Route::Instructions { .. } => {
                        i18n::text(language, "settings-about-how-to-update")
                    }
                    _ => i18n::text(language, "settings-about-update-now"),
                };
                action_button(&label, theme::ACCENT, Message::UpdateNow).enabled(!self.upgrading())
            }))
            .push(
                secondary(
                    &if asking {
                        i18n::text(language, "settings-about-checking")
                    } else {
                        i18n::text(language, "settings-about-check-for-updates")
                    },
                    Message::CheckLauncherNow,
                )
                    .enabled(!asking),
            )
            .spacing(theme::MD)
            .align_y(Alignment::Center);
        let mut rows = vec![setting(i18n::text(language, "settings-about-version"), status, None, controls)];
        if let Some(dirs) = &self.dirs {
            rows.push(setting(
                i18n::text(language, "settings-about-settings-file"),
                tilde(&dirs.config_file()),
                None,
                secondary(&i18n::text(language, "settings-about-open-folder"), Message::OpenConfigFolder).enabled(true),
            ));
        }
        column![setting_group(rows), self.diagnostics_group()].spacing(theme::MD).into()
    }

    /// `lightforge doctor`, one row per check, and a copy for a bug report.
    fn diagnostics_group(&self) -> Element<'_, Message> {
        let language = &self.config.interface_language;
        let running = self.diagnostics == Diagnostics::Running;
        let checks: &[lightforge_core::doctor::Check] = match &self.diagnostics {
            Diagnostics::Done(checks) => checks,
            _ => &[],
        };
        let controls = row![
            secondary(
                &if running {
                    i18n::text(language, "settings-about-checking")
                } else {
                    i18n::text(language, "settings-about-run-checks")
                },
                Message::RunDiagnostics,
            )
                .enabled(!running && self.dirs.is_some()),
            secondary(&i18n::text(language, "settings-about-copy"), Message::CopyDiagnostics)
                .enabled(!checks.is_empty()),
        ]
        .spacing(theme::SM);

        let mut rows = vec![setting(
            i18n::text(language, "settings-about-diagnostics"),
            i18n::text(language, "settings-about-diagnostics-about"),
            None,
            controls,
        )];
        rows.extend(checks.iter().map(|check| {
            use lightforge_core::doctor::Status;
            let color = match check.status {
                Status::Ok => theme::OK,
                Status::Warn => theme::WARN,
                Status::Fail => theme::ERROR,
            };
            let more = check.hint.as_ref().map(|hint| muted(lightforge_core::debug::scrub(&i18n::doctor_hint(language, hint)), theme::CAPTION));
            setting(
                i18n::doctor_name(language, &check.name),
                lightforge_core::debug::scrub(&i18n::doctor_detail(language, check, &check.detail)),
                more,
                text(i18n::doctor_status(language, check.status)).size(theme::LABEL).font(theme::MONO).color(color),
            )
        }));
        setting_group(rows)
    }
}
