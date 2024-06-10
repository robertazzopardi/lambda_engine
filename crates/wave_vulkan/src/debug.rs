use crate::utility::EntryInstance;
use ash::{ext::debug_utils, vk, Entry};
use std::{borrow::Cow, ffi::CStr};

pub(crate) const ENABLE_VALIDATION_LAYERS: bool = cfg!(debug_assertions);
pub(crate) const VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];

#[derive(Clone)]
pub(crate) struct Debug {
    pub messenger: vk::DebugUtilsMessengerEXT,
    pub utils: debug_utils::Instance,
}

pub(crate) fn check_validation_layer_support(entry: &Entry) -> bool {
    unsafe {
        let mut layer_properties = entry
            .enumerate_instance_layer_properties()
            .expect("Could not enumerate instance layer properties");

        for validation_layer in VALIDATION_LAYERS.into_iter() {
            let mut found_layer = false;

            for property in layer_properties.iter_mut() {
                let terminated_string =
                    String::from_utf8(property.layer_name.into_iter().map(|b| b as u8).collect())
                        .unwrap();
                let property_layer = terminated_string.trim_matches(char::from(0));

                if validation_layer == property_layer {
                    found_layer = true;
                    break;
                }
            }

            if !found_layer {
                return false;
            }
        }
    }

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

/// VkDebugUtilsMessageSeverityFlagBitsEXT
#[derive(Debug, Clone, Copy)]
pub struct MessageLevel {
    pub flags: vk::DebugUtilsMessageSeverityFlagsEXT,
}

impl MessageLevel {
    /// MessageLevel default
    ///
    /// Info message severity enabled by default
    pub const fn default() -> Self {
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

    pub fn all() -> Self {
        Self::default().error().verbose().warning()
    }
}

impl Default for MessageLevel {
    fn default() -> Self {
        Self::default().error().verbose().warning()
    }
}

/// VkDebugUtilsMessageTypeFlagBitsEXT
#[derive(Debug, Clone, Copy)]
pub struct MessageType {
    pub flags: vk::DebugUtilsMessageTypeFlagsEXT,
}

impl MessageType {
    /// MessageType default
    ///
    /// General type message enabled by default
    pub const fn default() -> Self {
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

    pub fn all() -> Self {
        Self::default().validation().performance()
    }
}

impl Default for MessageType {
    fn default() -> Self {
        Self::default().validation().performance()
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Debugger {
    pub message_level: MessageLevel,
    pub message_type: MessageType,
}

impl Debugger {
    pub fn new(message_level: MessageLevel, message_type: MessageType) -> Self {
        Self {
            message_level,
            message_type,
        }
    }

    pub fn all() -> Self {
        Self::new(MessageLevel::all(), MessageType::all())
    }
}

pub(crate) fn debugger(
    EntryInstance { entry, instance }: &EntryInstance,
    debug_properties: Debugger,
) -> Debug {
    let create_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
        .message_severity(debug_properties.message_level.flags)
        .message_type(debug_properties.message_type.flags)
        .pfn_user_callback(Some(vulkan_debug_callback));

    let utils = debug_utils::Instance::new(entry, instance);

    let messenger = unsafe {
        utils
            .create_debug_utils_messenger(&create_info, None)
            .expect("Failed to create debug messenger")
    };

    Debug { messenger, utils }
}
