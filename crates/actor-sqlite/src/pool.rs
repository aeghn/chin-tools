use std::sync::Arc;

use flume::{Receiver, Sender};

use crate::worker::WorkerConfig;
use crate::{ActorSqlError, Result, model::*};

use super::{client::ActorSqliteConnClient, worker::ActorSqliteWorker};

pub struct InnerActorSqlitePool {
    worker_tx: Sender<RspWrapper<ConnCmdReq, ConnCmdRsp>>,
    worker_rx: Receiver<RspWrapper<ConnCmdReq, ConnCmdRsp>>,
    config: WorkerConfig,
}

pub type ActorSqlitePool = Arc<InnerActorSqlitePool>;

impl TryFrom<WorkerConfig> for ActorSqlitePool {
    type Error = ActorSqlError;

    fn try_from(config: WorkerConfig) -> Result<Self> {
        let (worker_tx, worker_rx) = flume::unbounded();

        for i in 0..config.pool_size.unwrap_or(4) {
            log::info!("creating initial worker-{i}");
            ActorSqliteWorker::builder()
                .path(&config.path)
                .spawn(worker_rx.clone())?;
        }
        let inner = InnerActorSqlitePool {
            worker_tx,
            worker_rx,
            config,
        };
        Ok(inner.into())
    }
}

impl InnerActorSqlitePool {
    pub fn check_size(&self) -> Result<()> {
        let full_count = self.config.pool_size.unwrap_or(4) as usize;
        loop {
            if self.worker_tx.receiver_count() < full_count {
                ActorSqliteWorker::builder()
                    .path(&self.config.path)
                    .spawn(self.worker_rx.clone())?;
            }
        }
    }

    pub async fn get(&self) -> Result<ActorSqliteConnClient> {
        Ok(ActorSqliteConnClient {
            inner: self.worker_tx.clone(),
        })
    }
}
