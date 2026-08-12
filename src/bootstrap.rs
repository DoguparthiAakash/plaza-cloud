use std::sync::Arc;
use crate::api::{CloudProviderApi, SimulatedProvider};
use crate::volume::VolumeManager;
use crate::metadata::MetadataServer;

/// Cloud subsystem bootstrap — wires together all cloud components.
pub struct CloudService {
    pub provider: Arc<dyn CloudProviderApi>,
    pub volumes: Arc<VolumeManager>,
    pub metadata: Arc<tokio::sync::RwLock<MetadataServer>>,
}

impl CloudService {
    /// Create a cloud service with the simulated provider (for dev/test).
    pub fn simulated() -> Self {
        Self {
            provider: Arc::new(SimulatedProvider::new()),
            volumes: Arc::new(VolumeManager::new()),
            metadata: Arc::new(tokio::sync::RwLock::new(MetadataServer::new())),
        }
    }

    /// Create a cloud service with a custom provider implementation.
    pub fn with_provider(provider: Arc<dyn CloudProviderApi>) -> Self {
        Self {
            provider,
            volumes: Arc::new(VolumeManager::new()),
            metadata: Arc::new(tokio::sync::RwLock::new(MetadataServer::new())),
        }
    }
}
