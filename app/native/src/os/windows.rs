#[allow(non_camel_case_types)]
pub type QUERY_DEVICE_CONFIG_FLAGS = u32;

pub static QDC_ALL_PATHS: QUERY_DEVICE_CONFIG_FLAGS = 0x00000001;
pub static QDC_ONLY_ACTIVE_PATHS: QUERY_DEVICE_CONFIG_FLAGS = 0x00000002;
pub static QDC_DATABASE_CURRENT: QUERY_DEVICE_CONFIG_FLAGS = 0x00000004;
