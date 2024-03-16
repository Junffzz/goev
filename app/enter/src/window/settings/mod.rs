mod backend_panel;
pub mod sidebar;
// pub mod settings_window;
pub mod account_panel;
pub mod options_panel;
pub mod about_panel;

pub struct CentralUIPanelGroup {
    pub account: account_panel::AccountPanel,
    pub options:options_panel::OptionsPanel,
    pub about:about_panel::AboutPanel,
}