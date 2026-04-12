#[cfg(all(test, feature = "experimental"))]
mod tests {
    #[allow(unused_imports)]
    use crate::codegen::gpu_dispatch::*;
    use std::sync::Arc;

    #[test]
    fn test_software_backend_basic() {
        let backend = SoftwareBackend::new();
        assert_eq!(backend.backend_type(), GpuBackendType::Software);

        // 1. Allocation
        let size = 1024;
        let ptr = backend.alloc(size).expect("Allocation failed");
        assert!(ptr > 0);

        // 2. Host to Device Copy
        let mut host_data = vec![0u8; size];
        for i in 0..size {
            host_data[i] = (i % 255) as u8;
        }

        backend
            .memcpy_h2d(ptr, &host_data)
            .expect("H2D copy failed");

        // 3. Device to Host Copy
        let device_data = backend.memcpy_d2h(ptr, size).expect("D2H copy failed");
        assert_eq!(host_data, device_data);

        // 4. Kernel Launch
        let grid = [1, 1, 1];
        let block = [1, 1, 1];
        let launch_res = backend.launch_kernel("test_kernel", grid, block, 0, &[]);
        assert!(launch_res.is_ok());

        // 5. Free
        backend.free(ptr).expect("Free failed");
    }

    #[test]
    fn test_device_manager_fallback() {
        let mut manager = DeviceManager::new();
        // Should find SoftwareBackend as fallback
        let devices = manager.devices();
        assert!(!devices.is_empty());

        let software_dev = devices
            .iter()
            .find(|d| d.backend == GpuBackendType::Software);
        assert!(software_dev.is_some(), "Software fallback device not found");

        // Select it
        if let Some(pos) = devices
            .iter()
            .position(|d| d.backend == GpuBackendType::Software)
        {
            manager.select_device(pos).unwrap();
            let ctx = manager
                .context()
                .expect("Context not created for software device");
            assert_eq!(ctx.backend_type(), GpuBackendType::Software);
        }
    }

    #[test]
    fn test_cuda_backend_probe() {
        // This test attempts to load the CUDA driver.
        // It should not panic even if CUDA is missing.
        match CudaBackend::try_new() {
            Ok(backend) => {
                println!("CUDA driver found!");
                assert_eq!(backend.backend_type(), GpuBackendType::Cuda);
            }
            Err(e) => {
                println!("CUDA driver not found (expected in CI/CPU-only env): {}", e);
            }
        }
    }
}
