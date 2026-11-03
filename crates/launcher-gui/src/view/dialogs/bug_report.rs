//! The bug report card, its screenshot row, and what gets attached.

use crate::*;
use iced::widget::column;

impl Lightforge {
    /// What a bug report in this category carries beyond the base bundle.
    ///
    /// The addon list, the queue and drives, or the settings; read when the
    /// category is picked, shown, and scrubbed.
    pub(crate) fn report_attachment(&self, category: report::Category) -> String {
        use std::fmt::Write as _;
        use report::Category;

        let code = &self.selected.code;
        let mut out = String::new();
        match category {
            Category::Addons => {
                let dir = self.layout.addons_dir(code);
                let _ = writeln!(out, "\n--- {}: {} ---", i18n::text(&self.config.interface_language, "report-attachment-addons"), dir.display());
                let _ = writeln!(out, "{}: {}", i18n::text(&self.config.interface_language, "report-addon-flavor"), lightforge_core::addons::flavor_of(code));
                let installed = match lightforge_core::addons::scan(&dir, &lightforge_core::addons::Client::default()) {
                    Ok(installed) => installed,
                    Err(e) => {
                        let _ = writeln!(out, "{}", i18n::text_with_args(&self.config.interface_language, "report-addon-folder-error", &[("error", &chain(&e))]));
                        Vec::new()
                    }
                };
                // Read-only: the Addons page prunes the ledger as it lists, a
                // report must not change what it is reporting on.
                let ledger = AppDirs::discover()
                    .ok()
                    .map(|dirs| manage::ledger_path(dirs.data_dir(), &dir, code))
                    .map(|path| lightforge_core::addons::Ledger::load(&path));
                let ledger = match ledger {
                    Some(Ok(ledger)) => ledger,
                    Some(Err(e)) => {
                        let _ = writeln!(out, "{}", i18n::text_with_args(&self.config.interface_language, "report-addon-ledger-error", &[("error", &chain(&e))]));
                        lightforge_core::addons::Ledger::default()
                    }
                    None => lightforge_core::addons::Ledger::default(),
                };
                let rows = addonwork::rows_from(&installed, &ledger);
                let managed = rows.iter().filter(|r| r.managed.is_some() && r.child_of.is_none()).count();
                let _ = writeln!(out, "{}", i18n::text_with_args(&self.config.interface_language, "report-addon-folders", &[("count", &rows.len().to_string()), ("managed", &managed.to_string())]));
                for row in rows.iter().filter(|r| r.child_of.is_none()) {
                    let owner = match &row.managed {
                        Some(m) => {
                            let mut owner = format!("{}:{}", m.provider, m.id);
                            if let Some(file) = &m.file_id {
                                let _ = write!(owner, " file {file}");
                            }
                            if let Some(channel) = m.channel {
                                let _ = write!(owner, " {}", channel.as_str());
                            }
                            if m.pinned.is_some() {
                                owner.push_str(" pinned");
                            }
                            if m.ignored {
                                owner.push_str(" ignored");
                            }
                            owner
                        }
                        None => i18n::text(&self.config.interface_language, "report-addon-not-managed"),
                    };
                    let children = rows
                        .iter()
                        .filter(|r| r.child_of.as_deref() == Some(row.folder.as_str()))
                        .count();
                    let children = match children {
                        0 => String::new(),
                        n => format!(
                            " ({})",
                            i18n::text_with_args(
                                &self.config.interface_language,
                                "report-addon-child-folders",
                                &[("count", &n.to_string())],
                            )
                        ),
                    };
                    let _ = writeln!(out, "{}{children}  {}  {owner}", row.folder, row.version);
                }
            }
            Category::Downloader => {
                let _ = writeln!(out, "\n--- {} ---", i18n::text(&self.config.interface_language, "report-attachment-downloads"));
                if self.queue.is_empty() {
                    let _ = writeln!(out, "{}", i18n::text(&self.config.interface_language, "report-downloads-empty"));
                }
                for download in &self.queue {
                    let stage = match &download.stage {
                        Stage::Queued => i18n::text(&self.config.interface_language, "report-download-queued"),
                        Stage::Running { .. } => i18n::text(&self.config.interface_language, "report-download-running"),
                        Stage::Paused(p) => {
                            i18n::text_with_args(&self.config.interface_language, "report-download-paused", &[("done", &size_of(p.downloaded_bytes)), ("total", &size_of(p.total_bytes))])
                        }
                    };
                    let planned = download.job.total.map_or_else(|| i18n::text(&self.config.interface_language, "report-download-not-planned"), size_of);
                    let _ = writeln!(out, "{} ({})  {stage}  job {planned}", download.label, download.code.as_str());
                }
                let _ = writeln!(out, "\n--- {} ---", i18n::text(&self.config.interface_language, "report-attachment-libraries"));
                for library in &self.config.libraries {
                    let disk = library.disk().map_or_else(|| i18n::text(&self.config.interface_language, "report-library-size-unavailable"), |d| {
                        i18n::text_with_args(&self.config.interface_language, "report-library-free", &[("free", &size_of(d.free)), ("total", &size_of(d.total))])
                    });
                    let _ = writeln!(
                        out,
                        "{} {}  {}  {:?}  {disk}",
                        library.id,
                        library.label,
                        library.games().display(),
                        library.state()
                    );
                }
            }
            Category::Settings => {
                // The contact is in the report already, where it belongs.
                // Env values can be tokens: a flag like `1` or `+fps` helps
                // with the bug, anything longer or with a path stays here.
                let mut config = self.config.clone();
                config.contact = None;
                let flag = |value: &str| {
                    value.len() <= 16
                        && value.chars().all(|c| c.is_ascii_alphanumeric() || "_.,=:+-".contains(c))
                };
                let tweaks = std::iter::once(&mut config.battle_net_tweaks)
                    .chain(config.products.iter_mut().map(|p| &mut p.profile.tweaks));
                for value in tweaks.flat_map(|t| t.env.values_mut()) {
                    if !flag(value) {
                        *value = "(hidden)".to_owned();
                    }
                }
                let _ = writeln!(out, "\n--- config.toml ---");
                match config.to_toml() {
                    Ok(toml) => out.push_str(&toml),
                    Err(e) => {
                        let _ = writeln!(out, "{}", i18n::text_with_args(&self.config.interface_language, "report-addon-render-error", &[("error", &chain(&e))]));
                    }
                }
            }
            // The base bundle covers it. Battle.net's log names the account,
            // so it stays on the machine.
            Category::Game | Category::BattleNet | Category::Launcher | Category::General => {}
        }
        lightforge_core::debug::scrub(&out)
    }

    /// The bug report and the feedback dialog, one card: they differ in the
    /// prompt and in whether the bundle is shown and sent.
    pub(crate) fn report_card<'a>(&'a self, dialog: &'a ReportDialog) -> Element<'a, Message> {
        let language = &self.config.interface_language;
        let (title, prompt) = match dialog.kind {
            ReportKind::Bug => (i18n::text(language, "report-title-bug"), i18n::text(language, "report-prompt-bug")),
            ReportKind::Feedback => {
                (i18n::text(language, "app-menu-feedback"), i18n::text(language, "report-prompt-feedback"))
            }
        };
        // Editable right up until it is on its way: a report that failed, or
        // one the cooldown refused, is one somebody is about to change.
        let writing = !matches!(dialog.stage, ReportStage::Sending | ReportStage::Sent);

        let mut body = column![
            self.picker(
                i18n::text(language, "report-category-label"),
                report::Category::ALL.into_iter().map(|category| ReportCategoryLabel(category, i18n::text(language, category.message_id()))).collect(),
                dialog.category.map(|category| ReportCategoryLabel(category, i18n::text(language, category.message_id()))),
                i18n::text(language, "report-category-placeholder").as_str(),
                |option| Message::ReportCategory(option.0),
                Length::Fill,
                theme::BODY,
            ),
            // Five lines, and still fits a laptop. Escape leaves the field,
            // a second closes the card.
            clickable(
                "report note",
                text_editor(&dialog.note)
                    .placeholder(prompt)
                    .on_action(Message::ReportNote)
                    .height(Length::Fixed(theme::BODY * 1.4 * 5.0 + theme::SM * 2.0))
                    .style(theme::editor)
                    .size(theme::BODY)
                    .padding(theme::SM),
            ),
            // Required on a bug, said in the box; feedback can be anonymous.
            clickable(
                "report contact",
                text_input(
                    &if dialog.kind == ReportKind::Bug {
                        i18n::text(language, "report-contact-required")
                    } else {
                        i18n::text(language, "report-contact-optional")
                    },
                    &dialog.contact,
                )
                    .on_input_maybe(writing.then_some(Message::ReportContact))
                    .style(theme::field)
                    .size(theme::BODY)
                    .padding(theme::SM),
            ),
        ]
        .spacing(theme::MD);

        body = match dialog.kind {
            // Shown in full before sending, so the promise can be checked.
            ReportKind::Bug => body
                .push(muted(
                    i18n::text(language, "report-bug-data-promise"),
                    theme::LABEL,
                ))
                .push(
                    container(
                        scrollable(mono(dialog.debug.clone()))
                            .direction(scrollable::Direction::Vertical(theme::scrollbar_shape()))
                            .height(Length::Fixed(200.0))
                            .width(Length::Fill),
                    )
                    .padding(theme::SM)
                    .style(theme::row),
                )
                .push(self.shot_row(dialog)),
            ReportKind::Feedback => body.push(muted(
                i18n::text(language, "report-feedback-promise"),
                theme::LABEL,
            )),
        };

        if let Some((note, color)) = match dialog.stage {
            ReportStage::Sent => Some((i18n::text(language, "report-sent"), theme::OK)),
            ReportStage::Failed => Some((
                i18n::text(language, "report-send-failed"),
                theme::ERROR,
            )),
            ReportStage::TooSoon => Some((
                i18n::text(language, "report-too-soon"),
                theme::PLAY,
            )),
            _ => None,
        } {
            body = body.push(text(note).size(theme::LABEL).color(color));
        }

        let actions = report_actions(dialog, writing, language);

        container(column![section(title, body.into()), actions].spacing(theme::LG))
            .width(Length::Fixed(560.0))
            .padding(theme::LG)
            .style(theme::dialog)
            .into()
    }

    /// The screenshot line of the bug report card.
    ///
    /// A button for the desktop's file dialog, through the XDG portal.
    pub(crate) fn shot_row<'a>(&self, dialog: &'a ReportDialog) -> Element<'a, Message> {
        let language = &self.config.interface_language;
        let choose = |label: String| {
            tappable(
                "screenshot",
                Some(Message::ReportShotPick),
                button(text(label).size(theme::LABEL))
                    .padding(theme::tap([theme::XS, theme::SM]))
                    .style(theme::outlined),
            )
        };

        let Some(shot) = &dialog.shot else {
            return row![
                choose(i18n::text(language, "report-add-screenshot")),
                muted(i18n::text(language, "report-screenshot-optional"), theme::LABEL),
            ]
            .spacing(theme::SM)
            .align_y(Alignment::Center)
            .into();
        };

        let (width, height) = shot.size;
        row![
            container(
                image(shot.preview.clone())
                    .height(Length::Fixed(72.0))
                    .content_fit(ContentFit::Contain),
            )
            .padding(theme::XS)
            .style(theme::row),
            column![
                text(shot.name.clone()).size(theme::LABEL),
                muted(format!("{width}×{height}, {}", size_of(shot.bytes.len() as u64)), theme::CAPTION),
            ]
            .spacing(theme::XS)
            .width(Length::Fill),
            choose(i18n::text(language, "report-replace-screenshot")),
            tappable(
                "remove screenshot",
                Some(Message::ReportShotCleared),
                button(text(i18n::text(language, i18n::COMMON_REMOVE)).size(theme::LABEL))
                    .padding(theme::tap([theme::XS, theme::SM]))
                    .style(theme::outlined),
            ),
        ]
        .spacing(theme::SM)
        .align_y(Alignment::Center)
        .into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReportCategoryLabel(report::Category, String);

impl std::fmt::Display for ReportCategoryLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.1)
    }
}

/// Cancel, Send, and Copy once a send has failed; Close once it has arrived.
fn report_actions<'a>(dialog: &'a ReportDialog, writing: bool, language: &str) -> Element<'a, Message> {
    let choice = |label: String| {
        button(text(label).size(theme::LABEL).width(Length::Fill).center())
            .width(Length::Fixed(150.0))
            .padding(theme::tap([theme::SM, theme::MD]))
    };
    // Once it has arrived there is nothing left to cancel, so the card
    // offers one button that says what it does.
    match dialog.stage {
        ReportStage::Sent => row![
            Space::new().width(Length::Fill),
            tappable(
                "close",
                Some(Message::ReportCancel),
                choice(crate::i18n::text(language, "common-close")).style(theme::filled(theme::ACCENT)),
            ),
        ],
        _ => row![
            Space::new().width(Length::Fill),
            // Only once there is something to lose; text only.
            match dialog.stage {
                ReportStage::Failed => tappable(
                    "copy report",
                    Some(Message::ReportCopy),
                    choice(crate::i18n::text(language, "report-copy-report")).style(theme::outlined),
                ),
                _ => Element::from(Space::new().width(Length::Shrink)),
            },
            tappable("cancel", Some(Message::ReportCancel), choice(crate::i18n::text(language, "common-cancel")).style(theme::outlined)),
            tappable(
                "send",
                // A description, a category, and for a bug a contact.
                (writing && dialog.stage != ReportStage::TooSoon && dialog.can_send())
                    .then_some(Message::ReportSend),
                choice(match dialog.stage {
                    ReportStage::Sending => crate::i18n::text(language, "report-sending"),
                    ReportStage::Failed => crate::i18n::text(language, "report-try-again"),
                    ReportStage::TooSoon => crate::i18n::text(language, "report-just-sent"),
                    _ => match dialog.kind {
                        ReportKind::Bug => crate::i18n::text(language, "report-send-bug"),
                        ReportKind::Feedback => crate::i18n::text(language, "app-menu-feedback"),
                    },
                })
                .style(theme::filled(theme::ACCENT)),
            ),
        ],
    }
    .spacing(theme::SM)
    .align_y(Alignment::Center)
    .into()
}
