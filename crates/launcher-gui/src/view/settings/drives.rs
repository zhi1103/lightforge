//! Storage: the drives, what is on each, and freeing space.

use crate::*;
use iced::widget::column;

impl Lightforge {
    /// Where the library lives, how full its disk is, and what is using it.
    pub(crate) fn storage_section(&self) -> Element<'_, Message> {
        let language = &self.config.interface_language;
        let report = self.storage.as_ref();

        // ---- One card per drive.
        let mut drives = column![].spacing(theme::MD);
        match report {
            None => drives = drives.push(muted(i18n::text(language, "settings-storage-measuring"), theme::LABEL)),
            Some(report) => {
                for (index, library) in report.libraries.iter().enumerate() {
                    // The first library on a disk speaks for its free space.
                    let same_disk_as = report.libraries[..index]
                        .iter()
                        .find(|earlier| earlier.mount == library.mount)
                        .map(|earlier| earlier.label.as_str());
                    drives = drives.push(self.drive_card(library, same_disk_as));
                }
                drives = drives.spacing(theme::SM);
                drives = drives.extend(self.hint(i18n::text(language, "settings-storage-default-drive-hint")).map(Element::from));
            }
        }

        // ---- Partitions that could become one.
        let offered: Vec<(usize, &library::Partition)> = self
            .partitions
            .iter()
            .enumerate()
            .filter(|(_, partition)| !partition.in_use)
            .collect();
        let add: Element<'_, Message> = match offered.is_empty() {
            true => setting_group([setting(
                i18n::text(language, "settings-storage-nothing-to-add"),
                i18n::text(language, "settings-storage-no-writable-partition"),
                None,
                Space::new(),
            )]),
            false => setting_group(offered.into_iter().map(|(index, partition)| {
                let free = match partition.disk {
                    Some(disk) => format!("{} free of {}", size_of(disk.free), size_of(disk.total)),
                    None => i18n::text(language, "settings-storage-size-unknown"),
                };
                setting(
                    partition.label.clone(),
                    i18n::text_with_args(language, "settings-storage-partition-details", &[
                        ("filesystem", &partition.fstype),
                        ("device", &partition.device.display().to_string()),
                        ("mount", &partition.point.display().to_string()),
                    ]),
                    None,
                    row![
                        muted(free, theme::LABEL),
                        secondary(&i18n::text(language, "common-add"), Message::AddDrive(index)).enabled(true),
                    ]
                    .spacing(theme::MD)
                    .align_y(Alignment::Center),
                )
            })),
        };
        let add_hint = self.hint(
            i18n::text(language, "settings-storage-add-drive-hint"),
        );

        // ---- Lightforge's own directories, which follow XDG, not the
        // drives, so they are listed apart, not attributed to one.
        let mut usage = column![].spacing(theme::SM);
        match report {
            None => usage = usage.push(muted(i18n::text(language, "settings-storage-measuring"), theme::LABEL)),
            Some(report) => {
                let clearing = self.cache_notice.as_deref() == Some("clearing…");
                usage = usage.push(setting_group(report.app.iter().map(|item| {
                    let action = item.clearable.then(|| {
                        secondary(&i18n::text(language, "common-clear"), Message::ClearCache).enabled(item.bytes > 0 && !clearing)
                    });
                    setting(
                        item.label.clone(),
                        tilde(&item.path),
                        None,
                        // The button's slot is kept on every row, so the
                        // sizes line up whether or not there is one.
                        row![
                            text(size_of(item.bytes)).size(theme::BODY).font(theme::MONO),
                            container(action.unwrap_or_else(|| Space::new().into()))
                                .width(Length::Fixed(ACTION_COLUMN))
                                .align_x(Alignment::End),
                        ]
                        .spacing(theme::MD)
                        .align_y(Alignment::Center),
                    )
                })));
                usage = usage.extend(self.hint(i18n::text(language, "settings-storage-cache-hint")).map(Element::from));
                if let Some(notice) = &self.cache_notice {
                    usage = usage.push(muted(notice.clone(), theme::LABEL));
                }
                if self.storage_measuring {
                    usage = usage.push(muted(i18n::text(language, "settings-storage-updating"), theme::CAPTION));
                }
            }
        }

        column![
            group_label(i18n::text(language, "settings-storage-drives"), None),
            drives,
            group_label(
                i18n::text(language, "settings-storage-add-drive"),
                Some(secondary(&i18n::text(language, "settings-storage-rescan-drives"), Message::RefreshDrives).enabled(true)),
            ),
            add,
        ]
        .extend(add_hint.map(Element::from))
        .push(group_label(i18n::text(language, "settings-storage-lightforge-files"), None))
        .push(usage)
        .spacing(theme::SM)
        .into()
    }

    /// One drive's card: how full it is, what Lightforge put there, and the
    /// three things you can do to it.
    pub(crate) fn drive_card<'a>(
        &'a self,
        library: &'a storage::LibraryUsage,
        same_disk_as: Option<&'a str>,
    ) -> Element<'a, Message> {
        let language = &self.config.interface_language;
        let mut card = column![drive_title(library, language), muted(tilde(&library.path), theme::LABEL)]
            .spacing(theme::SM)
            .extend(drive_space(library, same_disk_as, language));
        // Only when true: an unclaimed library can still hold games.
        if library.state == library::State::New && library.total() == 0 {
            card = card.push(muted(
                i18n::text(language, "settings-storage-no-games-yet"),
                theme::CAPTION,
            ));
        }

        card = card.extend(self.drive_usage(library));
        card = card.extend(self.unused_rows(library));
        card = card.push(self.drive_actions(library));
        setting_card(card)
    }

    /// One row per kind of thing on the drive, with a Clear button where it
    /// can be cleared, and what the last clear said.
    fn drive_usage<'a>(&'a self, library: &'a storage::LibraryUsage) -> Vec<Element<'a, Message>> {
        let language = &self.config.interface_language;
        let mut rows = Vec::new();
        for item in &library.usage {
            // Nothing is a fact better told by absence than by a row of
            // zeroes with a dead button beside it.
            if item.bytes == 0 {
                continue;
            }
            let notice = self
                .drive_notice
                .as_ref()
                .filter(|(which, _)| *which == library.id)
                .map(|(_, note)| note.clone());
            let clearing = notice.as_deref() == Some("clearing…");
            let action = item.clearable.then(|| {
                secondary(&i18n::text(language, "common-clear"), Message::ClearDrive(library.id.clone()))
                    .enabled(item.bytes > 0 && !clearing)
            });
            rows.push(usage_row(
                text(item.label.clone()).size(theme::BODY).into(),
                item.bytes,
                action,
            ));
        }
        if let Some((which, note)) = &self.drive_notice {
            if *which == library.id {
                rows.push(muted(note.clone(), theme::LABEL).into());
            }
        }
        rows
    }

    /// Open, make default, remove.
    fn drive_actions<'a>(&'a self, library: &'a storage::LibraryUsage) -> Element<'a, Message> {
        let language = &self.config.interface_language;
        let id = &library.id;
        let mut actions = row![Space::new().width(Length::Fill)]
            .spacing(theme::SM)
            .align_y(Alignment::Center);
        // `is_present`: only a drive that is away cannot be opened or made default.
        actions = actions.push(
            secondary(&i18n::text(language, "settings-storage-open-folder"), Message::OpenDrive(id.clone()))
                .enabled(library.state.is_present()),
        );
        actions = actions.push(
            secondary(&i18n::text(language, "settings-storage-make-default"), Message::SetDefaultDrive(id.clone()))
                .enabled(!library.is_default && library.state.is_present()),
        );
        // Not the default while another exists, never the last.
        actions = actions.push(
            secondary(&i18n::text(language, i18n::COMMON_REMOVE), Message::RemoveDrive(id.clone()))
                .enabled(!library.is_default && self.config.libraries.len() > 1),
        );
        actions.into()
    }

    /// A drive card's "Unused game data" row and the line under it.
    pub(crate) fn unused_rows<'a>(&'a self, library: &'a storage::LibraryUsage) -> Vec<Element<'a, Message>> {
        let language = &self.config.interface_language;
        let label = || text(i18n::text(language, "settings-storage-unused-game-data")).size(theme::BODY).into();
        let Some(state) = self.unused.get(&library.id) else {
            return Vec::new();
        };
        match state {
            UnusedData::Checking => vec![muted(i18n::text(language, "settings-storage-checking-unused"), theme::CAPTION).into()],
            // Nothing worth a button: told by absence, like an empty cache.
            UnusedData::Ready(plan) if plan.freeable < UNUSED_WORTH_SHOWING => Vec::new(),
            UnusedData::Ready(plan) => {
                let busy = self
                    .config
                    .libraries
                    .iter()
                    .find(|l| l.id == library.id)
                    .is_some_and(|l| self.downloading_into(&self.layout.wow_install_in(l)));
                let held = self.store_writer().is_some() || self.in_play();
                let action =
                    secondary(&i18n::text(language, "settings-storage-free-up"), Message::FreeUnused(library.id.clone())).enabled(!busy && !held);
                let note = match busy {
                    true => i18n::text(language, "settings-storage-finish-download-first"),
                    false if plan.moved > 0 => i18n::text_with_args(language, "settings-storage-unused-explanation-copy", &[("size", &size_of(plan.moved))]),
                    false => i18n::text(language, "settings-storage-unused-explanation"),
                };
                vec![
                    usage_row(label(), plan.freeable, Some(action)),
                    muted(note, theme::CAPTION).into(),
                ]
            }
            UnusedData::Freeing { copied, total } => {
                let done = copied.load(std::sync::atomic::Ordering::Relaxed).min(*total);
                vec![muted(
                    i18n::text_with_args(language, "settings-storage-freeing-unused", &[("done", &size_of(done)), ("total", &size_of(*total))]),
                    theme::LABEL,
                )
                .into()]
            }
            UnusedData::Said(note) => vec![muted(note.clone(), theme::LABEL).into()],
        }
    }
}

/// The drive's name, its signs, and the two letters Wine knows it by.
fn drive_title<'a>(library: &'a storage::LibraryUsage, language: &str) -> Element<'a, Message> {
    let mut title = row![text(library.label.clone()).size(theme::HEADING)]
        .spacing(theme::SM)
        .align_y(Alignment::Center);
    if library.is_default {
        title = title.push(
            container(text(i18n::text(language, "settings-storage-default")).size(theme::CAPTION).color(theme::ACCENT))
                .padding([2.0, theme::SM])
                .style(theme::tag(theme::ACCENT)),
        );
    }
    if let Some(reason) = i18n::library_state_reason(language, library.state, &library.label) {
        title = title.push(
            container(text(reason).size(theme::CAPTION).color(theme::ERROR))
                .padding([2.0, theme::SM])
                .style(theme::tag(theme::ERROR)),
        );
    }
    title = title.push(Space::new().width(Length::Fill));
    title = title.push(mono(format!(
        "{} → games   {} → wow",
        library.letters.0.to_uppercase(),
        library.letters.1.to_uppercase(),
    )));
    title.into()
}

/// How full the disk under the drive is, or why that is not shown.
fn drive_space<'a>(
    library: &'a storage::LibraryUsage,
    same_disk_as: Option<&'a str>,
    language: &str,
) -> Vec<Element<'a, Message>> {
    match library.disk {
        // Already drawn on the card above: one bar per disk.
        Some(_) if same_disk_as.is_some() => vec![muted(
            i18n::text_with_args(language, "settings-storage-same-disk", &[
                ("other", same_disk_as.unwrap_or_default()),
                ("size", &size_of(library.total())),
            ]),
            theme::CAPTION,
        )
        .into()],
        Some(disk) => {
            let used = disk.total.saturating_sub(disk.free);
            let fraction = match disk.total {
                0 => 0.0,
                total => used as f32 / total as f32,
            };
            vec![
                progress_bar(0.0..=1.0, fraction).girth(6).into(),
                muted(
                    i18n::text_with_args(language, "settings-storage-disk-space", &[
                        ("free", &size_of(disk.free)),
                        ("total", &size_of(disk.total)),
                        ("lightforge", &size_of(library.total())),
                    ]),
                    theme::CAPTION,
                )
                .into(),
            ]
        }
        // An absent drive is not measured: its mount point shows what is beneath.
        None => vec![muted(
            i18n::text(language, "settings-storage-drive-away"),
            theme::CAPTION,
        )
        .into()],
    }
}
