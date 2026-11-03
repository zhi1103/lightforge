//! Popover menus and the picker that replaces a pick list.

use crate::*;
use iced::widget::column;

impl Lightforge {
    /// The menu under the wordmark: the app's own business.
    pub(crate) fn app_menu(&self) -> Element<'_, Message> {
        // No destination yet means no press and a dimmed face: the entry says
        // what is coming without pretending it is here.
        let entry = |icon: image::Handle, id: &'static str, message: Option<Message>| {
            let live = message.is_some();
            let face = row![
                image(icon)
                    .width(Length::Fixed(MENU_GLYPH))
                    .height(Length::Fixed(MENU_GLYPH))
                    .opacity(match live {
                        true => 1.0_f32,
                        false => DIMMED,
                    }),
                match live {
                    true => text(i18n::text(&self.config.interface_language, id)).size(theme::BODY),
                    false => muted(i18n::text(&self.config.interface_language, id), theme::BODY),
                },
            ]
            .spacing(theme::SM)
            .align_y(Alignment::Center);
            self.ring(
                focus::Target::Menu(id),
                message.clone(),
                button(face.width(Length::Fill))
                    .padding(theme::tap([theme::XS, theme::SM]))
                    .style(theme::ghost)
                    .on_press_maybe(message),
            )
        };
        let square = |icon: image::Handle, url: Option<&'static str>| {
            self.ring(
                focus::Target::Link(url.unwrap_or_default()),
                url.map(Message::OpenLink),
                link_square(icon, url, MENU_GLYPH),
            )
        };

        let rows = column![
            entry(gear_icon(), i18n::MENU_SETTINGS, Some(Message::Show(Screen::Settings))),
            entry(updates_icon(), i18n::MENU_UPDATES, Some(Message::Show(Screen::Updates))),
            container(Space::new().height(Length::Fixed(1.0)))
                .width(Length::Fill)
                .style(theme::divider),
            entry(bug_icon(), i18n::MENU_REPORT_BUG, Some(Message::OpenReport(ReportKind::Bug))),
            entry(
                feedback_icon(),
                i18n::MENU_FEEDBACK,
                Some(Message::OpenReport(ReportKind::Feedback)),
            ),
            container(Space::new().height(Length::Fixed(1.0)))
                .width(Length::Fill)
                .style(theme::divider),
            entry(exit_icon(), i18n::MENU_EXIT, Some(Message::Exit)),
            // One row of recognisable marks.
            row![
                square(discord_icon(), DISCORD_URL),
                square(x_icon(), X_URL),
                Space::new().width(Length::Fill),
                square(kofi_icon(), KOFI_URL),
                square(paypal_icon(), PAYPAL_URL),
            ]
            .spacing(theme::XS),
        ]
        .spacing(theme::XS)
        .width(Length::Fixed(240.0));

        focus::layer(container(rows).padding(theme::SM).style(theme::menu))
    }

    /// One addon's own settings: what is true of this addon, not what
    /// to do to it right now.
    ///
    /// A menu: the row is full.
    pub(crate) fn addon_row_menu(
        &self,
        managed: &addonwork::Managed,
        version: &str,
        name: &str,
    ) -> Element<'static, Message> {
        let language = self.config.interface_language.clone();
        let (provider, id) = (managed.provider.clone(), managed.id.clone());
        let default_auto = self.config.addon_auto_update;
        let entry = |id: &'static str, label: String, message: Message| {
            focus::ring(
                focus::Target::Key(format!("addon menu entry {id}")),
                focus::Press::Send(message.clone()),
                button(text(label).size(theme::BODY).width(Length::Fill))
                    .padding(theme::tap([theme::XS, theme::SM]))
                    .style(theme::ghost)
                    .on_press(message),
            )
        };

        // What this addon does today: its own choice where it has one, the
        // app's default otherwise.
        let auto = managed.auto_update.unwrap_or(default_auto);
        let mut rows = column![
            entry(
                "ignore",
                match managed.ignored {
                    true => i18n::text(&language, "menu-addon-stop-ignoring"),
                    false => i18n::text(&language, "menu-addon-ignore"),
                },
                Message::AddonIgnore(provider.clone(), id.clone(), !managed.ignored),
            ),
            entry(
                "auto-update",
                match auto {
                    true => i18n::text(&language, "menu-addon-no-auto-update"),
                    false => i18n::text(&language, "menu-addon-auto-update"),
                },
                Message::AddonAutoUpdate(provider.clone(), id.clone(), Some(!auto)),
            ),
        ]
        .spacing(theme::XS)
        .width(Length::Fixed(240.0));

        // Only worth offering once the addon has an opinion to give up, and
        // worth naming the default so clearing it is not a leap of faith.
        if managed.auto_update.is_some() {
            rows = rows.push(entry(
                "follow-default",
                match default_auto {
                    true => i18n::text(&language, "menu-addon-follow-default-on"),
                    false => i18n::text(&language, "menu-addon-follow-default-off"),
                },
                Message::AddonAutoUpdate(provider.clone(), id.clone(), None),
            ));
        }

        // Choosing an older build; only CurseForge publishes a build list.
        if managed.provider.eq_ignore_ascii_case("curse") {
            rows = rows.push(entry(
                "details",
                i18n::text(&language, "menu-addon-details-changelog"),
                Message::ShowAddonDetail(provider.clone(), id.clone(), name.to_owned()),
            ));
            rows = rows.push(entry(
                "choose-version",
                i18n::text(&language, "menu-addon-choose-version"),
                Message::PickVersion(
                    provider.clone(),
                    id.clone(),
                    name.to_owned(),
                    managed.file_id.clone(),
                ),
            ));
        }

        // Names the version it will hold.
        match (&managed.pinned, &managed.file_id) {
            (Some(_), _) => {
                rows = rows.push(entry(
                    "unpin",
                    i18n::text_with_args(&language, "menu-addon-unpin-version", &[("version", version)]),
                    Message::AddonPin(provider, id, None),
                ));
            }
            (None, Some(file_id)) => {
                rows = rows.push(entry(
                    "pin",
                    i18n::text_with_args(&language, "menu-addon-pin-version", &[("version", version)]),
                    Message::AddonPin(provider, id, Some(file_id.clone())),
                ));
            }
            (None, None) => {}
        }

        // Links out, above the rule, the project page first (where CurseForge
        // pays authors). Each only when known.
        let support = managed.support_url().map(str::to_owned);
        if managed.website_url.is_some() || support.is_some() {
            rows = rows.push(
                container(Space::new().height(Length::Fixed(1.0)))
                    .width(Length::Fill)
                    .style(theme::divider),
            );
        }
        if let Some(url) = managed.website_url.clone() {
            rows = rows.push(entry("project-page", i18n::text(&language, "menu-addon-open-project"), Message::OpenUrl(url)));
        }
        // See `support_url`; the label promises help, not a payment form.
        if let Some(url) = support {
            rows = rows.push(entry("support-author", i18n::text(&language, "menu-addon-support-author"), Message::OpenUrl(url)));
        }

        // An action, under its own rule; disabled while busy.
        rows = rows.push(
            container(Space::new().height(Length::Fixed(1.0)))
                .width(Length::Fill)
                .style(theme::divider),
        );
        let reinstall = (!self.addons_locked())
            .then(|| Message::ReinstallAddon(managed.provider.clone(), managed.id.clone()));
        rows = rows.push(self.ring(
            focus::Target::Key("addon menu entry reinstall".to_owned()),
            reinstall.clone(),
            button(text(i18n::text(&language, "menu-addon-reinstall")).size(theme::BODY).width(Length::Fill))
                .padding(theme::tap([theme::XS, theme::SM]))
                .style(theme::ghost)
                .on_press_maybe(reinstall),
        ));

        // Why either is worth having, in the menu that offers them: the words
        // alone do not say that the addon stays installed and working.
        rows = rows.push(muted(
            match (managed.ignored, managed.pinned.is_some()) {
                (true, _) => i18n::text(&language, "menu-addon-ignored-hint"),
                (false, true) => i18n::text(&language, "menu-addon-pinned-hint"),
                (false, false) => i18n::text(&language, "menu-addon-update-hint"),
            },
            theme::CAPTION,
        ));

        focus::layer(container(rows).padding(theme::SM).style(theme::menu))
    }

    /// The games the band had no room for.
    pub(crate) fn overflow_menu(&self, hidden: &[ProductLine]) -> Element<'_, Message> {
        let mut rows = column![].spacing(theme::XS).width(Length::Fixed(220.0));
        for line in hidden {
            let mut entry = row![].spacing(theme::SM).align_y(Alignment::Center);
            if let Some(mark) = self.tile_art(line) {
                entry = entry.push(
                    image(mark.handle.clone())
                        .content_fit(ContentFit::Contain)
                        .width(Length::Fixed(theme::TILE * 0.6))
                        .height(Length::Fixed(theme::TILE * 0.6)),
                );
            }
            entry = entry.push(text(self.name_of(line)).size(theme::BODY));
            rows = rows.push(self.ring(
                focus::Target::Overflow(line.clone()),
                Some(Message::LineSelected(line.clone())),
                button(entry.width(Length::Fill))
                    .padding(theme::tap([theme::XS, theme::SM]))
                    .style(theme::ghost)
                    .on_press(Message::LineSelected(line.clone())),
            ));
        }
        focus::layer(container(rows).padding(theme::SM).style(theme::menu))
    }

    /// A dropdown the ring can open and move through.
    ///
    /// Not a `pick_list`, which a keyboard cannot open. `key` must be unique
    /// on the page.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn picker<'a, T: ToString + PartialEq + Clone>(
        &self,
        key: String,
        options: Vec<T>,
        current: Option<T>,
        placeholder: &str,
        on_pick: impl Fn(T) -> Message,
        width: Length,
        size: f32,
    ) -> Element<'a, Message> {
        let label = match &current {
            Some(current) => text(current.to_string()).size(size),
            None => text(placeholder.to_owned()).size(size).color(theme::MUTED),
        };
        let toggle = Message::TogglePicker(key.clone());
        let face = button(
            row![
                label.width(Length::Fill),
                image(chevron_icon())
                    .width(Length::Fixed(APP_MENU_CHEVRON))
                    .height(Length::Fixed(APP_MENU_CHEVRON)),
            ]
            .spacing(theme::SM)
            .align_y(Alignment::Center),
        )
        .padding(theme::tap([theme::XS + 2.0, theme::SM]))
        .width(width)
        .style(theme::picker_button)
        .on_press(toggle.clone());
        let face = focus::ring(focus::Target::Key(format!("picker {key}")), focus::Press::Send(toggle), face);

        let menu = (self.picker_open.as_ref() == Some(&key)).then(|| {
            // As wide as the button it opens from, where that has a width.
            let menu_width = match width {
                Length::Fixed(button) => (button - 2.0 * theme::XS).max(240.0),
                _ => 240.0,
            };
            let mut rows = column![].spacing(theme::XS).width(Length::Fixed(menu_width));
            for option in options {
                let chosen = Message::Picked(Box::new(on_pick(option.clone())));
                let name = option.to_string();
                let entry = button(text(name.clone()).size(size).width(Length::Fill))
                    .padding(theme::tap([theme::XS, theme::SM]))
                    .width(Length::Fill)
                    .style(match current.as_ref() == Some(&option) {
                        true => theme::selected,
                        false => theme::ghost,
                    })
                    .on_press(chosen.clone());
                rows = rows.push(focus::ring(
                    focus::Target::Key(format!("pick {key} {name}")),
                    focus::Press::Send(chosen),
                    entry,
                ));
            }
            focus::layer(
                mouse_area(container(rows).padding(theme::XS).style(theme::menu))
                    .on_press(Message::MenuClick),
            )
        });
        popover::popover(face, menu).align_left().into()
    }

    /// The menu beside a running Battle.net: bring it up, or quit it.
    pub(crate) fn client_menu(&self) -> Element<'_, Message> {
        let language = &self.config.interface_language;
        let free = !self.busy && !self.quitting_client;
        let item = |id: &'static str, label: String, message: Message| {
            self.ring(
                focus::Target::Menu(id),
                free.then(|| message.clone()),
                button(text(label).size(theme::LABEL))
                    .style(theme::ghost)
                    .padding(theme::tap([theme::SM, theme::MD]))
                    .width(Length::Fill)
                    .on_press_maybe(free.then_some(message)),
            )
        };
        let menu = container(
            column![
                item("client-show", i18n::text(language, "menu-show-battlenet"), Message::ShowBattleNet),
                item("client-quit", i18n::text(language, "menu-quit-battlenet"), Message::AskQuitBattleNet),
            ]
            .spacing(theme::XS),
        )
        .padding(theme::XS)
        .width(Length::Fixed(200.0))
        .style(theme::floating);

        focus::layer(mouse_area(menu).on_press(Message::MenuClick))
    }

    /// The cog menu: what you do *about* a game, not to play it.
    pub(crate) fn gear_menu(&self) -> Element<'_, Message> {
        let language = &self.config.interface_language;
        let item = |id: &'static str, label: String, message: Message, enabled: bool| {
            self.ring(
                focus::Target::Menu(id),
                enabled.then(|| message.clone()),
                button(text(label).size(theme::LABEL))
                    .style(theme::ghost)
                    .padding(theme::tap([theme::SM, theme::MD]))
                    .width(Length::Fill)
                    .on_press_maybe(enabled.then_some(message)),
            )
        };

        let installed = self.local.version.is_some();
        let can = self.can_maintain();
        // Anything queued for this product can be given up.
        let queued = self.queue.iter().any(|d| d.code == self.selected.code);
        // Battle.net's order and wording: the game's own entries, then the
        // two that change the install below a rule.
        let menu = container(
            column![
                item("game-settings", i18n::text(language, "menu-game-settings"), Message::OpenGameSettings, installed),
                item("show-in-explorer", i18n::text(language, "menu-show-in-explorer"), Message::ShowInExplorer, installed),
                item("game-version", i18n::text(language, "menu-game-version"), Message::AskGameVersion, installed),
                item("scan-repair", i18n::text(language, "menu-scan-repair"), Message::ScanAndRepair, can.scan),
                item("check-updates", i18n::text(language, "menu-check-updates"), Message::CheckForUpdates, !self.locked()),
            ]
            .extend(
                self.owed_repair
                    .is_some()
                    .then(|| item("repair-files", i18n::text(language, "menu-repair-damaged-files"), Message::RepairDamaged, can.repair)))
            .extend(queued.then(|| {
                item(
                    "cancel-download",
                    i18n::text(language, "menu-cancel-download"),
                    Message::AskCancelDownload(self.selected.code.clone()),
                    true,
                )
            }))
            .push(container(Space::new()).height(Length::Fixed(1.0)).width(Length::Fill).style(theme::divider))
            .push(item("modify-install", i18n::text(language, "menu-modify-install"), Message::AskModifyInstall, can.scan))
            .push(item("uninstall", i18n::text(language, "common-uninstall"), Message::AskUninstall, can.uninstall))
            .push(item("copy-debug", i18n::text(language, "menu-copy-debug-info"), Message::CopyDebug, true))
            .spacing(theme::XS),
        )
        .padding(theme::XS)
        // The width of the split button beneath it, so the menu reads as
        // belonging to that control instead of floating near it.
        .width(Length::Fixed(240.0))
        .style(theme::floating);

        focus::layer(mouse_area(menu).on_press(Message::MenuClick))
    }
}
