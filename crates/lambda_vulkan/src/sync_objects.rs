use ash::{vk, Device};

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub type ImageSemaphoreArray = [vk::Semaphore; MAX_FRAMES_IN_FLIGHT];
pub type FenceArray = [vk::Fence; MAX_FRAMES_IN_FLIGHT];

#[derive(Clone)]
pub struct SyncObjects {
    pub image_available_semaphores: ImageSemaphoreArray,
    pub render_finished_semaphores: ImageSemaphoreArray,
    pub in_flight_fences: FenceArray,
    pub images_in_flight: Vec<vk::Fence>,
}

impl SyncObjects {
    pub fn new(device: &Device) -> Self {
        let semaphore_create_info = vk::SemaphoreCreateInfo::default();

        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        let mut image_available_semaphores: ImageSemaphoreArray = Default::default();
        let mut render_finished_semaphores: ImageSemaphoreArray = Default::default();
        let mut in_flight_fences: FenceArray = Default::default();
        let images_in_flight: Vec<vk::Fence> = vec![vk::Fence::null(); 3];

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
        }

        Self {
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            images_in_flight,
        }
    }
}
