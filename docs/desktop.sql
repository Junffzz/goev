--账户是连接远程设备的概念，一个远程连接代表一个账号，设置项和账号强关联的
CREATE TABLE accounts(
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL UNIQUE,
    remote_ip TEXT NOT NULL,
    remote_port INTEGER NOT NULL,
    is_default BOOLEAN DEFAULT 0,--是否默认账号
    device_id INTEGER NOT NULL,
    remote_os TEXT NOT NULL, --远端桌面的操作系统window10、macos
    auto_connect BOOLEAN DEFAULT 0,--是否自动连接
    remarks TEXT
);

-- 配置项(桌面配置、VR的配置可以通过远端同步过来)，双端保持同步
create table configs
(
    id                       INTEGER                  not null
        primary key,
    account_id               INTEGER                  not null UNIQUE,
    screen_brightness        REAL    default 1.0      not null,--屏幕亮度,最大100%
    screen_quality           TEXT    default "medium" not null,--桌面环境清晰度：高质量会占用battery使用。low,medium,high
    screen_frame_rate        INTEGER default 72       not null,-- 刷新率，高帧率减少filcker但是增加能耗，72fps 90fps
    desktop_bitrate          INTEGER default 100      not null,--桌面码率，提高串流桌面质量，但会增加能耗,100m
    microphone_passthrough   BOOLEAN default 1        not null,-- 麦克风
    microphone_volume        REAL    default 0.8      not null,-- 麦克风音量。单位%
    gamma                    REAL    default 1.0      not null,--伽马值，调节暗部亮度。默认：1.00
    vr_graphics_quality      TEXT    default "medium" not null,--VR画面质量。low(gtx 1070/rx 5500-xt),medium,high,ultra,godlike
    vr_frame_rate            INTEGER default 72       not null,-- VR画面帧率，72fps,90fps
    vr_bitrate               INTEGER default 100      not null,--VR码率，默认100Mbps
    sharpening               REAL    default 0.75     not null,--锐化，百分比75%
    ssw                      BOOLEAN default 0        not null,--算法补帧。Synchronous Spacewarp：disabled,automatic,always enabled
    show_performance_overlay BOOLEAN default 0        not null --是否显示性能参数面板
);
CREATE INDEX account_idx on configs(account_id);

-- 存储通用的配置项
CREATE TABLE kv(
            id INTEGER PRIMARY KEY,
            key TEXT NOT NULL UNIQUE,
            value TEXT NOT NULL
        );
