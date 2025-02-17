use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum MainError {
    #[error("No action error")]
    NoAction,
    #[error("RPC error")]
    RpcError,
    #[error("Can't commit block to database")]
    Database,
    #[error("Failed to join async task")]
    TaskJoinError,
}

pub trait AsRpcError<T> {
    fn into_rpc_error(self) -> Result<T, MainError>;
}

impl<T> AsRpcError<T> for anyhow::Result<T> {
    #[inline]
    fn into_rpc_error(self) -> Result<T, MainError> {
        self.map_err(|reason| {
            tracing::error!(?reason, "RPC error");
            MainError::RpcError
        })
    }
}

pub trait AsDbError<T> {
    fn into_db_error(self) -> Result<T, MainError>;
}

impl<T> AsDbError<T> for anyhow::Result<T> {
    #[inline]
    fn into_db_error(self) -> Result<T, MainError> {
        self.map_err(|reason| {
            tracing::error!(?reason, "Database error");
            MainError::Database
        })
    }
}

pub trait AsTaskJoinError<T> {
    fn into_task_join_error(self) -> Result<T, MainError>;
}

impl<T> AsTaskJoinError<T> for anyhow::Result<T> {
    #[inline]
    fn into_task_join_error(self) -> Result<T, MainError> {
        self.map_err(|reason| {
            tracing::error!(?reason, "{}", MainError::TaskJoinError);
            MainError::TaskJoinError
        })
    }
}

pub trait ContextDbInteractError<T> {
    fn context_db_interact_error(self) -> anyhow::Result<T>;
}

impl<T, E> ContextDbInteractError<T> for Result<T, E> {
    fn context_db_interact_error(self) -> anyhow::Result<T> {
        self.map_err(|_| anyhow::anyhow!("Failed to interact with db"))
    }
}
