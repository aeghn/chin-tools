use std::sync::Arc;

use flume::Sender;
use rusqlite::types::Value;

use crate::Result;

#[derive(Debug)]
pub enum CmdReq {
    Exec {
        sql: String,
        params: Vec<Value>,
    },
    QueryMap {
        sql: String,
        params: Vec<Value>,
    },
}

#[derive(Debug)]
pub enum CmdResult {
    Exec(usize),
    QueryMap(Vec<SVRow>),
}

#[derive(Debug)]
pub enum ConnCmdRsp {
    Cmd(CmdResult),
    Tx(TxInner),
}

#[derive(Debug)]
pub enum ConnCmdReq {
    Transaction,
    Command(CmdReq),
}

#[derive(Debug)]
pub enum TxCmdReq {
    Command(CmdReq),
    Commit,
    Rollback,
}

#[derive(Debug)]
pub enum TxCmdRsp {
    Cmd(CmdResult),
    Committed,
    Rollbacked,
    Closed,
}

#[derive(Debug)]
pub struct RspWrapper<T, V> {
    pub command: T,
    pub otx: oneshot::Sender<Result<V>>,
}

#[derive(Clone, Debug)]
pub struct ActorSqliteRow {
    pub cells: Vec<(Arc<str>, Value)>
}

pub(crate) type SVRow = ActorSqliteRow;
pub(crate) type SqlValueVec = Vec<Value>;
pub(crate) type TxInner = Sender<RspWrapper<TxCmdReq, TxCmdRsp>>;
