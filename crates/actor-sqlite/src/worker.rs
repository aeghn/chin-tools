use flume::Receiver;
use log::{debug, error};
use rusqlite::{Connection, Statement, Transaction, types::Value};
use std::sync::Arc;

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
    log::debug!("conn run {command:#?}");

    match command {
        ConnCmdReq::Transaction => {
            let tranaction = match conn.transaction() {
                Ok(transaction) => transaction,
                Err(err) => {
                    otx.send(Err(ActorSqlError::CustomRusqliteError(err)))?;
                    return Ok(());
                }
            };
            let (tx, rx) = flume::unbounded();
            otx.send(Ok(ConnCmdRsp::Tx(tx)))?;
            debug!("actlite: created tranaction");
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
        log::debug!("begin to recv on transaction");
        let RspWrapper { command, otx } = rx.recv()?;
        log::debug!("transaction run {command:#?}");
        match command {
            TxCmdReq::Command(cmd) => match executor.handle(cmd) {
                Ok(rsp) => {
                    debug!("actlite: transaction execute done {rsp:?}");
                    otx.send(Ok(TxCmdRsp::Cmd(rsp)))?;
                }
                Err(err) => {
                    otx.send(Err(err))?;
                }
            },
            TxCmdReq::Commit => {
                match tx.commit() {
                    Ok(_) => {
                        debug!("actlite: transaction commit");
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
                        debug!("actlite: transaction rollback");
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
    pub(crate) fn loop_handle(
        in_rx: flume::Receiver<RspWrapper<ConnCmdReq, ConnCmdRsp>>,
        mut conn: Connection,
    ) {
        loop {
            match in_rx.recv() {
                Ok(cb) => {
                    let result = conn_run(&mut conn, cb);
                    match result {
                        Ok(_) => {}
                        Err(err) => {
                            error!("loophandle -> ActorSqlError {err}");
                            break;
                        }
                    }
                }
                Err(err) => {
                    error!("loophandle -> unable to receive {err}");
                    break;
                }
            }
        }
    }
}
