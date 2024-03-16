use crate::error::{CoreError, CoreResult};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, OptionalExtension, Row};
use serde::Serialize;

#[derive(Default,Debug, Clone, Serialize)]
pub struct Account {
    pub id: u16,
    pub title: String,
    pub remote_ip: String,
    pub remote_port: u16,
    pub is_default: bool,
    pub device_id: i64,
    pub remote_os: String,
    pub auto_connect: bool,
    pub remarks: String,
}

pub struct AccountRepository {
    pool: Pool<SqliteConnectionManager>,
}

impl AccountRepository {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn ensure_table(&self) -> CoreResult<()> {
        let conn = self.pool.get()?;

        const COMMAND: &str = r"
        CREATE TABLE IF NOT EXISTS accounts(
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL UNIQUE,
            remote_ip TEXT NOT NULL,
            remote_port INTEGER NOT NULL,
            is_default BOOLEAN DEFAULT 0,
            device_id INTEGER NOT NULL,
            remote_os TEXT NOT NULL,
            auto_connect BOOLEAN DEFAULT 0,
            remarks TEXT NOT NULL
        )";

        conn.execute(COMMAND, [])?;

        Ok(())
    }

    pub fn add_account(&self, mut domain: Account) -> CoreResult<Account> {
        const COMMAND: &str = r#"
        INSERT INTO accounts(
            title,
            remote_ip,
            remote_port,
            is_default,
            device_id,
            remote_os,
            auto_connect,
            remarks
        )
        VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?)"#;

        let conn = self.pool.get()?;
        conn.execute(
            COMMAND,
            params![
                domain.title,
                domain.remote_ip,
                domain.remote_port,
                domain.is_default,
                domain.device_id,
                domain.remote_os,
                domain.auto_connect,
                domain.remarks,
            ],
        )?;

        domain.id = conn.last_insert_rowid() as u16;

        Ok(domain)
    }

    pub fn get_default_account(&self) -> CoreResult<Account> {
        const COMMAND: &str = r"SELECT * FROM accounts WHERE is_default = 1 LIMIT 1";

        self.pool
            .get()?
            .query_row_and_then(COMMAND, [], parse_domain)
    }

    pub fn account_exist(&self, name: &str) -> CoreResult<bool> {
        const COMMAND: &str = r"SELECT 1 FROM accounts WHERE name = ?";

        let res = self
            .pool
            .get()?
            .query_row(COMMAND, [name], |row| row.get::<_, u32>(0))
            .optional()?;

        Ok(res.is_some())
    }

    pub fn get_account_id_and_names(&self) -> CoreResult<Vec<(i64, String)>> {
        const COMMAND: &str = r"SELECT id, name FROM accounts";

        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(COMMAND)?;
        let rows = stmt.query_and_then([], |row| -> CoreResult<(i64, String)> {
            Ok((row.get(0)?, row.get(1)?))
        })?;

        let mut id_and_names = Vec::new();
        for row in rows {
            id_and_names.push(row?);
        }

        Ok(id_and_names)
    }

    pub fn get_account_by_name(&self, name: String) -> CoreResult<Account> {
        const COMMAND: &str = r"SELECT * FROM accounts WHERE name = ? LIMIT 1";

        let domain = self
            .pool
            .get()?
            .query_row_and_then(COMMAND, [name], parse_domain)?;

        Ok(domain)
    }

    pub fn get_account_by_id(&self, domain_id: i64) -> CoreResult<Account> {
        const COMMAND: &str = r"SELECT * FROM accounts WHERE id = ?";

        let domain = self
            .pool
            .get()?
            .query_row_and_then(COMMAND, [domain_id], parse_domain)?;

        Ok(domain)
    }

    pub fn get_accounts(&self, page: u32, limit: u32) -> CoreResult<(u32, Vec<Account>)> {
        const COUNT_COMMAND: &str = r"SELECT COUNT(*) FROM accounts";
        const PAGINATION_COMMAND: &str = r"SELECT * FROM accounts LIMIT ? OFFSET ?";

        let conn = self.pool.get()?;

        let count = conn.query_row_and_then(COUNT_COMMAND, [], |row| -> CoreResult<u32> {
            Ok(row.get(0)?)
        })?;

        let mut stmt = conn.prepare(PAGINATION_COMMAND)?;
        let rows = stmt.query_and_then([limit, (page - 1) * limit], parse_domain)?;

        let mut domains = Vec::new();
        for row in rows {
            domains.push(row?);
        }

        Ok((count, domains))
    }

    pub fn get_account_count(&self) -> CoreResult<u32> {
        const COMMAND: &str = r"SELECT COUNT(*) FROM accounts";
        self.pool
            .get()?
            .query_row_and_then(COMMAND, [], |row| Ok(row.get(0)?))
    }

    pub fn set_account_is_default(&self, domain_id: i64) -> CoreResult<()> {
        const UNSET_PRIMARY_COMMAND: &str =
            r"UPDATE accounts SET is_default = 0 WHERE is_primary = 1";
        const SET_PRIMARY_COMMAND: &str = r"UPDATE accounts SET is_default = 1 WHERE id = ?";

        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        if let Err(err) = tx.execute(UNSET_PRIMARY_COMMAND, []) {
            tx.rollback()?;
            return Err(CoreError::SQLiteError(err));
        }

        if let Err(err) = tx.execute(SET_PRIMARY_COMMAND, [domain_id]) {
            tx.rollback()?;
            return Err(CoreError::SQLiteError(err));
        }

        tx.commit()?;

        Ok(())
    }

    pub fn set_account_device_id(&self, domain_id: i64, device_id: i64) -> CoreResult<()> {
        const COMMAND: &str = r"UPDATE accounts SET device_id = ? WHERE id =?";

        self.pool
            .get()?
            .execute(COMMAND, params![device_id, domain_id])?;

        Ok(())
    }

    pub fn set_account_remarks(&self, domain_id: i64, remarks: &str) -> CoreResult<()> {
        const COMMAND: &str = r"UPDATE accounts SET remarks = ? WHERE id =?";

        self.pool
            .get()?
            .execute(COMMAND, params![remarks, domain_id])?;

        Ok(())
    }

    pub fn delete_account(&self, domain_id: i64) -> CoreResult<()> {
        const COMMAND: &str = r"DELETE FROM accounts WHERE id = ?";

        self.pool.get()?.execute(COMMAND, [domain_id])?;

        Ok(())
    }
}

fn parse_domain(row: &Row) -> CoreResult<Account> {
    Ok(Account {
        id: row.get(0)?,
        title: row.get(1)?,
        remote_ip: row.get(2)?,
        remote_port: row.get(3)?,
        is_default: row.get(4)?,
        device_id: row.get(5)?,
        remote_os: row.get(6)?,
        auto_connect: row.get(7)?,
        remarks: row.get(8)?
    })
}
