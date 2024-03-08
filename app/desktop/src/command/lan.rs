use crate::{command::AppState};
use crate::window::{ChildrenWindow, create_desktop_window, desktop_views_window};
use app_kernel::{
    api::endpoint::{
        create_desktop_active_endpoint_client, create_file_manager_active_endpoint_client,
        id::EndPointID, EndPointStream,
    },
    component::lan::{LANProvider, Node},
    core_error,
    error::CoreResult,
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

// #[tracing::instrument()]
pub async fn lan_connect(
    addr: String,
    ctx: &egui::Context,
    ui: &mut egui::Ui,
) -> CoreResult<()> {
    let remote_ip: IpAddr = addr
        .parse()
        .map_err(|_| core_error!("parse addr to IpAddr failed"))?;

    let window_label = format!("Desktop:{}", remote_ip.to_string().replace('.', "_"));

    let window_title = format!("MirrorX {remote_ip}");
    let remote_addr = SocketAddr::new(remote_ip, 48001);

    let endpoint_id = EndPointID::LANID {
        local_ip: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        remote_ip,
    };

    let (client, render_frame_rx) = create_desktop_active_endpoint_client(
        endpoint_id,
        None,
        EndPointStream::ActiveTCP(remote_addr),
        None,
    )
        .await?;

    let mut new_window: desktop_views_window::DesktopView = Default::default();
    let mut is_open = true;
    new_window.show(ctx, &mut is_open);
    // if let Err(err) = egui_plugin.create_window(
    //     window_label.clone(),
    //     Box::new(move |cc| {
    //         if let Some(gl_context) = cc.gl.as_ref() {
    //             Box::new(create_desktop_window(
    //                 cc,
    //                 gl_context.clone(),
    //                 endpoint_id,
    //                 client,
    //                 render_frame_rx,
    //             ))
    //         } else {
    //             panic!("get gl context failed");
    //         }
    //     }),
    //     window_label,
    //     eframe::NativeOptions {
    //         // hardware_acceleration: HardwareAcceleration::Required,
    //         ..Default::default()
    //     },
    // ) {
    //     tracing::error!(?err, "create desktop window failed");
    //     return Err(core_error!("create remote desktop window failed"));
    // }

    Ok(())
}