pub mod common;
pub mod rest;
pub mod websocket;

pub trait ExchangeClient: common::Common + rest::Rest + websocket::Websocket + Send + Sync {}

impl<T> ExchangeClient for T where
    T: common::Common + rest::Rest + websocket::Websocket + Send + Sync
{
}
