//! One addon's detail card: its description, versions and links.

use crate::*;
use iced::widget::column;

impl Lightforge {
    /// One addon's project page: what it is, and what changed in the build
    /// you would get.
    ///
    /// Tabs: under a long description the changelog would be lost.
    pub(crate) fn detail_card<'a>(&'a self, detail: &'a AddonDetail) -> Element<'a, Message> {
        let language = &self.config.interface_language;
        let title = detail
            .loaded
            .as_ref()
            .map(|d| d.project.name.clone())
            .unwrap_or_else(|| detail.name.clone());
        let mut head = row![text(title).size(theme::TITLE)]
            .spacing(theme::SM)
            .align_y(Alignment::Center);

        if let Some(loaded) = &detail.loaded {
            if loaded.project.download_count > 0 {
                head = head.push(muted(
                    i18n::text_with_args(language, "addons-download-count", &[("count", &downloads_of(loaded.project.download_count))]),
                    theme::CAPTION,
                ));
            }
        }

        let mut body = column![head].spacing(theme::SM).width(Length::Fixed(620.0));

        // The author line sits under the name, not beside it: addons
        // with four maintainers are common and the row would wrap badly.
        if let Some(loaded) = &detail.loaded {
            if !loaded.project.authors.is_empty() {
                body = body.push(muted(
                    i18n::text_with_args(language, "addons-authors", &[("authors", &loaded.project.authors.join(", "))]),
                    theme::CAPTION,
                ));
            }
            if let Some(summary) = &loaded.project.summary {
                body = body.push(text(summary.clone()).size(theme::BODY));
            }
        }

        match (&detail.loaded, &detail.problem) {
            (None, None) => {
                body = body.push(muted(i18n::text(language, "dialog-asking-catalogue"), theme::BODY));
            }
            (_, Some(problem)) => {
                body = body.push(empty(
                    &i18n::error_text(language, problem),
                    &i18n::text(language, "addon-detail-only-curseforge"),
                ));
            }
            (Some(loaded), None) => {
                body = body.push(self.detail_tabs(detail, loaded));
                body = body.push(self.detail_body(detail, loaded));
            }
        }

        // The action is here, so nobody has to find the row again.
        let installed = self.addon_is_installed(&detail.provider, &detail.id);

        let mut footer = row![Space::new().width(Length::Fill)].spacing(theme::SM);
        if let Some(url) = detail.loaded.as_ref().and_then(|d| d.project.website_url.clone()) {
            footer = footer.push(secondary(&i18n::text(language, "addon-detail-open-page"), Message::OpenUrl(url)).enabled(true));
        }
        // As in the installed row's menu.
        if let Some(url) = detail
            .loaded
            .as_ref()
            .and_then(|d| d.project.source_url.clone().or_else(|| d.project.author_url.clone()))
        {
            footer =
                footer.push(secondary(&i18n::text(language, "menu-addon-support-author"), Message::OpenUrl(url)).enabled(true));
        }
        let project = detail.loaded.as_ref().map(|d| &d.project);
        let opted_out = project.is_some_and(|p| p.opted_out);
        let github = project
            .filter(|p| p.opted_out)
            .and_then(|p| p.source_url.as_deref())
            .and_then(lightforge_core::addons::browse::github_repo_of);
        if opted_out {
            body = body.push(
                text(match github {
                    Some(_) => i18n::text(language, "addon-detail-opted-out-github"),
                    None => i18n::text(language, "addon-detail-opted-out-page"),
                })
                .size(theme::LABEL)
                .color(theme::UPDATE),
            );
        }
        if !installed {
            let install: Option<Element<'static, Message>> = match (opted_out, github) {
                (false, _) => Some(
                    action_button(
                        &i18n::text(language, "dialog-start-install"),
                        theme::INSTALL,
                        Message::InstallAddon(detail.provider.clone(), detail.id.clone()),
                    )
                    .enabled(!self.addons_locked()),
                ),
                (true, Some(repo)) => Some(
                    action_button(
                        &i18n::text(language, "menu-addon-get-from-github"),
                        theme::INSTALL,
                        Message::InstallAddon("github".to_owned(), repo),
                    )
                    .enabled(!self.addons_locked()),
                ),
                (true, None) => None,
            };
            if let Some(install) = install {
                footer = footer.push(install);
            }
        }
        footer = footer.push(secondary(&i18n::text(language, "common-close"), Message::CloseAddonDetail).enabled(true));
        body = body.push(footer);

        container(container(body).padding(theme::LG).style(theme::floating))
            .center(Length::Fill)
            .padding(theme::XL)
            .into()
    }

    /// The two tab buttons, each naming what it holds.
    ///
    /// An empty tab is disabled, not hidden: that is an answer too.
    pub(crate) fn detail_tabs<'a>(
        &'a self,
        detail: &'a AddonDetail,
        loaded: &'a addonwork::Detail,
    ) -> Element<'a, Message> {
        let language = &self.config.interface_language;
        let tab = |id: &'static str, label: String, which: DetailTab, available: bool| {
            let selected = detail.tab == which;
            let key = format!("detail tab {id}");
            let tab = button(text(label).size(theme::BODY))
                .padding([theme::XS, theme::MD])
                .style(match selected {
                    true => theme::selected,
                    false => theme::ghost,
                });
            tappable(key, (available && !selected).then_some(Message::AddonDetailTab(which)), tab)
        };

        let changelog_label = match &loaded.changelog_build {
            Some(build) => i18n::text_with_args(language, "addon-detail-changelog-version", &[("version", build)]),
            None => i18n::text(language, "addon-detail-changelog"),
        };
        row![
            tab(
                "changelog",
                changelog_label,
                DetailTab::Changelog,
                loaded.changelog.as_ref().is_some_and(|d| !d.is_empty()),
            ),
            tab(
                "about",
                i18n::text(language, "addon-detail-about"),
                DetailTab::About,
                loaded.description.as_ref().is_some_and(|d| !d.is_empty()),
            ),
        ]
        .spacing(theme::XS)
        .into()
    }

    /// Whichever document the open tab names, scrolled.
    pub(crate) fn detail_body<'a>(
        &'a self,
        detail: &'a AddonDetail,
        loaded: &'a addonwork::Detail,
    ) -> Element<'a, Message> {
        let language = &self.config.interface_language;
        let (document, missing) = match detail.tab {
            DetailTab::Changelog => (
                loaded.changelog.as_ref(),
                "addon-detail-no-changelog",
            ),
            DetailTab::About => (
                loaded.description.as_ref(),
                "addon-detail-no-description",
            ),
        };

        let Some(document) = document.filter(|d| !d.is_empty()) else {
            return empty(&i18n::text(language, missing), &i18n::text(language, "addon-detail-curseforge-more"));
        };

        let mut content = column![].spacing(theme::SM);
        for block in &document.blocks {
            content = content.push(rich_block(block));
        }
        if document.truncated {
            content = content.push(muted(
                i18n::text(language, "addon-detail-truncated"),
                theme::CAPTION,
            ));
        }

        focus::region(
            "addon detail",
            scrollable(container(content).padding(theme::MD))
                .height(Length::Fixed(320.0))
                .style(theme::scrollbar),
        )
    }
}
