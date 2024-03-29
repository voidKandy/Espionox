use crate::{
    agents::error::AgentError, errors::error_chain_fmt, language_models::error::ModelEndpointError,
};

use std::fmt::{Debug, Display, Formatter};

pub use super::dispatch::listeners::error::ListenerError;

#[derive(thiserror::Error)]
pub enum EnvHandleError {
    CouldNotOwnNotifications,
    MissingNotifications,
    MissingHandleData,
    MissingThreadHandle,
    ThreadAlreadySpawned,
    EnvError(#[from] EnvError),
}

#[derive(thiserror::Error)]
pub enum EnvError {
    #[error(transparent)]
    Undefined(#[from] anyhow::Error),
    Listener(#[from] ListenerError),
    Dispatch(#[from] DispatchError),
    Join(#[from] tokio::task::JoinError),
    Timeout(#[from] tokio::time::error::Elapsed),
    MissingHandleData,
    Request(String),

    Send,
}

impl From<ModelEndpointError> for DispatchError {
    fn from(value: ModelEndpointError) -> Self {
        Self::Agent(value.into())
    }
}

#[derive(thiserror::Error)]
pub enum DispatchError {
    Undefined(#[from] anyhow::Error),
    Listener(#[from] ListenerError),
    Agent(#[from] AgentError),
    Timeout(#[from] tokio::time::error::Elapsed),
    NoApiKey,
    AgentIsNone,
    Send,
}

impl Debug for EnvError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl Display for EnvError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Debug for EnvHandleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl Display for EnvHandleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Debug for DispatchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl Display for DispatchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
