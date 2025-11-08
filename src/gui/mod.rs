mod controller;
mod state;
mod theme;
mod view;

use controller::GuiController;
use gpui::{px, size, App, AppContext, Application, Bounds, WindowBounds, WindowOptions};
use view::MainView;

pub fn run() -> anyhow::Result<()> {
    let controller = GuiController::new()?;
    let application = Application::new();

    application.run(move |cx: &mut App| {
        gpui_component::init(cx);
        theme::install(cx);

        let bounds = Bounds::centered(None, size(px(1180.0), px(760.0)), cx);
        let controller = controller.clone();

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |window, cx| {
                let controller = controller.clone();
                let view = cx.new(|cx| MainView::new(window, cx, controller.clone()));
                cx.new(|cx| gpui_component::Root::new(view.into(), window, cx))
            },
        )
        .expect("failed to open GPUI window");

        cx.activate(true);
    });

    Ok(())
}
