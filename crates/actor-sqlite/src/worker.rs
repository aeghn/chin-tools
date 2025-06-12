use flume::Receiver;
use rusqlite::{Connection, OpenFlags, Statement, Transaction, types::Value};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    thread,
};

use crate::{ActorSqlError, Result, model::*};

pub(super) struct ActorSqliteWorker;

enum CmdExecutor<'a> {
    Tx(&'a Transaction<'a>),
    Conn(&'a mut Connection),
}

impl<'a> From<&'a Transaction<'a>> for CmdExecutor<'a> {
    fn from(value: &'a Transaction<'a>) -> Self {
        Self::Tx(value)
    }
}

impl<'a> From<&'a mut Connection> for CmdExecutor<'a> {
    fn from(value: &'a mut Connection) -> Self {
        Self::Conn(value)
    }
}

impl CmdExecutor<'_> {
    fn prepare(&self, sql: &str) -> Result<Statement<'_>> {
        match self {
            CmdExecutor::Tx(transaction) => Ok(transaction.prepare(sql)?),
            CmdExecutor::Conn(connection) => Ok(connection.prepare(sql)?),
        }
    }

    fn handle(&self, req: CmdReq) -> Result<CmdResult> {
        match req {
            CmdReq::Exec { sql, params } => {
                let stmt = self.prepare(&sql)?;

                let res = CmdExecutor::handle_exec(stmt, params);
                res.map(CmdResult::Exec)
            }
            CmdReq::QueryMap { sql, params } => {
                let stmt = self.prepare(&sql)?;

                let res = CmdExecutor::handle_query(stmt, params);
                res.map(CmdResult::QueryMap)
            }
        }
    }

    fn handle_exec(mut stmt: Statement<'_>, params: SqlValueVec) -> Result<usize> {
        let res = stmt.execute(
            params
                .iter()
                .map(|e| e as &dyn rusqlite::types::ToSql)
                .collect::<Vec<&dyn rusqlite::types::ToSql>>()
                .as_slice(),
        )?;
        Ok(res)
    }

    fn handle_query(mut stmt: Statement<'_>, params: SqlValueVec) -> Result<Vec<SVRow>> {
        let columns: Vec<String> = stmt
            .column_names()
            .into_iter()
            .map(|e| e.to_owned())
            .collect();

        let res: std::result::Result<Vec<SVRow>, rusqlite::Error> = stmt
            .query_map(
                params
                    .iter()
                    .map(|e| e as &dyn rusqlite::types::ToSql)
                    .collect::<Vec<&dyn rusqlite::types::ToSql>>()
                    .as_slice(),
                |row| {
                    let cells: Vec<(Arc<str>, Value)> = columns
                        .iter()
                        .map(|idx| (idx.to_string().into(), row.get_unwrap(idx.as_str())))
                        .collect();
                    Ok(SVRow { cells })
                },
            )?
            .collect();

        match res {
            Ok(r) => Ok(r),
            Err(err) => Err(err.into()),
        }
    }
}

pub(crate) fn conn_run(
    conn: &mut Connection,
    req: RspWrapper<ConnCmdReq, ConnCmdRsp>,
) -> Result<()> {
    let RspWrapper { command, otx } = req;
    log::debug!("conn run {:#?}", command);

    match command {
        ConnCmdReq::Transaction => {
            let tranaction = match conn.transaction() {
                Ok(tx) => tx,
                Err(err) => {
                    otx.send(Err(ActorSqlError::CustomRusqliteError(err)))?;
                    return Ok(());
                }
            };
            let (tx, rx) = flume::unbounded();
            otx.send(Ok(ConnCmdRsp::Tx(tx)))?;
            tx_run(tranaction, rx)?;
        }
        ConnCmdReq::Command(cmd) => match CmdExecutor::from(conn).handle(cmd) {
            Ok(rsp) => {
                otx.send(Ok(ConnCmdRsp::Cmd(rsp)))?;
            }
            Err(err) => {
                otx.send(Err(err))?;
            }
        },
    }

    Ok(())
}

pub(crate) fn tx_run<'a>(
    tx: Transaction<'a>,
    rx: Receiver<RspWrapper<TxCmdReq, TxCmdRsp>>,
) -> Result<()> {
    let executor = CmdExecutor::from(&tx);
    loop {
        let RspWrapper { command, otx } = rx.recv()?;
        log::debug!("conn run {:#?}", command);
        match command {
            TxCmdReq::Command(cmd) => match executor.handle(cmd) {
                Ok(rsp) => {
                    otx.send(Ok(TxCmdRsp::Cmd(rsp)))?;
                }
                Err(err) => {
                    otx.send(Err(err))?;
                }
            },
            TxCmdReq::Commit => {
                match tx.commit() {
                    Ok(_) => {
                        otx.send(Ok(TxCmdRsp::Committed))?;
                    }
                    Err(err) => {
                        otx.send(Err(err.into()))?;
                    }
                };
                break;
            }
            TxCmdReq::Rollback => {
                match tx.rollback() {
                    Ok(_) => {
                        otx.send(Ok(TxCmdRsp::Rollbacked))?;
                    }
                    Err(err) => {
                        otx.send(Err(err.into()))?;
                    }
                };

                break;
            }
        }
    }

    Ok(())
}

impl ActorSqliteWorker {
    pub fn builder() -> WorkerConfig {
        WorkerConfig::default()
    }

    fn loop_handle(
        in_rx: flume::Receiver<RspWrapper<ConnCmdReq, ConnCmdRsp>>,
        mut conn: Connection,
    ) {
        loop {
            match in_rx.recv() {
                Ok(cb) => {
                    conn_run(&mut conn, cb).unwrap();
                }
                Err(err) => {
                    println!("unable to receive {}", err);
                    unimplemented!()
                }
            }
        }
    }
}

#[derive(Default)]
// Stolen from https://github.com/ryanfowler/async-sqlite/blob/main/src/client.rs
pub struct WorkerConfig {
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

impl WorkerConfig {
    pub fn path<P: AsRef<Path>>(self, path: P) -> Self {
        Self {
            path: path.as_ref().to_owned(),
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
                    "unablt to set journal_mode: {:?}",
                    journal_mode
                )));
            }
        }

        Ok(conn)
    }
}
