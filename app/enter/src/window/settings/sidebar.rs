

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ContentPanelEnum {
    AccountPanel,
    OptionsPanel,
    AboutPanel,
    Other,
}

impl Default for ContentPanelEnum {
    fn default() -> Self {
        Self::AccountPanel
    }
}

pub struct SidebarPanel {
    pub selected_panel: ContentPanelEnum,
}

impl Default for SidebarPanel {
    fn default() -> Self {
        Self {
            selected_panel: Default::default(),
        }
    }
}

impl SidebarPanel {
    pub fn update(&mut self, ctx: &egui::Context) {

    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("Left Panel");
        });
        ui.selectable_value(&mut self.selected_panel, ContentPanelEnum::AccountPanel, "Account");
        ui.selectable_value(&mut self.selected_panel, ContentPanelEnum::OptionsPanel, "OptionsPanel");
        ui.selectable_value(&mut self.selected_panel, ContentPanelEnum::AboutPanel, "AboutPanel");

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.label("Hello from deferred viewport");
        });
    }
}