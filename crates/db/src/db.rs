use std::path::Path;

use color_eyre::eyre::Result;
use diesel::{Connection, SqliteConnection};

/// SQLite Database, single connection
pub struct Db {
    connection: SqliteConnection,
}

impl Db {
    pub fn connect(url: &Path) -> Result<Self> {
        Ok(Self { connection: SqliteConnection::establish(&url.to_string_lossy())? })
    }
}
