use device_query::{DeviceState, DeviceQuery};
use mouse_keyboard_input::VirtualDevice;
use notify_rust::Notification;
use std::{thread, time::Duration, num::NonZeroU32};
use std::sync::{Arc, Mutex};

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::window::{Window, WindowId, WindowLevel};
use winit::dpi::{Position, LogicalSize, LogicalPosition};

use softbuffer::{Context, Surface};

use ini::Ini;

// --- AUTO SCROLL FUNCTIONALITY --- 
#[derive(Clone)]
struct AutoScrollerConfig
{
    size: f64,
    radius: i32,
    max_scroll_power: i32,
    friction: f32,
    show_notif: bool,
    color0: u32,
    color1: u32
}

#[derive(Clone)]
struct ScrollState
{
    scroll_mode : bool,
    mx: i32,
    my: i32
}

#[derive(Debug, Clone, Copy)]
enum AutoScrollerEvent // For the window functionality
{
    WindowToggleOn(i32, i32),
    WindowToggleOff
}

// Define an Autoscroller
struct AutoScroller
{
    config: AutoScrollerConfig,
    window: Option<Arc<Window>>,
    surface: Option<Surface<Arc<Window>, Arc<Window>>>,
}

// Define an Autoscroller Worker
struct AutoScrollWorker
{
    config: AutoScrollerConfig,
    proxy: EventLoopProxy<AutoScrollerEvent>,
    vdevice: VirtualDevice,
    device_state: DeviceState,
    state: Arc<Mutex<ScrollState>>,
    delay: u64, // Millisecond delay
    delay0: u64, // Millisecond delay, second value
}

// Define the box that will be drawn inside the window! :D
fn draw_pixel_buffer(surface : &mut Surface<Arc<Window>, Arc<Window>>, window : &Window, color0: u32, color1: u32)
{
    
    let size = window.inner_size();

    if size.width == 0 || size.height == 0
    {
        return;
    }

    surface.resize(
        NonZeroU32::new(size.width).unwrap(),
        NonZeroU32::new(size.height).unwrap(),
    ).unwrap();

    let mut buffer = surface.buffer_mut().unwrap();

    // Draw the window:

    let width = size.width as usize;
    let height = size.height as usize;

    let border_thickness = 2;
    let dot_radius = 2;

    let cx = (width / 2) as i32;
    let cy = (height / 2) as i32;

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;

            // Determine if Border
            let is_border =
                x < border_thickness ||
                x >= width - border_thickness ||
                y < border_thickness ||
                y >= height - border_thickness;

            // Determine if Center Dot
            let dx = x as i32 - cx;
            let dy = y as i32 - cy;
            let is_center_dot = dx * dx + dy * dy <= dot_radius * dot_radius;

            buffer[idx] = if is_border || is_center_dot {
                color0
            } else {
                color1
            };
        }
    }

    buffer.present().unwrap();
}

// Define windows for Application Handler
impl ApplicationHandler<AutoScrollerEvent> for AutoScroller
{
    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AutoScrollerEvent)
    {
        match event
        {
            AutoScrollerEvent::WindowToggleOn(x, y) => 
            { // Turn window on
                
                if self.window.is_none()
                { // First time opening a window
                    // Create the window
                    let window = Arc::new(event_loop.create_window(
                        Window::default_attributes()
                        .with_title("s-autoscroller")
                        .with_inner_size(LogicalSize::new(self.config.size, self.config.size))
                        .with_max_inner_size(LogicalSize::new(self.config.size, self.config.size))
                        .with_min_inner_size(LogicalSize::new(self.config.size, self.config.size))
                        .with_window_level(WindowLevel::AlwaysOnTop)
                        .with_position(Position::Logical(LogicalPosition::new(
                                    x as f64 - (self.config.size/2.0), y as f64 - (self.config.size/2.0)
                        )))
                        .with_decorations(false)
                        .with_resizable(false)
                        .with_maximized(false)
                        .with_visible(true)
                    ).unwrap());

                    let context = Context::new(window.clone()).unwrap();
                    let surface = Surface::new(&context, window.clone()).unwrap();

                    self.surface = Some(surface);
                    self.window = Some(window);

                    // Set window attributes and colour it
                    if let Some(window) = &self.window
                    {
                        window.set_outer_position(Position::Logical(LogicalPosition::new(
                                    x as f64 - (self.config.size/2.0), y as f64 - (self.config.size/2.0)
                        )));
                        let _ = window.request_inner_size(LogicalSize::new(self.config.size, self.config.size));
                        window.request_redraw();
                        //draw_pixel_buffer(&window, self.config.color0, self.config.color1);
                    }
                }
                else if let Some(window) = &self.window
                { // Not the first time opening a window
                    window.set_visible(true);
                    window.set_outer_position(Position::Logical(LogicalPosition::new(
                                x as f64 - (self.config.size/2.0), y as f64 - (self.config.size/2.0)
                    )));
                    let _ = window.request_inner_size(LogicalSize::new(self.config.size, self.config.size));
                    window.request_redraw();
                    //draw_pixel_buffer(&window, self.config.color0, self.config.color1);
                }

            }
            AutoScrollerEvent::WindowToggleOff =>
            { // Turn window off
                
                if let Some(window) = &self.window
                {
                    window.set_visible(false);
                }

            }
        }
    }

    fn resumed(&mut self, _event_loop: &ActiveEventLoop) { }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) 
    {
        match event 
        {
            WindowEvent::RedrawRequested => 
            {
                // Draw when the OS requests a redraw
                if let (Some(surface), Some(window)) = (&mut self.surface, &self.window) 
                {
                    draw_pixel_buffer(surface, window, self.config.color0, self.config.color1);
                }
            }
            WindowEvent::Resized(_) => 
            {
                // Request redraw on resize
                if let Some(window) = &self.window 
                {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

}

// Define functions around an autoscroller
impl AutoScrollWorker
{
  
    fn new(
        proxy: EventLoopProxy<AutoScrollerEvent>,
        state: Arc<Mutex<ScrollState>>,
        config: AutoScrollerConfig
    ) -> Self
    {
        Self 
        {
            config: config,
            proxy,
            vdevice: VirtualDevice::default().unwrap(),
            device_state: DeviceState::new(),
            state,
            delay: 20,
            delay0: 1
        }
    }

    // Send a notification to the system about the scroll mode being enabled
    fn notify_scroll_mode(&mut self, enabled: bool)
    {
        if self.config.show_notif
        {
            let txt = if enabled {"ENABLED"} else {"DISABLED"};
            Notification::new()
                .summary("Mouse Scroll Mode")
                .body(txt)
                .show()
                .unwrap();
        }
    }

    // Either create or destroy a window with the icon for the autoscroll
    fn toggle_icon(&mut self, enabled: bool, x: i32, y: i32)
    {
        if enabled
        { // Create window
            let _ = self.proxy.send_event(AutoScrollerEvent::WindowToggleOn(x, y));
        }
        else
        { // Destroy or Hide window
            let _ = self.proxy.send_event(AutoScrollerEvent::WindowToggleOff);
        }
    }

    // Detect if the middle click is being pressed at the moment
    fn detect_middle_click(&mut self, scroll_mode: bool) -> (bool, i32, i32)
    {
        let mouse_state = self.device_state.get_mouse();
        let pressed = &mouse_state.button_pressed;

        // Save the mouse coordinates in the parameter
        let x = mouse_state.coords.0;
        let y = mouse_state.coords.1;

        // IDX 1 is left click, 
        // 2 is middle click,
        // 3 is right click
        if pressed[2]
        {
            return (true, x, y);
        }

        if scroll_mode
        { // exit the scroll mode on left or right click
            if pressed[1] || pressed[3]
            {
                return (true, x, y);
            }
        }

        return (false, x, y);
    }

    // Update the scroll mode according to the toggle button being released
    fn update_scroll_mode(&mut self)
    {
        let scroll_mode = 
        {
            let state = self.state.lock().unwrap();
            state.scroll_mode
        };

        let (mut toggle_button_clicked, mut x, mut y) = self.detect_middle_click(scroll_mode);

        if toggle_button_clicked
        {
            // Wait until toggle button that is clicked is false
            // This way we wait until the button is released :D
            while toggle_button_clicked
            {
                thread::sleep(Duration::from_millis(50));

                let new_scroll_mode = 
                {
                    let state = self.state.lock().unwrap();
                    state.scroll_mode
                };

                (toggle_button_clicked, x, y) = self.detect_middle_click(new_scroll_mode);
            }
            
            let mut state = self.state.lock().unwrap();

            // Update the scroll mode
            state.scroll_mode = if state.scroll_mode {false} else {true};

            // Save the middle mouse coordinates
            if state.scroll_mode
            {
                state.mx = x;
                state.my = y;
            }
            else
            {
                state.mx = -1;
                state.my = -1;
            }

            let scroll_mode = state.scroll_mode;
            let mx = state.mx;
            let my = state.my;
            drop(state);

            // Update UI for the user to notice
            self.notify_scroll_mode(scroll_mode); // Updated scroll mode
            self.toggle_icon(scroll_mode, mx, my); // Updated scroll mode
        }
    }

    // Scroll towards (x,y) using (mx, my) as the middle point
    // delay = max_delay - (distance / radius) * (max_delay - min_delay)
    fn scroll_towards_vertical(&mut self, my: i32, y: i32)
    {
        // Scroll calculations for 'y'
        let delta_y = my - y;
        let abs_delta_y = delta_y.abs();
        if abs_delta_y <= self.config.radius { return; } // In dead zone
        
        let effective_distance = abs_delta_y - self.config.radius; // Distance from the deadzone
        let scaled_distance = (effective_distance as f32 / self.config.friction) as i32;

        let clamped = scaled_distance.min(self.config.max_scroll_power);
        let scroll_y = if delta_y < 0 {-clamped} else {clamped};
        // Delay calculation
        let t = (scaled_distance as f32 / self.config.max_scroll_power as f32).clamp(0.0, 1.0);
        let dynamic_delay = self.delay as f32 - t * (self.delay as f32 - self.delay0 as f32);

        self.vdevice.smooth_scroll(0, scroll_y).unwrap();
        thread::sleep(Duration::from_millis(dynamic_delay as u64));
    }

    // Start the autoscroll main loop
    fn run(&mut self)
    {
        // Create window
        loop
        {
            self.update_scroll_mode();

            let state = self.state.lock().unwrap();
            let scroll_mode = state.scroll_mode;
            let my = state.my;
            drop(state);

            if scroll_mode
            {
                // Get current mouse coordinates
                let mouse_state = self.device_state.get_mouse();
                // Decided to discard the x value, becomes too clumsy
                self.scroll_towards_vertical(my, mouse_state.coords.1);
            }
            else
            {
                thread::sleep(Duration::from_millis(50));
            }
        }
    }

}

fn load_config() -> AutoScrollerConfig
{
    // Try to load config.ini, fall back to defaults if it fails
    let conf = match Ini::load_from_file("config.ini") {
        Ok(c) => c,
        Err(e) => {
            println!("Warning: Could not load config.ini ({}), using defaults", e);
            return default_config();
        }
    };
    
    // Helper function to parse with defaults
    fn get_or_default<T: std::str::FromStr>(
        conf: &Ini,
        section: &str,
        key: &str,
        default: T
    ) -> T {
        conf.section(Some(section))
            .and_then(|s| s.get(key))
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }
    
    // Parse hex color
    fn parse_hex_color(conf: &Ini, section: &str, key: &str, default: u32) -> u32 {
        conf.section(Some(section))
            .and_then(|s| s.get(key))
            .and_then(|s| {
                let s = s.trim().trim_start_matches("0x").trim_start_matches("0X");
                u32::from_str_radix(s, 16).ok()
            })
            .unwrap_or(default)
    }
    
    AutoScrollerConfig {
        size: get_or_default(&conf, "autoscroller", "size", 15.0),
        radius: get_or_default(&conf, "autoscroller", "radius", 15),
        max_scroll_power: get_or_default(&conf, "autoscroller", "max_scroll_power", 50),
        friction: get_or_default(&conf, "autoscroller", "friction", 15.0),
        show_notif: get_or_default(&conf, "autoscroller", "show_notif", false),
        color0: parse_hex_color(&conf, "autoscroller", "color0", 0xff07553b),
        color1: parse_hex_color(&conf, "autoscroller", "color1", 0xFFCED46A),
    }
}

fn default_config() -> AutoScrollerConfig 
{
    AutoScrollerConfig {
        size: 15.0,
        radius: 15,
        max_scroll_power: 50,
        friction: 15.0,
        show_notif: false,
        color0: 0xff07553b,
        color1: 0xFFCED46A,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>>
{

    let config = load_config();
    
    let event_loop: EventLoop<AutoScrollerEvent> = EventLoop::<AutoScrollerEvent>::with_user_event().build()?;
    let proxy = event_loop.create_proxy();

    event_loop.set_control_flow(ControlFlow::Wait);

    let state = Arc::new(
        Mutex::new(
            ScrollState
            {
                scroll_mode: false,
                mx: -1,
                my: -1
            }
        )
    );

    let worker_proxy = proxy.clone();
    let worker_state = state.clone();
    let worker_config = config.clone();

    thread::spawn(move || {
        let mut worker = AutoScrollWorker::new(worker_proxy, worker_state, worker_config);
        worker.run();
    });

    let mut app = AutoScroller
    {
        config: config,
        window: None,
        surface : None,
    };
    
    let _ = event_loop.run_app(&mut app);

    Ok(())
}
