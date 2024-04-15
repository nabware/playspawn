use ash::{
    ext::debug_utils,
    vk::{self, make_api_version, SurfaceCapabilitiesKHR, SurfaceFormatKHR},
    Entry,
};
use num::clamp;
use winit::{
    event_loop::EventLoop,
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::WindowBuilder,
};

use core::panic;
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
    let event_loop = match EventLoop::new() {
        Ok(event_loop) => event_loop,
        Err(error) => panic!("{}", error),
    };
    let window = match WindowBuilder::new().build(&event_loop) {
        Ok(window) => window,
        Err(error) => panic!("{}", error),
    };

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

    // Define required instance layers
    #[cfg(debug_assertions)]
    let required_instance_layer_names = [VALIDATION_LAYER_NAME.as_ptr()];
    #[cfg(not(debug_assertions))]
    let required_instance_layer_names = [];

    // Check instance layer support
    let available_layers = unsafe {
        match entry.enumerate_instance_layer_properties() {
            Ok(available_layers) => available_layers,
            Err(error) => panic!("{}", error),
        }
    };
    for required_name_ptr in required_instance_layer_names {
        let required_name_cstr = unsafe { CStr::from_ptr(required_name_ptr) };
        let mut layer_found = false;
        for vk::LayerProperties { layer_name, .. } in available_layers.iter() {
            let available_name_cstr = unsafe { CStr::from_ptr(layer_name.as_ptr()) };
            if required_name_cstr == available_name_cstr {
                layer_found = true;
                break;
            };
        }
        if !layer_found {
            panic!("Instance layer not supported: {:?}", required_name_cstr);
        };
    }

    let raw_display_handle = match window.display_handle() {
        Ok(display_handle) => display_handle.as_raw(),
        Err(error) => panic!("{}", error),
    };

    // Define required instance extensions
    let mut required_instance_extension_names =
        match ash_window::enumerate_required_extensions(raw_display_handle) {
            Ok(v) => v.to_vec(),
            Err(e) => panic!("{}", e),
        };
    #[cfg(debug_assertions)]
    required_instance_extension_names.push(debug_utils::NAME.as_ptr());

    // Check instance extension support
    let available_instance_extensions = unsafe {
        match entry.enumerate_instance_extension_properties(None) {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        }
    };
    for required_name_ptr in required_instance_extension_names.iter() {
        let required_name_cstr = unsafe { CStr::from_ptr(*required_name_ptr) };
        let mut extension_found = false;
        for vk::ExtensionProperties { extension_name, .. } in available_instance_extensions.iter() {
            let available_name_cstr = unsafe { CStr::from_ptr(extension_name.as_ptr()) };
            if required_name_cstr == available_name_cstr {
                extension_found = true;
                break;
            };
        }
        if !extension_found {
            panic!("Instance extension not supported: {:?}", required_name_cstr);
        };
    }

    // Create debug utrils messenger callback
    #[cfg(debug_assertions)]
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
        pp_enabled_layer_names: required_instance_layer_names.as_ptr(),
        enabled_layer_count: required_instance_layer_names.len() as u32,
        pp_enabled_extension_names: required_instance_extension_names.as_ptr(),
        enabled_extension_count: required_instance_extension_names.len() as u32,
        _marker: PhantomData,
    };
    let instance = unsafe {
        match entry.create_instance(&create_info, None) {
            Ok(instance) => instance,
            Err(error) => panic!("{}", error),
        }
    };

    // Create debug utils messenger
    #[cfg(debug_assertions)]
    let debug_utils = ash::ext::debug_utils::Instance::new(&entry, &instance);
    #[cfg(debug_assertions)]
    let debug_utils_messenger = unsafe {
        match debug_utils.create_debug_utils_messenger(&debug_utils_messenger_create_info, None) {
            Ok(debug_utils_messenger) => debug_utils_messenger,
            Err(error) => panic!("{}", error),
        }
    };

    // Create surface
    let raw_window_handle = match window.window_handle() {
        Ok(window_handle) => window_handle.as_raw(),
        Err(error) => panic!("{}", error),
    };
    let surface = unsafe {
        match ash_window::create_surface(
            &entry,
            &instance,
            raw_display_handle,
            raw_window_handle,
            None,
        ) {
            Ok(surface) => surface,
            Err(error) => panic!("{}", error),
        }
    };
    let surface_functions = ash::khr::surface::Instance::new(&entry, &instance);

    // Select physical device that supports device extensions
    // and queue families that support graphics and presentation,
    // preferably a discrete GPU.
    let mut selected_physical_device = None;
    let mut selected_queue_family_indices = HashSet::new();
    let mut selected_graphics_queue_family_index = None;
    let mut selected_present_queue_family_index = None;
    // Define required device extensions
    let required_device_extension_names = [ash::khr::swapchain::NAME.as_ptr()];
    let mut selected_surface_formats = vec![];
    let mut selected_present_modes = vec![];
    let available_physical_devices = unsafe {
        match instance.enumerate_physical_devices() {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        }
    };
    for available_physical_device in available_physical_devices {
        // Check device extension support
        let available_device_extensions = unsafe {
            match instance.enumerate_device_extension_properties(available_physical_device) {
                Ok(available_device_extensions) => available_device_extensions,
                Err(error) => panic!("{}", error),
            }
        };
        for required_name_ptr in required_device_extension_names {
            let required_name_cstr = unsafe { CStr::from_ptr(required_name_ptr) };
            let mut extension_found = false;
            for vk::ExtensionProperties { extension_name, .. } in available_device_extensions.iter()
            {
                let available_name_cstr = unsafe { CStr::from_ptr(extension_name.as_ptr()) };
                if required_name_cstr == available_name_cstr {
                    extension_found = true;
                    break;
                }
            }
            if !extension_found {
                continue;
            }
        }
        // Check swapchain support
        let surface_formats = unsafe {
            match surface_functions
                .get_physical_device_surface_formats(available_physical_device, surface)
            {
                Ok(v) => v,
                Err(e) => panic!("{}", e),
            }
        };
        if surface_formats.len() == 0 {
            continue;
        }
        let present_modes = unsafe {
            match surface_functions
                .get_physical_device_surface_present_modes(available_physical_device, surface)
            {
                Ok(v) => v,
                Err(e) => panic!("{}", e),
            }
        };
        if present_modes.len() == 0 {
            continue;
        }
        // Check graphics and presentation support
        let mut graphics_queue_family_index = None;
        let mut presentation_queue_family_index = None;
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
            // Check for graphics support
            if queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                graphics_queue_family_index = Some(current_queue_family_index);
            }
            // Check for presentation support
            if unsafe {
                match surface_functions.get_physical_device_surface_support(
                    available_physical_device,
                    current_queue_family_index,
                    surface,
                ) {
                    Ok(has_present_support) => has_present_support,
                    Err(error) => panic!("{}", error),
                }
            } {
                presentation_queue_family_index = Some(current_queue_family_index);
            }
            current_queue_family_index += 1;
        }
        if graphics_queue_family_index.is_some() && presentation_queue_family_index.is_some() {
            selected_physical_device = Some(available_physical_device);
            let graphics_queue_family_index = match graphics_queue_family_index {
                Some(graphics_queue_family_index) => graphics_queue_family_index,
                None => panic!("Missing graphics queue family index."),
            };
            let presentation_queue_family_index = match presentation_queue_family_index {
                Some(presentation_queue_family_index) => presentation_queue_family_index,
                None => panic!("Missing presentation queue family index."),
            };
            selected_queue_family_indices.clear();
            selected_queue_family_indices.insert(graphics_queue_family_index);
            selected_queue_family_indices.insert(presentation_queue_family_index);
            selected_surface_formats = surface_formats;
            selected_present_modes = present_modes;
            selected_graphics_queue_family_index = Some(graphics_queue_family_index);
            selected_present_queue_family_index = Some(presentation_queue_family_index);
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
    for queue_family_index in selected_queue_family_indices.iter() {
        let queue_create_info = vk::DeviceQueueCreateInfo {
            s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceQueueCreateFlags::empty(),
            queue_family_index: *queue_family_index,
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
        enabled_layer_count: required_instance_layer_names.len() as u32,
        pp_enabled_layer_names: required_instance_layer_names.as_ptr(),
        enabled_extension_count: required_device_extension_names.len() as u32,
        pp_enabled_extension_names: required_device_extension_names.as_ptr(),
        p_enabled_features: &physical_device_features,
        _marker: PhantomData,
    };
    let device: ash::Device = unsafe {
        match instance.create_device(selected_physical_device, &device_create_info, None) {
            Ok(device) => device,
            Err(error) => panic!("{}", error),
        }
    };

    let SurfaceCapabilitiesKHR {
        min_image_count,
        current_transform,
        min_image_extent,
        max_image_extent,
        ..
    } = unsafe {
        match surface_functions
            .get_physical_device_surface_capabilities(selected_physical_device, surface)
        {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        }
    };
    let SurfaceFormatKHR {
        format,
        color_space,
    } = selected_surface_formats[0];
    let (image_sharing_mode, queue_family_index_count, queue_family_indices) =
        if selected_queue_family_indices.len() > 1 {
            (
                vk::SharingMode::CONCURRENT,
                2,
                vec![
                    match selected_graphics_queue_family_index {
                        Some(v) => v,
                        None => panic!("Missing graphics queue family index."),
                    },
                    match selected_present_queue_family_index {
                        Some(v) => v,
                        None => panic!("Missing present queue family index."),
                    },
                ],
            )
        } else {
            (vk::SharingMode::EXCLUSIVE, 0, vec![])
        };
    let present_mode = selected_present_modes[0];
    let window_size = window.inner_size();
    let extent = vk::Extent2D {
        width: clamp(
            window_size.width,
            min_image_extent.width,
            max_image_extent.width,
        ),
        height: clamp(
            window_size.height,
            min_image_extent.height,
            max_image_extent.height,
        ),
    };

    // Create swapchain
    let swapchain_create_info = vk::SwapchainCreateInfoKHR {
        s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
        p_next: ptr::null(),
        flags: vk::SwapchainCreateFlagsKHR::empty(),
        surface,
        min_image_count: min_image_count + 1,
        image_color_space: color_space,
        image_format: format,
        image_extent: extent,
        image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
        image_sharing_mode,
        p_queue_family_indices: queue_family_indices.as_ptr(),
        queue_family_index_count,
        pre_transform: current_transform,
        composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
        present_mode,
        clipped: vk::TRUE,
        old_swapchain: vk::SwapchainKHR::null(),
        image_array_layers: 1,
        _marker: PhantomData,
    };
    let swapchain_functions = ash::khr::swapchain::Device::new(&instance, &device);
    let swapchain = unsafe {
        match swapchain_functions.create_swapchain(&swapchain_create_info, None) {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        }
    };

    // Cleanup
    unsafe {
        swapchain_functions.destroy_swapchain(swapchain, None);
        device.destroy_device(None);
        surface_functions.destroy_surface(surface, None);
        #[cfg(debug_assertions)]
        debug_utils.destroy_debug_utils_messenger(debug_utils_messenger, None);
        instance.destroy_instance(None);
    }
}
