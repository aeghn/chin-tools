use chin_sql::{SqlValueOwned, SqlValueRow};
use chin_tools::AResult;
use flume::Sender;

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

pub enum CmdResult {
    Exec(usize),
    QueryMap(Vec<SVRow>),
}

pub enum ConnCmdRsp {
    Cmd(CmdResult),
    Tx(TxInner),
}

pub enum ConnCmdReq {
    Transaction,
    Command(CmdReq),
}

pub enum TxCmdReq {
    Command(CmdReq),
    Commit,
    Rollback,
}

pub enum TxCmdRsp {
    Cmd(CmdResult),
    Closed,
}

pub struct RspWrapper<T, V> {
    pub command: T,
    pub otx: oneshot::Sender<AResult<V>>,
}

pub(crate) type SVRow = SqlValueRow<SqlValueOwned>;
pub(crate) type SqlValueVec = Vec<SqlValueOwned>;
pub(crate) type TxInner = Sender<RspWrapper<TxCmdReq, TxCmdRsp>>;
