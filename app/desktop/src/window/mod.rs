pub mod desktop;
pub mod desktop_views_window;

use self::desktop::DesktopWindow;
use app_kernel::{
    api::endpoint::{client::EndPointClient, id::EndPointID},
    DesktopDecodeFrame,
};
use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Arc};
use eframe::CreationContext;
use egui::{FontData, FontDefinitions, FontFamily};

static PROPORTIONAL_FONTS: Lazy<HashMap<&str, &[u8]>> = Lazy::new(|| {
    [
        (
            "NotoSans",
            include_bytes!("../../assets/fonts/NotoSans-Regular.ttf").as_ref(),
        ),
        (
            "NotoSansJP",
            include_bytes!("../../assets/fonts/NotoSansJP-Regular.otf").as_ref(),
        ),
        (
            "NotoSansKR",
            include_bytes!("../../assets/fonts/NotoSansKR-Regular.otf").as_ref(),
        ),
        (
            "NotoSansSC",
            include_bytes!("../../assets/fonts/NotoSansSC-Regular.otf").as_ref(),
        ),
        (
            "NotoSansTC",
            include_bytes!("../../assets/fonts/NotoSansTC-Regular.otf").as_ref(),
        ),
    ]
        .into_iter()
        .collect()
});

static MONOSPACE_FONTS: Lazy<HashMap<&str, &[u8]>> = Lazy::new(|| {
    [(
        "NotoSansMono",
        include_bytes!("../../assets/fonts/NotoSansMono-Regular.ttf").as_ref(),
    )]
        .into_iter()
        .collect()
});

#[allow(clippy::too_many_arguments)]
pub fn create_desktop_window(
    cc: &CreationContext,
    gl_context: Arc<eframe::glow::Context>,
    endpoint_id: EndPointID,
    client: Arc<EndPointClient>,
    render_frame_rx: tokio::sync::mpsc::Receiver<DesktopDecodeFrame>,
) -> DesktopWindow {
    set_fonts(&cc.egui_ctx);

    // cc.egui_ctx.set_debug_on_hover(true);

    crate::window::desktop::DesktopWindow::new(endpoint_id, gl_context, client, render_frame_rx)
}

fn set_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    add_normal_font_families(&mut fonts, &PROPORTIONAL_FONTS, FontFamily::Proportional);
    add_normal_font_families(&mut fonts, &MONOSPACE_FONTS, FontFamily::Monospace);
    // add_custom_family_font_families(&mut fonts);

    ctx.set_fonts(fonts);
}

fn add_normal_font_families<'a>(
    fonts: &mut FontDefinitions,
    font_data: &'a HashMap<&'a str, &'a [u8]>,
    family: FontFamily,
) {
    let mut new_fonts = Vec::new();
    for (name, font_data) in font_data.iter() {
        new_fonts.push(name.to_string());
        fonts
            .font_data
            .insert(name.to_string(), FontData::from_owned(font_data.to_vec()));
    }

    let old_fonts = fonts.families.entry(family.clone()).or_default();

    new_fonts.append(old_fonts);

    fonts.families.insert(family, new_fonts);
}

// fn add_custom_family_font_families(fonts: &mut FontDefinitions) {
//     fonts.font_data.insert(
//         "LiquidCrystal".into(),
//         FontData::from_static(
//             include_bytes!("../../assets/fonts/LiquidCrystal-Light.otf").as_ref(),
//         )
//         .tweak(FontTweak {
//             scale: 1.0,
//             y_offset_factor: 0.0,
//             y_offset: 0.5,
//         }),
//     );

//     fonts.families.insert(
//         FontFamily::Name("LiquidCrystal".into()),
//         vec!["LiquidCrystal".into()],
//     );
// }

pub trait View {
    fn ui(&mut self, ui: &mut egui::Ui);
}

/// Something to view
pub trait ChildrenWindow {
    /// Is the demo enabled for this integration?
    fn is_enabled(&self, _ctx: &egui::Context) -> bool {
        true
    }

    /// `&'static` so we can also use it as a key to store open/close state.
    fn name(&self) -> &'static str;

    /// Show windows, etc
    fn show(&mut self, ctx: &egui::Context, open: &mut bool);
}