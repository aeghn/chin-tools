use chin_sql::{SqlValueOwned, SqlValueRow};
use chin_tools::AResult;
use flume::Sender;

#[derive(Debug)]
pub enum CmdReq {
    Exec {
        sql: String,
        params: Vec<SqlValueOwned>,
    },
    QueryMap {
        sql: String,
        params: Vec<SqlValueOwned>,
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
    pub otx: oneshot::Sender<AResult<V>>,
}

pub(crate) type SVRow = SqlValueRow<SqlValueOwned>;
pub(crate) type SqlValueVec = Vec<SqlValueOwned>;
pub(crate) type TxInner = Sender<RspWrapper<TxCmdReq, TxCmdRsp>>;
