mod controller;
mod window;
mod utility;
mod service;
mod model;

use futures::FutureExt;

use std::ops::{Deref, DerefMut};
use std::sync::{Arc, mpsc, RwLock};
use eframe::egui;
use crate::window::{ChildrenWindow};
use futures::{StreamExt, TryFutureExt};
use std::thread::JoinHandle;
use crate::model::setting_bridge::{SettingBridge, SettingData};
use crate::service::GService;

use crate::window::{
    get_ui_fonts,
    settings::{CentralUIPanelGroup,sidebar::SidebarPanel}
};

struct WrapApp {
    sidebar_panel: SidebarPanel,
    central_panel_group: CentralUIPanelGroup,
    data: Arc<SettingData>,
    bridge_sender: mpsc::Sender<SettingBridge>,

    threads: Vec<(JoinHandle<()>, mpsc::SyncSender<egui::Context>)>,
    on_done_tx: mpsc::SyncSender<()>,
    on_done_rc: mpsc::Receiver<()>,
}


fn main() -> Result<(), eframe::Error> {
    GService();// 保证服务初始化lazy_static能先触发

    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().
            with_inner_size([820.0, 520.0]).
            with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| Box::new(<WrapApp>::new(cc))),
    )
}

impl eframe::App for WrapApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // 侧边栏
        egui::SidePanel::left("left_panel")
            // .resizable(true)
            .default_width(200.0)
            //.width_range(80.0..=200.0)
            .show(ctx, |ui| {
                self.sidebar_panel.ui(ui);
            });
        // 主要内容展示区
        egui::CentralPanel::default().show(ctx, |ui| {
            // enter.label("Hello from deferred viewport");
            // enter.label(format!("frame1: {}", 1));
            //
            // // 使用布局来设置CentralPanel中内容的布局和尺寸
            // enter.with_layout(egui::Layout::top_down(egui::Align::Center), |enter| {
            //     // 添加你的UI元素，例如按钮、文本等
            //
            //     enter.label("Hello from deferred viewport");
            //     enter.label(format!("frame1: {}", 1));
            // });

            match self.sidebar_panel.selected_panel {
                crate::window::settings::sidebar::ContentPanelEnum::AccountPanel => {
                    self.central_panel_group.account.ui(ctx,ui);
                }
                crate::window::settings::sidebar::ContentPanelEnum::OptionsPanel => {
                    self.central_panel_group.options.ui(ui);
                }
                crate::window::settings::sidebar::ContentPanelEnum::AboutPanel => {
                    self.central_panel_group.about.ui(ui);
                }
                _ => {self.central_panel_group.account.ui(ctx,ui);}
            }
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Hello World!");
                ui.label("Hello World!");
            });
        });

    }
}

impl std::ops::Drop for WrapApp {
    fn drop(&mut self) {
        for (handle, show_tx) in self.threads.drain(..) {
            std::mem::drop(show_tx);
            handle.join().unwrap();
        }
    }
}

impl WrapApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // let gl = cc.gl.as_ref()?;
        cc.egui_ctx.set_fonts(get_ui_fonts());

        let threads = Vec::with_capacity(3);
        let (on_done_tx, on_done_rc) = mpsc::sync_channel(0);

        let (bridge_sender,setting_data)=GService().init_bridge_setting_ui();
        #[allow(unused_mut)]
            let mut slf = Self {
            sidebar_panel: Default::default(),
            central_panel_group: CentralUIPanelGroup {
                account: Default::default(),
                options: crate::window::settings::options_panel::OptionsPanel::new(bridge_sender.clone(), setting_data.clone()),
                about: Default::default()
            },
            data: setting_data.clone(),
            bridge_sender,
            threads,
            on_done_tx,
            on_done_rc,
        };

        slf
    }
}



