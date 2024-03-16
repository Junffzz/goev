use crate::error::CoreResult;
use serde::{Deserialize, Serialize};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

#[derive(Debug, Serialize, Deserialize)]
pub enum BroadcastPacket {
    TargetLive(TargetLivePacket),
    TargetDead(String),
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TargetLivePacket {
    pub hostname: String,
    pub os: String,
    pub os_version: String,
}

pub struct Discover {
    write_exit_tx: Option<tokio::sync::oneshot::Sender<()>>,
    read_exit_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl Discover {
    /*
    这段Rust代码定义了一个异步函数new，它用于在局域网中发现其他设备。函数接收五个参数：主机名、接口名、IP地址、一个表示设备是否可发现的原子布尔值，以及一个发送器，用于发送包含目标地址和广播包的元组。
首先，函数创建并绑定一个UDP套接字到给定的IP地址和48000端口，并设置该套接字以允许广播。然后，它创建两个序列化的BroadcastPacket，一个表示目标设备已死亡（TargetDead），一个表示目标设备活着（TargetLive）。

接着，函数创建了两个套接字引用，一个用于读取，一个用于写入。同时，它创建了两个单向通道，用于在读写循环结束时发送退出信号。
然后，函数启动了两个异步任务。
第一个任务用于接收广播消息。在每次循环中，它会尝试接收一个消息，并将其反序列化为BroadcastPacket。如果接收到退出信号，它将退出循环。
第二个任务用于定期发送广播消息。如果设备被设置为可发现，它会每11秒发送一个TargetLive消息。如果接收到退出信号，它会发送一个TargetDead消息并退出循环。

最后，函数返回一个包含两个退出信号发送器的新实例。

这个函数的主要用途是在局域网中发现和跟踪其他设备的状态。
*/
    pub async fn new(
        hostname: &str,
        interface_name: &str,
        ip: IpAddr,
        discoverable: Arc<AtomicBool>,
        packet_tx: tokio::sync::mpsc::Sender<(SocketAddr, BroadcastPacket)>, // 多生产者，单消费者
    ) -> CoreResult<Self> {
        let stream = tokio::net::UdpSocket::bind((ip, 48000)).await?;
        stream.set_broadcast(true)?; // 设置该套接字以允许广播

        tracing::info!(interface = interface_name, ?ip, "lan discover listen");

        // 它创建两个序列化的BroadcastPacket，一个表示目标设备已死亡（TargetDead），一个表示目标设备活着（TargetLive）。
        let dead_packet = bincode::serialize(&BroadcastPacket::TargetDead(hostname.to_string()))?;
        let live_packet =
            bincode::serialize(&BroadcastPacket::TargetLive(create_live_packet(hostname)?))?;

        // 广播包，局域网的所有设备都能收到
        let writer = Arc::new(stream);
        let reader = writer.clone();

        // 创建了两个单向通道，用于在读写循环结束时发送退出信号。
        let (write_exit_tx, mut write_exit_rx) = tokio::sync::oneshot::channel();
        let (read_exit_tx, mut read_exit_rx) = tokio::sync::oneshot::channel();

        tokio::spawn(async move {
            let mut buffer = [0u8; 512];

            loop {
                // 在每次循环中，它会尝试接收一个消息，并将其反序列化为BroadcastPacket。如果接收到退出信号，它将退出循环。
                let Err(tokio::sync::oneshot::error::TryRecvError::Empty) = read_exit_rx.try_recv() else {
                    tracing::info!("lan discover broadcast recv loop exit");
                    return;
                };
                // 从局域网其它设备接收广播包（来源stream：socket 48000）
                let (buffer_len, target_addr) = match reader.recv_from(&mut buffer).await {
                    Ok(v) => v,
                    Err(err) => {
                        tracing::error!(?err, "lan discover broadcast packet recv failed");
                        continue;
                    }
                };

                // 解析广播包
                let packet = match bincode::deserialize::<BroadcastPacket>(&buffer[..buffer_len]) {
                    Ok(v) => v,
                    Err(err) => {
                        tracing::error!(
                            ?err,
                            ?target_addr,
                            "deserialize lan discover broadcast packet failed"
                        );
                        continue;
                    }
                };
                // stream的socket链接接收到的数据包，通过packet_tx发送给其他的socket链接
                let _ = packet_tx.send((target_addr, packet)).await;
            }
        });

        tokio::spawn(async move {
            // 创建定时器，每11秒监测一次
            let mut ticker = tokio::time::interval(Duration::from_secs(11));

            loop {
                tokio::select! {
                    _ = ticker.tick() => (),
                    _ = &mut write_exit_rx => {
                        // 收到退出信号，就发一个死亡广播包，然后退出循环
                        let _ = writer.send(&dead_packet).await;
                        tracing::info!("lan discover broadcast loop exit");
                        return;
                    }
                };
                // 如果不可被发现，就跳过
                if !discoverable.load(Ordering::SeqCst) {
                    continue;
                }

                // 如果设备被设置为可发现，它会每11秒发送一个TargetLive消息。
                if let Err(err) = writer
                    .send_to(&live_packet, (Ipv4Addr::BROADCAST, 38000))
                    .await
                {
                    tracing::warn!(?err, "lan discover broadcast failed");
                }
            }
        });

        // 函数返回一个包含两个退出信号发送器的新实例。
        Ok(Self {
            write_exit_tx: Some(write_exit_tx),
            read_exit_tx: Some(read_exit_tx),
        })
    }
}

impl Drop for Discover {
    fn drop(&mut self) {
        if let Some(tx) = self.write_exit_tx.take() {
            let _ = tx.send(());
        }

        if let Some(tx) = self.read_exit_tx.take() {
            let _ = tx.send(());
        }
    }
}

fn create_live_packet(hostname: &str) -> CoreResult<TargetLivePacket> {
    let os_info = os_info::get();
    let os_version = os_info.version().to_string();
    let os = match os_info.os_type() {
        os_info::Type::Linux
        | os_info::Type::Alpine
        | os_info::Type::Arch
        | os_info::Type::Debian
        | os_info::Type::EndeavourOS
        | os_info::Type::Garuda
        | os_info::Type::Gentoo
        | os_info::Type::Manjaro
        | os_info::Type::Mariner
        | os_info::Type::Mint
        | os_info::Type::NixOS
        | os_info::Type::OracleLinux
        | os_info::Type::Pop
        | os_info::Type::Raspbian
        | os_info::Type::Solus => "Linux",

        os_info::Type::HardenedBSD
        | os_info::Type::MidnightBSD
        | os_info::Type::NetBSD
        | os_info::Type::OpenBSD
        | os_info::Type::DragonFly => "BSD",

        os_info::Type::Unknown
        | os_info::Type::Emscripten
        | os_info::Type::Redox
        | os_info::Type::Illumos => "Unknown",

        os_info::Type::Amazon => "Amazon",
        os_info::Type::FreeBSD => "FreeBSD",
        os_info::Type::Android => "Android",
        os_info::Type::CentOS => "CentOS",
        os_info::Type::Fedora => "Fedora",
        os_info::Type::Macos => "macOS",
        os_info::Type::openSUSE => "openSUSE",
        os_info::Type::Redhat => "Redhat",
        os_info::Type::RedHatEnterprise => "Redhat Enterprise",
        os_info::Type::SUSE => "SUSE",
        os_info::Type::Ubuntu => "Ubuntu",
        os_info::Type::Windows => "Windows",

        _ => "Unknown",
    }
    .to_string();

    Ok(TargetLivePacket {
        hostname: hostname.to_string(),
        os,
        os_version,
    })
}
