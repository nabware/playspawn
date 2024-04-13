use ash::{
    ext::debug_utils,
    vk::{self, make_api_version},
    Entry,
};
use ash_window::enumerate_required_extensions;
use winit::{event_loop::EventLoop, raw_window_handle::HasDisplayHandle, window::WindowBuilder};

use std::{
    collections::HashSet,
    ffi::{CStr, CString},
    marker::PhantomData,
    os::raw::c_void,
    ptr,
};

const APP_NAME: &'static str = "Playspawn";
const APP_VERSION: u32 = make_api_version(0, 1, 0, 0);
const VULKAN_VERSION: u32 = make_api_version(0, 1, 3, 0);
const VALIDATION_LAYER_NAME: &CStr =
    unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0") };

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // Load vulkan functions
    let entry = unsafe {
        match Entry::load() {
            Ok(entry) => entry,
            Err(error) => panic!("{}", error),
        }
    };

    // Create app info
    let app_name_ptr = match CString::new(APP_NAME) {
        Ok(app_name) => app_name,
        Err(error) => panic!("{}", error),
    }
    .as_ptr();
    let engine_name_ptr = match CString::new(format!("{APP_NAME} Engine")) {
        Ok(engine_name) => engine_name,
        Err(error) => panic!("{}", error),
    }
    .as_ptr();
    let app_info = vk::ApplicationInfo {
        s_type: vk::StructureType::APPLICATION_INFO,
        p_next: ptr::null(),
        p_application_name: app_name_ptr,
        application_version: APP_VERSION,
        p_engine_name: engine_name_ptr,
        engine_version: APP_VERSION,
        api_version: VULKAN_VERSION,
        _marker: PhantomData,
    };

    // Define required layers
    #[cfg(debug_assertions)]
    let required_layer_names: [*const i8; 1] = [VALIDATION_LAYER_NAME.as_ptr()];
    #[cfg(not(debug_assertions))]
    let required_layer_names: [*const i8; 0] = [];

    // Check layer support
    let available_layers = unsafe {
        match entry.enumerate_instance_layer_properties() {
            Ok(available_layers) => available_layers,
            Err(error) => panic!("{}", error),
        }
    };
    for required_layer_name_ptr in required_layer_names {
        let required_layer_name = unsafe {
            match CStr::from_ptr(required_layer_name_ptr).to_str() {
                Ok(required_layer_name) => required_layer_name,
                Err(error) => panic!("{}", error),
            }
        };
        let mut layer_found = false;
        for available_layer in available_layers.iter() {
            let available_layer_name = unsafe {
                match CStr::from_ptr(available_layer.layer_name.as_ptr()).to_str() {
                    Ok(available_layer_name) => available_layer_name,
                    Err(error) => panic!("{}", error),
                }
            };
            if required_layer_name == available_layer_name {
                layer_found = true;
                break;
            };
        }
        if !layer_found {
            panic!("Layer not supported: {}", required_layer_name);
        };
    }

    let raw_display_handle = window.display_handle().unwrap().as_raw();

    // Define required extensions
    let required_extension_names = match enumerate_required_extensions(raw_display_handle) {
        Ok(required_extension_names) => required_extension_names,
        Err(error) => panic!("{}", error),
    };
    #[cfg(debug_assertions)]
    let required_extension_names =
        [required_extension_names, &[debug_utils::NAME.as_ptr()]].concat();

    // Check extension support
    let available_extensions = unsafe {
        match entry.enumerate_instance_extension_properties(None) {
            Ok(available_extensions) => available_extensions,
            Err(error) => panic!("{}", error),
        }
    };
    for required_extension_name_ptr in required_extension_names.iter() {
        let required_extension_name = unsafe {
            match CStr::from_ptr(*required_extension_name_ptr).to_str() {
                Ok(required_extension_name) => required_extension_name,
                Err(error) => panic!("{}", error),
            }
        };
        let mut extension_found = false;
        for available_extension in available_extensions.iter() {
            let available_extension_name = unsafe {
                match CStr::from_ptr(available_extension.extension_name.as_ptr()).to_str() {
                    Ok(available_extension_name) => available_extension_name,
                    Err(error) => panic!("{}", error),
                }
            };
            if required_extension_name == available_extension_name {
                extension_found = true;
                break;
            };
        }
        if !extension_found {
            panic!("Extension not supported: {}", required_extension_name);
        };
    }

    // Create info for debug utils messenger
    #[cfg(debug_assertions)]
    let debug_utils_messenger_create_info = vk::DebugUtilsMessengerCreateInfoEXT {
        s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
        p_next: ptr::null(),
        flags: vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
            | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        pfn_user_callback: Some(debug_utils_messenger_callback),
        p_user_data: ptr::null_mut(),
        _marker: PhantomData,
    };

    // Create debug utils messenger for instance creation and destruction
    #[cfg(debug_assertions)]
    let debug_utils_messenger_create_info_ptr =
        &debug_utils_messenger_create_info as *const _ as *const c_void;
    #[cfg(not(debug_assertions))]
    let debug_utils_messenger_create_info_ptr = ptr::null();

    // Create vulkan instance
    let create_info = vk::InstanceCreateInfo {
        s_type: vk::StructureType::INSTANCE_CREATE_INFO,
        p_next: debug_utils_messenger_create_info_ptr,
        flags: vk::InstanceCreateFlags::empty(),
        p_application_info: &app_info,
        pp_enabled_layer_names: required_layer_names.as_ptr(),
        enabled_layer_count: required_layer_names.len() as u32,
        pp_enabled_extension_names: required_extension_names.as_ptr(),
        enabled_extension_count: required_extension_names.len() as u32,
        _marker: PhantomData,
    };
    let instance = unsafe {
        match entry.create_instance(&create_info, None) {
            Ok(instance) => instance,
            Err(error) => panic!("{}", error),
        }
    };

    // Create debug utils messenger for everything else
    let debug_utils = ash::ext::debug_utils::Instance::new(&entry, &instance);
    let debug_utils_messenger = unsafe {
        match debug_utils.create_debug_utils_messenger(&debug_utils_messenger_create_info, None) {
            Ok(debug_utils_messenger) => debug_utils_messenger,
            Err(error) => panic!("{}", error),
        }
    };

    // Select physical device and queue family that supports graphics, preferably a discrete GPU
    let mut selected_physical_device = None;
    let mut selected_queue_family_indices = HashSet::new();
    let available_physical_devices = unsafe {
        match instance.enumerate_physical_devices() {
            Ok(physical_devices) => physical_devices,
            Err(error) => panic!("{}", error),
        }
    };
    for available_physical_device in available_physical_devices {
        let mut graphics_queue_family_index = None;
        let queue_families = unsafe {
            instance.get_physical_device_queue_family_properties(available_physical_device)
        };
        let mut current_queue_family_index = 0;
        for vk::QueueFamilyProperties {
            queue_count,
            queue_flags,
            ..
        } in queue_families
        {
            if queue_count == 0 {
                continue;
            }
            if queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                graphics_queue_family_index = Some(current_queue_family_index);
            }
            current_queue_family_index += 1;
        }
        if graphics_queue_family_index.is_some() {
            selected_physical_device = Some(available_physical_device);
            let graphics_queue_family_index = match graphics_queue_family_index {
                Some(graphics_queue_family_index) => graphics_queue_family_index,
                None => panic!("Missing graphics queue family index."),
            };
            selected_queue_family_indices.clear();
            selected_queue_family_indices.insert(graphics_queue_family_index);
            // Stop looking if this is a discrete GPU
            let vk::PhysicalDeviceProperties { device_type, .. } =
                unsafe { instance.get_physical_device_properties(available_physical_device) };
            if device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
                break;
            }
        }
    }
    let selected_physical_device = match selected_physical_device {
        Some(selected_physical_device) => selected_physical_device,
        None => panic!("Failed to find physical device that supports graphics."),
    };

    // Queue create infos and device features
    let queue_priorities = [1.0_f32];
    let mut queue_create_infos = vec![];
    for queue_family_index in selected_queue_family_indices {
        let queue_create_info = vk::DeviceQueueCreateInfo {
            s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceQueueCreateFlags::empty(),
            queue_family_index,
            p_queue_priorities: queue_priorities.as_ptr(),
            queue_count: queue_priorities.len() as u32,
            _marker: PhantomData,
        };
        queue_create_infos.push(queue_create_info);
    }
    let physical_device_features = vk::PhysicalDeviceFeatures {
        ..Default::default()
    };

    // Create logical device
    #[allow(deprecated)]
    let device_create_info = vk::DeviceCreateInfo {
        s_type: vk::StructureType::DEVICE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::DeviceCreateFlags::empty(),
        queue_create_info_count: queue_create_infos.len() as u32,
        p_queue_create_infos: queue_create_infos.as_ptr(),
        enabled_layer_count: required_layer_names.len() as u32,
        pp_enabled_layer_names: required_layer_names.as_ptr(),
        enabled_extension_count: 0,
        pp_enabled_extension_names: ptr::null(),
        p_enabled_features: &physical_device_features,
        _marker: PhantomData,
    };
    let device: ash::Device = unsafe {
        match instance.create_device(selected_physical_device, &device_create_info, None) {
            Ok(device) => device,
            Err(error) => panic!("{}", error),
        }
    };

    // Cleanup
    unsafe {
        #[cfg(debug_assertions)]
        debug_utils.destroy_debug_utils_messenger(debug_utils_messenger, None);
        device.destroy_device(None);
        instance.destroy_instance(None);
    }
}

unsafe extern "system" fn debug_utils_messenger_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    let severity = match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => "[Verbose]",
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => "[Warning]",
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => "[Error]",
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => "[Info]",
        _ => "[Unknown]",
    };
    let types = match message_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "[General]",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[Performance]",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[Validation]",
        _ => "[Unknown]",
    };
    let message = CStr::from_ptr((*p_callback_data).p_message);
    println!("[Debug]{}{}{:?}", severity, types, message);

    vk::FALSE
}
