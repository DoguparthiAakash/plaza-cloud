use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use plaza_foundation::core::{PlazaResult, PlazaError};
use serde::{Serialize, Deserialize};

/// Supported cloud providers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CloudProvider {
    Aws,
    Azure,
    Gcp,
    DigitalOcean,
    Custom(String),
}

/// Cloud region.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudRegion {
    pub provider: CloudProvider,
    pub region_id: String,
    pub display_name: String,
    pub available: bool,
}

/// Credentials for authenticating with a cloud provider.
#[derive(Debug, Clone)]
pub struct CloudCredentials {
    pub provider: CloudProvider,
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
    pub endpoint: Option<String>,
}

/// Represents a remote VM instance in the cloud.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudInstance {
    pub id: String,
    pub provider: CloudProvider,
    pub region: String,
    pub instance_type: String,
    pub status: CloudInstanceStatus,
    pub public_ip: Option<String>,
    pub private_ip: Option<String>,
    pub created_at: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudInstanceStatus {
    Pending,
    Running,
    Stopping,
    Stopped,
    Terminated,
    Error,
}

/// Abstract cloud provider interface.
#[async_trait]
pub trait CloudProviderApi: Send + Sync {
    async fn launch_instance(&self, config: &LaunchConfig) -> PlazaResult<CloudInstance>;
    async fn terminate_instance(&self, instance_id: &str) -> PlazaResult<()>;
    async fn get_instance(&self, instance_id: &str) -> PlazaResult<CloudInstance>;
    async fn list_instances(&self) -> PlazaResult<Vec<CloudInstance>>;
    async fn list_regions(&self) -> PlazaResult<Vec<CloudRegion>>;
}

/// Configuration for launching a cloud instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchConfig {
    pub name: String,
    pub instance_type: String,
    pub region: String,
    pub image_id: String,
    pub ssh_key_name: Option<String>,
    pub tags: HashMap<String, String>,
}

/// Simulated cloud provider for testing without real cloud accounts.
pub struct SimulatedProvider {
    instances: RwLock<HashMap<String, CloudInstance>>,
}

impl SimulatedProvider {
    pub fn new() -> Self {
        Self {
            instances: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl CloudProviderApi for SimulatedProvider {
    async fn launch_instance(&self, config: &LaunchConfig) -> PlazaResult<CloudInstance> {
        let instance = CloudInstance {
            id: uuid::Uuid::new_v4().to_string(),
            provider: CloudProvider::Custom("simulated".into()),
            region: config.region.clone(),
            instance_type: config.instance_type.clone(),
            status: CloudInstanceStatus::Running,
            public_ip: Some(format!("203.0.113.{}", rand_byte())),
            private_ip: Some(format!("10.0.0.{}", rand_byte())),
            created_at: chrono::Utc::now().to_rfc3339(),
            metadata: config.tags.clone(),
        };
        let mut store = self.instances.write().await;
        store.insert(instance.id.clone(), instance.clone());
        Ok(instance)
    }

    async fn terminate_instance(&self, instance_id: &str) -> PlazaResult<()> {
        let mut store = self.instances.write().await;
        let inst = store.get_mut(instance_id)
            .ok_or_else(|| PlazaError::NotFound(format!("Instance {}", instance_id)))?;
        inst.status = CloudInstanceStatus::Terminated;
        Ok(())
    }

    async fn get_instance(&self, instance_id: &str) -> PlazaResult<CloudInstance> {
        let store = self.instances.read().await;
        store.get(instance_id).cloned()
            .ok_or_else(|| PlazaError::NotFound(format!("Instance {}", instance_id)))
    }

    async fn list_instances(&self) -> PlazaResult<Vec<CloudInstance>> {
        let store = self.instances.read().await;
        Ok(store.values().cloned().collect())
    }

    async fn list_regions(&self) -> PlazaResult<Vec<CloudRegion>> {
        Ok(vec![
            CloudRegion { provider: CloudProvider::Custom("simulated".into()), region_id: "sim-east-1".into(), display_name: "Simulated East".into(), available: true },
            CloudRegion { provider: CloudProvider::Custom("simulated".into()), region_id: "sim-west-1".into(), display_name: "Simulated West".into(), available: true },
        ])
    }
}

fn rand_byte() -> u8 {
    (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().subsec_nanos() % 254 + 1) as u8
}
