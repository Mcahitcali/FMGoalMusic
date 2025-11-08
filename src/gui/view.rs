use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use gpui::{
    div, img, px, AnyElement, AppContext, Bounds, ClickEvent, Context, CursorStyle, Entity,
    FocusHandle, Focusable, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, ObjectFit, ParentElement, Pixels, Point, Render, SharedString,
    Styled, StyledImage, Subscription, Window,
};
use gpui_component::{
    button::{Button, ButtonVariants},
    input::{Input, InputState},
    scroll::ScrollbarAxis,
    select::{Select, SelectEvent, SelectItem, SelectState},
    slider::{Slider, SliderEvent, SliderState},
    switch::Switch,
    ActiveTheme, Disableable, IndexPath, Selectable, Sizable, StyledExt,
};

use super::controller::{GuiController, RegionCapture};
use super::state::AppTab;
use crate::audio::AudioManager;
use crate::state::{MusicEntry, ProcessState};

struct PreviewSound {
    manager: AudioManager,
    path: PathBuf,
}

struct CachedAudio {
    path: PathBuf,
    data: Arc<Vec<u8>>,
}

struct AddTeamForm {
    expanded: bool,
    use_existing_league: bool,
    selected_league: Option<String>,
    error: Option<String>,
    team_name_input: Entity<InputState>,
    new_league_input: Entity<InputState>,
    variations_input: Entity<InputState>,
}

struct RegionSelection {
    image_path: PathBuf,
    physical_size: (u32, u32),
    logical_size: (u32, u32),
    device_scale: f32,
    render_scale: f32,
    drag_start: Option<Point<gpui::Pixels>>,
    drag_current: Option<Point<gpui::Pixels>>,
    dragging: bool,
    points_local: bool,
}

impl RegionSelection {
    fn from_capture(capture: RegionCapture) -> Self {
        let physical_w = capture.physical_size.0.max(1) as f32;
        let render_scale = (960.0 / physical_w).min(1.0);
        Self {
            image_path: capture.image_path,
            physical_size: capture.physical_size,
            logical_size: capture.logical_size,
            device_scale: capture.device_scale.max(0.0001),
            render_scale,
            drag_start: None,
            drag_current: None,
            dragging: false,
            points_local: false,
        }
    }

    fn display_size(&self) -> (f32, f32) {
        (
            self.physical_size.0 as f32 * self.render_scale,
            self.physical_size.1 as f32 * self.render_scale,
        )
    }

    fn adjust_point(point: Point<gpui::Pixels>, bounds: Bounds<Pixels>) -> Point<gpui::Pixels> {
        Point::new(point.x - bounds.origin.x, point.y - bounds.origin.y)
    }

    fn ensure_local(&mut self, bounds: Bounds<Pixels>) {
        if self.points_local {
            return;
        }
        if let Some(start) = self.drag_start {
            self.drag_start = Some(Self::adjust_point(start, bounds));
        }
        if let Some(current) = self.drag_current {
            self.drag_current = Some(Self::adjust_point(current, bounds));
        }
        self.points_local = true;
    }

    fn start_drag(&mut self, point: Point<gpui::Pixels>, bounds: Option<Bounds<Pixels>>) {
        if let Some(bounds) = bounds {
            let local = Self::adjust_point(point, bounds);
            self.drag_start = Some(local);
            self.drag_current = Some(local);
            self.points_local = true;
        } else {
            self.drag_start = Some(point);
            self.drag_current = Some(point);
            self.points_local = false;
        }
        self.dragging = true;
    }

    fn update_drag(&mut self, point: Point<gpui::Pixels>, bounds: Option<Bounds<Pixels>>) {
        if self.dragging {
            if let Some(bounds) = bounds {
                if !self.points_local {
                    self.ensure_local(bounds);
                }
                let local = Self::adjust_point(point, bounds);
                self.drag_current = Some(local);
            } else {
                self.drag_current = Some(point);
            }
        }
    }

    fn finish_drag(&mut self, point: Point<gpui::Pixels>, bounds: Option<Bounds<Pixels>>) {
        if self.dragging {
            if let Some(bounds) = bounds {
                if !self.points_local {
                    self.ensure_local(bounds);
                }
                let local = Self::adjust_point(point, bounds);
                self.drag_current = Some(local);
            } else {
                self.drag_current = Some(point);
            }
            self.dragging = false;
        }
    }

    fn logical_rect(&self) -> Option<[u32; 4]> {
        let start = self.drag_start?;
        let current = self.drag_current?;
        let (disp_w, disp_h) = self.display_size();
        let clamp = |value: f32, max| value.clamp(0.0, max);
        let sx = clamp(f32::from(start.x), disp_w);
        let sy = clamp(f32::from(start.y), disp_h);
        let cx = clamp(f32::from(current.x), disp_w);
        let cy = clamp(f32::from(current.y), disp_h);
        let width = (sx - cx).abs();
        let height = (sy - cy).abs();
        if width < 2.0 || height < 2.0 {
            return None;
        }

        let physical_min_x = sx.min(cx) / self.render_scale;
        let physical_min_y = sy.min(cy) / self.render_scale;
        let physical_width = width / self.render_scale;
        let physical_height = height / self.render_scale;

        let mut logical_x = (physical_min_x / self.device_scale).round() as i64;
        let mut logical_y = (physical_min_y / self.device_scale).round() as i64;
        let mut logical_w = (physical_width / self.device_scale).round().max(1.0) as i64;
        let mut logical_h = (physical_height / self.device_scale).round().max(1.0) as i64;

        let max_w = self.logical_size.0.max(1) as i64;
        let max_h = self.logical_size.1.max(1) as i64;
        logical_x = logical_x.clamp(0, max_w.saturating_sub(1));
        logical_y = logical_y.clamp(0, max_h.saturating_sub(1));
        if logical_x + logical_w > max_w {
            logical_w = max_w - logical_x;
        }
        if logical_y + logical_h > max_h {
            logical_h = max_h - logical_y;
        }

        Some([
            logical_x.max(0) as u32,
            logical_y.max(0) as u32,
            logical_w.max(1) as u32,
            logical_h.max(1) as u32,
        ])
    }

    fn overlay_rect(&self) -> Option<(f32, f32, f32, f32)> {
        let start = self.drag_start?;
        let current = self.drag_current?;
        let (disp_w, disp_h) = self.display_size();
        let clamp = |value: f32, max| value.clamp(0.0, max);
        let sx = clamp(f32::from(start.x), disp_w);
        let sy = clamp(f32::from(start.y), disp_h);
        let cx = clamp(f32::from(current.x), disp_w);
        let cy = clamp(f32::from(current.y), disp_h);
        let width = (sx - cx).abs();
        let height = (sy - cy).abs();
        if width < 1.0 || height < 1.0 {
            return None;
        }
        Some((sx.min(cx), sy.min(cy), width, height))
    }
}
#[derive(Clone)]
struct MonitorOption {
    label: SharedString,
    value: usize,
}

impl MonitorOption {
    fn new(label: impl Into<SharedString>, value: usize) -> Self {
        Self {
            label: label.into(),
            value,
        }
    }
}

impl SelectItem for MonitorOption {
    type Value = usize;

    fn title(&self) -> SharedString {
        self.label.clone()
    }

    fn value(&self) -> &Self::Value {
        &self.value
    }
}

pub struct MainView {
    controller: GuiController,
    focus_handle: FocusHandle,
    active_tab: AppTab,
    status_text: SharedString,
    active_league: Option<String>,
    music_volume_slider: Entity<SliderState>,
    ambiance_volume_slider: Entity<SliderState>,
    music_length_slider: Entity<SliderState>,
    ambiance_length_slider: Entity<SliderState>,
    ocr_slider: Entity<SliderState>,
    debounce_slider: Entity<SliderState>,
    subscriptions: Vec<Subscription>,
    music_preview: Option<PreviewSound>,
    music_preview_playing: bool,
    ambiance_preview: Option<PreviewSound>,
    ambiance_preview_playing: bool,
    cached_audio: Option<CachedAudio>,
    add_team_form: AddTeamForm,
    monitor_select: Entity<SelectState<Vec<MonitorOption>>>,
    monitor_options: Vec<MonitorOption>,
    region_selection: Option<RegionSelection>,
    region_canvas_bounds: Option<Bounds<Pixels>>,
}

impl MainView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>, controller: GuiController) -> Self {
        let focus_handle = cx.focus_handle();
        let status_text: SharedString = controller.status_message().into();

        let (
            music_volume,
            ambiance_volume,
            music_length_ms,
            ambiance_length_ms,
            ocr_threshold,
            debounce_ms,
            selected_team,
            selected_monitor_index,
        ) = {
            let state = controller.state();
            let guard = state.lock();
            (
                guard.music_volume,
                guard.ambiance_volume,
                guard.music_length_ms,
                guard.ambiance_length_ms,
                guard.ocr_threshold,
                guard.debounce_ms,
                guard.selected_team.clone(),
                guard.selected_monitor_index,
            )
        };

        let music_volume_slider = cx.new(|_| {
            SliderState::new()
                .min(0.)
                .max(100.)
                .step(1.)
                .default_value((music_volume * 100.0).round())
        });
        let ambiance_volume_slider = cx.new(|_| {
            SliderState::new()
                .min(0.)
                .max(100.)
                .step(1.)
                .default_value((ambiance_volume * 100.0).round())
        });
        let music_length_slider = cx.new(|_| {
            SliderState::new()
                .min(1.)
                .max(60.)
                .step(1.)
                .default_value((music_length_ms as f32 / 1000.0).clamp(1.0, 60.0))
        });
        let ambiance_length_slider = cx.new(|_| {
            SliderState::new()
                .min(1.)
                .max(60.)
                .step(1.)
                .default_value((ambiance_length_ms as f32 / 1000.0).clamp(1.0, 60.0))
        });
        let ocr_slider = cx.new(|_| {
            SliderState::new()
                .min(0.)
                .max(255.)
                .step(1.)
                .default_value(ocr_threshold as f32)
        });
        let debounce_slider = cx.new(|_| {
            SliderState::new()
                .min(100.)
                .max(60_000.)
                .step(100.)
                .default_value(debounce_ms as f32)
        });

        let active_league = selected_team.as_ref().map(|team| team.league.clone());

        let team_name_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Team display name")
                .clean_on_escape()
        });
        let new_league_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("New league name")
                .clean_on_escape()
        });
        let variations_input = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line()
                .rows(4)
                .placeholder("Variations (one per line)")
        });

        let monitor_options = {
            let summaries = controller.monitor_summaries();
            if summaries.is_empty() {
                vec![MonitorOption::new("Display 1", 0)]
            } else {
                summaries
                    .into_iter()
                    .map(|summary| MonitorOption::new(summary.label, summary.index))
                    .collect::<Vec<_>>()
            }
        };
        let monitor_initial_ix = monitor_options
            .iter()
            .position(|option| option.value == selected_monitor_index)
            .or_else(|| (!monitor_options.is_empty()).then_some(0))
            .map(|idx| IndexPath::default().row(idx));
        let monitor_select = {
            let monitor_data = monitor_options.clone();
            cx.new(move |cx| SelectState::new(monitor_data.clone(), monitor_initial_ix, window, cx))
        };

        let mut view = Self {
            controller,
            focus_handle,
            active_tab: AppTab::Dashboard,
            status_text,
            active_league: active_league.clone(),
            music_volume_slider,
            ambiance_volume_slider,
            music_length_slider,
            ambiance_length_slider,
            ocr_slider,
            debounce_slider,
            subscriptions: Vec::new(),
            music_preview: None,
            music_preview_playing: false,
            ambiance_preview: None,
            ambiance_preview_playing: false,
            cached_audio: None,
            add_team_form: AddTeamForm {
                expanded: false,
                use_existing_league: true,
                selected_league: active_league,
                error: None,
                team_name_input,
                new_league_input,
                variations_input,
            },
            monitor_select,
            monitor_options,
            region_selection: None,
            region_canvas_bounds: None,
        };

        view.register_slider_subscriptions(cx);
        view.register_monitor_subscription(cx);
        view
    }

    fn refresh_status(&mut self) {
        self.status_text = self.controller.status_message().into();
    }

    fn register_slider_subscriptions(&mut self, cx: &mut Context<Self>) {
        let subscribe_volume = cx.subscribe(
            &self.music_volume_slider,
            |this, _, event: &SliderEvent, cx| match event {
                SliderEvent::Change(value) => {
                    let pct = value.start().clamp(0.0, 100.0);
                    if let Err(err) = this.controller.set_music_volume(pct / 100.0) {
                        this.status_text = format!("{err:#}").into();
                    } else {
                        this.refresh_status();
                    }
                    cx.notify();
                }
            },
        );
        self.subscriptions.push(subscribe_volume);

        let subscribe_ambiance_volume = cx.subscribe(
            &self.ambiance_volume_slider,
            |this, _, event: &SliderEvent, cx| match event {
                SliderEvent::Change(value) => {
                    let pct = value.start().clamp(0.0, 100.0);
                    if let Err(err) = this.controller.set_ambiance_volume(pct / 100.0) {
                        this.status_text = format!("{err:#}").into();
                    } else {
                        this.refresh_status();
                    }
                    cx.notify();
                }
            },
        );
        self.subscriptions.push(subscribe_ambiance_volume);

        let subscribe_music_length = cx.subscribe(
            &self.music_length_slider,
            |this, _, event: &SliderEvent, cx| match event {
                SliderEvent::Change(value) => {
                    let seconds = value.start().clamp(1.0, 60.0);
                    if let Err(err) = this
                        .controller
                        .set_music_length_ms((seconds * 1000.0).round() as u64)
                    {
                        this.status_text = format!("{err:#}").into();
                    } else {
                        this.refresh_status();
                    }
                    cx.notify();
                }
            },
        );
        self.subscriptions.push(subscribe_music_length);

        let subscribe_ambiance_length = cx.subscribe(
            &self.ambiance_length_slider,
            |this, _, event: &SliderEvent, cx| match event {
                SliderEvent::Change(value) => {
                    let seconds = value.start().clamp(1.0, 60.0);
                    if let Err(err) = this
                        .controller
                        .set_ambiance_length_ms((seconds * 1000.0).round() as u64)
                    {
                        this.status_text = format!("{err:#}").into();
                    } else {
                        this.refresh_status();
                    }
                    cx.notify();
                }
            },
        );
        self.subscriptions.push(subscribe_ambiance_length);

        let subscribe_ocr =
            cx.subscribe(
                &self.ocr_slider,
                |this, _, event: &SliderEvent, cx| match event {
                    SliderEvent::Change(value) => {
                        if let Err(err) = this
                            .controller
                            .set_ocr_threshold(value.start().round() as i16)
                        {
                            this.status_text = format!("{err:#}").into();
                        } else {
                            this.refresh_status();
                        }
                        cx.notify();
                    }
                },
            );
        self.subscriptions.push(subscribe_ocr);

        let subscribe_debounce = cx.subscribe(
            &self.debounce_slider,
            |this, _, event: &SliderEvent, cx| match event {
                SliderEvent::Change(value) => {
                    if let Err(err) = this
                        .controller
                        .set_debounce_ms(value.start().round().clamp(100.0, 60_000.0) as u64)
                    {
                        this.status_text = format!("{err:#}").into();
                    } else {
                        this.refresh_status();
                    }
                    cx.notify();
                }
            },
        );
        self.subscriptions.push(subscribe_debounce);
    }

    fn register_monitor_subscription(&mut self, cx: &mut Context<Self>) {
        let subscription = cx.subscribe(
            &self.monitor_select,
            |this, _, event: &SelectEvent<Vec<MonitorOption>>, _cx| {
                if let SelectEvent::Confirm(Some(index)) = event {
                    if let Err(err) = this.controller.set_monitor_index(*index) {
                        this.status_text = format!("{err:#}").into();
                    } else {
                        this.refresh_status();
                    }
                }
            },
        );
        self.subscriptions.push(subscription);
    }

    fn render_dashboard_tab(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        // Team callout tile
        let team_callout = div()
            .bg(cx.theme().group_box)
            .border_1()
            .border_color(cx.theme().border)
            .rounded_lg()
            .p_5()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            // PNG shield icon with emoji fallback
                            .child(self.render_png_icon("assets/icons/shield.png", 18.0, "🛡"))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(div().text_lg().font_semibold().child("Team Selection"))
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(cx.theme().muted_foreground)
                                            .child("The team currently being monitored. Click configure to change it."),
                                    ),
                            ),
                    )
                    .child(
                        Button::new("dash-open-team-selection")
                            .primary()
                            .label("Configure")
                            .on_click(cx.listener(|this, _event: &ClickEvent, _window, _cx| {
                                this.active_tab = AppTab::TeamSelection;
                            })),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .px_2()
                            .py_1()
                            .rounded_full()
                            .bg(cx.theme().muted)
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("Home Team"),
                    ),
            );

        // Music cards with a hero area
        let goal_music = div()
            .bg(cx.theme().group_box)
            .border_1()
            .border_color(cx.theme().border)
            .rounded_lg()
            
            .flex()
            .flex_col()
            .child(
                div()
                    .h(px(160.0))
                    .rounded_lg()
                    .bg(cx.theme().tab_active)
                    .flex()
                    .items_center()
                    .justify_center()
                    // PNG waveform icon with emoji fallback
                    .child(self.render_png_icon("assets/icons/waveform.png", 36.0, "🔈")),
            )
            .child(
                div()
                    .p_5()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(div().text_lg().font_semibold().child("Goal Music"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("Manage celebration tracks for goals."),
                    )
                    .child(
                        Button::new("dash-open-library-1")
                            .ghost()
                            .label("Browse Library →")
                            .on_click(cx.listener(|this, _event: &ClickEvent, _window, _cx| {
                                this.active_tab = AppTab::Library;
                            })),
                    ),
            );

        let other_music = div()
            .bg(cx.theme().group_box)
            .border_1()
            .border_color(cx.theme().border)
            .rounded_lg()
            .p_0()
            .flex()
            .flex_col()
            .child(
                div()
                    .h(px(160.0))
                    .rounded_lg()
                    .bg(cx.theme().tab)
                    .flex()
                    .items_center()
                    .justify_center()
                    // PNG music icon with emoji fallback
                    .child(self.render_png_icon("assets/icons/music.png", 36.0, "🎵")),
            )
            .child(
                div()
                    .p_5()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(div().text_lg().font_semibold().child("Other Music"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("Optional crowd or ambient layers."),
                    )
                    .child(
                        Button::new("dash-open-library-2")
                            .ghost()
                            .label("Browse Library →")
                            .on_click(cx.listener(|this, _event: &ClickEvent, _window, _cx| {
                                this.active_tab = AppTab::Library;
                            })),
                    ),
            );

        // Header with title (left) and status chip (right)
        let header = {
            let state = self.controller.state();
            let guard = state.lock();
            let process_state = guard.process_state;
            drop(guard);

            div()
                .flex()
                .justify_between()
                .items_center()
                .child(div().text_xl().font_semibold().child("Dashboard"))
                .child(self.render_status_chip(process_state, cx))
        };

        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(header)
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_5()
                    .child(div().flex_grow().min_w(px(360.0)).child(team_callout))
                    .child(div().flex_grow().min_w(px(360.0)).child(goal_music))
                    .child(div().flex_grow().min_w(px(360.0)).child(other_music)),
            )
    }

    fn render_png_icon(&self, file: &str, size: f32, alt: &str) -> AnyElement {
        if Path::new(file).exists() {
            img(file)
                .object_fit(ObjectFit::Contain)
                .w(px(size))
                .h(px(size))
                .into_any_element()
        } else {
            div().text_xl().child(alt).into_any_element()
        }
    }

    fn render_tab_icon(&self, tab: AppTab) -> AnyElement {
        match tab {
            AppTab::Dashboard => self.render_png_icon("assets/icons/dashboard.png", 18.0, "🏟️"),
            AppTab::Library => self.render_png_icon("assets/icons/library.png", 18.0, "🎵"),
            AppTab::TeamSelection => self.render_png_icon("assets/icons/team.png", 18.0, "⚽"),
            AppTab::Detection => self.render_png_icon("assets/icons/detection.png", 18.0, "🛰"),
            AppTab::Settings => self.render_png_icon("assets/icons/settings.png", 18.0, "⚙️"),
            AppTab::Help => self.render_png_icon("assets/icons/help.png", 18.0, "ℹ️"),
        }
    }

    fn render_sidebar(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.controller.state();
        let guard = state.lock();
        let process_state = guard.process_state;
        let detection_count = guard.detection_count;
        let selected_team = guard.selected_team.clone();
        drop(guard);

        let control_button = if process_state.is_running() {
            Button::new("stop-detection")
                .danger()
                .label("Stop Monitoring")
                .w_full()
                .on_click(cx.listener(|this, _event: &ClickEvent, _window, _cx| {
                    if let Err(err) = this.controller.stop_monitoring() {
                        this.status_text = format!("{err:#}").into();
                    } else {
                        this.refresh_status();
                    }
                }))
        } else {
            Button::new("start-detection")
                .primary()
                .label("Start Monitoring")
                .w_full()
                .on_click(cx.listener(|this, _event: &ClickEvent, _window, _cx| {
                    if let Err(err) = this.controller.start_monitoring() {
                        this.status_text = format!("{err:#}").into();
                    } else {
                        this.refresh_status();
                    }
                }))
        };

        let help_button = Button::new("jump-help")
            .ghost()
            .label("Help & Docs")
            .w_full()
            .on_click(cx.listener(|this, _event: &ClickEvent, _window, _cx| {
                this.active_tab = AppTab::Help;
            }));

        let team_summary = selected_team
            .map(|team| format!("Watching {}", team.display_name))
            .unwrap_or_else(|| "Watching any team".to_string());

        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_5()
            .bg(cx.theme().sidebar)
            .text_color(cx.theme().sidebar_foreground)
            .min_w(px(240.0))
            .max_w(px(320.0))
            .flex_shrink()
            .h_full()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(div().text_lg().font_semibold().child("FM Goal Musics"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(self.status_text.clone()),
                    ),
            )
            .child(self.render_status_chip(process_state, cx))
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("Detections today: {}", detection_count)),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(team_summary),
            )
            .child(control_button)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("Navigate"),
                    )
                    .children(AppTab::ALL.iter().enumerate().map(|(idx, tab)| {
                        let tab_value = *tab;
                        Button::new(("sidebar-tab", idx))
                            .ghost()
                            .selected(self.active_tab == tab_value)
                            .w_full()
                            .justify_start()
                            .on_click(cx.listener(
                                move |this, _event: &ClickEvent, _window, _cx| {
                                    this.active_tab = tab_value;
                                },
                            ))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_3()
                                    .child(self.render_tab_icon(tab_value))
                                    .child(div().flex().flex_col().gap_0().child(
                                        div().text_sm().font_semibold().child(tab_value.title()),
                                    )),
                            )
                    })),
            )
            .child(help_button)
    }

    fn render_library_tab(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.controller.state();
        let (music_list, selected_index, ambiance_enabled, ambiance_path) = {
            let guard = state.lock();
            (
                guard.music_list.clone(),
                guard.selected_music_index,
                guard.ambiance_enabled,
                guard.goal_ambiance_path.clone(),
            )
        };

        let list_column = div()
            .flex()
            .flex_col()
            .gap_4()
            .flex_grow()
            .min_w(px(360.0))
            .child(self.render_music_collection_panel(cx, &music_list, selected_index))
            .child(self.render_ambiance_panel(cx, ambiance_enabled, ambiance_path.clone()));

        let inspector = self.render_music_inspector_panel(cx, &music_list, selected_index);

        div()
            .flex()
            .flex_wrap()
            .gap_4()
            .items_start()
            .child(list_column)
            .child(inspector)
    }

    fn render_music_collection_panel(
        &mut self,
        cx: &mut Context<Self>,
        music_list: &[MusicEntry],
        selected_index: Option<usize>,
    ) -> impl IntoElement {
        let add_button = Button::new("library-add")
            .primary()
            .label("Add Tracks")
            .on_click(cx.listener(|_this, _event: &ClickEvent, window, cx| {
                cx.defer_in(window, move |this, _window, cx| {
                    if let Err(err) = this.add_music_via_dialog() {
                        this.status_text = err.into();
                    } else {
                        this.refresh_status();
                    }
                    cx.notify();
                });
            }));

        let preview_button = Button::new("library-preview")
            .label(if self.music_preview_playing {
                "Stop Preview"
            } else {
                "Preview"
            })
            .disabled(selected_index.is_none())
            .on_click(cx.listener(|this, _event: &ClickEvent, _window, cx| {
                if let Err(err) = this.toggle_music_preview() {
                    this.status_text = err.into();
                } else {
                    this.refresh_status();
                }
                cx.notify();
            }));

        let remove_button = Button::new("library-remove")
            .danger()
            .label("Remove")
            .disabled(selected_index.is_none())
            .on_click(cx.listener(|this, _event: &ClickEvent, _window, cx| {
                let selected = {
                    let state = this.controller.state();
                    let guard = state.lock();
                    guard.selected_music_index
                };
                if let Some(idx) = selected {
                    if let Err(err) = this.controller.remove_music(idx) {
                        this.status_text = format!("{err:#}").into();
                    } else {
                        this.refresh_status();
                    }
                    cx.notify();
                }
            }));

        let clear_button = Button::new("library-clear-selection")
            .ghost()
            .label("Clear Selection")
            .disabled(selected_index.is_none())
            .on_click(cx.listener(|this, _event: &ClickEvent, _window, cx| {
                this.controller.select_music(None);
                this.refresh_status();
                cx.notify();
            }));

        let toolbar = div()
            .flex()
            .flex_wrap()
            .gap_2()
            .child(add_button)
            .child(preview_button)
            .child(remove_button)
            .child(clear_button);

        let list_body = if music_list.is_empty() {
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("No celebration tracks yet. Add MP3/WAV files to get started.")
                .into_any_element()
        } else {
            div()
                .flex()
                .flex_col()
                .gap_1()
                .children(music_list.iter().enumerate().map(|(idx, entry)| {
                    let shortcut = entry.shortcut.clone().unwrap_or_default();
                    let label = if shortcut.is_empty() {
                        entry.name.clone()
                    } else {
                        format!("{} · {}", entry.name, shortcut)
                    };
                    Button::new(("music-row", idx))
                        .ghost()
                        .label(label)
                        .selected(selected_index == Some(idx))
                        .on_click(cx.listener(move |this, _event: &ClickEvent, _window, cx| {
                            this.controller.select_music(Some(idx));
                            this.refresh_status();
                            cx.notify();
                        }))
                }))
                .into_any_element()
        };

        div()
            .flex()
            .flex_col()
            .gap_3()
            .bg(cx.theme().list)
            .border_1()
            .border_color(cx.theme().border)
            .rounded_lg()
            .p_3()
            .child(
                div()
                    .text_lg()
                    .font_semibold()
                    .child("🎵 Celebration Tracks"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Files are converted to WAV and stored under config/musics."),
            )
            .child(toolbar)
            .child(list_body)
    }

    fn render_music_inspector_panel(
        &mut self,
        cx: &mut Context<Self>,
        music_list: &[MusicEntry],
        selected_index: Option<usize>,
    ) -> impl IntoElement {
        let selection = selected_index.and_then(|idx| music_list.get(idx));

        let body = if let Some(entry) = selection {
            let file_name = entry
                .path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| entry.name.clone());
            let shortcut = entry
                .shortcut
                .clone()
                .unwrap_or_else(|| "Not set".to_string());
            let location = entry.path.display().to_string();

            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(div().text_lg().font_semibold().child(file_name))
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!("Shortcut: {}", shortcut)),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!("Location: {}", location)),
                )
        } else {
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Select a track to see details and preview information.")
        };

        div()
            .flex()
            .flex_col()
            .gap_3()
            .flex_grow()
            .min_w(px(360.0))
            .bg(cx.theme().list)
            .border_1()
            .border_color(cx.theme().border)
            .rounded_lg()
            .p_3()
            .child(div().text_lg().font_semibold().child("Selection Details"))
            .child(body)
    }

    fn render_ambiance_panel(
        &mut self,
        cx: &mut Context<Self>,
        ambiance_enabled: bool,
        ambiance_path: Option<String>,
    ) -> impl IntoElement {
        let ambiance_file = ambiance_path
            .as_deref()
            .and_then(|p| {
                Path::new(p)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
            })
            .unwrap_or_else(|| "No crowd cheer selected".to_string());

        let toggle = Switch::new("ambiance-enabled")
            .label("Enable crowd ambiance layer")
            .checked(ambiance_enabled)
            .on_click(cx.listener(|this, checked: &bool, _window, cx| {
                if let Err(err) = this.controller.set_ambiance_enabled(*checked) {
                    this.status_text = format!("{err:#}").into();
                } else {
                    this.refresh_status();
                }
                cx.notify();
            }));

        let add_button = Button::new("ambiance-choose")
            .label("Choose Clip")
            .on_click(cx.listener(|_this, _event: &ClickEvent, window, cx| {
                cx.defer_in(window, move |this, _window, cx| {
                    if let Err(err) = this.pick_ambiance_file() {
                        this.status_text = err.into();
                    } else {
                        this.refresh_status();
                    }
                    cx.notify();
                });
            }));

        let preview_button = Button::new("ambiance-preview")
            .label(if self.ambiance_preview_playing {
                "Stop Preview"
            } else {
                "Preview"
            })
            .disabled(ambiance_path.is_none())
            .on_click(cx.listener(|this, _event: &ClickEvent, _window, cx| {
                if let Err(err) = this.toggle_ambiance_preview() {
                    this.status_text = err.into();
                } else {
                    this.refresh_status();
                }
                cx.notify();
            }));

        let clear_button = Button::new("ambiance-clear")
            .ghost()
            .label("Clear")
            .disabled(ambiance_path.is_none())
            .on_click(cx.listener(|this, _event: &ClickEvent, _window, cx| {
                if let Err(err) = this.controller.set_goal_ambiance_path(None) {
                    this.status_text = format!("{err:#}").into();
                } else {
                    this.refresh_status();
                }
                cx.notify();
            }));

        div()
            .flex()
            .flex_col()
            .gap_3()
            .bg(cx.theme().list)
            .border_1()
            .border_color(cx.theme().border)
            .rounded_lg()
            .p_3()
            .child(div().text_lg().font_semibold().child("🌆 Ambiance"))
            .child(toggle)
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .child(add_button)
                    .child(preview_button)
                    .child(clear_button),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("Current clip: {}", ambiance_file)),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Tip: keep ambiance clips short for quick fade-outs."),
            )
    }

    fn render_team_tab(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(database) = self.controller.team_database() else {
            return div()
                .flex()
                .flex_col()
                .gap_2()
                .child(div().text_lg().font_semibold().child("⚽ Team Selection"))
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("Team database not available. Download assets/teams/teams.json to enable this tab."),
                );
        };

        let (selected_team, leagues) = {
            let state = self.controller.state();
            let guard = state.lock();
            (guard.selected_team.clone(), database.get_leagues())
        };

        if self.active_league.is_none() {
            if let Some(team) = selected_team.as_ref() {
                self.active_league = Some(team.league.clone());
            } else if let Some(first) = leagues.first() {
                self.active_league = Some(first.clone());
            }
        }
        let active_league = self.active_league.clone();

        // Leagues rendered horizontally
        let leagues_row = div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Leagues"),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .children(leagues.iter().enumerate().map(|(idx, league)| {
                        let selected = active_league
                            .as_ref()
                            .map(|active| active == league)
                            .unwrap_or(false);
                        Button::new(("league", idx))
                            .ghost()
                            .selected(selected)
                            .label(league.clone())
                            .on_click(cx.listener({
                                let league = league.clone();
                                move |this, _event: &ClickEvent, _window, cx| {
                                    this.active_league = Some(league.clone());
                                    this.controller.set_league(Some(league.clone()));
                                    this.refresh_status();
                                    cx.notify();
                                }
                            }))
                    })),
            );

        let team_grid = if let Some(ref league) = active_league {
            if let Some(teams) = database.get_teams(league) {
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .children(teams.into_iter().enumerate().map(|(idx, (key, team))| {
                        let is_selected = selected_team
                            .as_ref()
                            .map(|st| st.team_key == key)
                            .unwrap_or(false);
                        Button::new(("team-card", idx))
                            .ghost()
                            .selected(is_selected)
                            .label(team.display_name.clone())
                            .on_click(cx.listener({
                                let league = league.clone();
                                let key = key.clone();
                                move |this, _event: &ClickEvent, _window, cx| {
                                    if let Err(err) = this.controller.select_team(&league, &key) {
                                        this.status_text = format!("{err:#}").into();
                                    } else {
                                        this.refresh_status();
                                    }
                                    cx.notify();
                                }
                            }))
                    }))
            } else {
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("No teams found for this league.")
            }
        } else {
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Pick a league to browse teams.")
        };

        let selection_summary = selected_team
            .as_ref()
            .map(|team| format!("Selected: {} ({})", team.display_name, team.league))
            .unwrap_or_else(|| "No team selected — celebrations trigger for all goals.".into());

        let clear_button = Button::new("clear-team")
            .ghost()
            .label("Clear Selection")
            .disabled(selected_team.is_none())
            .on_click(cx.listener(|this, _event: &ClickEvent, _window, cx| {
                if let Err(err) = this.controller.clear_team_selection() {
                    this.status_text = format!("{err:#}").into();
                } else {
                    this.refresh_status();
                }
                this.active_league = None;
                this.add_team_form.selected_league = None;
                cx.notify();
            }));

        // Main column with leagues, divider, teams box, and add-team section
        div()
            .flex()
            .flex_col()
            .gap_4()
            .flex_grow()
            .child(leagues_row)
            .child(div().h(px(1.0)).w_full().bg(cx.theme().border))
            .child(
                div()
                    .bg(cx.theme().list)
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded_lg()
                    .p_3()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div().text_lg().font_semibold().child(
                                    active_league
                                        .clone()
                                        .map(|league| format!("Teams in {league}"))
                                        .unwrap_or_else(|| "Teams".to_string()),
                                ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(selection_summary),
                                    )
                                    .child(clear_button),
                            ),
                    )
                    .child(team_grid),
            )
            .child(self.render_add_team_section(cx, &leagues))
    }

    fn submit_team_form(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        let team_name = {
            let value = self.add_team_form.team_name_input.read(cx).value();
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err("Enter a team name".to_string());
            }
            trimmed.to_string()
        };

        let league_name = if self.add_team_form.use_existing_league {
            self.add_team_form
                .selected_league
                .clone()
                .or_else(|| self.active_league.clone())
                .ok_or_else(|| "Select a league".to_string())?
        } else {
            let value = self.add_team_form.new_league_input.read(cx).value();
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err("Enter a league name".to_string());
            }
            trimmed.to_string()
        };

        let create_new = !self.add_team_form.use_existing_league;
        let variations_text = self.add_team_form.variations_input.read(cx).value();
        let mut variations: Vec<String> = variations_text
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .map(|line| line.to_string())
            .collect();
        if !variations
            .iter()
            .any(|variation| variation.eq_ignore_ascii_case(&team_name))
        {
            variations.insert(0, team_name.clone());
        }

        self.controller
            .add_custom_team(
                league_name.clone(),
                team_name.clone(),
                variations,
                create_new,
            )
            .map_err(|err| format!("{err:#}"))?;

        self.active_league = Some(league_name.clone());
        self.add_team_form.selected_league = Some(league_name.clone());
        self.controller.set_league(Some(league_name));
        self.clear_team_form(window, cx);
        self.refresh_status();
        Ok(())
    }

    fn clear_team_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.add_team_form.error = None;
        self.add_team_form
            .team_name_input
            .update(cx, |state, cx| state.set_value("", window, cx));
        self.add_team_form
            .new_league_input
            .update(cx, |state, cx| state.set_value("", window, cx));
        self.add_team_form
            .variations_input
            .update(cx, |state, cx| state.set_value("", window, cx));
    }

    fn render_add_team_section(
        &mut self,
        cx: &mut Context<Self>,
        leagues: &[String],
    ) -> impl IntoElement {
        let toggle_label = if self.add_team_form.expanded {
            "Hide form"
        } else {
            "Show form"
        };

        let toggle_button = Button::new("toggle-add-team")
            .ghost()
            .label(toggle_label)
            .on_click(cx.listener(|this, _event: &ClickEvent, _window, _cx| {
                this.add_team_form.expanded = !this.add_team_form.expanded;
            }));

        let header = div()
            .flex()
            .items_center()
            .justify_between()
            .child(div().text_lg().font_semibold().child("➕ Add Custom Team"))
            .child(toggle_button);

        let mut container_content = div()
            .flex()
            .flex_col()
            .gap_2()
            .border_1()
            .border_color(cx.theme().border)
            .rounded_md()
            .p_3()
            .child(header);

        if self.add_team_form.expanded {
            let league_switch = Switch::new("use-existing-league")
                .label("Add to an existing league")
                .checked(self.add_team_form.use_existing_league)
                .on_click(cx.listener(|this, checked: &bool, _window, _cx| {
                    this.add_team_form.use_existing_league = *checked;
                    if *checked && this.add_team_form.selected_league.is_none() {
                        this.add_team_form.selected_league = this.active_league.clone();
                    }
                }));

            let league_selector = if self.add_team_form.use_existing_league {
                if leagues.is_empty() {
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("No leagues available yet.")
                } else {
                    div()
                        .flex()
                        .flex_wrap()
                        .gap_1()
                        .children(leagues.iter().enumerate().map(|(idx, league)| {
                            let selected = self
                                .add_team_form
                                .selected_league
                                .as_ref()
                                .map(|current| current == league)
                                .unwrap_or(false);
                            Button::new(("team-form-league", idx))
                                .ghost()
                                .label(league.clone())
                                .selected(selected)
                                .on_click(cx.listener({
                                    let league = league.clone();
                                    move |this, _event: &ClickEvent, _window, _cx| {
                                        this.add_team_form.selected_league = Some(league.clone());
                                    }
                                }))
                        }))
                }
            } else {
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("New league name"),
                    )
                    .child(Input::new(&self.add_team_form.new_league_input).cleanable(true))
            }
            .into_any_element();

            let team_input = div()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("Team display name"),
                )
                .child(Input::new(&self.add_team_form.team_name_input).cleanable(true));

            let variations_input = div()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("Name variations (one per line)"),
                )
                .child(
                    Input::new(&self.add_team_form.variations_input)
                        .cleanable(true)
                        .h(px(140.0)),
                );

            let team_name_filled = {
                let value = self.add_team_form.team_name_input.read(cx).value();
                !value.trim().is_empty()
            };

            let league_ready = if self.add_team_form.use_existing_league {
                self.add_team_form.selected_league.is_some()
            } else {
                let value = self.add_team_form.new_league_input.read(cx).value();
                !value.trim().is_empty()
            };

            let submit_disabled = !(team_name_filled && league_ready);

            let submit_button = Button::new("submit-add-team")
                .primary()
                .label("Add Team")
                .disabled(submit_disabled)
                .on_click(cx.listener(|this, _event: &ClickEvent, window, cx| {
                    match this.submit_team_form(window, cx) {
                        Ok(()) => this.add_team_form.error = None,
                        Err(err) => this.add_team_form.error = Some(err),
                    }
                }));

            let clear_button = Button::new("clear-add-team")
                .ghost()
                .label("Clear")
                .on_click(cx.listener(|this, _event: &ClickEvent, window, cx| {
                    this.clear_team_form(window, cx);
                }));

            let mut form = div()
                .flex()
                .flex_col()
                .gap_2()
                .child(league_switch)
                .child(league_selector)
                .child(team_input)
                .child(variations_input)
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(submit_button)
                        .child(clear_button),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("Tip: include abbreviations and nicknames in the variations list."),
                );

            if let Some(error) = &self.add_team_form.error {
                form = form.child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().danger)
                        .child(error.clone()),
                );
            }

            container_content = container_content.child(form);
        }

        // Make the entire container clickable when form is collapsed
        let container = if !self.add_team_form.expanded {
            Button::new("add-team-container")
                .ghost()
                .w_full()
                .child(container_content)
                .on_click(cx.listener(|this, _event: &ClickEvent, _window, _cx| {
                    this.add_team_form.expanded = !this.add_team_form.expanded;
                }))
                .into_any_element()
        } else {
            container_content.into_any_element()
        };

        container
    }

    fn render_detection_tab(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let (region, monitor_index, morph_open, preview_path) = {
            let state = self.controller.state();
            let guard = state.lock();
            (
                guard.capture_region,
                guard.selected_monitor_index,
                guard.enable_morph_open,
                guard.preview_image_path.clone(),
            )
        };

        let mut panel = div()
            .flex()
            .flex_col()
            .gap_4()
            .child(self.render_capture_section(region, monitor_index, cx))
            .child(self.render_detection_section(morph_open, cx))
            .child(self.render_preview_panel(preview_path, cx));

        if let Some(selector) = self.render_region_selector(cx) {
            panel = panel.child(selector);
        }

        panel
    }

    fn render_settings_tab(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let auto_updates = {
            let state = self.controller.state();
            let guard = state.lock();
            guard.auto_check_updates
        };

        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(self.render_audio_section(cx))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_4()
                    .child(
                        div()
                            .flex_grow()
                            .min_w(px(320.0))
                            .child(self.render_update_section(auto_updates, cx)),
                    )
                    .child(
                        div()
                            .flex_grow()
                            .min_w(px(320.0))
                            .child(self.render_diagnostics_section(cx)),
                    ),
            )
    }

    fn render_capture_section(
        &mut self,
        region: [u32; 4],
        monitor_index: usize,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let region_controls = ["X", "Y", "Width", "Height"]
            .iter()
            .enumerate()
            .map(|(idx, label)| {
                let value = region[idx];
                let idx_copy = idx;
                let decrease = Button::new(("region-dec", idx))
                    .ghost()
                    .label("-10")
                    .on_click(cx.listener(move |this, _event: &ClickEvent, _window, _cx| {
                        if let Err(err) = this.controller.adjust_capture_region(idx_copy, -10) {
                            this.status_text = format!("{err:#}").into();
                        } else {
                            this.refresh_status();
                        }
                    }));
                let idx_copy = idx;
                let increase = Button::new(("region-inc", idx))
                    .ghost()
                    .label("+10")
                    .on_click(cx.listener(move |this, _event: &ClickEvent, _window, _cx| {
                        if let Err(err) = this.controller.adjust_capture_region(idx_copy, 10) {
                            this.status_text = format!("{err:#}").into();
                        } else {
                            this.refresh_status();
                        }
                    }));

                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(*label),
                    )
                    .child(div().text_lg().child(value.to_string()))
                    .child(div().flex().gap_1().child(decrease).child(increase))
            })
            .collect::<Vec<_>>();

        let reset_button = Button::new("reset-region")
            .ghost()
            .label("Reset Region")
            .on_click(cx.listener(|this, _event: &ClickEvent, _window, _cx| {
                let defaults = [0, 900, 1024, 50];
                if let Err(err) = this.controller.reset_capture_region(defaults) {
                    this.status_text = format!("{err:#}").into();
                } else {
                    this.refresh_status();
                }
            }));

        let monitor_dropdown = Select::new(&self.monitor_select)
            .small()
            .placeholder("Choose monitor")
            .menu_width(px(260.0));

        let select_region_button = Button::new("select-region")
            .primary()
            .label("Select Region")
            .on_click(cx.listener(|_this, _event: &ClickEvent, window, cx| {
                cx.defer_in(window, move |this, _window, cx| {
                    match this.controller.capture_fullscreen_for_selection() {
                        Ok(capture) => {
                            this.region_selection = Some(RegionSelection::from_capture(capture));
                            this.status_text =
                                "Drag on the screenshot to define the capture area.".into();
                        }
                        Err(err) => {
                            this.status_text = format!("{err:#}").into();
                        }
                    }
                    cx.notify();
                });
            }));

        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(div().text_lg().font_semibold().child("🖥️ Capture Region"))
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(
                    "Fine-tune the OCR region. Keep it tight around “GOAL FOR” to reduce noise.",
                ),
            )
            .child(div().flex().gap_4().children(region_controls))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .child(select_region_button)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .min_w(px(200.0))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Active monitor"),
                            )
                            .child(monitor_dropdown),
                    )
                    .child(reset_button)
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!(
                                "Monitoring Display {} of {}",
                                monitor_index + 1,
                                self.monitor_options.len().max(1)
                            )),
                    ),
            )
    }

    fn render_detection_section(
        &mut self,
        morph_open: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let ocr_value = self.slider_value(&self.ocr_slider, cx);
        let debounce_value = self.slider_value(&self.debounce_slider, cx);
        let ocr_label = if ocr_value <= 0.5 {
            "Auto (Otsu)".to_string()
        } else {
            format!("{:.0}", ocr_value)
        };

        let debounce_label = format!("{:.1}s", debounce_value / 1000.0);

        let morph_switch = Switch::new("morph-open")
            .label("Enable morphological opening (reduces bold UI noise)")
            .checked(morph_open)
            .on_click(cx.listener(|this, checked: &bool, _window, _cx| {
                if let Err(err) = this.controller.set_morph_open(*checked) {
                    this.status_text = format!("{err:#}").into();
                } else {
                    this.refresh_status();
                }
            }));

        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(div().text_lg().font_semibold().child("🔍 Detection"))
            .child(morph_switch)
            .child(
                div()
                    .flex()
                    .justify_between()
                    .child(div().text_sm().font_semibold().child("OCR Threshold"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(ocr_label),
                    ),
            )
            .child(Slider::new(&self.ocr_slider))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .child(div().text_sm().font_semibold().child("Goal debounce"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(debounce_label),
                    ),
            )
            .child(Slider::new(&self.debounce_slider))
    }

    fn render_preview_panel(
        &mut self,
        preview_path: Option<PathBuf>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let capture_button = Button::new("capture-preview")
            .primary()
            .label("Capture Preview")
            .on_click(cx.listener(|_this, _event: &ClickEvent, window, cx| {
                cx.defer_in(window, move |this, _window, _cx| {
                    if let Err(err) = this.controller.capture_preview() {
                        this.status_text = format!("{err:#}").into();
                    } else {
                        this.refresh_status();
                    }
                });
            }));

        let preview_display = if let Some(path) = preview_path {
            img(path)
                .object_fit(ObjectFit::Cover)
                .w_full()
                .h(px(220.0))
                .rounded_lg()
                .into_any_element()
        } else {
            div()
                .flex()
                .items_center()
                .justify_center()
                .h(px(220.0))
                .rounded_lg()
                .bg(cx.theme().list)
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Capture a preview to verify the monitored region.")
                .into_any_element()
        };

        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(div().text_lg().font_semibold().child("👁️ Region Preview"))
                    .child(capture_button),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(
                        "Use preview to confirm your capture window aligns with the scoreboard.",
                    ),
            )
            .child(preview_display)
    }

    fn render_region_selector(&mut self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let selection = self.region_selection.as_ref()?;
        let (display_w, display_h) = selection.display_size();
        let overlay: AnyElement = selection
            .overlay_rect()
            .map(|(x, y, w, h)| {
                div()
                    .absolute()
                    .left(px(x))
                    .top(px(y))
                    .w(px(w))
                    .h(px(h))
                    .border_2()
                    .border_color(cx.theme().primary)
                    .bg(cx.theme().primary.opacity(0.15))
                    .into_any_element()
            })
            .unwrap_or_else(|| div().into_any_element());

        let view_handle = cx.entity().clone();
        let canvas = div()
            .relative()
            .w(px(display_w))
            .h(px(display_h))
            .border_1()
            .border_color(cx.theme().border)
            .rounded_lg()
            .overflow_hidden()
            .cursor(CursorStyle::Crosshair)
            .on_mouse_down(MouseButton::Left, cx.listener(Self::region_mouse_down))
            .on_mouse_move(cx.listener(Self::region_mouse_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::region_mouse_up))
            .child(
                img(selection.image_path.clone())
                    .object_fit(ObjectFit::Contain)
                    .w(px(display_w))
                    .h(px(display_h)),
            )
            .child(overlay)
            .on_children_prepainted(move |child_bounds, _window, cx| {
                if let Some(bounds) = child_bounds.first().copied() {
                    let handle = view_handle.clone();
                    handle.update(cx, |this, _| {
                        this.region_canvas_bounds = Some(bounds);
                        if let Some(selection) = this.region_selection.as_mut() {
                            selection.ensure_local(bounds);
                        }
                    });
                }
            });

        let apply_button = Button::new("apply-region")
            .primary()
            .label("Apply Selection")
            .on_click(cx.listener(|this, _event: &ClickEvent, _window, cx| {
                let mut applied = false;
                if let Some(selection) = this.region_selection.as_ref() {
                    if let Some(region) = selection.logical_rect() {
                        match this.controller.update_capture_region(region) {
                            Ok(()) => {
                                this.refresh_status();
                                applied = true;
                            }
                            Err(err) => this.status_text = format!("{err:#}").into(),
                        }
                    } else {
                        this.status_text =
                            "Drag a rectangle on the screenshot before applying.".into();
                    }
                }
                if applied {
                    this.region_selection = None;
                    this.region_canvas_bounds = None;
                }
                cx.notify();
            }));

        let cancel_button = Button::new("cancel-region-selector")
            .ghost()
            .label("Cancel")
            .on_click(cx.listener(|this, _event: &ClickEvent, _window, cx| {
                this.region_selection = None;
                this.region_canvas_bounds = None;
                cx.notify();
            }));

        Some(
            div()
                .flex()
                .flex_col()
                .gap_3()
                .border_1()
                .border_color(cx.theme().border)
                .rounded_lg()
                .p_4()
                .child(div().text_lg().font_semibold().child("📐 Region Selection"))
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("Drag across the screenshot to capture the exact scoreboard area."),
                )
                .child(canvas)
                .child(
                    div()
                        .flex()
                        .justify_end()
                        .gap_2()
                        .child(cancel_button)
                        .child(apply_button),
                )
                .into_any_element(),
        )
    }

    fn region_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(selection) = self.region_selection.as_mut() {
            selection.start_drag(event.position, self.region_canvas_bounds);
            cx.notify();
        }
    }

    fn region_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(selection) = self.region_selection.as_mut() {
            selection.update_drag(event.position, self.region_canvas_bounds);
            cx.notify();
        }
    }

    fn region_mouse_up(
        &mut self,
        event: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(selection) = self.region_selection.as_mut() {
            selection.finish_drag(event.position, self.region_canvas_bounds);
            cx.notify();
        }
    }

    fn render_audio_section(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let music_volume = self.slider_value(&self.music_volume_slider, cx);
        let ambiance_volume = self.slider_value(&self.ambiance_volume_slider, cx);
        let music_length = self.slider_value(&self.music_length_slider, cx);
        let ambiance_length = self.slider_value(&self.ambiance_length_slider, cx);

        let slider_row = |label: &str, value: String, slider: Slider| {
            let label_text = label.to_string();
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .child(div().text_sm().font_semibold().child(label_text))
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(value),
                        ),
                )
                .child(slider)
        };

        div()
            .border_1()
            .border_color(cx.theme().border)
            .rounded_lg()
            .p_4()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(div().text_lg().font_semibold().child("🔊 Playback & Mix"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("Balance celebration tracks and ambiance timing."),
                    ),
            )
            .child(slider_row(
                "Music volume",
                format!("{:.0}%", music_volume),
                Slider::new(&self.music_volume_slider),
            ))
            .child(slider_row(
                "Ambiance volume",
                format!("{:.0}%", ambiance_volume),
                Slider::new(&self.ambiance_volume_slider),
            ))
            .child(slider_row(
                "Music playback length",
                format!("{:.0}s", music_length),
                Slider::new(&self.music_length_slider),
            ))
            .child(slider_row(
                "Ambiance playback length",
                format!("{:.0}s", ambiance_length),
                Slider::new(&self.ambiance_length_slider),
            ))
    }

    fn render_update_section(
        &mut self,
        auto_updates: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let switch = Switch::new("auto-updates")
            .label("Check for updates on startup")
            .checked(auto_updates)
            .on_click(cx.listener(|this, checked: &bool, _event, _cx| {
                if let Err(err) = this.controller.set_auto_check_updates(*checked) {
                    this.status_text = format!("{err:#}").into();
                } else {
                    this.refresh_status();
                }
            }));

        let manual_check = Button::new("manual-update-check")
            .label("Check for updates now")
            .on_click(cx.listener(|this, _event: &ClickEvent, _window, _cx| {
                this.controller.check_for_updates();
                this.refresh_status();
            }));

        div()
            .border_1()
            .border_color(cx.theme().border)
            .rounded_lg()
            .p_4()
            .flex()
            .flex_col()
            .gap_3()
            .child(div().text_lg().font_semibold().child("🔄 Updates"))
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Stay current with the latest detectors and fixes."),
            )
            .child(switch)
            .child(manual_check)
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("Status: {}", self.status_text)),
            )
    }

    fn render_diagnostics_section(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let config_path = self.controller.config_file_path();
        let logs_path = self.controller.logs_directory();

        let config_display = config_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "Unavailable".to_string());
        let logs_display = logs_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "Unavailable".to_string());

        let config_path_for_button = config_path.clone();
        let logs_path_for_button = logs_path.clone();

        let open_logs_button = Button::new("open-logs-folder")
            .ghost()
            .label("Open Logs Folder")
            .on_click(cx.listener(move |this, _event: &ClickEvent, _window, cx| {
                if let Some(path) = logs_path_for_button.clone() {
                    let label = path.display().to_string();
                    match open::that(&path) {
                        Ok(()) => {
                            this.controller
                                .set_status(format!("Opened logs at {}", label));
                            this.refresh_status();
                        }
                        Err(err) => {
                            this.status_text = format!("Failed to open logs: {err:#}").into()
                        }
                    }
                } else {
                    this.status_text = "Log folder unavailable.".into();
                }
                cx.notify();
            }));

        let reveal_config_button = Button::new("reveal-config-file")
            .ghost()
            .label("Reveal Config File")
            .on_click(cx.listener(move |this, _event: &ClickEvent, _window, cx| {
                if let Some(path) = config_path_for_button.clone() {
                    let label = path.display().to_string();
                    match open::that(&path) {
                        Ok(()) => {
                            this.controller
                                .set_status(format!("Opened config {}", label));
                            this.refresh_status();
                        }
                        Err(err) => {
                            this.status_text = format!("Failed to open config: {err:#}").into();
                        }
                    }
                } else {
                    this.status_text = "Config file unavailable.".into();
                }
                cx.notify();
            }));

        div()
            .border_1()
            .border_color(cx.theme().border)
            .rounded_lg()
            .p_4()
            .flex()
            .flex_col()
            .gap_3()
            .child(div().text_lg().font_semibold().child("🩺 Diagnostics"))
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Quick links for troubleshooting and sharing logs."),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Config file"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().foreground)
                                    .child(config_display),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Logs folder"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().foreground)
                                    .child(logs_display),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .child(reveal_config_button)
                    .child(open_logs_button),
            )
    }

    fn slider_value(&self, slider: &Entity<SliderState>, cx: &mut Context<Self>) -> f32 {
        slider.read(cx).value().start()
    }

    fn toggle_music_preview(&mut self) -> Result<(), String> {
        if self.music_preview_playing {
            self.stop_music_preview();
            self.controller.set_status("Music preview stopped");
            self.refresh_status();
            return Ok(());
        }

        let path = self
            .selected_music_path()
            .ok_or_else(|| "Select a music file to preview.".to_string())?;

        self.ensure_music_preview_loaded(&path)?;

        if let Some(preview) = self.music_preview.as_ref() {
            preview.manager.stop();
            preview
                .manager
                .play_sound()
                .map_err(|e| format!("Preview failed: {e}"))?;
            self.music_preview_playing = true;
            self.controller
                .set_status(format!("Previewing {}", preview.path.display()));
            self.refresh_status();
        }

        Ok(())
    }

    fn toggle_ambiance_preview(&mut self) -> Result<(), String> {
        if self.ambiance_preview_playing {
            self.stop_ambiance_preview();
            self.controller.set_status("Ambiance preview stopped");
            self.refresh_status();
            return Ok(());
        }

        let path = {
            let state = self.controller.state();
            let guard = state.lock();
            guard
                .goal_ambiance_path
                .as_ref()
                .map(|p| PathBuf::from(p))
                .ok_or_else(|| "Add a goal cheer sound first to preview.".to_string())?
        };

        self.ensure_ambiance_preview_loaded(&path)?;

        if let Some(preview) = self.ambiance_preview.as_ref() {
            preview.manager.stop();
            preview
                .manager
                .play_sound()
                .map_err(|e| format!("Ambiance preview failed: {e}"))?;
            self.ambiance_preview_playing = true;
            self.controller
                .set_status(format!("Previewing ambiance {}", preview.path.display()));
            self.refresh_status();
        }

        Ok(())
    }

    fn selected_music_path(&self) -> Option<PathBuf> {
        let state = self.controller.state();
        let guard = state.lock();
        guard
            .selected_music_index
            .and_then(|idx| guard.music_list.get(idx))
            .map(|entry| entry.path.clone())
    }

    fn ensure_music_preview_loaded(&mut self, path: &Path) -> Result<(), String> {
        let needs_reload = self
            .music_preview
            .as_ref()
            .map(|preview| preview.path.as_path() != path)
            .unwrap_or(true);
        if !needs_reload {
            return Ok(());
        }

        self.stop_music_preview();
        let data = self.get_or_load_audio_data(path)?;
        let manager =
            AudioManager::from_preloaded(data).map_err(|e| format!("Preview init failed: {e}"))?;
        self.music_preview = Some(PreviewSound {
            manager,
            path: path.to_path_buf(),
        });
        Ok(())
    }

    fn ensure_ambiance_preview_loaded(&mut self, path: &Path) -> Result<(), String> {
        let needs_reload = self
            .ambiance_preview
            .as_ref()
            .map(|preview| preview.path.as_path() != path)
            .unwrap_or(true);
        if !needs_reload {
            return Ok(());
        }

        self.stop_ambiance_preview();
        let data = self.get_or_load_audio_data(path)?;
        let manager =
            AudioManager::from_preloaded(data).map_err(|e| format!("Ambiance init failed: {e}"))?;
        self.ambiance_preview = Some(PreviewSound {
            manager,
            path: path.to_path_buf(),
        });
        Ok(())
    }

    fn get_or_load_audio_data(&mut self, path: &Path) -> Result<Arc<Vec<u8>>, String> {
        if let Some(cache) = &self.cached_audio {
            if cache.path.as_path() == path {
                return Ok(Arc::clone(&cache.data));
            }
        }

        let bytes = fs::read(path)
            .map_err(|e| format!("Failed to read audio file '{}': {e}", path.display()))?;
        let data = Arc::new(bytes);
        self.cached_audio = Some(CachedAudio {
            path: path.to_path_buf(),
            data: Arc::clone(&data),
        });
        Ok(data)
    }

    fn stop_music_preview(&mut self) {
        if let Some(preview) = &self.music_preview {
            preview.manager.stop();
        }
        self.music_preview = None;
        self.music_preview_playing = false;
    }

    fn stop_ambiance_preview(&mut self) {
        if let Some(preview) = &self.ambiance_preview {
            preview.manager.stop();
        }
        self.ambiance_preview = None;
        self.ambiance_preview_playing = false;
    }

    fn render_status_chip(
        &self,
        process_state: ProcessState,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        match process_state {
            ProcessState::Running { .. } => div()
                .px_3()
                .py_1()
                .rounded_full()
                .text_sm()
                .font_semibold()
                .bg(cx.theme().success)
                .text_color(cx.theme().success_foreground)
                .child("Running"),
            ProcessState::Stopped => div()
                .px_3()
                .py_1()
                .rounded_full()
                .text_sm()
                .font_semibold()
                .bg(cx.theme().danger)
                .text_color(cx.theme().danger_foreground)
                .child("Stopped"),
            // Transitional states are rendered as Idle (gray)
            ProcessState::Starting | ProcessState::Stopping => div()
                .px_3()
                .py_1()
                .rounded_full()
                .text_sm()
                .font_semibold()
                .bg(cx.theme().muted)
                .text_color(cx.theme().muted_foreground)
                .child("Idle"),
        }
    }

    fn add_music_via_dialog(&mut self) -> Result<(), String> {
        let picks = rfd::FileDialog::new()
            .add_filter("Audio", &["mp3", "wav", "ogg"])
            .pick_files()
            .or_else(|| {
                rfd::FileDialog::new()
                    .add_filter("Audio", &["mp3", "wav", "ogg"])
                    .pick_file()
                    .map(|file| vec![file])
            });

        let Some(paths) = picks else {
            return Ok(());
        };

        let mut last_error = None;
        for path in paths {
            if let Err(err) = self.controller.add_music_file(path.clone()) {
                last_error = Some(format!("{err:#}"));
            }
        }

        if let Some(err) = last_error {
            Err(err)
        } else {
            Ok(())
        }
    }

    fn pick_ambiance_file(&mut self) -> Result<(), String> {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("WAV", &["wav"])
            .pick_file()
        else {
            return Ok(());
        };

        self.controller
            .set_goal_ambiance_path(Some(path))
            .map_err(|err| format!("{err:#}"))
    }

    fn render_help_tab(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let quick_steps = [
            "Add celebration music and ambiance tracks from the Library tab.",
            "Pick your preferred leagues/teams so detections stay relevant.",
            "Capture and preview the goal banner region under Detection.",
            "Start monitoring (or press Cmd+1) before your match kicks off.",
        ];

        let quick_step_rows = quick_steps
            .iter()
            .map(|step| {
                div()
                    .flex()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("•"),
                    )
                    .child(div().text_sm().child(*step))
            })
            .collect::<Vec<_>>();

        let quick_start_button = self.help_link_button(
            "help-open-plan",
            "Open Setup Guide (Doc/Plan.md)",
            PathBuf::from("Doc/Plan.md"),
            cx,
        );

        let quick_start_card = div()
            .border_1()
            .border_color(cx.theme().border)
            .rounded_lg()
            .p_4()
            .flex()
            .flex_col()
            .gap_3()
            .child(div().text_lg().font_semibold().child("🚀 Quick Start"))
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Follow these steps to get cheers playing in minutes."),
            )
            .child(div().flex().flex_col().gap_2().children(quick_step_rows))
            .child(quick_start_button);

        let shortcuts = [
            ("Cmd + 1", "Start/stop monitoring instantly"),
            ("Cmd + Shift + R", "Capture a new goal region"),
            ("Cmd + K", "Focus the help/search drawer"),
            ("Cmd + Option + ←/→", "Switch tabs"),
        ];

        let shortcut_rows = shortcuts
            .iter()
            .map(|(combo, desc)| {
                div()
                    .flex()
                    .justify_between()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .font_semibold()
                            .text_color(cx.theme().primary)
                            .child(*combo),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().foreground)
                            .child(*desc),
                    )
            })
            .collect::<Vec<_>>();

        let shortcuts_button = self.help_link_button(
            "help-open-readme",
            "View full shortcut list (README.md)",
            PathBuf::from("README.md"),
            cx,
        );

        let shortcuts_card = div()
            .border_1()
            .border_color(cx.theme().border)
            .rounded_lg()
            .p_4()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .text_lg()
                    .font_semibold()
                    .child("⌨️ Keyboard Shortcuts"),
            )
            .child(div().flex().flex_col().gap_2().children(shortcut_rows))
            .child(shortcuts_button);

        let troubleshooting_tips = [
            "Region preview empty? Re-run Select Region and ensure permissions are granted.",
            "Missed detections? Increase OCR threshold or enable morphological opening.",
            "Audio stutters? Shorten clip length or lower volume balancing.",
        ];

        let troubleshooting_rows = troubleshooting_tips
            .iter()
            .map(|tip| {
                div()
                    .flex()
                    .gap_2()
                    .child(div().text_sm().text_color(cx.theme().accent).child("•"))
                    .child(div().text_sm().child(*tip))
            })
            .collect::<Vec<_>>();

        let logs_path = self.controller.logs_directory();
        let logs_button = {
            let logs_for_button = logs_path.clone();
            Button::new("help-open-logs")
                .ghost()
                .label("Open Logs Folder")
                .w_full()
                .on_click(cx.listener(move |this, _event: &ClickEvent, _window, cx| {
                    if let Some(path) = logs_for_button.clone() {
                        let label = path.display().to_string();
                        match open::that(&path) {
                            Ok(()) => {
                                this.controller
                                    .set_status(format!("Opened logs at {}", label));
                                this.refresh_status();
                            }
                            Err(err) => {
                                this.status_text = format!("Failed to open logs: {err:#}").into();
                            }
                        }
                    } else {
                        this.status_text = "Log folder unavailable.".into();
                    }
                    cx.notify();
                }))
        };

        let troubleshooting_card = div()
            .border_1()
            .border_color(cx.theme().border)
            .rounded_lg()
            .p_4()
            .flex()
            .flex_col()
            .gap_3()
            .child(div().text_lg().font_semibold().child("🧪 Troubleshooting"))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .children(troubleshooting_rows),
            )
            .child(div().flex().flex_wrap().gap_2().child(logs_button).child(
                self.help_link_button(
                    "help-open-detection-docs",
                    "Review detection tuning (Doc/Design.md)",
                    PathBuf::from("Doc/Design.md"),
                    cx,
                ),
            ));

        let support_card = div()
            .border_1()
            .border_color(cx.theme().border)
            .rounded_lg()
            .p_4()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .text_lg()
                    .font_semibold()
                    .child("🌐 Reference & Support"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Deep-dive into architecture, specs, and planning documents."),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(self.help_link_button(
                        "help-open-architecture",
                        "ARCHITECTURE.md",
                        PathBuf::from("ARCHITECTURE.md"),
                        cx,
                    ))
                    .child(self.help_link_button(
                        "help-open-project",
                        "openspec/project.md",
                        PathBuf::from("openspec/project.md"),
                        cx,
                    ))
                    .child(self.help_link_button(
                        "help-open-design",
                        "Doc/Design.md",
                        PathBuf::from("Doc/Design.md"),
                        cx,
                    )),
            );

        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_4()
                    .child(div().flex_grow().min_w(px(320.0)).child(quick_start_card))
                    .child(div().flex_grow().min_w(px(320.0)).child(shortcuts_card)),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_4()
                    .child(
                        div()
                            .flex_grow()
                            .min_w(px(320.0))
                            .child(troubleshooting_card),
                    )
                    .child(div().flex_grow().min_w(px(320.0)).child(support_card)),
            )
    }

    fn help_link_button(
        &mut self,
        id: &'static str,
        label: &'static str,
        target: PathBuf,
        cx: &mut Context<Self>,
    ) -> Button {
        Button::new(id)
            .ghost()
            .label(label)
            .w_full()
            .on_click(cx.listener(move |this, _event: &ClickEvent, _window, cx| {
                match open::that(&target) {
                    Ok(()) => {
                        this.controller.set_status(format!("Opened {label}"));
                        this.refresh_status();
                    }
                    Err(err) => {
                        this.status_text = format!("Failed to open {label}: {err:#}").into();
                    }
                }
                cx.notify();
            }))
    }

    fn render_footer(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.controller.state();
        let guard = state.lock();
        let region = guard.capture_region;
        drop(guard);

        div()
            .flex()
            .flex_wrap()
            .justify_between()
            .gap_2()
            .text_sm()
            .text_color(cx.theme().muted_foreground)
            .child(format!(
                "Capture region: X={}, Y={}, W={}, H={}",
                region[0], region[1], region[2], region[3]
            ))
            .child(
                div().child("Hotkeys: Cmd+1 start/stop · Cmd+Shift+R region selector · Cmd+K help"),
            )
    }
}

impl Render for MainView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.status_text = self.controller.status_message().into();

        let content = match self.active_tab {
            AppTab::Dashboard => self.render_dashboard_tab(cx).into_any_element(),
            AppTab::Library => self.render_library_tab(cx).into_any_element(),
            AppTab::TeamSelection => self.render_team_tab(cx).into_any_element(),
            AppTab::Detection => self.render_detection_tab(cx).into_any_element(),
            AppTab::Settings => self.render_settings_tab(cx).into_any_element(),
            AppTab::Help => self.render_help_tab(cx).into_any_element(),
        };

        div()
            .flex()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(self.render_sidebar(cx))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_grow()
                    .min_w(px(360.0))
                    .h_full()
                    .gap_4()
                    .p_5()
                    .child(
                        div()
                            .pr(px(6.0))
                            .child(content)
                            .scrollable(ScrollbarAxis::Vertical)
                            .flex_grow(),
                    )
                    .child(self.render_footer(cx)),
            )
    }
}

impl Focusable for MainView {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
