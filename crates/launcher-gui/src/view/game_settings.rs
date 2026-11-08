//! The Game Settings popup: launch options, environment, files and backups.

use crate::*;
use iced::widget::{column, text_editor};

impl Lightforge {
    /// The Game Settings popup.
    ///
    /// Rows shaped as in Settings; sections scroll under a fixed title.
    pub(crate) fn game_settings_card<'a>(&'a self, settings: &'a GameSettings) -> Element<'a, Message> {
        let language = &self.config.interface_language;
        // The figure is [`WINDOW_MIN`]'s height less the title, the footer and
        // the padding.
        const SECTIONS_MAX: f32 = 560.0;

        let sections = column![
            section(i18n::text(language, "game-settings-performance"), self.performance_group()),
            section(i18n::text(language, "game-settings-graphics"), self.graphics_group(settings)),
            section(i18n::text(language, "game-settings-environment"), self.environment_group(Scope::Product)),
            section(i18n::text(language, "game-settings-game"), self.game_group(settings)),
            section(i18n::text(language, "settings-proton-proton"), self.proton_group(settings)),
            section(i18n::text(language, "settings-nav-addons"), self.addons_group()),
            section(i18n::text(language, "game-settings-backups"), self.backups_group(settings)),
        ]
        .spacing(theme::LG);

        let actions = row![
            Space::new().width(Length::Fill),
            secondary(&i18n::text(language, "common-close"), Message::CloseGameSettings).enabled(true),
        ]
        .align_y(Alignment::Center);

        container(
            column![
                column![
                    text(i18n::text(language, "menu-game-settings")).size(theme::TITLE),
                    muted(settings.label.clone(), theme::LABEL),
                ]
                .spacing(theme::XS),
                container(focus::region(
                    "game settings",
                    scrollable(sections)
                        .direction(scrollable::Direction::Vertical(theme::scrollbar_shape()))
                        .style(theme::scrollbar),
                ))
                .max_height(SECTIONS_MAX),
                actions,
            ]
            .spacing(theme::LG),
        )
        .width(Length::Fixed(680.0))
        .padding(theme::LG)
        .style(theme::dialog)
        .into()
    }

    /// A switch for a tool that may not be installed: dimmed, and said so,
    /// when it is missing, not gone, so nobody wonders where it went.
    pub(crate) fn tool_switch<'a>(
        &self,
        key: &'static str,
        available: bool,
        on: bool,
        scope: Scope,
        which: Tweak,
    ) -> Element<'a, Message> {
        match available {
            true => clickable(
                key,
                iced::widget::toggler(on)
                    .on_toggle(move |on| Message::Tweak(scope, which, on))
                    .size(theme::tap_text(20.0))
                    .style(theme::switch),
            ),
            false => iced::widget::toggler(false)
                .size(theme::tap_text(20.0))
                .style(theme::switch)
                .into(),
        }
    }

    /// CPU profile and overlay, the two things a launch wraps the game in.
    fn performance_group(&self) -> Element<'_, Message> {
        let language = &self.config.interface_language;
        let tweaks = self.tweaks();
        let tools = &self.tools;
        let scope = Scope::Product;

        setting_group([
            setting(
                i18n::text(language, "settings-battlenet-cpu-title"),
                match tools.game_performance {
                    Some(_) => i18n::text(language, "game-settings-cpu-about"),
                    None => i18n::text(language, "settings-battlenet-cpu-missing"),
                },
                self.hint(i18n::text(language, "game-settings-cpu-hint")),
                self.tool_switch(
                    "game performance",
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
                    "mangohud",
                    tools.mangohud.is_some(),
                    tweaks.mangohud && tools.mangohud.is_some(),
                    scope,
                    Tweak::MangoHud,
                ),
            ),
            setting(
                i18n::text(language, "game-settings-shader-cache"),
                i18n::text(language, "game-settings-shader-cache-about"),
                None,
                secondary(&i18n::text(language, "common-clear"), Message::ClearShaderCache).enabled(true),
            ),
        ])
    }

    /// How the game reaches the screen.
    fn graphics_group<'a>(&'a self, settings: &'a GameSettings) -> Element<'a, Message> {
        let language = &self.config.interface_language;
        let tweaks = self.tweaks();
        let scope = Scope::Product;

        let mut rows = vec![setting(
            i18n::text(language, "settings-battlenet-wayland-title"),
            wayland_about(&self.tools, language),
            self.hint(i18n::text(language, "settings-battlenet-wayland-hint")),
            self.tool_switch(
                "wayland",
                self.tools.wayland,
                tweaks.wayland && self.tools.wayland,
                scope,
                Tweak::Wayland,
            ),
        )];
        if self.tools.hybrid {
            rows.push(setting(
                i18n::text(language, "game-settings-nvidia-gpu"),
                i18n::text(language, "game-settings-nvidia-gpu-about"),
                self.hint(i18n::text(language, "game-settings-nvidia-gpu-hint")),
                self.tool_switch("nvidia gpu", true, tweaks.nvidia_gpu, scope, Tweak::NvidiaGpu),
            ));
        }
        if self.tools.nvidia_card {
            rows.push(setting(
                i18n::text(language, "game-settings-nvidia-features"),
                i18n::text(language, "game-settings-nvidia-features-about"),
                self.hint(i18n::text(language, "game-settings-nvidia-features-hint")),
                self.picker(
                    "nvapi".to_owned(),
                    lightforge_core::tweaks::Nvapi::ALL
                        .into_iter()
                        .map(|choice| DisplayChoice(choice, i18n::text(language, match choice {
                            lightforge_core::tweaks::Nvapi::Proton => "game-settings-nvapi-proton",
                            lightforge_core::tweaks::Nvapi::On => "game-settings-nvapi-on",
                            lightforge_core::tweaks::Nvapi::Off => "game-settings-nvapi-off",
                        })))
                        .collect::<Vec<_>>(),
                    Some(DisplayChoice(tweaks.nvapi, i18n::text(language, match tweaks.nvapi {
                        lightforge_core::tweaks::Nvapi::Proton => "game-settings-nvapi-proton",
                        lightforge_core::tweaks::Nvapi::On => "game-settings-nvapi-on",
                        lightforge_core::tweaks::Nvapi::Off => "game-settings-nvapi-off",
                    }))),
                    &i18n::text(language, "game-settings-nvidia-features"),
                    |choice| Message::NvapiPicked(choice.0),
                    Length::Fixed(200.0),
                    theme::BODY,
                ),
            ));
        }

        // Only where the build has textures behind the option.
        if let Some(bytes) = settings.textures_bytes {
            let on = self.opted_optional(&settings.code).iter().any(|n| n == HIGH_RES);
            let working = matches!(
                settings.textures,
                TexturesStep::Planning | TexturesStep::Removing
            );
            let about = match (&settings.textures, on) {
                (TexturesStep::Planning, _) => i18n::text(language, "game-settings-textures-planning"),
                (TexturesStep::Removing, _) => i18n::text(language, "game-settings-textures-removing"),
                (_, true) => i18n::text_with_args(language, "game-settings-textures-downloaded", &[("size", &size_of(bytes))]),
                (_, false) => i18n::text_with_args(language, "game-settings-textures-optional", &[("size", &size_of(bytes))]),
            };
            let switch: Element<'a, Message> = match working || self.locked() {
                true => iced::widget::toggler(on)
                    .size(theme::tap_text(20.0))
                    .style(theme::switch)
                    .into(),
                false => clickable(
                    "high res textures",
                    iced::widget::toggler(on)
                        .on_toggle(|on| Message::OptionalContent(HIGH_RES.to_owned(), on))
                        .size(theme::tap_text(20.0))
                        .style(theme::switch),
                ),
            };
            rows.push(setting(optional_label(HIGH_RES, language), about, None, switch));

            // The question, in the group it belongs to, once it is asked.
            if let TexturesStep::Ask(plan) = &settings.textures {
                rows.push(setting(
                    i18n::text(language, "game-settings-delete-textures-question"),
                    i18n::text_with_args(language, "game-settings-delete-textures-about", &[("size", &size_of(plan.freed))]),
                    None,
                    row![
                        secondary(&i18n::text(language, "game-settings-keep-textures"), Message::KeepOptional).enabled(true),
                        action_button(&i18n::text(language, "common-delete"), theme::ERROR, Message::RemoveOptional)
                            .enabled(!self.locked()),
                    ]
                    .spacing(theme::SM),
                ));
            }
        }

        setting_group(rows)
    }

    /// The environment-variable editor of one scope, with what is wrong in it.
    pub(crate) fn environment_group(&self, scope: Scope) -> Element<'_, Message> {
        let language = &self.config.interface_language;
        let content = match scope {
            Scope::Product => &self.env_editor,
            Scope::BattleNet => &self.bnet_env_editor,
        };
        let key = match scope {
            Scope::Product => "environment",
            Scope::BattleNet => "bnet environment",
        };
        let (_, bad) = lightforge_core::tweaks::parse_env(&content.text());

        let mut body = column![
            muted(i18n::text(language, "game-settings-environment-about"), theme::LABEL),
            clickable(
                key,
                text_editor(content)
                    .placeholder("DXVK_HUD=fps")
                    .on_action(move |action| Message::EnvEdit(scope, action))
                    .height(Length::Fixed(theme::BODY * 1.4 * 5.0 + theme::SM * 2.0))
                    .style(theme::editor)
                    .size(theme::BODY)
                    .padding(theme::SM),
            ),
        ]
        .spacing(theme::SM);

        if !bad.is_empty() {
            let lines: Vec<String> = bad.iter().map(ToString::to_string).collect();
            body = body.push(
                text(i18n::text_with_args(language, "game-settings-environment-not-used", &[("variables", &lines.join(", "))]))
                    .size(theme::LABEL)
                    .color(theme::ERROR),
            );
        }
        body = body.extend(
            self.hint(i18n::text(language, "game-settings-environment-hint"))
            .map(Element::from),
        );

        setting_card(body)
    }

    /// Where the game lives, and what can be done to the files there.
    fn game_group<'a>(&'a self, settings: &'a GameSettings) -> Element<'a, Message> {
        let language = &self.config.interface_language;
        let can = self.can_maintain();
        let dir = self.layout.product_dir(&settings.code);
        let shown = match std::env::var_os("HOME").filter(|home| !home.is_empty()) {
            Some(home) => match dir.strip_prefix(&home) {
                Ok(rest) => Path::new("~").join(rest).display().to_string(),
                Err(_) => dir.display().to_string(),
            },
            None => dir.display().to_string(),
        };

        let mut rows = vec![
            setting(
                i18n::text(language, "game-settings-location"),
                shown,
                None,
                secondary(&i18n::text(language, "menu-show-in-explorer"), Message::OpenFiles).enabled(true),
            ),
            self.drive_row(settings, can.move_game),
        ];
        rows.extend(self.leftover_rows(settings, can.move_game));
        rows.extend([
            setting(
                i18n::text(language, "game-settings-game-files"),
                i18n::text(language, "game-settings-game-files-about"),
                None,
                row![
                    match self.deep_cancel.is_some() {
                        true => secondary(&i18n::text(language, "game-settings-stop-deep-scan"), Message::DeepScan).enabled(true),
                        false => secondary(
                            &i18n::text(language, "game-settings-deep-scan"),
                            Message::FromGameSettings(Box::new(Message::DeepScan)),
                        )
                        .enabled(can.deep_scan),
                    },
                    secondary(
                        &i18n::text(language, "game-settings-scan-repair"),
                        Message::FromGameSettings(Box::new(Message::ScanAndRepair)),
                    )
                    .enabled(can.scan),
                ]
                .spacing(theme::SM),
            ),
            setting(
                i18n::text(language, "game-settings-cache"),
                match settings.cache_cleared {
                    true => i18n::text(language, "game-settings-cache-cleared"),
                    false => i18n::text(language, "game-settings-cache-about"),
                },
                None,
                secondary(&i18n::text(language, "game-settings-clear-cache"), Message::ClearGameCache).enabled(!settings.cache_cleared && !self.in_play()),
            ),
            setting(
                i18n::text(language, "game-settings-remove-from-computer"),
                i18n::text(language, "game-settings-uninstall-about"),
                None,
                action_button(
                    &i18n::text(language, "common-uninstall"),
                    theme::ERROR,
                    Message::FromGameSettings(Box::new(Message::AskUninstall)),
                )
                .enabled(can.uninstall),
            ),
        ]);
        setting_group(rows)
    }

    /// A copy a move left in another library, offered for removal. Hidden
    /// while a move runs: its own last step is what removes it.
    fn leftover_rows<'a>(&'a self, settings: &'a GameSettings, can_remove: bool) -> Vec<Element<'a, Message>> {
        let language = &self.config.interface_language;
        if self.moving.as_ref().is_some_and(|m| m.code == settings.code) {
            return Vec::new();
        }
        settings
            .leftovers
            .iter()
            .map(|library| {
                let controls = match settings.leftover_confirm.as_deref() == Some(library.id.as_str()) {
                    true => row![
                        secondary(&i18n::text(language, "common-cancel"), Message::CancelRemoveLeftover).enabled(true),
                        action_button(&i18n::text(language, "common-delete"), theme::ERROR, Message::RemoveLeftover).enabled(can_remove),
                    ],
                    false => row![
                        secondary(&i18n::text(language, i18n::COMMON_REMOVE), Message::AskRemoveLeftover(library.id.clone())).enabled(can_remove),
                    ],
                };
                setting(
                    i18n::text_with_args(language, "game-settings-old-copy", &[("drive", &library.label)]),
                    i18n::text(language, "game-settings-old-copy-about"),
                    self.hint(i18n::text(language, "game-settings-old-copy-hint")),
                    controls.spacing(theme::SM),
                )
            })
            .collect()
    }

    /// Which library the game is on, and moving it to another one.
    fn drive_row<'a>(&'a self, settings: &'a GameSettings, can_move: bool) -> Element<'a, Message> {
        let language = &self.config.interface_language;
        let here = self.layout.library(&settings.code);
        if let Some(moving) = self.moving.as_ref().filter(|m| m.code == settings.code) {
            let progress = moving.progress.lock().map(|p| *p).unwrap_or_default();
            let percent = progress.bytes_done * 100 / progress.bytes_total.max(1);
            let doing = match (moving.releasing, progress.verifying) {
                (true, _) => i18n::text_with_args(language, "game-settings-moving-remove-old", &[("drive", &moving.from.label)]),
                (false, true) => i18n::text_with_args(language, "game-settings-moving-check", &[("drive", &moving.to.label), ("percent", &percent.to_string())]),
                (false, false) => i18n::text_with_args(language, "game-settings-moving-copy", &[("drive", &moving.to.label), ("percent", &percent.to_string())]),
            };
            return setting(i18n::text(language, "settings-storage-drives"), doing, None, secondary(&i18n::text(language, "common-cancel"), Message::CancelMove).enabled(!moving.releasing));
        }

        let about = here.map_or_else(String::new, |library| i18n::text_with_args(language, "game-settings-on-drive", &[("drive", &library.label)]));
        let targets: Vec<(String, String)> = self
            .config
            .libraries
            .iter()
            .filter(|library| here.is_none_or(|h| h.id != library.id))
            .map(|library| (library.label.clone(), library.id.clone()))
            .collect();
        if targets.is_empty() {
            return setting(i18n::text(language, "settings-storage-drives"), about, self.hint(i18n::text(language, "game-settings-add-drive-hint")), Space::new());
        }
        let picked = settings
            .move_to
            .as_ref()
            .and_then(|id| targets.iter().find(|(_, t)| t == id))
            .map(|(label, _)| label.clone());
        let labels = targets.iter().map(|(label, _)| label.clone()).collect();
        let picker = self.picker(
            "move to".to_owned(),
            labels,
            picked.clone(),
            i18n::text(language, "game-settings-another-drive").as_str(),
            move |label: String| {
                let id = targets.iter().find(|(l, _)| *l == label).map(|(_, id)| id.clone()).unwrap_or_default();
                Message::MoveTargetPicked(id)
            },
            Length::Shrink,
            theme::LABEL,
        );
        setting(
            i18n::text(language, "settings-storage-drives"),
            about,
            self.hint(i18n::text(language, "game-settings-move-hint")),
            row![picker, secondary(&i18n::text(language, "game-settings-move"), Message::MoveGame).enabled(can_move && picked.is_some())]
                .spacing(theme::SM)
                .align_y(Alignment::Center),
        )
    }

    /// The same pin as Settings › Proton, for whoever is here.
    fn proton_group<'a>(&'a self, settings: &'a GameSettings) -> Element<'a, Message> {
        let language = &self.config.interface_language;
        let code = &settings.code;
        if self.pin_follows_client(code) {
            return setting_group([setting(
                i18n::text(language, "settings-nav-proton"),
                i18n::text(language, super::settings::FOLLOWS_CLIENT),
                None,
                self.locked_picker(
                    ProtonChoice(None, Unpinned::BattleNet).label(language),
                    Length::Fixed(260.0),
                    theme::BODY,
                ),
            )]);
        }
        let unpinned = match Kind::of(code) {
            Kind::Managed => Unpinned::Default,
            Kind::HandOff => Unpinned::BattleNet,
        };
        let builds: Vec<DisplayChoice<ProtonChoice>> = std::iter::once(ProtonChoice(None, unpinned))
            .chain(
                self.setup
                    .iter()
                    .flat_map(|s| s.installed.iter())
                    .map(|build| ProtonChoice(Some(build.version.clone()), unpinned)),
            )
            .map(|choice| DisplayChoice(choice.clone(), choice.label(&self.config.interface_language)))
            .collect();
        let pin = self.config.product(code).and_then(|p| p.profile.proton_ge_version.clone());
        let current = ProtonChoice(pin, unpinned);
        let for_code = code.clone();

        setting_group([setting(
            i18n::text(language, "game-settings-proton-version"),
            i18n::text(language, "game-settings-proton-version-about"),
            None,
            self.picker(
                "game settings proton".to_owned(),
                builds,
                Some(DisplayChoice(current.clone(), current.label(language))),
                "",
                move |choice| Message::PinProductProton(for_code.clone(), choice.0),
                Length::Fixed(260.0),
                theme::BODY,
            ),
        )])
    }

    /// The way to the page that manages them.
    fn addons_group(&self) -> Element<'_, Message> {
        let language = &self.config.interface_language;
        setting_group([setting(
            i18n::text(language, "game-settings-installed-addons"),
            i18n::text(language, "game-settings-addons-about"),
            None,
            secondary(
                &i18n::text(language, "dashboard-manage-addons"),
                Message::FromGameSettings(Box::new(Message::Show(Screen::Addons))),
            )
            .enabled(true),
        )])
    }

    /// The WTF backups, each restorable or deletable, and the button that
    /// takes one now.
    fn backups_group<'a>(&'a self, settings: &'a GameSettings) -> Element<'a, Message> {
        let language = &self.config.interface_language;
        let idle = !settings.working && !self.locked();

        let mut rows: Vec<Element<'a, Message>> = vec![setting(
            i18n::text(language, "game-settings-your-settings"),
            i18n::text(language, "game-settings-your-settings-about"),
            None,
            secondary(&i18n::text(language, "game-settings-backup-now"), Message::BackupNow).enabled(idle),
        )];

        match settings.backups.as_deref() {
            None => rows.push(setting(i18n::text(language, "game-settings-reading-backups"), "", None, Space::new())),
            Some([]) => rows.push(setting(i18n::text(language, "game-settings-no-backups"), "", None, Space::new())),
            Some(list) => {
                for backup in list {
                    let asking = settings
                        .confirm
                        .as_ref()
                        .filter(|(path, _)| path == &backup.path)
                        .map(|(_, action)| *action);
                    // The question replaces the details under the date while
                    // it is asked, so the row only ever holds two buttons.
                    let about = match asking {
                        Some(BackupAction::Restore) if backup.accounts.len() > 1 => {
                            i18n::text(language, "game-settings-restore-accounts-question")
                        }
                        Some(BackupAction::Restore) => {
                            i18n::text(language, "game-settings-restore-question")
                        }
                        Some(BackupAction::Delete) => i18n::text(language, "game-settings-delete-backup-question"),
                        None => [
                            Some(backup.reason.label().to_owned()),
                            backup.build.clone(),
                            Some(size_of(backup.size)),
                        ]
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>()
                        .join(" · "),
                    };
                    let controls = match asking {
                        Some(BackupAction::Restore) if backup.accounts.len() > 1 => row![
                            secondary(&i18n::text(language, "common-cancel"), Message::CancelRestore).enabled(true),
                            action_button(&i18n::text(language, "game-settings-all-accounts"), theme::UPDATE, Message::ConfirmRestore(None))
                                .enabled(idle),
                        ]
                        .extend(backup.accounts.iter().map(|name| {
                            secondary(name, Message::ConfirmRestore(Some(name.clone()))).enabled(idle)
                        })),
                        Some(BackupAction::Restore) => row![
                            secondary(&i18n::text(language, "common-cancel"), Message::CancelRestore).enabled(true),
                            action_button(&i18n::text(language, "game-settings-restore"), theme::UPDATE, Message::ConfirmRestore(None))
                                .enabled(idle),
                        ],
                        Some(BackupAction::Delete) => row![
                            secondary(&i18n::text(language, "common-cancel"), Message::CancelRestore).enabled(true),
                            action_button(&i18n::text(language, "common-delete"), theme::ERROR, Message::ConfirmDeleteBackup)
                                .enabled(idle),
                        ],
                        None => row![
                            secondary(&i18n::text(language, "game-settings-restore"), Message::AskRestore(backup.path.clone()))
                                .enabled(idle),
                            secondary(&i18n::text(language, "common-delete"), Message::AskDeleteBackup(backup.path.clone()))
                                .enabled(idle),
                        ],
                    }
                    .spacing(theme::SM);
                    rows.push(setting(backup.when(), about, None, controls));
                }
            }
        }

        let mut body = column![setting_group(rows)].spacing(theme::SM);
        if let Some(note) = &settings.note {
            body = body.push(match note {
                Ok(line) => muted(line.clone(), theme::LABEL),
                Err(e) => text(e.clone()).size(theme::LABEL).color(theme::ERROR),
            });
        }
        body.into()
    }
}

/// The Wayland backend row's description, in Game Settings and Settings ›
/// Battle.net alike.
pub(crate) fn wayland_about(tools: &HostTools, language: &str) -> String {
    match tools.wayland {
        false => crate::i18n::text(language, "game-settings-wayland-not-available"),
        true if tools.nvidia_lacks_explicit_sync() => {
            crate::i18n::text(language, "game-settings-wayland-nvidia-flicker")
        }
        true => crate::i18n::text(language, "game-settings-wayland-about"),
    }
}
