use egui::DragValue;

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
#[derive(PartialEq)]
pub struct AccountPanel {
    num_lorem_ipsums: usize,
    show_add_account_window: bool,
}

impl Default for AccountPanel {
    fn default() -> Self {
        Self {
            num_lorem_ipsums: 2,
            show_add_account_window: false,
        }
    }
}

struct Device {
    hostname: String,
    ip: String,
    port: u16,
}

impl AccountPanel {
    pub fn new() -> Self {
        Self {
            num_lorem_ipsums: 2,
            show_add_account_window: false,
        }
    }

    pub fn ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let Self {
            num_lorem_ipsums,
            show_add_account_window,
        } = self;

        ui.separator();

        let mut devices = vec![
            Device {
                hostname: "hostname".to_string(),
                ip: "127.0.0.1".to_string(),
                port: 48001,
            }];

        ui.label("受信任设备列表");
        egui::ScrollArea::vertical().show(ui, |ui| {
            for device in devices.iter_mut() {
                ui.horizontal(|ui| {
                    ui.label(format!("名称: {}", device.hostname));
                    ui.label(format!("地址: {}", device.ip));
                    if ui.button("编辑").clicked() {
                        // 编辑设备逻辑
                    }
                    if ui.button("移除").clicked() {
                        // 移除设备逻辑
                    }
                });
                ui.separator();
            }
        });
        // todo: 手动添加UI改为在列表下方增加表单（可以展开和收缩），不做弹窗形式
        // if ui.button("手动添加").clicked() {
        //     *show_add_account_window = true;
        // }
        //
        // if *show_add_account_window {
        //     show_add_device_form(ctx, ui, &mut Device {
        //         hostname: "unknown".to_string(),
        //         ip: "127.0.0.1".to_string(),
        //         port: 48001,
        //     }, show_add_account_window);
        // }
    }
}

// 废弃
fn show_add_device_form(ctx: &egui::Context, ui: &mut egui::Ui, new_device: &mut Device, to_add: &mut bool) {
    let parent_size = ctx.available_rect(); // 获取父窗口的大小
    let window_size = [260.0, 220.0]; // 子窗口的大小
    let pos = egui::pos2(
        parent_size.center().x - window_size[0] / 2.0,
        parent_size.center().y - window_size[1] / 2.0,
    ); // 计算子窗口的位置
    let id = egui::Id::new("add_device_window");
    let should_close = egui::Window::new("添加设备")
        .id(id)
        .open(to_add)
        .default_pos(pos)
        .fixed_size(window_size)
        .show(ctx, |ui| {
            ui.label("添加设备");
            ui.horizontal(|ui| {
                ui.label("名称");
                ui.text_edit_singleline(&mut new_device.hostname);
            });
            ui.horizontal(|ui| {
                ui.label("IP");
                ui.text_edit_singleline(&mut new_device.ip);
                ui.label(":");
                ui.add(DragValue::new(&mut new_device.port).clamp_range(1..=65535));
            });
            let mut close_clicked = false;
            ui.horizontal(|ui| {
                if ui.button("取消").clicked() {
                    close_clicked = true;
                }
                if ui.button("保存").clicked() {
                    // 保存新设备逻辑
                    close_clicked = true;
                }
            });
            close_clicked
        });

    if should_close.map_or(false, |response| response.inner.unwrap_or(false)) {
        *to_add = false;
    }
}
