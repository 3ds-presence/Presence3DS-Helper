use ctru_sys::{FS_ArchiveResource, FSUSER_GetSdmcArchiveResource, fsExit, fsInit};
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct SdCardInfo {
    pub sector_size: u32,
    pub cluster_size: u32,
    pub total_clusters: u32,
    pub free_clusters: u32,
}

impl SdCardInfo {
    pub fn query() -> Option<Self> {
        if unsafe { fsInit() } < 0 {
            return None;
        }

        let mut resource: FS_ArchiveResource = unsafe { std::mem::zeroed() };
        let result = unsafe { FSUSER_GetSdmcArchiveResource(&raw mut resource) };

        unsafe { fsExit() };

        if result < 0 {
            return None;
        }

        Some(Self {
            sector_size: resource.sectorSize,
            cluster_size: resource.clusterSize,
            total_clusters: resource.totalClusters,
            free_clusters: resource.freeClusters,
        })
    }

    pub const fn sectors_per_cluster(&self) -> u32 {
        if self.sector_size == 0 {
            return 0;
        }

        self.cluster_size / self.sector_size
    }

    pub const fn total_size(&self) -> u64 {
        self.cluster_size as u64 * self.total_clusters as u64
    }

    pub const fn free_size(&self) -> usize {
        self.cluster_size as usize * self.free_clusters as usize
    }
}
