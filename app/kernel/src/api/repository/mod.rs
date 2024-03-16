pub mod account;
pub mod config;
pub mod kv;

use self::{account::AccountRepository, config::ConfigRepository, kv::KVRepository};
use crate::error::CoreResult;
use r2d2_sqlite::SqliteConnectionManager;
use std::{path::Path, sync::Arc};

#[derive(Clone)]
pub struct LocalStorage {
    account: Arc<AccountRepository>,
    config: Arc<ConfigRepository>,
    kv: Arc<KVRepository>,
}

impl LocalStorage {
    pub fn new<P>(db_path: P) -> CoreResult<LocalStorage>
        where
            P: AsRef<Path>,
    {
        let manager = SqliteConnectionManager::file(db_path);
        let pool = r2d2::Pool::new(manager)?;

        let account_repository = AccountRepository::new(pool.clone());
        account_repository.ensure_table()?;

        let config_repository = ConfigRepository::new(pool.clone());
        config_repository.ensure_table()?;

        let kv_repository = KVRepository::new(pool.clone());
        kv_repository.ensure_table()?;

        Ok(Self {
            account: Arc::new(account_repository),
            config: Arc::new(config_repository),
            kv: Arc::new(kv_repository),
        })
    }

    pub fn account(&self) -> &AccountRepository {
        &self.account
    }

    pub fn config(&self) -> &ConfigRepository {
        &self.config
    }

    pub fn kv(&self) -> &KVRepository {
        &self.kv
    }
}
