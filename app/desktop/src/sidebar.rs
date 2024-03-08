

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
#[derive(PartialEq)]
pub struct SidebarSettingGeneral {
    num_lorem_ipsums: usize,
}

impl Default for SidebarSettingGeneral {
    fn default() -> Self {
        Self {
            num_lorem_ipsums: 2,
        }
    }
}

impl SidebarSettingGeneral {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let Self {
            num_lorem_ipsums,
        } = self;


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



#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
#[derive(PartialEq)]
pub struct SidebarSettingOther {
    num_lorem_ipsums: usize,
}

impl Default for SidebarSettingOther {
    fn default() -> Self {
        Self {
            num_lorem_ipsums: 2,
        }
    }
}

impl SidebarSettingOther {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let Self {
            num_lorem_ipsums,
        } = self;


        ui.add_space(8.0);

        ui.weak("other effect.");

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