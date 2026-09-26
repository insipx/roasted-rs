use std::path::Path;

use color_eyre::eyre::Result;
use diesel::{Connection, SqliteConnection, dsl::insert_into, prelude::*};
use roasted_db_schema::schema::gaggimate_status_frames::dsl::*;
use roasted_types::{daemon::ShotSet, db::UUID};

use crate::models::NewStatusFrame;

/// SQLite Database, single connection
pub struct Db {
    connection: SqliteConnection,
}

impl Db {
    pub fn connect(url: &Path) -> Result<Self> {
        Ok(Self { connection: SqliteConnection::establish(&url.to_string_lossy())? })
    }

    pub fn record_frames(&mut self, frames: &ShotSet) -> Result<()> {
        let uuid = UUID::v7_now();
        let frames =
            frames.iter().map(|f| NewStatusFrame::builder().shot_id(&uuid).state(f).build());
        insert_into(gaggimate_status_frames)
            .values(frames.collect::<Vec<NewStatusFrame>>())
            .execute(&mut self.connection)?;
        Ok(())
    }
}
