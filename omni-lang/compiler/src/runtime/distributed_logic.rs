//! Advanced Distributed Logic
//!
//! ZeRO Optimizer, Gradient Bucketing, and Topology Discovery.
#![allow(dead_code)]

use std::collections::HashMap;

/// ZeRO (Zero Redundancy Optimizer) State Sharding
pub struct ZeroOptimizer {
    pub world_size: usize,
    pub rank: usize,
    pub partition_size: usize,
}

impl ZeroOptimizer {
    /// Shard optimizer state (momentum, variance) across ranks
    pub fn shard_state<T>(&self, full_param: &[T]) -> (Vec<T>, usize, usize)
    where
        T: Clone + Default,
    {
        let total_params = full_param.len();
        let chunk_size = (total_params + self.world_size - 1) / self.world_size;

        let start_idx = self.rank * chunk_size;
        let end_idx = (start_idx + chunk_size).min(total_params);

        if start_idx >= total_params {
            return (Vec::new(), 0, 0);
        }

        let local_shard = full_param[start_idx..end_idx].to_vec();
        (local_shard, start_idx, end_idx)
    }

    /// Gather full weights from all ranks (AllGather)
    /// In a single-node scenario, returns a copy of the local shard.
    /// In a distributed setting, would perform NCCL AllGather across ranks.
    pub fn gather_weights<T>(&self, local_shard: &[T]) -> Vec<T>
    where
        T: Clone + Default,
    {
        if self.world_size <= 1 {
            // Single rank - just return the local shard as the full tensor
            return local_shard.to_vec();
        }

        // Simulate AllGather by creating a buffer for all ranks
        // In real impl: ncclAllGather(local_shard, full_buffer, count, datatype, comm, stream)
        let chunk_size = local_shard.len();
        let total_size = chunk_size * self.world_size;
        let mut full_buffer = vec![T::default(); total_size];

        // Place our shard at the correct offset
        let offset = self.rank * chunk_size;
        for (i, val) in local_shard.iter().enumerate() {
            if offset + i < full_buffer.len() {
                full_buffer[offset + i] = val.clone();
            }
        }

        // In a real distributed runtime, the other ranks' data would be
        // filled in by the NCCL AllGather collective
        log::debug!(
            "ZeRO: AllGather rank {} contributing {} elements at offset {}",
            self.rank,
            chunk_size,
            offset
        );

        full_buffer
    }

    /// Reduce-scatter gradients across ranks
    pub fn reduce_scatter_gradients(&self, gradients: &[f32]) -> Vec<f32> {
        if self.world_size <= 1 {
            return gradients.to_vec();
        }

        let chunk_size = (gradients.len() + self.world_size - 1) / self.world_size;
        let start = self.rank * chunk_size;
        let end = (start + chunk_size).min(gradients.len());

        // In real impl: ncclReduceScatter with sum operation
        // Returns only this rank's chunk of the reduced gradients
        if start < gradients.len() {
            gradients[start..end].to_vec()
        } else {
            Vec::new()
        }
    }
}

/// Gradient Bucketing for PCIe saturation
pub struct GradientBucketeer {
    pub bucket_size_mb: usize,
    pub(crate) buckets: Vec<Bucket>,
    current_bucket: Bucket,
}

pub(crate) struct Bucket {
    pub params: Vec<String>, // Parameter names
    pub size_bytes: usize,
    pub data: Vec<f32>, // Flattened gradient buffer
}

impl GradientBucketeer {
    pub fn new(bucket_size_mb: usize) -> Self {
        Self {
            bucket_size_mb,
            buckets: Vec::new(),
            current_bucket: Bucket {
                params: Vec::new(),
                size_bytes: 0,
                data: Vec::new(),
            },
        }
    }

    pub fn add_param(&mut self, name: &str, size_bytes: usize) {
        if self.current_bucket.size_bytes + size_bytes > self.bucket_size_mb * 1024 * 1024 {
            // Flush bucket
            let full_bucket = std::mem::replace(
                &mut self.current_bucket,
                Bucket {
                    params: Vec::new(),
                    size_bytes: 0,
                    data: Vec::new(),
                },
            );
            self.buckets.push(full_bucket);
        }

        self.current_bucket.params.push(name.to_string());
        self.current_bucket.size_bytes += size_bytes;
    }

    pub fn flush(&mut self) {
        if !self.current_bucket.params.is_empty() {
            let full_bucket = std::mem::replace(
                &mut self.current_bucket,
                Bucket {
                    params: Vec::new(),
                    size_bytes: 0,
                    data: Vec::new(),
                },
            );
            self.buckets.push(full_bucket);
        }
    }
}

/// GPU Topology Discovery
pub struct TopologyDiscovery;

#[derive(Debug)]
pub enum LinkType {
    PCIe,
    NVLink,
    NVSwitch,
    QPI, // CPU-CPU
}

impl TopologyDiscovery {
    /// Discover GPU topology using system information
    /// Falls back to heuristic-based topology if NVML is not available
    pub fn discover() -> HashMap<(usize, usize), LinkType> {
        let mut map = HashMap::new();

        let gpu_count = Self::detect_gpu_count();

        if gpu_count == 0 {
            log::info!("TopologyDiscovery: No GPUs detected");
            return map;
        }

        log::info!("TopologyDiscovery: Detected {} GPUs", gpu_count);

        // Try to read topology from sysfs (Linux)
        #[cfg(target_os = "linux")]
        {
            if let Some(topology) = Self::read_sysfs_topology(gpu_count) {
                return topology;
            }
        }

        // Heuristic: GPUs on same NUMA node get NVLink, cross-NUMA get PCIe
        let gpus_per_socket = (gpu_count + 1) / 2;

        for i in 0..gpu_count {
            for j in 0..gpu_count {
                if i == j {
                    continue;
                }
                let link = if (i / gpus_per_socket) == (j / gpus_per_socket) {
                    LinkType::NVLink // Same socket/switch domain
                } else {
                    LinkType::PCIe // Cross-socket
                };
                map.insert((i, j), link);
            }
        }

        map
    }

    /// Detect the number of GPUs available
    fn detect_gpu_count() -> usize {
        // Check CUDA_VISIBLE_DEVICES
        if let Ok(devices) = std::env::var("CUDA_VISIBLE_DEVICES") {
            if devices.is_empty() {
                return 0;
            }
            return devices.split(',').count();
        }

        // Check for NVIDIA devices on Linux
        #[cfg(target_os = "linux")]
        {
            let mut count = 0;
            for i in 0..16 {
                if std::path::Path::new(&format!("/dev/nvidia{}", i)).exists() {
                    count += 1;
                }
            }
            if count > 0 {
                return count;
            }
        }

        // Check for ROCm devices
        if let Ok(devices) = std::env::var("HIP_VISIBLE_DEVICES") {
            return devices.split(',').count();
        }

        0
    }

    #[cfg(target_os = "linux")]
    fn read_sysfs_topology(gpu_count: usize) -> Option<HashMap<(usize, usize), LinkType>> {
        // Try to read /sys/bus/pci/drivers/nvidia/*/numa_node for NUMA topology
        let mut numa_nodes: Vec<Option<usize>> = vec![None; gpu_count];

        for i in 0..gpu_count {
            let path = format!("/proc/driver/nvidia/gpus/{}/information", i);
            if let Ok(content) = std::fs::read_to_string(&path) {
                // Parse NUMA node from driver info
                for line in content.lines() {
                    if line.contains("NUMA") {
                        if let Some(node_str) = line.split(':').last() {
                            if let Ok(node) = node_str.trim().parse::<usize>() {
                                numa_nodes[i] = Some(node);
                            }
                        }
                    }
                }
            }
        }

        // If we got any NUMA info, build topology
        if numa_nodes.iter().any(|n| n.is_some()) {
            let mut map = HashMap::new();
            for i in 0..gpu_count {
                for j in 0..gpu_count {
                    if i == j {
                        continue;
                    }
                    let link = match (numa_nodes[i], numa_nodes[j]) {
                        (Some(a), Some(b)) if a == b => LinkType::NVLink,
                        _ => LinkType::PCIe,
                    };
                    map.insert((i, j), link);
                }
            }
            return Some(map);
        }

        None
    }

    /// Get the optimal communication ring order
    pub fn optimal_ring_order(
        topology: &HashMap<(usize, usize), LinkType>,
        gpu_count: usize,
    ) -> Vec<usize> {
        // Greedy: prefer NVLink connections for ring neighbors
        let mut ring = vec![0usize];
        let mut visited = vec![false; gpu_count];
        visited[0] = true;

        for _ in 1..gpu_count {
            let last = *ring.last().unwrap();
            let mut best = None;
            let mut best_is_nvlink = false;

            for next in 0..gpu_count {
                if visited[next] {
                    continue;
                }
                let is_nvlink = matches!(
                    topology.get(&(last, next)),
                    Some(LinkType::NVLink) | Some(LinkType::NVSwitch)
                );
                if best.is_none() || (is_nvlink && !best_is_nvlink) {
                    best = Some(next);
                    best_is_nvlink = is_nvlink;
                }
            }

            if let Some(next) = best {
                ring.push(next);
                visited[next] = true;
            }
        }

        ring
    }
}
