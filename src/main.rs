use ash::{
    ext::debug_utils,
    vk::{self, api_version_major, api_version_minor, api_version_patch, make_api_version},
    Entry,
};
use ash_window::{create_surface, enumerate_required_extensions};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::WindowBuilder,
};

use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
    os::raw::{c_char, c_void},
    ptr,
};

const APP_NAME: &'static str = "Playspawn";
const APP_VERSION: u32 = make_api_version(0, 1, 0, 0);
const VULKAN_VERSION: u32 = make_api_version(0, 1, 3, 0);
#[cfg(not(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = false;
#[cfg(debug_assertions)]
const ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(debug_assertions)]
const VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];
#[cfg(not(debug_assertions))]
const VALIDATION_LAYERS: [&str; 0] = [];
const VALIDATION_LAYER_NAME: &CStr =
    unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0") };

struct QueueFamilyIndices {
    graphics_family: Option<u32>,
    present_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn new() -> QueueFamilyIndices {
        QueueFamilyIndices {
            graphics_family: None,
            present_family: None,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

struct VulkanApp {
    _entry: ash::Entry,
    instance: ash::Instance,
    debug_utils: ash::ext::debug_utils::Instance,
    debug_utils_messenger: ash::vk::DebugUtilsMessengerEXT,
    _physical_device: vk::PhysicalDevice,
    device: ash::Device, // Logical Device
    _graphics_queue: vk::Queue,
    _surface: vk::SurfaceKHR,
    _surface_instance: ash::khr::surface::Instance,
    _present_queue: vk::Queue,
}

impl VulkanApp {
    pub fn new(window: &winit::window::Window) -> VulkanApp {
        // Load vulkan functions
        let entry = unsafe {
            match Entry::load() {
                Ok(entry) => entry,
                Err(error) => panic!("{}", error),
            }
        };

        let app_name = match CString::new(APP_NAME) {
            Ok(app_name) => app_name,
            Err(error) => panic!("{}", error),
        };
        let engine_name = match CString::new(format!("{APP_NAME} Engine")) {
            Ok(engine_name) => engine_name,
            Err(error) => panic!("{}", error),
        };

        let app_info = vk::ApplicationInfo {
            s_type: vk::StructureType::APPLICATION_INFO,
            p_next: ptr::null(),
            p_application_name: app_name.as_ptr(),
            application_version: APP_VERSION,
            p_engine_name: engine_name.as_ptr(),
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
            debug_utils
                .create_debug_utils_messenger(&debug_utils_messenger_create_info, None)
                .unwrap()
        };

        let window_handle = window.window_handle().unwrap().as_raw();
        let surface = unsafe {
            create_surface(&entry, &instance, raw_display_handle, window_handle, None).unwrap()
        };
        let surface_instance = ash::khr::surface::Instance::new(&entry, &instance);
        let physical_device =
            VulkanApp::pick_physical_device(&instance, &surface_instance, surface);
        let (device, family_indices) = VulkanApp::create_logical_device(
            &instance,
            physical_device,
            &surface_instance,
            surface,
        );
        let graphics_queue =
            unsafe { device.get_device_queue(family_indices.graphics_family.unwrap(), 0) };
        let present_queue =
            unsafe { device.get_device_queue(family_indices.present_family.unwrap(), 0) };

        VulkanApp {
            _entry: entry,
            instance,
            debug_utils,
            debug_utils_messenger,
            _physical_device: physical_device,
            device,
            _graphics_queue: graphics_queue,
            _surface: surface,
            _surface_instance: surface_instance,
            _present_queue: present_queue,
        }
    }

    fn vk_to_string(raw_string_array: &[c_char]) -> String {
        let raw_string = unsafe {
            let pointer = raw_string_array.as_ptr();
            CStr::from_ptr(pointer)
        };

        raw_string
            .to_str()
            .expect("Failed to convert vulkan raw string.")
            .to_owned()
    }

    fn pick_physical_device(
        instance: &ash::Instance,
        surface_instance: &ash::khr::surface::Instance,
        surface: vk::SurfaceKHR,
    ) -> vk::PhysicalDevice {
        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Failed to enumerate Physical Devices!")
        };

        println!(
            "{} devices (GPU) found with vulkan support.",
            physical_devices.len()
        );

        let mut result = None;
        for &physical_device in physical_devices.iter() {
            if VulkanApp::is_physical_device_suitable(
                instance,
                physical_device,
                surface_instance,
                surface,
            ) {
                if result.is_none() {
                    result = Some(physical_device)
                }
            }
        }

        match result {
            None => panic!("Failed to find a suitable GPU!"),
            Some(physical_device) => physical_device,
        }
    }

    fn is_physical_device_suitable(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface_instance: &ash::khr::surface::Instance,
        surface: vk::SurfaceKHR,
    ) -> bool {
        let device_properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let device_features = unsafe { instance.get_physical_device_features(physical_device) };
        let device_queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let device_type = match device_properties.device_type {
            vk::PhysicalDeviceType::CPU => "Cpu",
            vk::PhysicalDeviceType::INTEGRATED_GPU => "Integrated GPU",
            vk::PhysicalDeviceType::DISCRETE_GPU => "Discrete GPU",
            vk::PhysicalDeviceType::VIRTUAL_GPU => "Virtual GPU",
            vk::PhysicalDeviceType::OTHER => "Unknown",
            _ => panic!(),
        };

        let device_name = VulkanApp::vk_to_string(&device_properties.device_name);
        println!(
            "\tDevice Name: {}, id: {}, type: {}",
            device_name, device_properties.device_id, device_type
        );

        let major_version = api_version_major(device_properties.api_version);
        let minor_version = api_version_minor(device_properties.api_version);
        let patch_version = api_version_patch(device_properties.api_version);

        println!(
            "\tAPI Version: {}.{}.{}",
            major_version, minor_version, patch_version
        );

        println!("\tSupport Queue Family: {}", device_queue_families.len());
        println!("\t\tQueue Count | Graphics, Compute, Transfer, Sparse Binding");
        for queue_family in device_queue_families.iter() {
            let is_graphics_support = if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            {
                "support"
            } else {
                "unsupport"
            };
            let is_compute_support = if queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE) {
                "support"
            } else {
                "unsupport"
            };
            let is_transfer_support = if queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER)
            {
                "support"
            } else {
                "unsupport"
            };
            let is_sparse_support = if queue_family
                .queue_flags
                .contains(vk::QueueFlags::SPARSE_BINDING)
            {
                "support"
            } else {
                "unsupport"
            };

            println!(
                "\t\t{}\t    | {},  {},  {},  {}",
                queue_family.queue_count,
                is_graphics_support,
                is_compute_support,
                is_transfer_support,
                is_sparse_support
            );
        }

        // there are plenty of features
        println!(
            "\tGeometry Shader support: {}",
            if device_features.geometry_shader == 1 {
                "Support"
            } else {
                "Unsupport"
            }
        );

        let indices =
            VulkanApp::find_queue_family(instance, physical_device, surface_instance, surface);

        return indices.is_complete();
    }

    fn find_queue_family(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface_instance: &ash::khr::surface::Instance,
        surface: vk::SurfaceKHR,
    ) -> QueueFamilyIndices {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let mut queue_family_indices = QueueFamilyIndices::new();

        let mut index = 0;
        for queue_family in queue_families.iter() {
            if queue_family.queue_count > 0
                && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            {
                queue_family_indices.graphics_family = Some(index);
            }

            let is_present_support = unsafe {
                surface_instance
                    .get_physical_device_surface_support(physical_device, index as u32, surface)
                    .unwrap()
            };
            if queue_family.queue_count > 0 && is_present_support {
                queue_family_indices.present_family = Some(index);
            }

            if queue_family_indices.is_complete() {
                break;
            }

            index += 1;
        }

        queue_family_indices
    }

    fn create_logical_device(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface_instance: &ash::khr::surface::Instance,
        surface: vk::SurfaceKHR,
    ) -> (ash::Device, QueueFamilyIndices) {
        let indices =
            VulkanApp::find_queue_family(instance, physical_device, surface_instance, surface);

        use std::collections::HashSet;
        let mut unique_queue_families = HashSet::new();
        unique_queue_families.insert(indices.graphics_family.unwrap());
        unique_queue_families.insert(indices.present_family.unwrap());

        let queue_priorities = [1.0_f32];
        let mut queue_create_infos = vec![];
        for &queue_family in unique_queue_families.iter() {
            let queue_create_info = vk::DeviceQueueCreateInfo {
                s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::DeviceQueueCreateFlags::empty(),
                queue_family_index: queue_family,
                p_queue_priorities: queue_priorities.as_ptr(),
                queue_count: queue_priorities.len() as u32,
                _marker: PhantomData,
            };
            queue_create_infos.push(queue_create_info);
        }

        let physical_device_features = vk::PhysicalDeviceFeatures {
            ..Default::default() // default just enable no feature.
        };

        let requred_validation_layer_raw_names: Vec<CString> = VALIDATION_LAYERS
            .iter()
            .map(|layer_name| CString::new(*layer_name).unwrap())
            .collect();
        let enable_layer_names: Vec<*const c_char> = requred_validation_layer_raw_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();

        #[allow(deprecated)]
        let device_create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceCreateFlags::empty(),
            queue_create_info_count: queue_create_infos.len() as u32,
            p_queue_create_infos: queue_create_infos.as_ptr(),
            enabled_layer_count: if ENABLE_VALIDATION_LAYERS {
                enable_layer_names.len()
            } else {
                0
            } as u32,
            pp_enabled_layer_names: if ENABLE_VALIDATION_LAYERS {
                enable_layer_names.as_ptr()
            } else {
                ptr::null()
            },
            enabled_extension_count: 0,
            pp_enabled_extension_names: ptr::null(),
            p_enabled_features: &physical_device_features,
            _marker: PhantomData,
        };

        let device: ash::Device = unsafe {
            instance
                .create_device(physical_device, &device_create_info, None)
                .expect("Failed to create logical Device!")
        };

        (device, indices)
    }
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        unsafe {
            #[cfg(debug_assertions)]
            self.debug_utils
                .destroy_debug_utils_messenger(self.debug_utils_messenger, None);
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let _vulkan_app = VulkanApp::new(&window);

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    // ControlFlow::Wait pauses the event loop if no events are available to
    // process. This is ideal for non-game applications that only update in
    // response to user input, and uses significantly less power/CPU time than
    // ControlFlow::Poll.
    event_loop.set_control_flow(ControlFlow::Wait);

    let _ = event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("The close button was pressed; stopping");
                elwt.exit();
            }
            Event::AboutToWait => {
                // Application update code.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need
                // to redraw in applications which do not always need to.
                // Applications that redraw continuously can render here
                // instead.
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render
                // continuously to render in this event rather than in
                // AboutToWait, since rendering in here allows the program to
                // gracefully handle redraws requested by the OS.
            }
            _ => (),
        }
    });
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
