use std::sync::Arc;

use druid::{
    widget::{CrossAxisAlignment, Flex, Label, LineBreaking, List},
    LensExt, LocalizedString, Menu, MenuItem, Selector, Widget, WidgetExt,
};

use crate::{
    cmd,
    data::{
        AppState, Ctx, Episode, Library, Nav, Show, ShowDetail, ShowEpisodes, ShowLink, WithCtx,
    },
    webapi::WebApi,
    widget::{Async, MyWidgetExt, RemoteImage},
};

use super::{
    library, theme,
    utils::{error_widget, placeholder_widget, spinner_widget},
};

pub const LOAD_DETAIL: Selector<ShowLink> = Selector::new("app.show.load-detail");

pub fn detail_widget() -> impl Widget<AppState> {
    Flex::column()
        .with_child(async_info_widget())
        .with_default_spacer()
        .with_child(async_episodes_widget())
}

fn async_info_widget() -> impl Widget<AppState> {
    Async::new(spinner_widget, info_widget, error_widget)
        .lens(
            Ctx::make(
                AppState::common_ctx,
                AppState::show_detail.then(ShowDetail::show),
            )
            .then(Ctx::in_promise()),
        )
        .on_command_async(
            LOAD_DETAIL,
            |d| WebApi::global().get_show(&d.id),
            |_, data, d| data.show_detail.show.defer(d),
            |_, data, (d, r)| data.show_detail.show.update((d, r)),
        )
}

fn info_widget() -> impl Widget<WithCtx<Arc<Show>>> {
    Label::raw().lens(Ctx::data().then(Show::name.in_arc()))
}

fn async_episodes_widget() -> impl Widget<AppState> {
    Async::new(spinner_widget, episodes_widget, error_widget)
        .lens(
            Ctx::make(
                AppState::common_ctx,
                AppState::show_detail.then(ShowDetail::episodes),
            )
            .then(Ctx::in_promise()),
        )
        .on_command_async(
            LOAD_DETAIL,
            |d| WebApi::global().get_show_episodes(&d.id),
            |_, data, d| data.show_detail.episodes.defer(d),
            |_, data, (d, r)| {
                let r = r.map(|episodes| ShowEpisodes {
                    show: d.clone(),
                    episodes,
                });
                data.show_detail.episodes.update((d, r))
            },
        )
}

fn episodes_widget() -> impl Widget<WithCtx<ShowEpisodes>> {
    List::new(episode_widget).lens(Ctx::map(ShowEpisodes::episodes))
}

fn episode_widget() -> impl Widget<WithCtx<Arc<Episode>>> {
    let cover = episode_cover_widget(theme::grid(4.0));

    let name = Label::raw()
        .with_font(theme::UI_FONT_MEDIUM)
        .with_line_break_mode(LineBreaking::WordWrap)
        .lens(Episode::name.in_arc());

    let description = Label::raw()
        .with_text_size(theme::TEXT_SIZE_SMALL)
        .with_text_color(theme::PLACEHOLDER_COLOR)
        .with_line_break_mode(LineBreaking::WordWrap)
        .lens(Episode::description.in_arc());

    Flex::row()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(cover)
        .with_default_spacer()
        .with_flex_child(
            Flex::column()
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .with_child(name)
                .with_default_spacer()
                .with_child(description),
            1.0,
        )
        .padding(theme::grid(1.0))
        .link()
        .rounded(theme::BUTTON_BORDER_RADIUS)
        .on_click(|ctx, row, _| {
            // TODO
        })
        .lens(Ctx::data())
}

pub fn show_widget() -> impl Widget<WithCtx<Arc<Show>>> {
    let show_image = show_cover_widget(theme::grid(7.0));

    let show_name = Label::raw()
        .with_font(theme::UI_FONT_MEDIUM)
        .with_line_break_mode(LineBreaking::Clip)
        .lens(Show::name.in_arc());

    let show_publisher = Label::raw()
        .with_text_size(theme::TEXT_SIZE_SMALL)
        .with_text_color(theme::PLACEHOLDER_COLOR)
        .lens(Show::publisher.in_arc());

    let show_info = Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(show_name)
        .with_spacer(1.0)
        .with_child(show_publisher);

    let show = Flex::row()
        .with_child(show_image)
        .with_default_spacer()
        .with_flex_child(show_info, 1.0)
        .lens(Ctx::data());

    show.padding(theme::grid(0.5))
        .link()
        .on_click(|ctx, show, _| {
            ctx.submit_command(cmd::NAVIGATE.with(Nav::ShowDetail(show.data.link())));
        })
        .context_menu(show_ctx_menu)
}

pub fn episode_cover_widget(size: f64) -> impl Widget<Arc<Episode>> {
    RemoteImage::new(placeholder_widget(), move |episode: &Arc<Episode>, _| {
        episode.image(size, size).map(|image| image.url.clone())
    })
    .fix_size(size, size)
}

fn show_cover_widget(size: f64) -> impl Widget<Arc<Show>> {
    RemoteImage::new(placeholder_widget(), move |show: &Arc<Show>, _| {
        show.image(size, size).map(|image| image.url.clone())
    })
    .fix_size(size, size)
}

fn show_ctx_menu(show: &WithCtx<Arc<Show>>) -> Menu<AppState> {
    show_menu(&show.data, &show.ctx.library)
}

fn show_menu(show: &Arc<Show>, library: &Arc<Library>) -> Menu<AppState> {
    let mut menu = Menu::empty();

    menu = menu.entry(
        MenuItem::new(
            LocalizedString::new("menu-item-copy-link").with_placeholder("Copy Link to Show"),
        )
        .command(cmd::COPY.with(show.link().url())),
    );

    menu = menu.separator();

    if library.contains_show(show) {
        menu = menu.entry(
            MenuItem::new(
                LocalizedString::new("menu-item-remove-from-library").with_placeholder("Unfollow"),
            )
            .command(library::UNSAVE_SHOW.with(show.link())),
        );
    } else {
        menu = menu.entry(
            MenuItem::new(
                LocalizedString::new("menu-item-save-to-library").with_placeholder("Follow"),
            )
            .command(library::SAVE_SHOW.with(show.clone())),
        );
    }

    menu
}