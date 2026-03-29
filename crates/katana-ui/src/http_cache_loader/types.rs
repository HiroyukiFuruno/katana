use serde::{Deserialize, Serialize};
use std::{sync::Arc, task::Poll};

#[derive(Clone)]
pub(crate) struct CachedFile {
    pub(crate) bytes: Arc<[u8]>,
    pub(crate) mime: Option<String>,
}

impl CachedFile {
    pub(crate) fn from_response(uri: &str, response: ehttp::Response) -> Result<Self, String> {
        if !response.ok {
            match response.text() {
                Some(response_text) => Err(format!(
                    "failed to load {uri:?}: {} {} {response_text}",
                    response.status, response.status_text
                )),
                None => Err(format!(
                    "failed to load {uri:?}: {} {}",
                    response.status, response.status_text
                )),
            }
        } else {
            let mime = response.content_type().map(ToOwned::to_owned);
            let bytes = response.bytes.into();
            Ok(Self { bytes, mime })
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct CacheMetadata {
    pub(crate) mime: Option<String>,
}

pub(crate) type Entry = Poll<Result<CachedFile, String>>;

pub(crate) struct HttpCacheEntry {
    pub(crate) uri: String,
    pub(crate) entry: Entry,
}
