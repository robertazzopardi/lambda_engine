use crate::utility::EntryInstance;
use ash::{extensions::ext::DebugUtils, vk, Entry};
use derive_new::new;
use std::{borrow::Cow, ffi::CStr};

pub const VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];

#[derive(Clone)]
pub struct Debug {
    pub messenger: vk::DebugUtilsMessengerEXT,
    pub utils: DebugUtils,
}

pub const ENABLE_VALIDATION_LAYERS: bool = cfg!(debug_assertions);

pub(crate) fn check_validation_layer_support(entry: &Entry) -> bool {
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

/// VkDebugUtilsMessageSeverityFlagBitsEXT
#[derive(Debug, Clone, Copy)]
pub struct MessageLevel {
    pub flags: vk::DebugUtilsMessageSeverityFlagsEXT,
}

impl MessageLevel {
    /// MessageLevel Builder
    ///
    /// Info message severity enabled by default
    pub const fn builder() -> Self {
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

impl Default for MessageLevel {
    fn default() -> Self {
        Self::builder().error().verbose().warning()
    }
}

/// VkDebugUtilsMessageTypeFlagBitsEXT
#[derive(Debug, Clone, Copy)]
pub struct MessageType {
    pub flags: vk::DebugUtilsMessageTypeFlagsEXT,
}

impl MessageType {
    /// MessageType Builder
    ///
    /// General type message enabled by default
    pub const fn builder() -> Self {
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

impl Default for MessageType {
    fn default() -> Self {
        Self::builder().validation().performance()
    }
}

#[derive(new, Debug, Default, Clone, Copy)]
pub struct DebugMessageProperties {
    pub message_level: MessageLevel,
    pub message_type: MessageType,
}

#[inline]
pub fn create_debug_messenger(
    debugging: DebugMessageProperties,
) -> vk::DebugUtilsMessengerCreateInfoEXT {
    vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(debugging.message_level.flags)
        .message_type(debugging.message_type.flags)
        .pfn_user_callback(Some(vulkan_debug_callback))
        .build()
}

pub fn debugger(
    EntryInstance { entry, instance }: &EntryInstance,
    debug_properties: DebugMessageProperties,
) -> Debug {
    let create_info = create_debug_messenger(debug_properties);

    let utils = DebugUtils::new(entry, instance);

    let messenger = unsafe {
        utils
            .create_debug_utils_messenger(&create_info, None)
            .expect("Failed to create debug messenger")
    };

    Debug { messenger, utils }
}
