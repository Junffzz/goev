use std::sync::Arc;
use std::sync::mpsc::Sender as stdSender;
use crate::model::setting_bridge::{MultiType, SettingBridge, SettingData};

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct OptionsPanel {
    num_lorem_ipsums: usize,
    ctx_data:Arc<SettingData>,
    bridge_sender: stdSender<SettingBridge>,
}

impl OptionsPanel {
    pub fn new(sender:stdSender<SettingBridge>,ctx_data:Arc<SettingData>) -> Self {
        Self {
            num_lorem_ipsums: 2,
            ctx_data,
            bridge_sender:sender,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let Self {
            num_lorem_ipsums, ..
        } = self;
        let mut ctx_data = self.ctx_data.clone();
        let mut data = ctx_data.config.write();
        // 添加滑块并显示数值
        ui.label("伽马值");
        if ui.add(egui::Slider::new(&mut data.gamma, 0.0f32..=3.0f32).fixed_decimals(1).text("伽马值")).changed() {
            println!("gamma: {}", data.gamma);

            if let Err(err) = self.bridge_sender.send(SettingBridge { cfg_key: "gamma".to_string(), cfg_val: MultiType::F32(data.gamma) }) {
                println!("Failed to send message, receiver might have been dropped,err={:}",err);
            }
            // self.bridge_sender.send(SettingBridge { cfg_key: "gamma".to_string(), cfg_val: MultiType::F32(data.gamma) }).expect("self.bridge_sender.send fail.");
        }

        ui.add_space(8.0);

        ui.weak("When to show scroll bars; resize the window to see the effect.");

        ui.add_space(8.0);

        ui.separator();

        ui.add(
            egui::Slider::new(num_lorem_ipsums, 1..=100)
                .text("Content length")
                .logarithmic(true),
        );

        ui.separator();
    }

}