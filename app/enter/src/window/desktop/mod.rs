pub(crate) mod render;
pub(crate) mod state;

use self::render::Render;
use app_kernel::{
    api::endpoint::{
        client::EndPointClient,
        id::EndPointID,
        message::{EndPointInput, EndPointMessage, InputEvent, KeyboardEvent, MouseEvent},
    },
    component::input::key::MouseKey,
    DesktopDecodeFrame,
};
use eframe::{CreationContext, egui_glow::CallbackFn, glow::{self, Context}};
use egui::{
    epaint::Shadow, style::Margin, Align, Button, CentralPanel, Color32, FontId, Frame, Layout,
    Pos2, Rect, RichText, Rounding, Sense, Stroke, Ui, Vec2,
};
use egui_extras::RetainedImage;
use state::State;
use std::{
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};
use crate::SidebarSettingEnum;

static ICON_MAXIMIZE_BYTES: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 448 512"><!--! Font Awesome Pro 6.2.0 by @fontawesome - https://fontawesome.com License - https://fontawesome.com/license (Commercial License) Copyright 2022 Fonticons, Inc. --><path style="fill:rgb(255,255,255)" d="M168 32H24C10.7 32 0 42.7 0 56V200c0 9.7 5.8 18.5 14.8 22.2s19.3 1.7 26.2-5.2l40-40 79 79L81 335 41 295c-6.9-6.9-17.2-8.9-26.2-5.2S0 302.3 0 312V456c0 13.3 10.7 24 24 24H168c9.7 0 18.5-5.8 22.2-14.8s1.7-19.3-5.2-26.2l-40-40 79-79 79 79-40 40c-6.9 6.9-8.9 17.2-5.2 26.2s12.5 14.8 22.2 14.8H424c13.3 0 24-10.7 24-24V312c0-9.7-5.8-18.5-14.8-22.2s-19.3-1.7-26.2 5.2l-40 40-79-79 79-79 40 40c6.9 6.9 17.2 8.9 26.2 5.2s14.8-12.5 14.8-22.2V56c0-13.3-10.7-24-24-24H280c-9.7 0-18.5 5.8-22.2 14.8s-1.7 19.3 5.2 26.2l40 40-79 79-79-79 40-40c6.9-6.9 8.9-17.2 5.2-26.2S177.7 32 168 32z"/></svg>"#;
static ICON_SCALE_BYTES: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 512"><!--! Font Awesome Pro 6.2.0 by @fontawesome - https://fontawesome.com License - https://fontawesome.com/license (Commercial License) Copyright 2022 Fonticons, Inc. --><path style="fill:rgb(255,255,255)" d="M32 64c17.7 0 32 14.3 32 32l0 320c0 17.7-14.3 32-32 32s-32-14.3-32-32V96C0 78.3 14.3 64 32 64zm214.6 73.4c12.5 12.5 12.5 32.8 0 45.3L205.3 224l229.5 0-41.4-41.4c-12.5-12.5-12.5-32.8 0-45.3s32.8-12.5 45.3 0l96 96c12.5 12.5 12.5 32.8 0 45.3l-96 96c-12.5 12.5-32.8 12.5-45.3 0s-12.5-32.8 0-45.3L434.7 288l-229.5 0 41.4 41.4c12.5 12.5 12.5 32.8 0 45.3s-32.8 12.5-45.3 0l-96-96c-12.5-12.5-12.5-32.8 0-45.3l96-96c12.5-12.5 32.8-12.5 45.3 0zM640 96V416c0 17.7-14.3 32-32 32s-32-14.3-32-32V96c0-17.7 14.3-32 32-32s32 14.3 32 32z"/></svg>"#;

pub type AppCreator = Box<dyn FnOnce(&CreationContext<'_>) -> Box<dyn eframe::App + Send> + Send>;

pub struct DesktopWindow {
    state: State,
    icon_maximize: RetainedImage,
    icon_scale: RetainedImage,
    render: Arc<RwLock<Render>>,
    render_call_back: Arc<CallbackFn>,
    last_show_cursor: bool,
    current_show_cursor: bool,// 当前光标显示状态
}

impl super::ChildrenWindow for DesktopWindow {
    fn name(&self) -> &'static str {
        "About egui"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.name())
            .default_width(320.0)
            .default_height(480.0)
            .open(open)
            .show(ctx, |ui| {
                use super::View as _;
                self.build_panel(ui);
                // self.enter(enter);
            });
    }
}

impl DesktopWindow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        endpoint_id: EndPointID,
        // gl_context: Arc<Context>,
        screen_render: Arc<RwLock<Render>>,
        client: Arc<EndPointClient>,
        render_frame_rx: tokio::sync::mpsc::Receiver<DesktopDecodeFrame>,
    ) -> Self {
        // 这是一个线程安全的引用计数指针，指向一个互斥锁，该锁保护对DesktopDecodeFrame的访问。
        let frame_slot = Arc::new(Mutex::new(DesktopDecodeFrame::default()));

        let state = State::new(endpoint_id,
                               client,
                               render_frame_rx, frame_slot.clone());

        // let desktop_render = Arc::new(RwLock::new(
        //     Render::new(gl_context.as_ref()).expect("create screen render failed"),
        // ));

        let desktop_render_clone = screen_render.clone();
        // 它创建了一个新的回调函数cb，该函数在每次调用时尝试获取desktop_render_clone的写锁和frame_slot的锁，然后尝试绘制帧。如果绘制失败，它将记录一个错误。
        let cb = CallbackFn::new(move |_info, painter| {
            let mut render = desktop_render_clone.write().unwrap();
            let frame = frame_slot.lock().unwrap();

            if let Err(err) = render.paint(painter.gl(), &frame, painter.intermediate_fbo()) {
                tracing::error!(?err, "screen render failed");
            }
        });

        Self {
            state,
            icon_maximize: RetainedImage::from_color_image(
                "fa_maximize",
                egui_extras::image::load_svg_bytes(ICON_MAXIMIZE_BYTES).unwrap(),
            ),
            icon_scale: RetainedImage::from_color_image(
                "fa_arrows-left-right-to-line",
                egui_extras::image::load_svg_bytes(ICON_SCALE_BYTES).unwrap(),
            ),
            render: screen_render,
            render_call_back: Arc::new(cb),
            last_show_cursor: true,
            current_show_cursor: true,
        }
    }

    pub fn build_panel(&mut self, ui: &mut Ui) {
        // match self.state.visit_state() {
        //     state::VisitState::Connecting => {
        //         enter.centered_and_justified(|enter| {
        //             let (rect, response) = enter.allocate_exact_size(
        //                 Vec2::new(160.0, 80.0),
        //                 Sense::focusable_noninteractive(),
        //             );

        //             enter.allocate_ui_at_rect(rect, |enter| {
        //                 enter.spinner();
        //                 enter.label("connecting");
        //             });

        //             response
        //         });
        //     }
        //     state::VisitState::Negotiating => {
        //         enter.centered_and_justified(|enter| {
        //             let (rect, response) = enter.allocate_exact_size(
        //                 Vec2::new(160.0, 80.0),
        //                 Sense::focusable_noninteractive(),
        //             );

        //             enter.allocate_ui_at_rect(rect, |enter| {
        //                 enter.spinner();
        //                 enter.label("negotiating");
        //             });

        //             response
        //         });
        //     }
        //     state::VisitState::Serving => {
        self.build_desktop_texture(ui);
        self.build_toolbar(ui);
        //     }
        //     state::VisitState::ErrorOccurred => {
        //         enter.centered_and_justified(|enter| {
        //             enter.label(
        //                 self.state
        //                     .last_error()
        //                     .map(|err| err.to_string())
        //                     .unwrap_or_else(|| String::from("An unknown error occurred")),
        //             );
        //         });
        //     }
        // }
    }

    // 这个函数的主要目的是根据当前的桌面帧大小和 UI 可用空间来构建和渲染桌面纹理。
    fn build_desktop_texture(&mut self, ui: &mut Ui) {
        // 获取帧宽度、高度
        let (frame_width, frame_height) = self.state.update_desktop_frame();

        if frame_width > 0 && frame_height > 0 {
            // 函数检查 UI 的可用宽度和高度是否小于桌面帧的宽度和高度。如果是，那么它将禁用桌面帧的缩放功能。
            // when client area bigger than original screen frame, disable scale button
            self.state.set_desktop_frame_scalable(
                ui.available_width() < frame_width as _
                    || ui.available_height() < frame_height as _,
            );
            // 如果桌面帧已经缩放
            if self.state.desktop_frame_scaled()
                && (ui.available_width() < frame_width as _
                    || ui.available_height() < frame_height as _)
            {
                // 函数将计算出新的左边和顶部位置，然后在这个位置上分配一个新的 UI 矩形。
                let left = ((ui.available_width() - frame_width as f32) / 2.0).max(0.0);
                let top = ((ui.available_height() - frame_height as f32) / 2.0).max(0.0);

                // 设置新的UI矩阵
                let mut available_rect = ui.available_rect_before_wrap();
                available_rect.min = Pos2::new(left, top);
                // 分配新的UI矩阵
                ui.allocate_ui_at_rect(available_rect, |ui| {
                    // 创建一个滚动区域，它可以在两个方向上滚动，但不会自动缩小
                    egui::ScrollArea::both()
                        .auto_shrink([false; 2])
                        .show_viewport(ui, |ui, view_port| {
                            ui.set_width(frame_width as f32);
                            ui.set_height(frame_height as f32);

                            // 绘制回调
                            let callback = egui::PaintCallback {
                                rect: ui.available_rect_before_wrap(),
                                callback: self.render_call_back.clone(),
                            };

                            ui.painter().add(callback);

                            // 函数获取 UI 的输入事件，并根据这些事件和当前的视口位置来更新光标的显示状态和位置。
                            let input = ui.ctx().input(|i| i.pixels_per_point());
                            // let events = input.events.as_slice();
                            // let left_top = view_port.left_top();
                            //
                            // self.current_show_cursor = !input.pointer.has_pointer();
                            //
                            // self.emit_input(events, move |pos| Some(pos + left_top.to_vec2()));
                        });
                });
            } else {// 桌面帧没有被缩放，将计算出一个新的桌面帧大小，然后在 UI 中心分配一个新的 UI 矩形。
                let available_width = ui.available_width();
                let available_height = ui.available_height();
                let aspect_ratio = (frame_width as f32) / (frame_height as f32);

                let desktop_size = if (available_width / aspect_ratio) < available_height {
                    (available_width, available_width / aspect_ratio)
                } else {
                    (available_height * aspect_ratio, available_height)
                };

                let scale_ratio = desktop_size.0 / (frame_width as f32);

                // 计算出一个新的缩放比例，以及在图像周围的空间。
                let space_around_image = Vec2::new(
                    (available_width - desktop_size.0) / 2.0,
                    (available_height - desktop_size.1) / 2.0,
                );

                let callback = egui::PaintCallback {
                    rect: Rect {
                        min: space_around_image.to_pos2(),
                        max: space_around_image.to_pos2() + desktop_size.into(),
                    },
                    callback: self.render_call_back.clone(),
                };

                ui.painter().add(callback);

                // 函数获取 UI 的输入事件，并根据这些事件和计算出的空间来更新光标的显示状态和位置。
                let input = ui.ctx().input(|i| i.pixels_per_point());
                // let events = input.events.as_slice();
                // if let Some(pos) = input.pointer.hover_pos() {
                //     if (space_around_image.x <= pos.x
                //         && pos.x <= space_around_image.x + desktop_size.0)
                //         && (space_around_image.y <= pos.y
                //         && pos.y <= space_around_image.y + desktop_size.1)
                //     {
                //         self.current_show_cursor = false;
                //     }
                // }

                // self.emit_input(events, move |pos| {
                //     if (space_around_image.x <= pos.x
                //         && pos.x <= space_around_image.x + desktop_size.0)
                //         && (space_around_image.y <= pos.y
                //         && pos.y <= space_around_image.y + desktop_size.1)
                //     {
                //         Some(Pos2::new(
                //             (pos.x - space_around_image.x).max(0.0) / scale_ratio,
                //             (pos.y - space_around_image.y).max(0.0) / scale_ratio,
                //         ))
                //     } else {
                //         None
                //     }
                // });
            }
        } else {
            // 如果桌面帧的宽度和高度都为0，函数将在 UI 的中心位置显示一个 "preparing" 的标签和一个旋转的加载指示器。
            ui.centered_and_justified(|ui| {
                let (rect, _) = ui
                    .allocate_exact_size(Vec2::new(160.0, 80.0), Sense::focusable_noninteractive());

                ui.allocate_ui_at_rect(rect, |ui| {
                    ui.spinner();
                    ui.label("preparing");
                });
            });
        }
    }

    // 用于在用户界面（UI）上构建一个工具栏。
    fn build_toolbar(&mut self, ui: &mut Ui) {
        // put the toolbar at central top
        // Sense::click()表示这个矩形可以响应点击事件
        let (mut rect, _) = ui.allocate_at_least(Vec2::new(220.0, 35.0), Sense::click());
        rect.set_center(Pos2::new(ui.max_rect().width() / 2.0, 50.0));

        ui.allocate_ui_at_rect(rect, |ui| {
            Frame::default()
                .inner_margin(Margin::symmetric(6.0, 2.0))// 设置边距
                .rounding(Rounding::same(12.0))// 设置圆角
                .fill(ui.style().visuals.window_fill())// 设置填充颜色
                .shadow(Shadow::small_light())// 设置阴影
                .stroke(Stroke::new(1.0, Color32::GRAY))// 设置边框
                .show(ui, |ui| {
                    ui.set_min_size(rect.size());// 设置最小尺寸为rect的尺寸
                    ui.style_mut().spacing.item_spacing = Vec2::new(6.0, 2.0);//设置间距
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        // remote device id
                        // 标签为设备ID
                        ui.label(
                            RichText::new(self.state.format_remote_device_id())
                                .font(FontId::monospace(22.0)),
                        );

                        ui.separator();
                        // 构建工具栏按钮
                        self.build_toolbar_button_scale(ui);

                        ui.separator();

                        // FPS
                        // 该标签显示当前帧率
                        // enter.label(
                        //     RichText::new(self.render.read().unwrap().frame_rate().to_string())
                        //         .font(FontId::monospace(24.0)), // FontFamily::Name("LiquidCrystal".into()))),
                        // );
                    })
                })
        });
    }

    fn build_toolbar_button_scale(&mut self, ui: &mut Ui) {
        // when use_original_resolution is true, the button should display 'fit size' icon
        ui.add_enabled_ui(self.state.desktop_frame_scalable(), |ui| {
            // enter.visuals_mut().widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
            // let button = if self.state.desktop_frame_scaled() {
            //     egui::ImageButton::new(
            //         self.icon_scale.texture_id(enter.ctx()),
            //     )
            // } else {
            //     egui::ImageButton::new(
            //         self.icon_maximize.texture_id(enter.ctx()),
            //     )
            // }
            //     .tint(enter.visuals().noninteractive().fg_stroke.color);

            let button = Button::new("test_descktop_frame_scaled");
            if ui.add(button).clicked() {
                self.state
                    .set_desktop_frame_scaled(!self.state.desktop_frame_scaled());
            }
        });
    }
}

impl DesktopWindow {
    // 用于处理输入事件
    fn emit_input(&mut self, events: &[egui::Event], pos_calc_fn: impl Fn(Pos2) -> Option<Pos2>) {
        let mut input_commands = Vec::new();// 用于存储输入命令
        for event in events.iter() {
            match event {
                // 如果是鼠标移动事件
                egui::Event::PointerMoved(pos) => {
                    if let Some(mouse_pos) = pos_calc_fn(*pos) {
                        // if mouse_pos != self.last_mouse_pos {
                        input_commands.push(InputEvent::Mouse(MouseEvent::Move(
                            MouseKey::None,
                            mouse_pos.x,
                            mouse_pos.y,
                        )));
                    }
                }
                //  如果是鼠标按键事件
                egui::Event::PointerButton {
                    pos,
                    button,
                    pressed,
                    ..
                } => {
                    let Some(mouse_pos) = pos_calc_fn(*pos) else {
                        continue;
                    };

                    let mouse_key = match button {
                        egui::PointerButton::Primary => MouseKey::Left,
                        egui::PointerButton::Secondary => MouseKey::Right,
                        egui::PointerButton::Middle => MouseKey::Wheel,
                        egui::PointerButton::Extra1 => MouseKey::SideBack,
                        egui::PointerButton::Extra2 => MouseKey::SideForward,
                    };

                    let mouse_event = if *pressed {
                        MouseEvent::Down(mouse_key, mouse_pos.x, mouse_pos.y)
                    } else {
                        MouseEvent::Up(mouse_key, mouse_pos.x, mouse_pos.y)
                    };

                    input_commands.push(InputEvent::Mouse(mouse_event));
                }
                // 如果是滚轮事件
                egui::Event::Scroll(scroll_vector) => {
                    input_commands
                        .push(InputEvent::Mouse(MouseEvent::ScrollWheel(scroll_vector.y)));
                }
                // 如果是键盘事件
                egui::Event::Key { key, pressed, .. } => {
                    tracing::info!(?key, "raw key");

                    // let keyboard_event = if *pressed {
                    //     KeyboardEvent::KeyDown(*key)
                    // } else {
                    //     KeyboardEvent::KeyUp(*key)
                    // };
                    //
                    // input_commands.push(InputEvent::Keyboard(keyboard_event))
                }
                _ => {}
            }
        }

        if input_commands.is_empty() {
            return;
        }

        if let Err(err) = self
            .state
            .endpoint_client().try_send(&EndPointMessage::InputCommand(EndPointInput {
                events: input_commands,
            }))
        {
            tracing::error!(?err, "send input event failed");
        }
    }
}

impl eframe::App for DesktopWindow {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        let update_instant = std::time::Instant::now();//获取当前时间

        self.current_show_cursor = true;
        // CentralPanel::default()
        //     .frame(egui::Frame::none())
        //     .show(ctx, |enter| {
        //         self.build_panel(enter);
        //     });

        if self.current_show_cursor != self.last_show_cursor {
            app_kernel::api::system::set_show_cursor(self.current_show_cursor);
            self.last_show_cursor = self.current_show_cursor;// 设置是否显示光标
        }

        let cost = update_instant.elapsed();// 计算时间差

        // 函数计算从获取update_instant到现在所经过的时间，并保存到cost变量中。然后，函数尝试从cost中减去16毫秒。如果结果是Some，函数请求在等待这个时间后重绘UI。否则，函数立即请求重绘UI。
        if let Some(wait) = cost.checked_sub(Duration::from_millis(16)) {
            ctx.request_repaint_after(wait);
        } else {
            ctx.request_repaint();
        }
    }

    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        if let Some(gl) = gl {
            self.render.write().unwrap().destroy(gl);
        }
    }
}
