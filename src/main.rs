use ash::ext::debug_utils;
use ash::khr::{surface, wayland_surface};
use ash::vk::{self, make_api_version};
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::os::raw::{c_char, c_void};
use std::ptr;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const APP_NAME: &'static str = "Playspawn";
const APP_VERSION: u32 = make_api_version(0, 1, 0, 0);
const VULKAN_VERSION: u32 = make_api_version(0, 1, 3, 0);
#[cfg(not(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = false;
#[cfg(debug_assertions)]
const ENABLE_VALIDATION_LAYERS: bool = true;
const REQUIRED_VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];

struct VulkanApp {
    _entry: ash::Entry,
    instance: ash::Instance,
}

impl VulkanApp {
    pub fn new() -> VulkanApp {
        let entry = unsafe { ash::Entry::load().unwrap() };

        if ENABLE_VALIDATION_LAYERS
            && VulkanApp::check_validation_layer_support(&entry) == false
        {
            panic!("Validation layers requested, but not available!");
        }

        let app_name = CString::new(APP_NAME).unwrap();
        let engine_name = CString::new(format!("{APP_NAME} Engine")).unwrap();

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

        let extension_names = vec![
            surface::NAME.as_ptr(),
            wayland_surface::NAME.as_ptr(),
            debug_utils::NAME.as_ptr(),
        ];

        let required_validation_layer_raw_names: Vec<CString> =
            REQUIRED_VALIDATION_LAYERS
                .iter()
                .map(|layer_name| CString::new(*layer_name).unwrap())
                .collect();
        let enable_layer_names: Vec<*const i8> =
            required_validation_layer_raw_names
                .iter()
                .map(|layer_name| layer_name.as_ptr())
                .collect();

        let create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_next: if ENABLE_VALIDATION_LAYERS {
                &VulkanApp::debug_utils_create_info()
                    as *const vk::DebugUtilsMessengerCreateInfoEXT
                    as *const c_void
            } else {
                ptr::null()
            },
            flags: vk::InstanceCreateFlags::empty(),
            p_application_info: &app_info,
            pp_enabled_layer_names: if ENABLE_VALIDATION_LAYERS {
                enable_layer_names.as_ptr()
            } else {
                ptr::null()
            },
            enabled_layer_count: if ENABLE_VALIDATION_LAYERS {
                enable_layer_names.len()
            } else {
                0
            } as u32,
            pp_enabled_extension_names: extension_names.as_ptr(),
            enabled_extension_count: extension_names.len() as u32,
            _marker: PhantomData,
        };

        let instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Failed to create instance!")
        };

        VulkanApp {
            _entry: entry,
            instance,
        }
    }

    fn check_validation_layer_support(entry: &ash::Entry) -> bool {
        // if support validation layer, then return true

        let layer_properties = unsafe {
            entry
                .enumerate_instance_layer_properties()
                .expect("Failed to enumerate Instance Layers Properties!")
        };

        if layer_properties.len() <= 0 {
            eprintln!("No available layers.");
            return false;
        } else {
            println!("Instance Available Layers: ");
            for layer in layer_properties.iter() {
                let layer_name = VulkanApp::vk_to_string(&layer.layer_name);
                println!("\t{}", layer_name);
            }
        }

        for required_layer_name in REQUIRED_VALIDATION_LAYERS.iter() {
            let mut is_layer_found = false;

            for layer_property in layer_properties.iter() {
                let test_layer_name =
                    VulkanApp::vk_to_string(&layer_property.layer_name);
                if (*required_layer_name) == test_layer_name {
                    is_layer_found = true;
                    break;
                }
            }

            if is_layer_found == false {
                return false;
            }
        }

        true
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

    fn debug_utils_create_info<'a>() -> vk::DebugUtilsMessengerCreateInfoEXT<'a>
    {
        vk::DebugUtilsMessengerCreateInfoEXT {
            s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
            p_next: ptr::null(),
            flags: vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
            message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
            pfn_user_callback: Some(VulkanApp::vulkan_debug_utils_callback),
            p_user_data: ptr::null_mut(),
            _marker: PhantomData,
        }
    }

    // Callback function used in Debug Utils.
    unsafe extern "system" fn vulkan_debug_utils_callback(
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
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}

fn main() {
    let _vulkan_app = VulkanApp::new();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

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
