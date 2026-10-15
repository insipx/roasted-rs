/// Max buffer size over serial connection
pub const MAX_BUFFER_SIZE: usize = 512;

pub mod net {
    pub const HOSTNAME: &str = "zippy";
}

pub mod db {
    pub const CURRENT_VERSION: u8 = 1;
    pub const AEAD: &[u8; 16] = b"roasted/cred/v01";
    pub const MEM_SIZE: usize = 1536;
    pub const TREE_SIZE: usize = 1024;
    pub const HMAC_LABEL: &[u8; 29] = b"roasted/zippy/database-key/v1";
    pub const HEADER_SIZE: usize = 8;
    pub const DB_HEADER: &[u8; HEADER_SIZE] = b"ZIPPY_DB";
    pub const DB_START: u32 = 8;
}
