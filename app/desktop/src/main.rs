mod command;
mod backend_panel;
mod window;
mod sidebar;
mod utility;

use eframe::egui;
use crate::window::desktop_views_window;
use futures::executor::block_on;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().
            with_inner_size([320.0, 240.0]).
            with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| Box::new(<WrapApp>::new(cc))),
    )
}

/// The state that we persist (serialize).
#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct State {
    backend_panel: backend_panel::BackendPanel,
}

#[derive(Clone, Copy, Debug)]
#[must_use]
enum Command {
    Nothing,
    ResetEverything,
}

struct WrapApp {
    state: State,
    sidebar_ui:SidebarUI,
    sidebar_settings: SidebarSettingEnum,
    name: String,
    age: u32,
    demo_desktop_window:desktop_views_window::DesktopView,
}

struct SidebarUI{
    general:sidebar::SidebarSettingGeneral,
    other:sidebar::SidebarSettingOther,
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
enum SidebarSettingEnum {
    SidebarGeneral,
    Other,
}

impl Default for SidebarSettingEnum {
    fn default() -> Self {
        Self::SidebarGeneral
    }
}

impl eframe::App for WrapApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx,  |ui| {
            self.state.backend_panel.update(ctx, frame);

            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.sidebar_settings, SidebarSettingEnum::SidebarGeneral, "General");
                ui.selectable_value(&mut self.sidebar_settings, SidebarSettingEnum::Other, "Other");
            });
            ui.separator();

            match self.sidebar_settings {
                SidebarSettingEnum::SidebarGeneral => {
                    self.sidebar_ui.general.ui(ui);
                }
                SidebarSettingEnum::Other => {
                    self.sidebar_ui.other.ui(ui);
                }
            }

            self.state.backend_panel.end_of_frame(ctx);

            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name)
                    .labelled_by(name_label.id);
            });
            if ui.button("Increment").clicked() {
                self.age += 1;
                let addr:String = "127.0.0.1".to_string();
                let future = command::lan::lan_connect(addr, ctx, ui);
                block_on(future);
            }
            ui.label(format!("Hello '{}', age {}", self.name, self.age));
        });
    }
}

impl WrapApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        #[allow(unused_mut)]
            let mut slf = Self {
            state: State::default(),

            sidebar_ui: SidebarUI { general: Default::default(), other: Default::default() },
            sidebar_settings: Default::default(),
            name: "zjfTest".to_string(),
            age: 31,
            demo_desktop_window:Default::default(),
            #[cfg(any(feature = "glow", feature = "wgpu"))]
            custom3d: crate::apps::Custom3d::new(cc),
        };

        #[cfg(feature = "persistence")]
        if let Some(storage) = cc.storage {
            if let Some(state) = eframe::get_value(storage, eframe::APP_KEY) {
                slf.state = state;
            }
        }

        slf
    }

    fn backend_panel(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) -> Command {
        // The backend-panel can be toggled on/off.
        // We show a little animation when the user switches it.
        let is_open =
            self.state.backend_panel.open || ctx.memory(|mem| mem.everything_is_visible());

        let mut cmd = Command::Nothing;

        egui::SidePanel::left("backend_panel")
            .resizable(false)
            .show_animated(ctx, is_open, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("💻 Backend");
                });

                ui.separator();
                self.backend_panel_contents(ui, frame, &mut cmd);
            });

        cmd
    }

    fn backend_panel_contents(
        &mut self,
        ui: &mut egui::Ui,
        frame: &mut eframe::Frame,
        cmd: &mut Command,
    ) {
        self.state.backend_panel.ui(ui, frame);

        ui.separator();

        ui.horizontal(|ui| {
            if ui
                .button("Reset egui")
                .on_hover_text("Forget scroll, positions, sizes etc")
                .clicked()
            {
                ui.ctx().memory_mut(|mem| *mem = Default::default());
                ui.close_menu();
            }

            if ui.button("Reset everything").clicked() {
                *cmd = Command::ResetEverything;
                ui.close_menu();
            }
        });
    }
}


