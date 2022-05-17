use crate::utility::EntryInstance;
use ash::{extensions::ext::DebugUtils, vk};
use derive_new::new;
use std::{borrow::Cow, ffi::CStr};
use winit::window::Window;

#[derive(Clone)]
pub(crate) struct Debug {
    pub messenger: vk::DebugUtilsMessengerEXT,
    pub utils: DebugUtils,
}

pub(crate) const fn enable_validation_layers() -> bool {
    cfg!(debug_assertions)
}

pub(crate) const fn check_validation_layer_support(_window_handle: &Window) -> bool {
    // let mut _layer_count: u32;

    true
}

/// # Safety
///
/// Expand on the safety of this function
unsafe extern "system" fn vulkan_debug_callback(
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

/// ## VkDebugUtilsMessageSeverityFlagBitsEXT
#[derive(Default, Clone, Copy, Debug)]
pub struct MessageLevel {
    flags: vk::DebugUtilsMessageSeverityFlagsEXT,
}

impl MessageLevel {
    /// ## MessageLevel Builder
    ///
    /// Info message severity enabled by default
    pub fn builder() -> Self {
        Self {
            flags: vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
        }
    }

    /// Enables error messages
    pub fn error(mut self) -> Self {
        self.flags |= vk::DebugUtilsMessageSeverityFlagsEXT::ERROR;
        self
    }

    /// Enables verbose messages
    pub fn verbose(mut self) -> Self {
        self.flags |= vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE;
        self
    }

    /// Enables warning messages
    pub fn warning(mut self) -> Self {
        self.flags |= vk::DebugUtilsMessageSeverityFlagsEXT::WARNING;
        self
    }
}

/// ## VkDebugUtilsMessageTypeFlagBitsEXT
#[derive(Default, Clone, Copy, Debug)]
pub struct MessageType {
    flags: vk::DebugUtilsMessageTypeFlagsEXT,
}

impl MessageType {
    /// ## MessageType Builder
    ///
    /// General type message enabled by default
    pub fn builder() -> Self {
        Self {
            flags: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL,
        }
    }

    /// Enables validation type messages
    pub fn validation(mut self) -> Self {
        self.flags |= vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION;
        self
    }

    /// Enables performance type messages
    pub fn performance(mut self) -> Self {
        self.flags |= vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE;
        self
    }
}

#[derive(new, Clone, Copy, Debug)]
pub struct DebugMessageProperties {
    pub message_level: MessageLevel,
    pub message_type: MessageType,
}

pub(crate) fn debugger(
    EntryInstance { entry, instance }: &EntryInstance,
    DebugMessageProperties {
        message_level,
        message_type,
    }: DebugMessageProperties,
) -> Debug {
    let create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(message_level.flags)
        .message_type(message_type.flags)
        .pfn_user_callback(Some(vulkan_debug_callback));

    let utils = DebugUtils::new(entry, instance);

    let messenger = unsafe {
        utils
            .create_debug_utils_messenger(&create_info, None)
            .unwrap()
    };

    Debug { messenger, utils }
}
