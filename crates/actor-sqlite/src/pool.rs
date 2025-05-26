use std::sync::{Arc, atomic::AtomicU8};

use chin_tools::AResult;
use flume::{Receiver, Sender};

use crate::model::*;
use crate::worker::WorkerConfig;

use super::{client::ActorSqliteConnClient, worker::ActorSqliteWorker};

pub struct InnerActorSqlitePool {
    worker_tx: Sender<RspWrapper<ConnCmdReq, ConnCmdRsp>>,
    worker_rx: Receiver<RspWrapper<ConnCmdReq, ConnCmdRsp>>,
    config: WorkerConfig,
    worker_count: AtomicU8,
}

#[derive(Clone)]
pub struct ActorSqlitePool {
    inner: Arc<InnerActorSqlitePool>,
}

//
impl ActorSqlitePool {
    pub fn create(config: WorkerConfig) -> AResult<ActorSqlitePool> {
        let (worker_tx, worker_rx) = flume::unbounded();

        for i in 0..config.pool_size.unwrap_or(4) {
            ActorSqliteWorker::builder()
                .path(&config.path)
                .spawn(worker_rx.clone())?;
        }
        let inner = InnerActorSqlitePool {
            worker_tx,
            worker_rx,
            config,
            worker_count: 0.into(),
        };

        Ok(Self {
            inner: inner.into(),
        })
    }

    pub async fn get(&self) -> AResult<ActorSqliteConnClient> {
        Ok(ActorSqliteConnClient {
            inner: self.inner.worker_tx.clone(),
        })
    }
}
