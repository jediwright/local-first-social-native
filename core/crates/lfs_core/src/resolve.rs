//! B4 — one live DID → PDS endpoint resolve via atrium-identity. Read-only.
use atrium_api::types::string::Did;
use atrium_common::resolver::Resolver;
use atrium_identity::did::{CommonDidResolver, CommonDidResolverConfig, DEFAULT_PLC_DIRECTORY_URL};
use atrium_xrpc_client::reqwest::ReqwestClient;
use std::sync::Arc;

use crate::CoreError;

pub async fn resolve_pds(did: &str) -> Result<String, CoreError> {
    let did: Did = did.parse().map_err(|e: &str| CoreError::Network(e.to_string()))?;
    let resolver = CommonDidResolver::new(CommonDidResolverConfig {
        plc_directory_url: DEFAULT_PLC_DIRECTORY_URL.to_string(),
        http_client: Arc::new(ReqwestClient::new("https://bsky.social")),
    });
    let doc = resolver.resolve(&did).await.map_err(|e| CoreError::Network(e.to_string()))?;
    doc.get_pds_endpoint()
        .ok_or_else(|| CoreError::Network("no PDS service in DID document".into()))
}

pub fn resolve_pds_blocking(did: &str) -> Result<String, CoreError> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| CoreError::Network(e.to_string()))?;
    rt.block_on(resolve_pds(did))
}
