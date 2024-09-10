#![forbid(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

use anyhow::bail;
use core::slice;
use std::mem;

use metal::{Device, DeviceRef, MTLResourceOptions};

/// A vector dot product abstraction that supports CPU and Apple Silicon GPU
pub trait DotProduct {
    /// Executes the chosen dot product function on two vectors and returns a result
    fn execute(&self, v: &[u32], w: &[u32]) -> anyhow::Result<Vec<u32>>;
}

/// Creates a DotProduct for either a CPU or an Apple Silicon GPU
pub fn new(use_cpu: bool) -> Box<dyn DotProduct> {
    if use_cpu {
        Box::new(DotProductCPU::new())
    } else {
        Box::new(DotProductGPU::new())
    }
}

struct DotProductCPU {}

impl DotProductCPU {
    fn new() -> DotProductCPU {
        DotProductCPU {}
    }
}

impl DotProduct for DotProductCPU {
    fn execute(&self, v: &[u32], w: &[u32]) -> anyhow::Result<Vec<u32>> {
        println!("Using the CPU");

        let mut result = Vec::new();

        for i in 0..v.len() {
            result.push(v[i] * w[i]);
        }

        Ok(result)
    }
}

const METAL_LIB_DATA: &[u8] = include_bytes!("../target/dotprod.metallib");

struct DotProductGPU {}

impl DotProductGPU {
    fn new() -> DotProductGPU {
        DotProductGPU {}
    }
}

impl DotProduct for DotProductGPU {
    fn execute(&self, v: &[u32], w: &[u32]) -> anyhow::Result<Vec<u32>> {
        println!("Using the GPU");

        // will return a raw pointer to the result
        // the system will assign a GPU to use.
        let device = match Device::system_default() {
            Some(device) => device,
            None => bail!("No device found"),
        };
        let device: &DeviceRef = &device;

        // represents the library which contains the kernel.
        let lib = match device.new_library_with_data(METAL_LIB_DATA) {
            Ok(lib) => lib,
            Err(errstr) => bail!(errstr),
        };

        // create function pipeline.
        // this compiles the function, so a pipline can't be created in performance sensitive code.
        let function: metal::Function = match lib.get_function("dot_product", None) {
            Ok(function) => function,
            Err(errstr) => bail!(errstr),
        };

        let pipeline: metal::ComputePipelineState =
            match device.new_compute_pipeline_state_with_function(&function) {
                Ok(pipeline) => pipeline,
                Err(errstr) => bail!(errstr),
            };

        let length = v.len() as u64;
        let size = length * core::mem::size_of::<u32>() as u64;
        assert_eq!(v.len(), w.len());

        let buffer_a = device.new_buffer_with_data(
            unsafe { mem::transmute(v.as_ptr()) },
            size,
            MTLResourceOptions::StorageModeShared,
        );
        let buffer_b = device.new_buffer_with_data(
            unsafe { mem::transmute(w.as_ptr()) },
            size,
            MTLResourceOptions::StorageModeShared,
        );
        let buffer_result = device.new_buffer(
            size, // the operation will return an array with the same size.
            MTLResourceOptions::StorageModeShared,
        );

        // a command queue for sending instructions to the device.
        let command_queue = device.new_command_queue();
        // for sending commands, a command buffer is needed.
        let command_buffer = command_queue.new_command_buffer();
        // to write commands into a buffer an encoder is needed, in our case a compute encoder.
        let compute_encoder = command_buffer.new_compute_command_encoder();
        compute_encoder.set_compute_pipeline_state(&pipeline);
        compute_encoder.set_buffers(
            0,
            &[Some(&buffer_a), Some(&buffer_b), Some(&buffer_result)],
            &[0; 3],
        );

        // specify thread count and organization
        let grid_size = metal::MTLSize::new(length, 1, 1);
        let threadgroup_size = metal::MTLSize::new(length, 1, 1);
        compute_encoder.dispatch_threads(grid_size, threadgroup_size);

        // end encoding and execute commands
        compute_encoder.end_encoding();
        command_buffer.commit();

        command_buffer.wait_until_completed();

        let ptr = buffer_result.contents() as *const u32;
        let len = buffer_result.length() as usize / mem::size_of::<u32>();
        let slice = unsafe { slice::from_raw_parts(ptr, len) };
        Ok(slice.to_vec())
    }
}
