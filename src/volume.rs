use std::collections::HashMap;
use tokio::sync::RwLock;
use plaza_foundation::core::{PlazaResult, PlazaError};
use serde::{Serialize, Deserialize};

/// Represents a cloud-attached block volume.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudVolume {
    pub id: String,
    pub name: String,
    pub size_gb: u64,
    pub volume_type: VolumeType,
    pub attached_to: Option<String>,
    pub status: VolumeStatus,
    pub encrypted: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VolumeType {
    Ssd,
    Hdd,
    NvMe,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VolumeStatus {
    Available,
    InUse,
    Detaching,
    Deleting,
    Error,
}

/// Cloud volume manager for persistent storage across deployments.
pub struct VolumeManager {
    volumes: RwLock<HashMap<String, CloudVolume>>,
}

impl VolumeManager {
    pub fn new() -> Self {
        Self { volumes: RwLock::new(HashMap::new()) }
    }

    pub async fn create_volume(&self, name: &str, size_gb: u64, volume_type: VolumeType, encrypted: bool) -> PlazaResult<CloudVolume> {
        let vol = CloudVolume {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            size_gb,
            volume_type,
            attached_to: None,
            status: VolumeStatus::Available,
            encrypted,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        let mut store = self.volumes.write().await;
        store.insert(vol.id.clone(), vol.clone());
        Ok(vol)
    }

    pub async fn attach_volume(&self, volume_id: &str, instance_id: &str) -> PlazaResult<()> {
        let mut store = self.volumes.write().await;
        let vol = store.get_mut(volume_id).ok_or_else(|| PlazaError::NotFound(format!("Volume {}", volume_id)))?;
        if vol.status != VolumeStatus::Available {
            return Err(PlazaError::Internal("Volume is not available for attachment".into()));
        }
        vol.attached_to = Some(instance_id.to_string());
        vol.status = VolumeStatus::InUse;
        Ok(())
    }

    pub async fn detach_volume(&self, volume_id: &str) -> PlazaResult<()> {
        let mut store = self.volumes.write().await;
        let vol = store.get_mut(volume_id).ok_or_else(|| PlazaError::NotFound(format!("Volume {}", volume_id)))?;
        vol.attached_to = None;
        vol.status = VolumeStatus::Available;
        Ok(())
    }

    pub async fn delete_volume(&self, volume_id: &str) -> PlazaResult<()> {
        let mut store = self.volumes.write().await;
        let vol = store.get(volume_id).ok_or_else(|| PlazaError::NotFound(format!("Volume {}", volume_id)))?;
        if vol.status == VolumeStatus::InUse {
            return Err(PlazaError::Internal("Cannot delete volume that is in use".into()));
        }
        store.remove(volume_id);
        Ok(())
    }

    pub async fn list_volumes(&self) -> Vec<CloudVolume> {
        self.volumes.read().await.values().cloned().collect()
    }

    pub async fn resize_volume(&self, volume_id: &str, new_size_gb: u64) -> PlazaResult<()> {
        let mut store = self.volumes.write().await;
        let vol = store.get_mut(volume_id).ok_or_else(|| PlazaError::NotFound(format!("Volume {}", volume_id)))?;
        if new_size_gb < vol.size_gb {
            return Err(PlazaError::Internal("Cannot shrink volume".into()));
        }
        vol.size_gb = new_size_gb;
        Ok(())
    }
}
