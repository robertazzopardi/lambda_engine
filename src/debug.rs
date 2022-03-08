use ash::{extensions::ext::DebugUtils, vk, Entry, Instance};
use std::{borrow::Cow, ffi::CStr};
use winit::window::Window;

pub struct Debug {
    pub debug_messenger: vk::DebugUtilsMessengerEXT,
    pub debug_utils: DebugUtils,
}

pub fn enable_validation_layers() -> bool {
    cfg!(debug_assertions)
}

pub fn check_validation_layer_support(_window_handle: &Window) -> bool {
    // let mut _layer_count: u32;

    true
}

/// # Safety
///
/// Expand on the safety of this function
pub unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number: i32 = callback_data.message_id_number as i32;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    println!(
        "{:?}:\n{:?} [{} ({})] : {}",
        message_severity,
        message_type,
        message_id_name,
        &message_id_number.to_string(),
        message,
    );

    vk::FALSE
}

pub fn setup_debug_messenger(instance: &Instance, entry: &Entry) -> Option<Debug> {
    if enable_validation_layers() {
        let create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::default())
            .message_type(vk::DebugUtilsMessageTypeFlagsEXT::default())
            .pfn_user_callback(Some(vulkan_debug_callback));

        let debug_utils_loader = DebugUtils::new(entry, instance);
        unsafe {
            return Some(Debug {
                debug_messenger: debug_utils_loader
                    .create_debug_utils_messenger(&create_info, None)
                    .unwrap(),
                debug_utils: debug_utils_loader,
            });
        }
    }
    None
}
