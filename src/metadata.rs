use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Instance metadata service — provides cloud metadata to running VMs
/// (similar to AWS IMDS / GCP metadata server / Azure IMDS).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceMetadata {
    pub instance_id: String,
    pub hostname: String,
    pub region: String,
    pub availability_zone: String,
    pub instance_type: String,
    pub public_ipv4: Option<String>,
    pub private_ipv4: Option<String>,
    pub tags: HashMap<String, String>,
    pub user_data: Option<String>,
}

/// Metadata server that VM instances can query for self-identification.
pub struct MetadataServer {
    metadata: HashMap<String, InstanceMetadata>,
}

impl MetadataServer {
    pub fn new() -> Self {
        Self { metadata: HashMap::new() }
    }

    pub fn register(&mut self, instance_id: &str, meta: InstanceMetadata) {
        self.metadata.insert(instance_id.to_string(), meta);
    }

    pub fn get(&self, instance_id: &str) -> Option<&InstanceMetadata> {
        self.metadata.get(instance_id)
    }

    pub fn get_tag(&self, instance_id: &str, key: &str) -> Option<String> {
        self.metadata.get(instance_id)
            .and_then(|m| m.tags.get(key).cloned())
    }

    pub fn get_user_data(&self, instance_id: &str) -> Option<String> {
        self.metadata.get(instance_id)
            .and_then(|m| m.user_data.clone())
    }

    pub fn unregister(&mut self, instance_id: &str) {
        self.metadata.remove(instance_id);
    }

    /// Generate metadata JSON response (mimics cloud IMDS endpoint format).
    pub fn to_json(&self, instance_id: &str) -> Option<String> {
        self.metadata.get(instance_id)
            .map(|m| serde_json::to_string_pretty(m).unwrap_or_default())
    }
}
