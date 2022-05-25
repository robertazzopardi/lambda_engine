use crate::time::Time;
use lambda_camera::camera::Camera;
use lambda_geometry::{GeomBehavior, Geometries};
use lambda_vulkan::{
    command_buffer::{self, VkCommander},
    create_surface,
    debug::{self, DebugMessageProperties},
    device::{self, Devices},
    frame_buffer, renderer,
    resource::Resources,
    swap_chain::{self, SwapChain},
    sync_objects::{SyncObjects, MAX_FRAMES_IN_FLIGHT},
    uniform_buffer::UniformBufferObject,
    utility::{EntryInstance, InstanceDevices},
    Vulkan, WindowSize,
};
use lambda_window::window::{self, Display};
use winit::platform::run_return::EventLoopExtRunReturn;

pub struct Engine {
    vulkan: Vulkan,
    current_frame: usize,
    is_frame_buffer_resized: bool,
    models: Geometries,
    time: Time,
}

impl Engine {
    pub fn new(
        display: &Display,
        camera: &mut Camera,
        models: Geometries,
        debugging: Option<DebugMessageProperties>,
    ) -> Self {
        let entry_instance = EntryInstance::new(&display.window);

        let debugger =
            debugging.map(|debug_properties| debug::debugger(&entry_instance, debug_properties));

        let surface = lambda_window::create_surface(
            &display.window,
            &entry_instance.instance,
            &entry_instance.entry,
        );

        let surface_loader = create_surface(&entry_instance);

        let devices = Devices::new(&entry_instance.instance, &surface, &surface_loader);

        let instance_devices = InstanceDevices::new(entry_instance.instance, devices);

        let swap_chain =
            SwapChain::new(&instance_devices, surface, &surface_loader, &display.window);

        let render_pass = renderer::create_render_pass(&instance_devices, &swap_chain);

        let resources = Resources::new(&swap_chain, &instance_devices);

        let frame_buffers = frame_buffer::create_frame_buffers(
            &swap_chain,
            &render_pass,
            &instance_devices,
            &resources,
        );

        let command_pool =
            command_buffer::create_command_pool(&instance_devices, &surface_loader, &surface);

        let sync_objects = SyncObjects::new(&instance_devices);

        let swap_chain_len = swap_chain.images.len() as u32;

        let mut models = models;

        let mut vulkan_objects = Vec::new();
        models.iter_mut().for_each(|property| {
            property.defer_build(
                &command_pool,
                swap_chain_len,
                &swap_chain,
                &render_pass,
                &instance_devices,
            );
            vulkan_objects.push(property.vulkan_object());
        });

        let command_buffers = command_buffer::create_command_buffers(
            &command_pool,
            &swap_chain,
            &instance_devices,
            &render_pass,
            &frame_buffers,
            &vulkan_objects,
        );

        let commander = VkCommander::new(command_buffers, command_pool);

        let ubo = UniformBufferObject::new(&swap_chain.extent, camera);

        let time = Time::new(60.);

        let vulkan = Vulkan {
            commander,
            render_pass,
            resources,
            surface,
            surface_loader,
            swap_chain,
            sync_objects,
            ubo,
            debugger,
            frame_buffers,
            instance_devices,
        };

        Self {
            vulkan,
            current_frame: 0,
            is_frame_buffer_resized: false,
            models,
            time,
        }
    }

    pub fn run(&mut self, display: &mut Display, mut camera: Camera) {
        let mut mouse_pressed = false;

        display.event_loop.run_return(|event, _, control_flow| {
            self.time.tick();

            window::handle_inputs(
                control_flow,
                event,
                &display.window,
                &mut camera,
                &mut mouse_pressed,
            );

            self.time.step(
                &mut camera,
                &mut self.vulkan.ubo,
                &WindowSize(self.vulkan.swap_chain.extent),
            );

            // not efficient but for now works
            let mut vulkan_objects = Vec::new();
            self.models
                .iter()
                .for_each(|model| vulkan_objects.push(model.vulkan_object()));

            unsafe {
                renderer::render(
                    &mut self.vulkan,
                    &display.window,
                    &mut camera,
                    &mut self.current_frame,
                    &mut self.is_frame_buffer_resized,
                    &vulkan_objects,
                )
            };
        });
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        let device = &self.vulkan.instance_devices.devices.logical.device;

        unsafe {
            device.device_wait_idle().unwrap();

            let mut vulkan_objects = Vec::new();
            self.models
                .iter()
                .for_each(|model| vulkan_objects.push(model.vulkan_object()));

            swap_chain::cleanup_swap_chain(&self.vulkan, &vulkan_objects);

            self.models.iter().for_each(|model| {
                device::destroy(model.vulkan_object(), device);
            });

            for i in 0..MAX_FRAMES_IN_FLIGHT {
                device.destroy_semaphore(
                    self.vulkan.sync_objects.image_available_semaphores[i],
                    None,
                );
                device.destroy_semaphore(
                    self.vulkan.sync_objects.render_finished_semaphores[i],
                    None,
                );
                device.destroy_fence(self.vulkan.sync_objects.in_flight_fences[i], None);
            }

            device.destroy_command_pool(*self.vulkan.commander.pool, None);

            println!("here");

            device.destroy_device(None);

            if debug::enable_validation_layers() {
                if let Some(debugger) = &self.vulkan.debugger {
                    debugger
                        .utils
                        .destroy_debug_utils_messenger(debugger.messenger, None);
                }
            }

            self.vulkan
                .surface_loader
                .destroy_surface(self.vulkan.surface, None);

            self.vulkan.instance_devices.instance.destroy_instance(None);
        }
    }
}
