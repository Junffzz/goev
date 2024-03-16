use crate::error::{ CoreResult};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Row, ToSql};
use serde::Serialize;

#[derive(Default,Debug, Clone, Serialize)]
pub struct Config {
    pub id: i64,
    pub account_id: i64,
    pub screen_brightness: f32,
    pub screen_quality: String,
    pub screen_frame_rate: u8,
    pub desktop_bitrate: u8,
    pub microphone_passthrough: bool,
    pub microphone_volume: f32,
    pub gamma: f32,
    pub vr_graphics_quality: String,//VR画面质量。low(gtx 1070/rx 5500-xt),medium,high,ultra,godlike
    pub vr_frame_rate: u8,
    pub vr_bitrate: u8,
    pub sharpening: f32,
    pub ssw: bool,// 算法补帧。Synchronous Spacewarp：disabled,automatic,always enabled
    pub show_performance_overlay: bool,
}

pub struct ConfigRepository {
    pool: Pool<SqliteConnectionManager>,
}

impl ConfigRepository {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn ensure_table(&self) -> CoreResult<()> {
        let conn = self.pool.get()?;

        const COMMAND: &str = r#"
        CREATE TABLE IF NOT EXISTS configs
        (
            id                       INTEGER                  not null
                primary key,
            account_id               INTEGER                  not null,
            screen_brightness        REAL    default 1.0      not null,
            screen_quality           TEXT    default "medium" not null,
            screen_frame_rate        INTEGER default 72       not null,
            desktop_bitrate          INTEGER default 100      not null,
            microphone_passthrough   BOOLEAN default 1        not null,
            microphone_volume        REAL    default 0.8      not null,
            gamma                    REAL    default 1.0      not null,
            vr_graphics_quality      TEXT    default "medium" not null,
            vr_frame_rate            INTEGER default 72       not null,
            vr_bitrate               INTEGER default 100      not null,
            sharpening               REAL    default 0.75     not null,
            ssw                      BOOLEAN default 0        not null,
            show_performance_overlay BOOLEAN default 0        not null
        )"#;

        conn.execute(COMMAND, [])?;

        Ok(())
    }

    pub fn create_default_config(&self, mut cfg: Config) -> CoreResult<Config> {
        const COMMAND: &str = r#"
        INSERT INTO configs(
            account_id,
            screen_brightness,
            screen_quality,
            screen_frame_rate,
            desktop_bitrate,
            microphone_passthrough,
            microphone_volume,
            gamma,
            vr_graphics_quality,
            vr_frame_rate,
            vr_bitrate,
            sharpening,
            ssw,
            show_performance_overlay
        )
        VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?)"#;

        let conn = self.pool.get()?;
        conn.execute(
            COMMAND,
            params![
                cfg.account_id,
                cfg.screen_brightness,
                cfg.screen_quality,
                cfg.screen_frame_rate,
                cfg.desktop_bitrate,
                cfg.microphone_passthrough,
                cfg.microphone_volume,
                cfg.gamma,
                cfg.vr_graphics_quality,
                cfg.vr_frame_rate,
                cfg.vr_bitrate,
                cfg.sharpening,
                cfg.ssw,
                cfg.show_performance_overlay,

            ],
        )?;

        cfg.id = conn.last_insert_rowid();

        Ok(cfg)
    }

    pub fn update_config(&self, mut cfg: Config) -> CoreResult<()> {
        const COMMAND: &str = r#"
        UPDATE configs
        SET
            account_id= ?,
            screen_brightness= ?,
            screen_quality= ?,
            screen_frame_rate = ?,
            desktop_bitrate = ?,
            microphone_passthrough = ?,
            microphone_volume = ?,
            gamma = ?,
            vr_graphics_quality = ?,
            vr_frame_rate = ?,
            vr_bitrate = ?,
            sharpening = ?,
            ssw = ?,
            show_performance_overlay= ?
        WHERE id = ?"#;

        let conn = self.pool.get()?;
        conn.execute(
            COMMAND,
            params![
                cfg.account_id,
                cfg.screen_brightness,
                cfg.screen_quality,
                cfg.screen_frame_rate,
                cfg.desktop_bitrate,
                cfg.microphone_passthrough,
                cfg.microphone_volume,
                cfg.gamma,
                cfg.vr_graphics_quality,
                cfg.vr_frame_rate,
                cfg.vr_bitrate,
                cfg.sharpening,
                cfg.ssw,
                cfg.show_performance_overlay,
                cfg.id,
            ],
        )?;

        Ok(())
    }

    pub fn get_account_config(&self, account_id: u16) -> CoreResult<Config> {
        const COMMAND: &str = r"SELECT * FROM configs WHERE id = ? LIMIT 1";

        let domain = self
            .pool
            .get()?
            .query_row_and_then(COMMAND, [account_id], parse_config)?;

        Ok(domain)
    }

    pub fn set_config_by_key<T:ToSql>(&self, id: u16, key: String, value: T) -> CoreResult<()> {
        let command = format!("UPDATE configs SET {} = ? WHERE id =?", key);

        self.pool
            .get()?
            .execute(&command, params![value,id])?;

        Ok(())
    }

    pub fn delete_config(&self, domain_id: i64) -> CoreResult<()> {
        const COMMAND: &str = r"DELETE FROM accounts WHERE id = ?";

        self.pool.get()?.execute(COMMAND, [domain_id])?;

        Ok(())
    }
}

fn parse_config(row: &Row) -> CoreResult<Config> {
    Ok(Config {
        id: row.get(0)?,
        account_id: row.get(1)?,
        screen_brightness: row.get(2)?,
        screen_quality: row.get(3)?,
        screen_frame_rate: row.get(4)?,
        desktop_bitrate: row.get(5)?,
        microphone_passthrough: row.get(6)?,
        microphone_volume: row.get(7)?,
        gamma: row.get(8)?,
        vr_graphics_quality: row.get(9)?,
        vr_frame_rate: row.get(10)?,
        vr_bitrate: row.get(11)?,
        sharpening: row.get(12)?,
        ssw: row.get(13)?,
        show_performance_overlay: row.get(14)?,
    })
}
