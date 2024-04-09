use ash::ext::debug_utils;
use ash::khr::{surface, wayland_surface};
use ash::vk::{self, make_api_version};
use std::ffi::CString;
use std::marker::PhantomData;
use std::ptr;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const APP_NAME: &'static str = "Playspawn";
const APP_VERSION: u32 = make_api_version(0, 1, 0, 0);
const VULKAN_VERSION: u32 = make_api_version(0, 1, 3, 0);

struct VulkanApp {
    _entry: ash::Entry,
    instance: ash::Instance,
}

impl VulkanApp {
    pub fn new() -> VulkanApp {
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

        let create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::InstanceCreateFlags::empty(),
            p_application_info: &app_info,
            pp_enabled_layer_names: ptr::null(),
            enabled_layer_count: 0,
            pp_enabled_extension_names: extension_names.as_ptr(),
            enabled_extension_count: extension_names.len() as u32,
            _marker: PhantomData,
        };

        let entry = unsafe { ash::Entry::load().unwrap() };
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
