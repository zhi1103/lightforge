//! The installed addons: one row each, with its channel, pin and update state.

use crate::*;
use iced::widget::column;

impl Lightforge {
    pub(crate) fn addons_mine(&self) -> Element<'_, Message> {
        let language = &self.config.interface_language;
        let outdated = self.updates.iter().filter(|u| u.has_update()).count();

        let mut body = column![self.mine_toolbar(outdated)].spacing(theme::SM);

        // Only when something is unmanaged.
        let unmanaged = self
            .addons
            .rows
            .iter()
            .filter(|row| row.managed.is_none() && row.child_of.is_none())
            .count();

        if unmanaged > 0 {
            body = body.push(self.adopt_offer(unmanaged));
        }

        // A header, because four right-hand columns of bare dates are not
        // self-describing. Indented and spaced to match a row's own columns.
        if !self.addons.rows.is_empty() {
            body = body.push(mine_header(language));
        }

        // Two providers writing one folder undo each other's updates.
        for (folder, providers) in &self.addons.shared {
            body = body.push(
                text(format!(
                    "{folder} is installed from both {}: an update from one overwrites the other. Remove one of them.",
                    providers.join(" and ")
                ))
                .size(theme::CAPTION)
                .color(theme::WARN),
            );
        }

        // What the unattended pass did, said only here.
        if let Some(note) = &self.addon_auto_note {
            body = body.push(muted(note.clone(), theme::CAPTION));
        }

        body = body.extend(self.unavailable_rows(AddonTab::Mine));

        if self.addons.rows.is_empty() {
            body = body.push(empty(
                &i18n::text(language, "addons-mine-empty"),
                &i18n::text(language, "addons-mine-empty-hint"),
            ));
        }

        // Outdated first, then by name.
        let needle = self.addon_filter.trim().to_lowercase();
        let mut shown: Vec<&_> = self
            .addons
            .rows
            .iter()
            // A package's other folders belong to the addon above them;
            // showing BigWigs' eleven folders as eleven addons would be a lie.
            .filter(|addon| addon.child_of.is_none())
            .filter(|addon| {
                needle.is_empty()
                    || addon.title.to_lowercase().contains(&needle)
                    || addon.folder.to_lowercase().contains(&needle)
            })
            .collect();
        shown.sort_by_cached_key(|addon| {
            let outdated = self.update_status(addon).is_some_and(|s| s.has_update());
            (!outdated, addon.title.to_lowercase())
        });

        if !needle.is_empty() && shown.is_empty() && !self.addons.rows.is_empty() {
            body = body.push(empty(
                &i18n::text_with_args(language, "addons-mine-no-filter-match", &[("filter", self.addon_filter.trim())]),
                &i18n::text(language, "addons-mine-clear-filter-hint"),
            ));
        }

        for addon in shown {
            body = body.push(self.mine_row(addon));
        }

        // Where all of this lands, kept quiet: "Open folder" in the toolbar is
        // the way there, and this is for the times you need the path itself.
        body = body.push(muted(self.addons.dir.display().to_string(), theme::CAPTION));
        body.into()
    }

    /// The filter, the update count, and the three buttons over the list.
    fn mine_toolbar(&self, outdated: usize) -> Element<'_, Message> {
        let language = &self.config.interface_language;
        // Skipped by the check and by Update all; said, or a ledger of pins
        // reads as "no updates" (#31).
        let pinned = self
            .addons
            .rows
            .iter()
            .filter(|row| row.child_of.is_none() && row.managed.as_ref().is_some_and(|m| m.pinned.is_some()))
            .count();
        let checked = match (self.updates.is_empty(), outdated) {
            (true, _) => None,
            (false, 0) if pinned > 0 => Some(i18n::text(language, "addons-rest-up-to-date")),
            (false, 0) => Some(i18n::text(language, "addons-everything-up-to-date")),
            (false, n) => Some(i18n::text_with_args(language, "addons-updates-available", &[("count", &n.to_string())])),
        };
        let summary = match (checked, pinned) {
            (None, _) => String::new(),
            (Some(checked), 0) => checked,
            (Some(_), n) if n == self.addons.managed => i18n::text_with_args(language, "addons-pinned-not-checked", &[("count", &n.to_string())]),
            (Some(checked), n) => i18n::text_with_args(language, "addons-pinned-and-checked", &[("count", &n.to_string()), ("checked", &checked)]),
        };
        container(
            row![
                // Sized, or it reads as a search.
                clickable(
                    "addon filter",
                    text_input(&i18n::text(language, "addons-filter-placeholder"), &self.addon_filter)
                        .on_input(Message::AddonFilterChanged)
                        .style(theme::field)
                        .size(theme::BODY)
                        .padding(theme::SM)
                        .width(Length::Fixed(260.0)),
                ),
                Space::new().width(Length::Fill),
                muted(summary, theme::LABEL),
                // One shape for the three; blue is the one to press.
                action_button(&i18n::text(language, "addons-check-updates"), theme::PAUSE, Message::CheckAddonUpdates)
                    .enabled(!self.addons_locked() && self.addons.managed > 0),
                action_button(&i18n::text(language, "addons-update-all"), theme::ACCENT, Message::UpdateAddons)
                    .enabled(!self.addons_locked() && self.addons.managed > 0),
                action_button(&i18n::text(language, "settings-storage-open-folder"), theme::PAUSE, Message::OpenAddonsDir)
                    .enabled(true),
            ]
            .push((pinned > 0).then(|| {
                secondary(&i18n::text(language, "addons-unpin-all"), Message::UnpinAddons).enabled(!self.addons_locked())
            }))
            .spacing(theme::SM)
            .align_y(Alignment::Center),
        )
        .padding(theme::MD)
        .style(theme::card)
        .into()
    }

    /// The offer to take in addons installed by hand.
    fn adopt_offer(&self, unmanaged: usize) -> Element<'_, Message> {
        let language = &self.config.interface_language;
        container(
            row![
                column![
                    text(match unmanaged {
                        1 => i18n::text(language, "addons-manual-one"),
                        n => i18n::text_with_args(language, "addons-manual-many", &[("count", &n.to_string())]),
                    })
                    .size(theme::BODY),
                    muted(
                        i18n::text(language, "addons-recognise-about"),
                        theme::CAPTION,
                    ),
                ]
                .spacing(theme::XS)
                .width(Length::Fill),
                action_button(&i18n::text(language, "addons-recognise"), theme::ACCENT, Message::AdoptAddons)
                    .enabled(!self.addons_locked()),
            ]
            .spacing(theme::SM)
            .align_y(Alignment::Center),
        )
        .padding(theme::MD)
        .style(theme::card)
        .into()
    }

    /// One installed addon: its name and state, the three dated columns,
    /// and what can be done to it.
    fn mine_row<'a>(&'a self, addon: &'a addonwork::Row) -> Element<'a, Message> {
        let language = &self.config.interface_language;
        let extra = self
            .addons
            .rows
            .iter()
            .filter(|r| r.child_of.as_deref() == Some(addon.folder.as_str()))
            .count();
        let status = self.update_status(addon);
        // Its own folders' needs too: they are listed under it, not apart.
        let mut missing: Vec<&String> = self
            .addons
            .rows
            .iter()
            .filter(|r| r.folder == addon.folder || r.child_of.as_deref() == Some(addon.folder.as_str()))
            .flat_map(|r| &r.missing)
            .collect();
        missing.sort();
        missing.dedup();

        let mut words = column![
            self.addon_heading(addon, status),
            muted(
                match (&addon.managed, extra) {
                    (Some(m), 0) => i18n::text_with_args(language, "addons-managed-provider", &[("provider", &m.provider)]),
                    (Some(m), n) => i18n::text_with_args(language, "addons-managed-provider-folders", &[("provider", &m.provider), ("count", &n.to_string())]),
                    (None, _) => i18n::text(language, "addons-installed-by-hand"),
                },
                theme::CAPTION,
            ),
        ]
        .spacing(theme::XS)
        .width(Length::Fill);
        for dep in missing {
            words = words.push(
                row![
                    text(i18n::text_with_args(language, "addons-needs-dependency", &[("dependency", dep)])).size(theme::CAPTION).color(theme::WARN),
                    secondary(&i18n::text(language, "addons-find"), Message::FindAddon(dep.clone())).enabled(true),
                ]
                .spacing(theme::SM)
                .align_y(Alignment::Center),
            );
        }

        let mut line = row![words]
        .spacing(theme::SM)
        .align_y(Alignment::Center);

        // A dash for an unknown fact; a blank reads as a layout bug.
        let managed_row = addon.managed.as_ref();
        line = line.push(cell(
            status
                .and_then(|s| s.available_name.as_deref())
                .map(|name| release_version(name, &addon.title))
                .or_else(|| {
                    // Up to date: what is latest is what is installed, and
                    // saying so beats an empty column.
                    managed_row.map(|_| addon.version.clone())
                }),
        ));
        line = line.push(cell(
            status.and_then(|s| s.available_date.as_deref().map(short_day)),
        ));
        line = line.push(cell(
            managed_row.and_then(|m| m.installed_at.as_deref().map(short_day)),
        ));
        // A fixed-width box for the controls, even an empty one, so the
        // columns line up with the header.
        let actions: Element<'a, Message> = match &addon.managed {
            Some(managed) => self.addon_actions(addon, managed, status),
            None => row![].into(),
        };
        line = line.push(
            container(actions)
                .width(Length::Fixed(ADDON_ACTIONS))
                .align_x(Alignment::End),
        );

        container(line).padding(theme::MD).style(theme::row).into()
    }

    /// The title and version, and a pill for whatever is unusual about it.
    fn addon_heading<'a>(
        &self,
        addon: &'a addonwork::Row,
        status: Option<&'a lightforge_core::addons::manage::UpdateStatus>,
    ) -> Element<'a, Message> {
        let language = &self.config.interface_language;
        let mut heading = row![text(addon.title.clone()).size(theme::BODY), mono(addon.version.clone())]
            .spacing(theme::SM)
            .align_y(Alignment::Center);
        match addon.managed.as_ref() {
            // Said once, in the place an update would be announced: this
            // row is why "Update all" leaves something behind.
            Some(m) if m.ignored => {
                heading = heading.push(pill(i18n::text(language, "addons-ignored"), theme::MUTED));
            }
            // The version it is held at is already beside the title, so
            // the pill says the state and not the number twice.
            Some(m) if m.pinned.is_some() => {
                heading = heading.push(pill(i18n::text(language, "addons-pinned"), theme::MUTED));
            }
            // Only when it differs from the default.
            Some(m) if !m.auto_update.unwrap_or(self.config.addon_auto_update) => {
                heading = heading.push(pill(i18n::text(language, "addons-manual"), theme::MUTED));
            }
            _ => {}
        }
        // What the game would say about it, which an update may not change.
        if addon.out_of_date {
            heading = heading.push(pill(i18n::text(language, "addons-out-of-date"), theme::WARN));
        }
        if let Some(status) = status.filter(|s| s.has_update()) {
            // The provider's version name; a channel switch says so instead.
            heading = heading.push(match status.is_channel_switch() {
                true => pill(
                    i18n::text_with_args(language, "addons-build-label", &[(
                        "build",
                        status.available_build.map(|c| c.as_str()).unwrap_or("different"),
                    )]),
                    theme::ACCENT,
                ),
                false => pill(
                    format!(
                        "→ {}",
                        status
                            .available_name
                            .as_deref()
                            .map(|name| release_version(name, &addon.title))
                            .or_else(|| status.available.clone())
                            .unwrap_or_default()
                    ),
                    theme::UPDATE,
                ),
            });
        }
        heading.into()
    }

    /// The channel picker, Update, Remove, and the menu with the rest.
    fn addon_actions<'a>(
        &'a self,
        addon: &'a addonwork::Row,
        managed: &'a addonwork::Managed,
        status: Option<&'a lightforge_core::addons::manage::UpdateStatus>,
    ) -> Element<'a, Message> {
        let language = &self.config.interface_language;
        let mut actions = row![].spacing(theme::SM).align_y(Alignment::Center);
        // Which builds this addon accepts: first, fixed width, button height.
        actions = actions.push(self.picker(
            format!("channel {} {}", managed.provider, managed.id),
            RowChannel::ALL.map(|choice| DisplayChoice(choice, choice.label(language))).to_vec(),
            Some(DisplayChoice(RowChannel(managed.channel), RowChannel(managed.channel).label(language))),
            "",
            |choice| Message::AddonChannelFor(managed.provider.clone(), managed.id.clone(), choice.0),
            Length::Fixed(120.0),
            theme::LABEL,
        ));

        // One addon at a time. `install_one` fetches the newest file
        // and does not pin it, which is exactly what updating is, so
        // this is the same path "Update all" takes for every row.
        //
        // A pin is a statement about which version this addon is on,
        // so the row stops offering to move it, unlike Ignore, which
        // only means "not in bulk". Unpinning brings the button back.
        let pinned = managed.pinned.is_some();
        if let Some(status) = status.filter(|s| s.has_update() && !pinned) {
            // "Update" would be a lie when the channel changed: going
            // back to stable from an alpha installs an *older* build,
            // which is right, but not what the word promises.
            let label = match status.is_channel_switch() {
                true => i18n::text(language, "addons-switch-channel"),
                false => i18n::text(language, "addons-update"),
            };
            actions = actions.push(
                action_button(
                    &label,
                    theme::UPDATE,
                    Message::InstallAddon(
                        managed.provider.clone(),
                        managed.id.clone(),
                    ),
                )
                .enabled(!self.addons_locked()),
            );
        }

        // Two clicks, because removing deletes the addon's folders off
        // disk and there is no undo. The second button is the way out,
        // so the destructive one is never the only thing to press.
        let armed = self.addon_remove_armed.as_ref().is_some_and(|(p, id)| {
            p == &managed.provider && id == &managed.id
        });
        // Remove and the Delete it turns into are one place for the
        // ring, which stays on it while the question is asked.
        let remove = format!("addon remove {} {}", managed.provider, managed.id);
        actions = match armed {
            true => actions
                .push(
                    action_button(
                        &i18n::text(language, "common-delete"),
                        theme::ERROR,
                        Message::RemoveAddon(
                            managed.provider.clone(),
                            managed.id.clone(),
                        ),
                    )
                    .keyed(remove, !self.addons_locked()),
                )
                .push(
                    secondary(&i18n::text(language, "addons-keep"), Message::CancelRemoveAddon).enabled(true),
                ),
            false => actions.push(
                secondary(
                    &i18n::text(language, i18n::COMMON_REMOVE),
                    Message::ArmRemoveAddon(
                        managed.provider.clone(),
                        managed.id.clone(),
                    ),
                )
                .keyed(remove, !self.addons_locked()),
            ),
        };

        // Everything that is a setting, not an action, lives
        // behind this, so the row keeps its width as the settings
        // grow. Only one menu is open at a time, and the panel hangs
        // off the button itself, not the row's corner.
        let key = (managed.provider.clone(), managed.id.clone());
        let open = self.addon_menu_open.as_ref() == Some(&key);
        let toggle = Message::AddonMenu(match open {
            true => None,
            false => Some(key.clone()),
        });
        actions = actions.push(popover::popover(
            focus::ring(
                focus::Target::Key(format!("addon menu {} {}", managed.provider, managed.id)),
                focus::Press::Send(toggle.clone()),
                button(
                    image(ellipsis_icon())
                        .width(Length::Fixed(MENU_GLYPH))
                        .height(Length::Fixed(MENU_GLYPH)),
                )
                .padding(theme::SM)
                .style(theme::ghost)
                .on_press(toggle),
            ),
            open.then(|| self.addon_row_menu(managed, &addon.version, &addon.title)),
        ));
        actions.into()
    }
}

/// The column names over the list, indented and spaced to match a row.
fn mine_header(language: &str) -> Element<'static, Message> {
    container(
        row![
            muted(crate::i18n::text(language, "addons-column-name"), theme::CAPTION).width(Length::Fill),
            column_head(&crate::i18n::text(language, "addons-column-latest")),
            column_head(&crate::i18n::text(language, "addons-column-released")),
            column_head(&crate::i18n::text(language, "addons-column-installed")),
            // Standing in for the controls each row carries, so
            // the three headings sit over their own columns,
            // not over the buttons.
            Space::new().width(Length::Fixed(ADDON_ACTIONS)),
        ]
        .spacing(theme::SM)
        .align_y(Alignment::Center),
    )
    .padding([0.0, theme::MD])
    .into()
}
