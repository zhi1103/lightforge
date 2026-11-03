//! Installing and uninstalling: the install, downgrade, cancel and uninstall cards.

use crate::*;
use iced::widget::column;

impl Lightforge {
    /// The install dialog's card.
    pub(crate) fn install_card<'a>(&'a self, dialog: &'a InstallDialog) -> Element<'a, Message> {
        let language = &self.config.interface_language;
        let mut drives = column![].spacing(theme::SM);
        for (index, target) in dialog.targets.iter().enumerate() {
            drives = drives.push(install_target_row(index, target, index == dialog.chosen, language));
        }
        let location: Element<Message> = column![drives]
            .extend(
                self.hint(i18n::text(language, "dialog-install-location-hint"))
                .map(Element::from),
            )
            .spacing(theme::XS)
            .into();

        let mut body = column![field_row(i18n::text(language, "wizard-install-to"), location)].spacing(theme::MD);

        let mut ready = false;
        match &dialog.stage {
            DialogStage::Calculating => {
                body = body.push(muted(i18n::text(language, "dialog-calculating-size"), theme::LABEL));
            }
            DialogStage::Failed(e) => {
                body = body.push(
                    text(format!("⚠ {}: {e}", i18n::text(language, "dialog-build-read-failed")))
                        .size(theme::LABEL)
                        .color(theme::ERROR),
                );
            }
            DialogStage::Ready(offerings) => {
                let extra = offerings
                    .optional_bytes
                    .iter()
                    .find(|(tag, _)| tag == HIGH_RES)
                    .map(|(_, bytes)| *bytes)
                    // Every build names the tag; only some have textures
                    // behind it. Retail's adds nothing, and a "+0 MiB"
                    // switch is not a choice.
                    .filter(|bytes| *bytes > 0);
                if let Some(extra) = extra {
                    body = body.push(field_row(
                        i18n::text(language, "dialog-optional"),
                        row![
                            clickable(
                                "high res",
                                tick(dialog.high_res)
                                    .label(optional_label(HIGH_RES, &self.config.interface_language))
                                    .on_toggle(Message::InstallHighRes)
                                    .text_size(theme::tap_text(theme::BODY)),
                            ),
                            muted(format!("+{}", size_of(extra)), theme::LABEL),
                        ]
                        .spacing(theme::SM)
                        .align_y(Alignment::Center)
                        .into(),
                    ));
                }

                let need = offerings.base_bytes
                    + if dialog.high_res { extra.unwrap_or(0) } else { 0 };
                // Exact: priced against the store, the same test the plan
                // applies, so shared files are already left out. Download
                // size, not disk size, the extracted install files (the
                // executables and the like) come on top, which is most of
                // why Battle.net's "required" reads a little higher.
                body = body.push(field_row(
                    i18n::text(language, "dialog-download"),
                    mono(i18n::text_with_args(language, "dialog-download-size", &[("size", &size_of(need))])).into(),
                ));
                match dialog.target().and_then(|t| t.disk) {
                    Some(disk) => {
                        body = body.push(field_row(
                            i18n::text(language, "dialog-free-space"),
                            mono(size_of(disk.free)).into(),
                        ));
                        ready = disk.free > need;
                        if !ready {
                            body = body.push(
                                text(i18n::text(language, "dialog-not-enough-space"))
                                    .size(theme::LABEL)
                                    .color(theme::ERROR),
                            );
                        }
                    }
                    None => ready = true,
                }
            }
        }

        // A pair, so one size: the same height and width, labels centred,
        // with only the fill telling the default from the way out.
        let choice = |label: String| {
            button(text(label).size(theme::LABEL).width(Length::Fill).center())
                .width(Length::Fixed(150.0))
                .padding(theme::tap([theme::SM, theme::MD]))
        };
        let actions = row![
            Space::new().width(Length::Fill),
            tappable("cancel", Some(Message::InstallCancel), choice(i18n::text(language, "common-cancel")).style(theme::outlined)),
            tappable(
                "start install",
                ready.then_some(Message::InstallStart),
                choice(i18n::text(language, "dialog-start-install")).style(theme::filled(theme::INSTALL)),
            ),
        ]
        .spacing(theme::SM)
        .align_y(Alignment::Center);

        container(
            column![
                section(i18n::text_with_args(language, "dialog-install-game", &[("game", &dialog.label)]), body.into()),
                actions,
            ]
            .spacing(theme::LG),
        )
        .width(Length::Fixed(640.0))
        .padding(theme::LG)
        .style(theme::dialog)
        .into()
    }

    /// The card asking whether to open the shared prefix with an older build.
    ///
    /// Two buttons, no third option. "Launch anyway" is the one
    /// the user came for, so it is not hidden behind an explanation; the
    /// explanation is there because the cost falls on games they are not
    /// looking at.
    pub(crate) fn downgrade_card<'a>(&'a self, dialog: &'a DowngradeDialog) -> Element<'a, Message> {
        let language = &self.config.interface_language;
        let what = match dialog.what {
            Launch::Game => i18n::text_with_args(language, "dialog-launching-game", &[("game", &dialog.label)]),
            Launch::BattleNet => i18n::text_with_args(language, "dialog-starting-game-through-battlenet", &[("game", &dialog.label)]),
            Launch::Client => i18n::text(language, "dialog-opening-battlenet"),
            Launch::Install => i18n::text(language, "dialog-opening-install-flow"),
        };

        let body = column![
            field_row(i18n::text(language, "dialog-would-run"), mono(dialog.pick.clone()).into()),
            field_row(i18n::text(language, "dialog-prefix-built-by"), mono(dialog.built_by.clone()).into()),
            text(i18n::text_with_args(language, "dialog-downgrade-warning", &[("what", &what), ("pick", &dialog.pick), ("built_by", &dialog.built_by)]))
            .size(theme::LABEL)
            .color(theme::UPDATE),
            muted(
                i18n::text(language, "dialog-downgrade-explanation"),
                theme::LABEL,
            ),
        ]
        .spacing(theme::MD);

        let choice = |label: String| {
            button(text(label).size(theme::LABEL).width(Length::Fill).center())
                .width(Length::Fixed(170.0))
                .padding(theme::tap([theme::SM, theme::MD]))
        };
        let actions = row![
            Space::new().width(Length::Fill),
            tappable(
                "cancel",
                Some(Message::DowngradeCancel),
                choice(i18n::text(language, "common-cancel")).style(theme::outlined),
            ),
            tappable(
                "launch anyway",
                Some(Message::DowngradeConfirm),
                choice(i18n::text(language, "dialog-launch-anyway")).style(theme::filled(theme::UPDATE)),
            ),
        ]
        .spacing(theme::SM)
        .align_y(Alignment::Center);

        container(
            column![section(i18n::text(language, "dialog-older-proton-title"), body.into()), actions]
                .spacing(theme::LG),
        )
        .width(Length::Fixed(560.0))
        .padding(theme::LG)
        .style(theme::dialog)
        .into()
    }

    /// The confirmation for giving a download up.
    ///
    /// Two shapes, because an update and a first install are not the same
    /// question. An update's files are never removed: the game on disk plays
    /// right now, and what has already come down is a head start on the next
    /// attempt, not waste.
    ///
    /// One line of consequence under the title, and two buttons.
    pub(crate) fn cancel_card<'a>(&'a self, dialog: &'a CancelDialog) -> Element<'a, Message> {
        let language = &self.config.interface_language;
        let updating = dialog.installed.is_some();
        let (title, hint, keep, give_up) = match &dialog.installed {
            Some(version) => (
                i18n::text_with_args(language, "dialog-cancel-update-title", &[("game", &dialog.label)]),
                i18n::text_with_args(language, "dialog-cancel-update-body", &[("game", &dialog.label), ("version", version)]),
                i18n::text(language, "dialog-keep-downloading"),
                i18n::text(language, "dialog-cancel-update"),
            ),
            None => (
                i18n::text_with_args(language, "dialog-cancel-install-title", &[("game", &dialog.label)]),
                i18n::text(language, "dialog-cancel-install-body"),
                i18n::text(language, "dialog-keep-installing"),
                i18n::text(language, "dialog-cancel-install"),
            ),
        };

        let body = column![muted(hint, theme::BODY)]
            .spacing(theme::SM)
            .width(Length::Fixed(520.0));

        // One shape for both, so neither reads as the safe one by being
        // bigger. Only the fill says which is which.
        let choice = |label: String| {
            button(text(label).size(theme::LABEL).width(Length::Fill).center())
                .width(Length::Fixed(170.0))
                .padding(theme::tap([theme::SM, theme::MD]))
        };
        let actions = row![
            Space::new().width(Length::Fill),
            tappable("keep", Some(Message::CancelDownloadDismiss), choice(keep).style(theme::outlined)),
            tappable(
                "give up",
                Some(Message::CancelDownloadConfirm { delete: !updating }),
                choice(give_up).style(theme::filled(theme::ERROR)),
            ),
        ]
        .spacing(theme::SM)
        .align_y(Alignment::Center);

        container(column![section(title, body.into()), actions].spacing(theme::LG))
            .width(Length::Fixed(520.0))
            .padding(theme::LG)
            .style(theme::dialog)
            .into()
    }

    /// The uninstall confirmation's card.
    pub(crate) fn uninstall_card<'a>(&'a self, dialog: &'a UninstallDialog) -> Element<'a, Message> {
        let language = &self.config.interface_language;
        let mut body = column![].spacing(theme::MD);
        let mut ready = false;
        match &dialog.plan {
            None => {
                body = body.push(muted(i18n::text(language, "dialog-uninstall-planning"), theme::LABEL));
            }
            Some(Err(e)) => {
                body = body.push(text(format!("⚠ {e}")).size(theme::LABEL).color(theme::ERROR));
            }
            Some(Ok(preview)) if preview.plan.is_empty() => {
                body = body.push(muted(
                    i18n::text(language, "dialog-uninstall-empty"),
                    theme::LABEL,
                ));
            }
            Some(Ok(preview)) => {
                ready = true;
                let plan = &preview.plan;
                let reclaim = preview.reclaim.as_ref().and_then(|r| r.as_ref().ok());
                let deletes = column(
                    plan.flavor_dir
                        .iter()
                        .chain(plan.store.iter())
                        .map(|path| mono(path.display().to_string()).into()),
                )
                .spacing(theme::XS);
                body = body
                    .push(field_row(i18n::text(language, "dialog-deletes"), deletes.into()))
                    .push(field_row(
                        i18n::text(language, "dialog-frees"),
                        mono(size_of(plan.bytes + reclaim.map_or(0, |r| r.freed))).into(),
                    ));

                let (note, color) = match plan.is_whole_install() {
                    true => (
                        i18n::text(language, "dialog-uninstall-last-version"),
                        theme::ERROR,
                    ),
                    false => {
                        let others = plan
                            .remaining
                            .iter()
                            .map(|c| self.label_of(c))
                            .collect::<Vec<_>>()
                            .join(", ");
                        let note = match &preview.reclaim {
                            // Only what nobody else uses goes; an archive
                            // it shares with them keeps its bytes as dead
                            // space until something rewrites it.
                            Some(Ok(r)) if r.stranded > 0 => i18n::text_with_args(language, "dialog-uninstall-shared-stranded", &[("games", &others), ("size", &size_of(r.stranded))]),
                            Some(Ok(_)) => i18n::text_with_args(language, "dialog-uninstall-shared", &[("games", &others)]),
                            _ => i18n::text_with_args(language, "dialog-uninstall-shared-unknown", &[("games", &others)]),
                        };
                        (note, theme::MUTED)
                    }
                };
                body = body.push(text(note).size(theme::LABEL).color(color));
                if let Some(r) = reclaim.filter(|r| r.moved > 0) {
                    let line = match dialog.working {
                        true => i18n::text_with_args(language, "dialog-uninstall-compacting", &[
                            ("copied", &size_of(dialog.copied.load(std::sync::atomic::Ordering::Relaxed).min(r.moved))),
                            ("total", &size_of(r.moved)),
                        ]),
                        false => i18n::text_with_args(language, "dialog-uninstall-copying-first", &[("size", &size_of(r.moved))]),
                    };
                    body = body.push(muted(line, theme::LABEL));
                }
                body = body.push(muted(
                    i18n::text(language, "dialog-uninstall-addon-settings-note"),
                    theme::LABEL,
                ));
            }
        }

        let choice = |label: String| {
            button(text(label).size(theme::LABEL).width(Length::Fill).center())
                .width(Length::Fixed(150.0))
                .padding(theme::tap([theme::SM, theme::MD]))
        };
        let actions = row![
            Space::new().width(Length::Fill),
            tappable(
                "cancel",
                (!dialog.working).then_some(Message::UninstallCancel),
                choice(i18n::text(language, "common-cancel")).style(theme::outlined),
            ),
            tappable(
                "uninstall",
                (ready && !dialog.working).then_some(Message::UninstallConfirm),
                choice(if dialog.working { i18n::text(language, "dialog-uninstalling") } else { i18n::text(language, "common-uninstall") })
                    .style(theme::filled(theme::ERROR)),
            ),
        ]
        .spacing(theme::SM)
        .align_y(Alignment::Center);

        container(
            column![section(i18n::text_with_args(language, "dialog-uninstall-game", &[("game", &dialog.label)]), body.into()), actions]
                .spacing(theme::LG),
        )
        .width(Length::Fixed(560.0))
        .padding(theme::LG)
        .style(theme::dialog)
        .into()
    }
}

/// One drive the install can go on: its name and path, its free space, and
/// whether it would share a data store with what is already there.
fn install_target_row<'a>(index: usize, target: &'a InstallTarget, chosen: bool, language: &str) -> Element<'a, Message> {
    // Fixed, and wide enough for the longest thing that goes in it.
    // Left to shrink it was squeezed by the path beside it until
    // "1.37 TiB free" came out as three stacked words.
    let mut right = column![]
        .spacing(2)
        .width(Length::Fixed(110.0))
        .align_x(Alignment::End);
    right = match target.disk {
        Some(disk) => right.push(mono(i18n::text_with_args(language, "dialog-drive-free", &[("size", &size_of(disk.free))]))),
        None => right.push(muted("—".to_owned(), theme::LABEL)),
    };
    // Battle.net's own wording and colour for the same fact: a second
    // WoW version in a folder that already has one shares its data
    // store, so what is already downloaded is not fetched twice.
    if target.shares {
        right = right.push(text(i18n::text(language, "dialog-sharing-data")).size(theme::CAPTION).color(theme::OK));
    }

    let mut left = column![row![
        text(target.label.clone()).size(theme::BODY),
        Space::new().width(Length::Fixed(theme::SM)),
    ]
    .align_y(Alignment::Center)]
    .spacing(2);
    // `~/Games/...`, not `/home/<name>/Games/...`: the home
    // prefix is the longest and least informative part of the path,
    // and this row is where the width is tightest.
    left = left.push(muted(home_relative(&target.install), theme::CAPTION));
    if let Some(reason) = i18n::library_state_reason(language, target.state, &target.label) {
        left = left.push(text(reason).size(theme::CAPTION).color(theme::ERROR));
    }

    let usable = target.state != library::State::Missing;
    tappable(
        format!("install target {index}"),
        usable.then_some(Message::InstallTarget(index)),
        button(
            // The left column takes what the right one does not, so
            // the path is the only thing that ever wraps.
            row![left.width(Length::Fill), right]
                .spacing(theme::MD)
                .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .padding([theme::SM, theme::MD])
        .style(match chosen {
            true => theme::selected,
            false => theme::card_button,
        }),
    )
}
