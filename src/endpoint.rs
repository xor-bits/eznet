use once_cell::sync::OnceCell;
use quinn::{ClientConfig, Incoming};
use rustls::{client::ServerCertVerifier, Certificate};
use std::{
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    sync::Arc,
};

//

#[derive(Debug, Clone)]
pub struct Endpoint {
    inner: quinn::Endpoint,
}

//

impl Endpoint {
    pub fn addr(&self) {}

    pub fn inner(&self) -> &quinn::Endpoint {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut quinn::Endpoint {
        &mut self.inner
    }

    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.inner.local_addr()
    }

    pub async fn wait_idle(&self) {
        self.inner.wait_idle().await
    }
}

impl From<quinn::Endpoint> for Endpoint {
    fn from(inner: quinn::Endpoint) -> Self {
        Self { inner }
    }
}
