use super::AppState;
use crate::window::create_desktop_window;
use app_kernel::{
    api::{
        endpoint::{
            create_desktop_active_endpoint_client, create_file_manager_active_endpoint_client,
            id::EndPointID, EndPointStream,
        },
        signaling::{http_message::Response, SignalingClient},
    },
    core_error,
    error::CoreResult,
};
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs};
use http::Uri;
// use tauri_egui::EguiPluginHandle;

/*
这段代码定义了一个名为 signaling_connect 的异步函数，它接收两个参数：一个 app_state（应用状态）和一个布尔值 force。这个函数的主要目的是建立一个信号连接。
首先，它获取了当前的信号客户端 current_signaling 和存储 storage。如果存储未初始化，它将返回一个错误。
然后，它获取了存储中的主域 primary_domain。如果当前的信号客户端已经连接到这个主域，并且 force 参数为 false，那么函数将直接返回，不进行任何操作。
接下来，它尝试解析主域的地址 addr。如果地址可以被解析为 IPv4 或 IPv6 地址，它将创建一个包含这个地址和订阅端口的 SocketAddr 向量。如果地址是一个 URL，它将尝试获取主机名，并在一个新的任务中解析这个主机名和订阅端口到 SocketAddr。如果解析成功，它将返回解析出的地址向量，否则返回错误。如果地址既不是 IP 地址也不是 URL，它将返回错误。
然后，它创建了一个新的信号客户端 client，并使用之前解析出的地址、设备 ID、指纹和存储来订阅主域。
最后，它将当前的信号客户端设置为新创建的客户端，并返回成功。
*/
// #[tauri::controller]
#[tracing::instrument(skip(app_state))]
pub async fn signaling_connect(
    app_state: &AppState,
    force: bool,
) -> CoreResult<()> {
    let mut current_signaling = app_state.signaling_client.lock().await;
    // app_state.storage来源config的配置存储sqlite
    let Some(ref storage) = *app_state.storage.lock().await else {
        return Err(core_error!("storage not initialize"));
    };

    let primary_domain = storage.domain().get_primary_domain()?;

    if let Some((current_domain_id, _)) = *current_signaling {
        // 如果当前的信号客户端已经连接到这个主域，并且 force 参数为 false，那么函数将直接返回，不进行任何操作。
        if current_domain_id == primary_domain.id && !force {
            return Ok(());
        }
    }

    // addrs存储的是订阅端口28001的socket地址
    let addrs: Vec<SocketAddr> = if let Ok(ipv4_addr) = primary_domain.addr.parse::<Ipv4Addr>() {
        vec![(ipv4_addr, primary_domain.subscribe_port).into()]
    } else if let Ok(ipv6_addr) = primary_domain.addr.parse::<Ipv6Addr>() {
        vec![(ipv6_addr, primary_domain.subscribe_port).into()]
    } else if let Ok(url_addr) = primary_domain.addr.parse::<Uri>() {
        if let Some(host) = url_addr.host() {
            let host = host.to_string();
            let (tx, rx) = tokio::sync::oneshot::channel();
            tokio::task::spawn_blocking(move || {
                match (host, primary_domain.subscribe_port).to_socket_addrs() {
                    Ok(addrs) => {
                        let addrs: Vec<SocketAddr> = addrs.collect();
                        let _ = tx.send(Some(addrs));
                    }
                    Err(_) => {
                        let _ = tx.send(None);
                    }
                };
            });

            match rx.await {
                Ok(addrs) => match addrs {
                    Some(addrs) => addrs,
                    None => {
                        return Err(core_error!("resolve empty socket addr"));
                    }
                },
                Err(_) => {
                    return Err(core_error!(
                        "receive addr resolve result failed, this shouldn't happen"
                    ));
                }
            }
        } else {
            return Err(core_error!("invalid domain addr"));
        }
    } else {
        return Err(core_error!("invalid domain addr"));
    };

    let mut client = SignalingClient::new(primary_domain.addr)?;

    client
        .subscribe(
            addrs,
            primary_domain.device_id,
            &primary_domain.finger_print,
            storage.clone(),
        )
        .await?;

    *current_signaling = Some((primary_domain.id, client));

    Ok(())
}

// #[tauri::controller]
#[tracing::instrument(skip( app_state, password))]
pub async fn signaling_visit(
    // app_handle: tauri::AppHandle,
    app_state: &AppState,
    // egui_plugin: tauri::State<'_, EguiPluginHandle>,
    remote_device_id: String,
    password: String,
) -> CoreResult<()> {
    let window_label = format!("Desktop:{remote_device_id}");

    let window_title = format!("MirrorX {remote_device_id}");

    let Some(ref storage) = *app_state.storage.lock().await else {
        return Err(core_error!("storage not initialize"));
    };

    let Some((_,ref signaling_client)) = *app_state.signaling_client.lock().await else {
        return Err(core_error!("storage not initialize"));
    };

    let remote_device_id_num = remote_device_id.replace('-', "").parse()?;
    let primary_domain = storage.domain().get_primary_domain()?;
    let local_device_id = primary_domain.device_id;
    let resp = signaling_client
        .visit(
            primary_domain.device_id,
            remote_device_id_num,
            password,
            true,
        )
        .await?;

    let (endpoint_addr, visit_credentials, opening_key, sealing_key) = match resp {
        Response::Message(result) => match result {
            Ok(v) => v,
            Err(reason) => return Err(core_error!("Visit Failed ({:?})", reason)),
        },
        Response::Error(err) => return Err(core_error!("Visit Failed ({:?})", err)),
    };

    let endpoint_addr: SocketAddr = endpoint_addr
        .parse()
        .map_err(|_| core_error!("parse endpoint addr failed"))?;

    tracing::info!(?local_device_id, ?remote_device_id, "key exchange success");

    let endpoint_id = EndPointID::DeviceID {
        local_device_id,
        remote_device_id: remote_device_id_num,
    };


    let (client, render_frame_rx) = create_desktop_active_endpoint_client(
        endpoint_id,
        Some((opening_key, sealing_key)),
        EndPointStream::ActiveTCP(endpoint_addr),
        Some(visit_credentials),
    )
        .await?;

    // if let Err(err) = egui_plugin.create_window(
    //     window_label,
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
    //     window_title,
    //     tauri_egui::eframe::NativeOptions {
    //         // hardware_acceleration: HardwareAcceleration::Required,
    //         ..Default::default()
    //     },
    // ) {
    //     tracing::error!(?err, "create screen window failed");
    //     return Err(core_error!("create remote screen window failed"));
    // }


    let _ = storage
        .history()
        .create(remote_device_id_num, &primary_domain.name);

    Ok(())
}
