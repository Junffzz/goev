use std::cell::RefCell;
use crate::{controller::AppState};
use crate::window::{ChildrenWindow};
use app_kernel::{api::endpoint::{
    create_desktop_active_endpoint_client, create_file_manager_active_endpoint_client,
    id::EndPointID, EndPointStream,
}, component::lan::{LANProvider, Node}, core_error, DesktopDecodeFrame, error::CoreResult};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use eframe::CreationContext;
use tokio::sync::mpsc::Sender;
use app_kernel::api::endpoint::client::EndPointClient;
use app_kernel::api::signaling::http_message::Response;


#[tracing::instrument(skip(app_state))]
pub async fn lan_init(app_state: &AppState, force: bool) -> CoreResult<()> {
    let mut lan_provider = app_state.lan_provider.lock().await;

    if force || lan_provider.is_none() {
        *lan_provider = Some(LANProvider::new().await?);
    }

    Ok(())
}

// #[tracing::instrument()]
pub async fn lan_connect(
    addr: String,
    render_frame_tx: Sender<DesktopDecodeFrame>,
) -> CoreResult<(EndPointID,
                 Arc<EndPointClient>,
                 // Arc<tokio::sync::mpsc::Receiver<DesktopDecodeFrame>>,
)> {
    let remote_ip: IpAddr = addr
        .parse()
        .map_err(|_| core_error!("parse addr to IpAddr failed"))?;

    let remote_addr = SocketAddr::new(remote_ip, 48001); // LANProvider注册的端口

    let endpoint_id = EndPointID::LANID {
        local_ip: IpAddr::V4(Ipv4Addr::UNSPECIFIED), // 0.0.0.0
        remote_ip,// 远端ip
    };

    let client = create_desktop_active_endpoint_client(
        endpoint_id,
        render_frame_tx,
        None,
        EndPointStream::ActiveTCP(remote_addr),
        None,
    )
        .await?;

    Ok((endpoint_id,client,))
}


// #[tauri::controller]
#[tracing::instrument(skip(app_state))]
pub async fn lan_nodes_list(app_state: &AppState) -> CoreResult<Vec<Node>> {
    if let Some(ref discover) = *app_state.lan_provider.lock().await {
        Ok(discover.nodes().await)
    } else {
        Err(core_error!("lan discover is empty"))
    }
}

//#[tauri::controller]
#[tracing::instrument(skip(app_state))]
pub async fn lan_nodes_search(
    app_state: &AppState,
    keyword: String,
) -> CoreResult<Vec<Node>> {
    if let Some(ref discover) = *app_state.lan_provider.lock().await {
        let mut nodes = discover.nodes().await;
        let nodes_count = nodes.len();

        for i in 0..nodes_count {
            if !nodes[i].display_name.contains(&keyword) {
                nodes.remove(i);
            }

            for ip in nodes[i].addrs.keys() {
                if !ip.to_string().contains(&keyword) {
                    nodes.remove(i);
                    break;
                }
            }
        }

        Ok(nodes)
    } else {
        Err(core_error!("lan discover is empty"))
    }
}

//#[tauri::controller]
#[tracing::instrument(skip(app_state))]
pub async fn lan_discoverable_set(
    app_state: &AppState,
    discoverable: bool,
) -> CoreResult<()> {
    if let Some(ref discover) = *app_state.lan_provider.lock().await {
        discover.set_discoverable(discoverable);
        Ok(())
    } else {
        Err(core_error!("lan discover is empty"))
    }
}

//#[tauri::controller]
#[tracing::instrument(skip(app_state))]
pub async fn lan_discoverable_get(app_state: &AppState) -> CoreResult<bool> {
    if let Some(ref discover) = *app_state.lan_provider.lock().await {
        Ok(discover.discoverable())
    } else {
        Err(core_error!("lan discover is empty"))
    }
}