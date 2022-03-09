use crate::swapchain::SwapChain;
use ash::{vk, Device};

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct SyncObjects {
    pub image_available_semaphores: [vk::Semaphore; MAX_FRAMES_IN_FLIGHT],
    pub render_finished_semaphores: [vk::Semaphore; MAX_FRAMES_IN_FLIGHT],
    pub in_flight_fences: [vk::Fence; MAX_FRAMES_IN_FLIGHT],
    pub images_in_flight: Vec<vk::Fence>,
}

impl SyncObjects {
    pub fn new(device: &Device, _swapchain: &SwapChain) -> Self {
        let semaphore_create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            ..Default::default()
        };

        let fence_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            flags: vk::FenceCreateFlags::SIGNALED,
            ..Default::default()
        };

        let mut image_available_semaphores: [vk::Semaphore; MAX_FRAMES_IN_FLIGHT] =
            Default::default();
        let mut render_finished_semaphores: [vk::Semaphore; MAX_FRAMES_IN_FLIGHT] =
            Default::default();
        let mut in_flight_fences: [vk::Fence; MAX_FRAMES_IN_FLIGHT] = Default::default();
        let images_in_flight: Vec<vk::Fence> = [vk::Fence::null(); 3].to_vec();

        unsafe {
            for i in 0..MAX_FRAMES_IN_FLIGHT {
                image_available_semaphores[i] = device
                    .create_semaphore(&semaphore_create_info, None)
                    .unwrap();
                render_finished_semaphores[i] = device
                    .create_semaphore(&semaphore_create_info, None)
                    .unwrap();
                in_flight_fences[i] = device.create_fence(&fence_info, None).unwrap();
            }

            Self {
                image_available_semaphores,
                render_finished_semaphores,
                in_flight_fences,
                images_in_flight,
            }
        }
    }
}
