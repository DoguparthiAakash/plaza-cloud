//! # plaza-cloud
//!
//! Cloud provider orchestration for PlazaVM.
//!
//! Provides:
//! - **Provider API**: Abstract multi-cloud interface (AWS, Azure, GCP, custom)
//! - **Volumes**: Cloud-attached block storage with lifecycle management
//! - **Metadata**: Instance metadata service (IMDS-compatible)
//! - **Bootstrap**: Composition root for cloud subsystem

pub mod api;
pub mod volume;
pub mod metadata;
pub mod bootstrap;

pub use api::{CloudProvider, CloudInstance, CloudInstanceStatus, CloudProviderApi, SimulatedProvider, LaunchConfig};
pub use volume::{VolumeManager, CloudVolume, VolumeType, VolumeStatus};
pub use metadata::{MetadataServer, InstanceMetadata};
pub use bootstrap::CloudService;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cloud_instance_lifecycle() {
        let svc = CloudService::simulated();
        let config = LaunchConfig {
            name: "test-vm".into(),
            instance_type: "t3.micro".into(),
            region: "sim-east-1".into(),
            image_id: "ami-12345".into(),
            ssh_key_name: None,
            tags: Default::default(),
        };

        let instance = svc.provider.launch_instance(&config).await.unwrap();
        assert_eq!(instance.status, CloudInstanceStatus::Running);
        assert!(instance.public_ip.is_some());

        let fetched = svc.provider.get_instance(&instance.id).await.unwrap();
        assert_eq!(fetched.id, instance.id);

        svc.provider.terminate_instance(&instance.id).await.unwrap();
        let terminated = svc.provider.get_instance(&instance.id).await.unwrap();
        assert_eq!(terminated.status, CloudInstanceStatus::Terminated);
    }

    #[tokio::test]
    async fn test_volume_lifecycle() {
        let svc = CloudService::simulated();

        let vol = svc.volumes.create_volume("data-vol", 100, VolumeType::Ssd, true).await.unwrap();
        assert_eq!(vol.status, VolumeStatus::Available);
        assert!(vol.encrypted);

        svc.volumes.attach_volume(&vol.id, "instance-1").await.unwrap();
        let vols = svc.volumes.list_volumes().await;
        assert_eq!(vols[0].status, VolumeStatus::InUse);

        svc.volumes.detach_volume(&vol.id).await.unwrap();
        svc.volumes.resize_volume(&vol.id, 200).await.unwrap();
        svc.volumes.delete_volume(&vol.id).await.unwrap();
        assert!(svc.volumes.list_volumes().await.is_empty());
    }
}
