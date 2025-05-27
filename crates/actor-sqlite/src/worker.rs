use std::{
    borrow::Cow,
    collections::HashMap,
    path::{Path, PathBuf},
    thread,
};

use chin_sql::{SqlValue, SqlValueOwned};
use chin_tools::{AResult, EResult, aanyhow};
use flume::Receiver;
use rusqlite::{Connection, OpenFlags, Statement, Transaction};

use crate::model::*;

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
    fn prepare(&self, sql: &str) -> AResult<Statement<'_>> {
        match self {
            CmdExecutor::Tx(transaction) => Ok(transaction.prepare(sql)?),
            CmdExecutor::Conn(connection) => Ok(connection.prepare(sql)?),
        }
    }

    fn handle(&self, req: CmdReq) -> AResult<CmdResult> {
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

    fn handle_exec(mut stmt: Statement<'_>, params: SqlValueVec) -> AResult<usize> {
        let params: Vec<SqlValue<'_>> = params.into_iter().map(|p| p.into()).collect();

        let res = stmt.execute(
            params
                .iter()
                .map(|e| e as &dyn rusqlite::types::ToSql)
                .collect::<Vec<&dyn rusqlite::types::ToSql>>()
                .as_slice(),
        )?;
        Ok(res)
    }

    fn handle_query(mut stmt: Statement<'_>, params: SqlValueVec) -> AResult<Vec<SVRow>> {
        let columns: Vec<String> = stmt
            .column_names()
            .into_iter()
            .map(|e| e.to_owned())
            .collect();

        let res: Result<Vec<SVRow>, rusqlite::Error> = stmt
            .query_map(
                params
                    .iter()
                    .map(|e| e as &dyn rusqlite::types::ToSql)
                    .collect::<Vec<&dyn rusqlite::types::ToSql>>()
                    .as_slice(),
                |row| {
                    let cs: HashMap<String, SqlValueOwned> = columns
                        .iter()
                        .map(|idx| {
                            (idx.to_string(), {
                                let sv: SqlValue<'_> = match row.get_ref_unwrap(idx.as_str()) {
                                    rusqlite::types::ValueRef::Null => SqlValue::Opt(None),
                                    rusqlite::types::ValueRef::Integer(i) => SqlValue::I64(i),
                                    rusqlite::types::ValueRef::Real(f) => SqlValue::F64(f),
                                    rusqlite::types::ValueRef::Text(items) => {
                                        SqlValue::Str(String::from_utf8_lossy(items))
                                    }
                                    rusqlite::types::ValueRef::Blob(items) => {
                                        SqlValue::Blob(Cow::Owned(items.to_vec()))
                                    }
                                };
                                SqlValueOwned::from(sv)
                            })
                        })
                        .collect();
                    Ok(SVRow { row: cs })
                },
            )?
            .collect();

        match res {
            Ok(r) => Ok(r),
            Err(err) => Err(err.into()),
        }
    }
}

pub(crate) fn conn_run(conn: &mut Connection, req: RspWrapper<ConnCmdReq, ConnCmdRsp>) -> EResult {
    let RspWrapper { command, otx } = req;
    log::debug!("conn run {:#?}", command);

    match command {
        ConnCmdReq::Transaction => {
            let tranaction = match conn.transaction() {
                Ok(tx) => tx,
                Err(err) => {
                    otx.send(Err(aanyhow!(err)))?;
                    return Ok(());
                }
            };
            let (tx, rx) = flume::unbounded();
            otx.send(Ok(ConnCmdRsp::Tx(tx)))?;
            tx_run(tranaction, rx)?;
        }
        ConnCmdReq::Command(cmd) => {
            let rsp = CmdExecutor::from(conn).handle(cmd)?;
            otx.send(Ok(ConnCmdRsp::Cmd(rsp)))?;
        }
    }

    Ok(())
}

pub(crate) fn tx_run<'a>(
    tx: Transaction<'a>,
    rx: Receiver<RspWrapper<TxCmdReq, TxCmdRsp>>,
) -> EResult {
    let executor = CmdExecutor::from(&tx);
    loop {
        let RspWrapper { command, otx } = rx.recv()?;
        log::debug!("conn run {:#?}", command);
        match command {
            TxCmdReq::Command(cmd) => {
                let rsp = executor.handle(cmd)?;
                otx.send(Ok(TxCmdRsp::Cmd(rsp)))?;
            }
            TxCmdReq::Commit => {
                tx.commit()?;
                otx.send(Ok(TxCmdRsp::Committed))?;

                break;
            }
            TxCmdReq::Rollback => {
                tx.rollback()?;
                otx.send(Ok(TxCmdRsp::Rollbacked))?;

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
                Err(_) => {
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

    pub fn spawn(self, in_rx: Receiver<RspWrapper<ConnCmdReq, ConnCmdRsp>>) -> EResult {
        let conn = self.build_conn()?;

        thread::spawn(move || {
            let conn = conn;
            ActorSqliteWorker::loop_handle(in_rx, conn);
        });

        Ok(())
    }

    fn build_conn(mut self) -> AResult<Connection> {
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
                return Err(aanyhow!("unablt to set journal_mode: {:?}", journal_mode));
            }
        }

        Ok(conn)
    }
}
