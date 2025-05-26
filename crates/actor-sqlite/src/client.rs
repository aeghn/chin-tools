use chin_sql::{SqlValueOwned, SqlValueRow};
use chin_tools::{AResult, EResult};
use flume::Sender;

use crate::model::*;

#[derive(Clone)]
pub struct ActorSqliteTxClient {
    pub(super) inner: flume::Sender<RspWrapper<TxCmdReq, TxCmdRsp>>,
}

pub struct ActorSqliteConnClient {
    pub(super) inner: Sender<RspWrapper<ConnCmdReq, ConnCmdRsp>>,
}

impl ActorSqliteConnClient {
    async fn inner(&self, req: ConnCmdReq) -> AResult<ConnCmdRsp> {
        let (otx, orx) = oneshot::channel();
        self.inner.send(RspWrapper { command: req, otx })?;
        orx.await?
    }

    pub async fn execute(&self, sql: String, params: SqlValueVec) -> AResult<usize> {
        match self
            .inner(ConnCmdReq::Command(CmdReq::Exec { sql, params }))
            .await?
        {
            ConnCmdRsp::Cmd(CmdResult::Exec(count)) => Ok(count),
            _ => unreachable!(),
        }
    }

    pub async fn query(
        &self,
        sql: String,
        params: SqlValueVec,
    ) -> AResult<Vec<SqlValueRow<SqlValueOwned>>> {
        match self
            .inner(ConnCmdReq::Command(CmdReq::QueryMap { sql, params }))
            .await?
        {
            ConnCmdRsp::Cmd(CmdResult::QueryMap(res)) => Ok(res),
            _ => unreachable!(),
        }
    }

    pub async fn transaction(&mut self) -> AResult<ActorSqliteTxClient> {
        match self.inner(ConnCmdReq::Transaction).await? {
            ConnCmdRsp::Tx(tx) => Ok(ActorSqliteTxClient { inner: tx }),
            _ => unreachable!(),
        }
    }
}

impl ActorSqliteTxClient {
    async fn inner(&self, command: TxCmdReq) -> AResult<TxCmdRsp> {
        let (otx, orx) = oneshot::channel();
        let _ = self.inner.send(RspWrapper { command, otx });
        orx.await?
    }

    pub async fn execute(&self, sql: String, params: SqlValueVec) -> AResult<usize> {
        match self
            .inner(TxCmdReq::Command(CmdReq::Exec { sql, params }))
            .await?
        {
            TxCmdRsp::Cmd(CmdResult::Exec(res)) => Ok(res),
            _ => unreachable!(),
        }
    }

    pub async fn query(
        &self,
        sql: String,
        params: SqlValueVec,
    ) -> AResult<Vec<SqlValueRow<SqlValueOwned>>> {
        match self
            .inner(TxCmdReq::Command(CmdReq::QueryMap { sql, params }))
            .await?
        {
            TxCmdRsp::Cmd(CmdResult::QueryMap(res)) => Ok(res),
            _ => unreachable!(),
        }
    }

    pub async fn commit(&self) -> EResult {
        match self.inner(TxCmdReq::Commit).await? {
            TxCmdRsp::Closed => Ok(()),
            _ => unreachable!(),
        }
    }

    pub async fn rollback(&self) -> EResult {
        match self.inner(TxCmdReq::Rollback).await? {
            TxCmdRsp::Closed => Ok(()),
            _ => unreachable!(),
        }
    }
}
