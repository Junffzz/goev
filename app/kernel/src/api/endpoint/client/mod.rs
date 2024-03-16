mod tcp;
mod udp;

use self::{tcp::serve_tcp, udp::serve_udp};
use super::{
    handlers::negotiate_desktop_params::handle_negotiate_desktop_params_request, id::EndPointID,
    message::*, EndPointStream,
};
use crate::{
    api::endpoint::handlers::{
        fs_download_file::handle_download_file_request, fs_send_file::handle_send_file_request,
        fs_visit_directory::handle_visit_directory_request, input::handle_input,
        negotiate_finished::handle_negotiate_finished_request,
    },
    call,
    component::{
        desktop::monitor::Monitor,
        fs::transfer::{append_file_block, delete_file_append_session},
    },
    core_error,
    error::{CoreError, CoreResult},
    utility::{
        bincode::{bincode_deserialize, bincode_serialize},
        nonce_value::NonceValue,
    },
};
use bytes::Bytes;
use ring::aead::{OpeningKey, SealingKey};
use scopeguard::defer;
use serde::de::DeserializeOwned;
use std::{
    fmt::Display,
    ops::Deref,
    sync::{atomic::AtomicU16, Arc},
    time::Duration,
};
use tokio::sync::{mpsc::Sender, RwLock};

const RECV_MESSAGE_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone)]
pub struct EndPointClient {
    endpoint_id: EndPointID,
    monitor: Arc<RwLock<Option<Arc<Monitor>>>>,
    tx: Sender<Vec<u8>>,
    call_id: Arc<AtomicU16>,
    call_store: Arc<moka::sync::Cache<u16, Sender<Vec<u8>>>>,
}

impl EndPointClient {
    pub fn new(endpoint_id: EndPointID)->Self{
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        let slf=Self{
            endpoint_id, // 初始化ID
            monitor: Arc::new(RwLock::new(Some(Arc::from(Monitor::default())))), // 初始时没有Monitor
            tx, // 使用空的Sender初始化
            call_id: Arc::new(AtomicU16::new(0)),
            call_store: Arc::new(moka::sync::Cache::new(100)), // 假设Cache的容量是100
        };
        slf
    }

    pub async fn new_desktop_active(
        endpoint_id: EndPointID,
        stream_key: Option<(OpeningKey<NonceValue>, SealingKey<NonceValue>)>,
        stream: EndPointStream,
        // video_frame_tx: Sender<EndPointVideoFrame>,
        // audio_frame_tx: Sender<EndPointAudioFrame>,
        visit_credentials: Option<Vec<u8>>,
    ) -> CoreResult<Arc<EndPointClient>> {
        EndPointClient::create(
            true,
            endpoint_id,
            stream_key,
            stream,
            // Some(video_frame_tx),
            // Some(audio_frame_tx),
            visit_credentials,
        )
        .await
    }

    pub async fn new_file_manager_active(
        endpoint_id: EndPointID,
        stream_key: Option<(OpeningKey<NonceValue>, SealingKey<NonceValue>)>,
        stream: EndPointStream,
        visit_credentials: Option<Vec<u8>>,
    ) -> CoreResult<Arc<EndPointClient>> {
        EndPointClient::create(
            true,
            endpoint_id,
            stream_key,
            stream,
            // None,
            // None,
            visit_credentials,
        )
        .await
    }

    pub async fn new_passive(
        endpoint_id: EndPointID,
        key_pair: Option<(OpeningKey<NonceValue>, SealingKey<NonceValue>)>,
        stream: EndPointStream,
        visit_credentials: Option<Vec<u8>>,
    ) -> CoreResult<()> {
        let _ = EndPointClient::create(
            false,
            endpoint_id,
            key_pair,
            stream,
            // None,
            // None,
            visit_credentials,
        )
        .await?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn create(
        active: bool,
        endpoint_id: EndPointID,
        key_pair: Option<(OpeningKey<NonceValue>, SealingKey<NonceValue>)>,
        stream: EndPointStream,
        // video_frame_tx: Option<Sender<EndPointVideoFrame>>,
        // audio_frame_tx: Option<Sender<EndPointAudioFrame>>,
        visit_credentials: Option<Vec<u8>>,
    ) -> CoreResult<Arc<EndPointClient>> {
        let (opening_key, sealing_key) = match key_pair {
            Some((opening_key, sealing_key)) => (Some(opening_key), Some(sealing_key)),
            None => (None, None),
        };

        let (tx, mut rx) = match stream {
            EndPointStream::ActiveTCP(addr) => {
                // 建立TCP连接
                let stream = tokio::time::timeout(
                    Duration::from_secs(10),
                    tokio::net::TcpStream::connect(addr),// addr来源于create_desktop_active_endpoint_client的remote_addr:48001
                )
                .await
                .map_err(|_| CoreError::Timeout)??;

                serve_tcp(
                    stream,
                    endpoint_id,
                    sealing_key,
                    opening_key,
                    visit_credentials,// 生成的随机visit_credentials，是否访问凭证
                )
                .await?
            }
            EndPointStream::ActiveUDP(_) => panic!("not support yet"),
            EndPointStream::PassiveTCP(stream) => {
                serve_tcp(
                    stream,
                    endpoint_id,
                    sealing_key,
                    opening_key,
                    visit_credentials,
                )
                .await?
            }
            EndPointStream::PassiveUDP { socket, .. } => {
                serve_udp(
                    socket,
                    endpoint_id,
                    sealing_key,
                    opening_key,
                    visit_credentials,
                )
                .await?
            }
        };

        // active endpoint should start negotiate with passive endpoint
        // 如果 active 为 true，并且 video_frame_tx 和 audio_frame_tx 都存在，那么将启动与被动端点的协商过程，并获取 primary_monitor。否则，primary_monitor 为 None。
        // let primary_monitor = if active && video_frame_tx.is_some() && audio_frame_tx.is_some() {
        //     // 主要目的是通过发送请求、接收响应、处理响应来进行一次协商过程，以确定桌面参数。这个过程是异步的，因此函数返回一个Future，可以在其他地方使用await关键字来等待它的完成。
        //     let params = serve_active_negotiate(&tx, &mut rx).await?;
        //     Some(Arc::new(params.primary_monitor))
        // } else {
        //     None
        // };

        let primary_monitor=None;

        let call_store = moka::sync::CacheBuilder::new(32)
            .time_to_live(Duration::from_secs(60))
            .build();

        let client = Arc::new(EndPointClient {
            endpoint_id,
            monitor: Arc::new(RwLock::new(primary_monitor)),
            tx,
            call_id: Arc::new(AtomicU16::new(0)),
            call_store: Arc::new(call_store),
        });

        handle_message(client.clone(), rx,
                       // video_frame_tx, audio_frame_tx
        );

        Ok(client)
    }
}

impl EndPointClient {
    pub async fn monitor(&self) -> Option<Arc<Monitor>> {
        (*self.monitor.read().await).clone()
    }

    pub async fn set_monitor(&self, monitor: Monitor) {
        (*self.monitor.write().await) = Some(Arc::new(monitor))
    }
}

impl EndPointClient {
    pub fn try_send(&self, message: &EndPointMessage) -> CoreResult<()> {
        let buffer = bincode_serialize(message)?;
        self.tx
            .try_send(buffer)
            .map_err(|_| CoreError::OutgoingMessageChannelDisconnect)
    }

    pub fn blocking_send(&self, message: &EndPointMessage) -> CoreResult<()> {
        let buffer = bincode_serialize(message)?;
        self.tx
            .blocking_send(buffer)
            .map_err(|_| CoreError::OutgoingMessageChannelDisconnect)
    }

    pub async fn send(&self, message: &EndPointMessage) -> CoreResult<()> {
        let buffer = bincode_serialize(message)?;
        self.tx
            .send(buffer)
            .await
            .map_err(|_| CoreError::OutgoingMessageChannelDisconnect)
    }

    pub async fn call<TReply>(&self, message: EndPointCallRequest) -> CoreResult<TReply>
    where
        TReply: DeserializeOwned,
    {
        let call_id = self
            .call_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let (tx, mut rx) = tokio::sync::mpsc::channel(1);

        self.call_store.insert(call_id, tx);
        defer! {
            self.call_store.invalidate(&call_id);
        }

        self.send(&EndPointMessage::CallRequest(call_id, message))
            .await?;

        let reply_bytes = rx.recv().await.ok_or(CoreError::Timeout)?;

        bincode_deserialize::<Result<TReply, String>>(&reply_bytes)?
            .map_err(|err_str| core_error!("{}", err_str))
    }
}

impl Display for EndPointClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EndPointClient({})", self.endpoint_id)
    }
}

/*
这段代码定义了一个异步函数serve_active_negotiate，它接收两个参数：一个发送器tx和一个接收器rx。这个函数的主要目的是进行一次协商过程，以确定桌面参数。

首先，函数创建了一个NegotiateDesktopParamsRequest消息，该消息包含一个视频编码器列表，这里只有一个H264编码器。然后，它使用bincode_serialize函数将此消息序列化为字节缓冲区，然后通过发送器tx发送出去。

接下来，函数等待接收协商响应。它使用tokio::time::timeout函数设置了一个超时，如果在RECV_MESSAGE_TIMEOUT时间内没有收到任何消息，就会返回一个CoreError::Timeout错误。如果接收到了消息，它会尝试使用bincode_deserialize函数将其反序列化为EndPointMessage。

然后，函数检查反序列化后的消息是否为NegotiateDesktopParamsResponse类型。如果不是，它会返回一个错误。如果是，它会检查响应中的参数。如果响应表示视频错误或显示错误，函数会记录错误并返回。如果响应包含参数，函数会记录这些参数并继续。

最后，函数创建并发送一个NegotiateFinishedRequest消息，表示协商过程已完成。然后，它返回协商得到的参数。

总的来说，这个函数的主要目的是通过发送请求、接收响应、处理响应来进行一次协商过程，以确定桌面参数。这个过程是异步的，因此函数返回一个Future，可以在其他地方使用await关键字来等待它的完成。
*/
async fn serve_active_negotiate(
    tx: &Sender<Vec<u8>>,
    rx: &mut tokio::sync::mpsc::Receiver<Bytes>,
) -> CoreResult<EndPointNegotiateVisitDesktopParams> {
    // 主动端发送协商桌面参数请求
    let negotiate_request_buffer = bincode_serialize(
        &EndPointMessage::NegotiateDesktopParamsRequest(EndPointNegotiateDesktopParamsRequest {
            video_codecs: vec![VideoCodec::H264],
        }),
    )?;

    // 走网络请求48000端口
    tx.send(negotiate_request_buffer)
        .await
        .map_err(|_| CoreError::OutgoingMessageChannelDisconnect)?;
    // 走网络请求从被动端收到协商响应
    let negotiate_response_buffer = tokio::time::timeout(RECV_MESSAGE_TIMEOUT, rx.recv())
        .await
        .map_err(|_| CoreError::Timeout)?
        .ok_or(CoreError::OutgoingMessageChannelDisconnect)?;

    let EndPointMessage::NegotiateDesktopParamsResponse(negotiate_response) =
        bincode_deserialize(negotiate_response_buffer.deref())? else {
            return Err(core_error!("unexpected negotiate reply"));
        };

    let params = match negotiate_response {
        EndPointNegotiateDesktopParamsResponse::VideoError(err) => {
            tracing::error!(?err, "negotiate failed with video error");
            return Err(core_error!("negotiate failed ({})", err));
        }
        EndPointNegotiateDesktopParamsResponse::MonitorError(err) => {
            tracing::error!(?err, "negotiate failed with display error");
            return Err(core_error!("negotiate failed ({})", err));
        }
        EndPointNegotiateDesktopParamsResponse::Params(params) => {
            tracing::info!(?params, "negotiate success");
            params
        }
    };

    let negotiate_request_buffer = bincode_serialize(&EndPointMessage::NegotiateFinishedRequest(
        EndPointNegotiateFinishedRequest {
            expected_frame_rate: 60,
        },
    ))?;
    // 创建并发送一个NegotiateFinishedRequest消息，表示协商过程已完成。
    tx.send(negotiate_request_buffer)
        .await
        .map_err(|_| CoreError::OutgoingMessageChannelDisconnect)?;

    Ok(params)
}

fn handle_message(
    client: Arc<EndPointClient>,
    mut rx: tokio::sync::mpsc::Receiver<Bytes>,
    // video_frame_tx: Option<Sender<EndPointVideoFrame>>,
    // audio_frame_tx: Option<Sender<EndPointAudioFrame>>,
) {
    tokio::spawn(async move {
        loop {
            // 网络中转过来的数据
            let buffer = match rx.recv().await {
                Some(buffer) => buffer,
                None => {
                    tracing::info!("message handle channel is closed");
                    break;
                }
            };

            let message = match bincode_deserialize(&buffer) {
                Ok(message) => message,
                Err(err) => {
                    tracing::error!(?err, "deserialize endpoint message failed");
                    continue;
                }
            };
            // println!("handle_message message: {:?}", message);
            match message {
                EndPointMessage::Error => {
                    // handle_error(active_device_id, passive_device_id);
                }
                // 协商桌面请求参数，这个消息是由被动端(vr端)点发送的，表示被动端点希望与主动端点协商桌面参数。被动端通过serve_active_negotiate()方法发送协商请求，然后走remote_addr:48001端口，被动端serve_active_negotiate()里rx接收到请求后，会返回协商响应。
                EndPointMessage::NegotiateDesktopParamsRequest(req) => {
                    handle_negotiate_desktop_params_request(client.clone(), req).await
                }
                EndPointMessage::NegotiateDesktopParamsResponse(_) => {
                    // this message should not received at handle_message loop because it already handled
                    // at negotiate stage from active endpoint
                }
                // 主动端在serve_active_negotiate()方法里发送的协商完成请求，被动端接收到后，会处理这个请求然后获取屏幕发送给主动端显示。
                EndPointMessage::NegotiateFinishedRequest(_) => {
                    handle_negotiate_finished_request(client.clone());
                }
                // 客户端：接收到的视频帧后通过tx.send发送到解码器
                // EndPointMessage::VideoFrame(video_frame) => {
                //     if let Some(ref tx) = video_frame_tx {
                //         if let Err(err) = tx.send(video_frame).await {
                //             tracing::error!(%err, "endpoint video frame message channel send failed");
                //             return;
                //         }
                //     } else {
                //         tracing::error!("as passive endpoint, shouldn't receive video frame");
                //     }
                // }
                // EndPointMessage::AudioFrame(audio_frame) => {
                //     if let Some(ref tx) = audio_frame_tx {
                //         if let Err(err) = tx.send(audio_frame).await {
                //             tracing::error!(%err, "endpoint audio frame message channel send failed");
                //             return;
                //         }
                //     } else {
                //         tracing::error!("as passive endpoint, shouldn't receive audio frame");
                //     }
                // }
                EndPointMessage::InputCommand(input_event) => {
                    handle_input(client.clone(), input_event).await
                }
                EndPointMessage::CallRequest(call_id, message) => {
                    let client = client.clone();
                    tokio::spawn(async move {
                        let reply = match message {
                            EndPointCallRequest::VisitDirectoryRequest(req) => {
                                call!(handle_visit_directory_request(req).await)
                            }
                            EndPointCallRequest::SendFileRequest(req) => {
                                call!(handle_send_file_request(req).await)
                            }
                            EndPointCallRequest::DownloadFileRequest(req) => {
                                call!(handle_download_file_request(client.clone(), req).await)
                            }
                        };

                        match reply {
                            Ok(reply_bytes) => {
                                if let Err(err) = client
                                    .send(&EndPointMessage::CallReply(call_id, reply_bytes))
                                    .await
                                {
                                    tracing::error!(?err, "reply Call send message failed");
                                }
                            }
                            Err(err) => {
                                tracing::error!(?err, "reply Call failed");
                            }
                        }
                    });
                }
                EndPointMessage::CallReply(call_id, reply) => {
                    tracing::info!(?call_id, "receive call reply");
                    if let Some(tx) = client.call_store.get(&call_id) {
                        let _ = tx.send(reply).await;
                    }

                    client.call_store.invalidate(&call_id)
                }
                EndPointMessage::FileTransferBlock(block) => {
                    append_file_block(client.clone(), block).await
                }
                EndPointMessage::FileTransferError(message) => {
                    delete_file_append_session(&message.id).await
                }
                _ => {}
            }
        }

        tracing::info!("message handle loop exit");
    });
}
