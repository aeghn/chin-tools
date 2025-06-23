use flume::Receiver;
use rusqlite::{Connection, OpenFlags};
use std::{
    path::{Path, PathBuf},
    thread,
};

use crate::{ActorSqlError, Result};

use crate::{
    model::{ConnCmdReq, ConnCmdRsp, RspWrapper},
    worker::ActorSqliteWorker,
};

#[derive(Default, Clone)]
// Stolen from https://github.com/ryanfowler/async-sqlite/blob/main/src/client.rs
pub struct PoolConfig {
    pub(crate) path: PathBuf,
    flags: OpenFlags,
    journal_mode: Option<JournalMode>,
    vfs: Option<String>,
    pub(crate) pool_size: Option<u8>,
}

/// The possible sqlite journal modes.
///
/// For more information, please see the [sqlite docs](https://www.sqlite.org/pragma.html#pragma_journal_mode).
#[derive(Clone, Copy, Debug)]
pub enum JournalMode {
    Delete,
    Truncate,
    Persist,
    Memory,
    Wal,
    Off,
}

impl JournalMode {
    /// Returns the appropriate string representation of the journal mode.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Delete => "DELETE",
            Self::Truncate => "TRUNCATE",
            Self::Persist => "PERSIST",
            Self::Memory => "MEMORY",
            Self::Wal => "WAL",
            Self::Off => "OFF",
        }
    }
}

impl PoolConfig {
    pub fn path<P: AsRef<Path>>(self, path: P) -> Self {
        Self {
            path: path.as_ref().to_owned(),
            ..self
        }
    }

    pub fn pool_size(self, pool_size: u8) -> Self {
        Self {
            pool_size: Some(pool_size),
            ..self
        }
    }

    pub fn journal_mode(self, journal_mode: JournalMode) -> Self {
        Self {
            journal_mode: Some(journal_mode),
            ..self
        }
    }

    pub fn spawn(self, in_rx: Receiver<RspWrapper<ConnCmdReq, ConnCmdRsp>>) -> Result<()> {
        let conn = self.build_conn()?;

        thread::spawn(move || {
            let conn = conn;
            ActorSqliteWorker::loop_handle(in_rx, conn);
        });

        Ok(())
    }

    fn build_conn(mut self) -> Result<Connection> {
        let path = self.path.clone();
        let conn = if let Some(vfs) = self.vfs.take() {
            Connection::open_with_flags_and_vfs(path, self.flags, &vfs)?
        } else {
            Connection::open_with_flags(path, self.flags)?
        };

        if let Some(journal_mode) = self.journal_mode.take() {
            let val = journal_mode.as_str();
            let out: String =
                conn.pragma_update_and_check(None, "journal_mode", val, |row| row.get(0))?;
            if !out.eq_ignore_ascii_case(val) {
                return Err(ActorSqlError::RusqliteBuildError(format!(
                    "unable to set journal_mode: {:?} -- {}",
                    journal_mode, out
                )));
            }
        }

        Ok(conn)
    }
}
