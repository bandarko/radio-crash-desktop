use crate::{
    dsp::Meters,
    tray_meter::{Frame, Layout},
};
use cpal::traits::{DeviceTrait, HostTrait};
use std::time::{Duration, Instant};
pub fn devices() -> Vec<String> {
    cpal::default_host()
        .output_devices()
        .map(|ds| {
            ds.filter_map(|d| d.description().ok().map(|n| n.name().to_owned()))
                .collect()
        })
        .unwrap_or_default()
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    None,
    Restore,
    Quit,
    Updates,
}
pub struct Tray {
    icon: tray_icon::TrayIcon,
    show: tray_icon::menu::MenuId,
    quit: tray_icon::menu::MenuId,
    updates: tray_icon::menu::MenuItem,
    update_label: String,
    last: Instant,
    levels: [f32; 2],
    frame: Frame,
    pub ticks: u64,
    pub uploads: u64,
}
impl Tray {
    pub fn new() -> anyhow::Result<Self> {
        use tray_icon::{
            Icon, TrayIconBuilder,
            menu::{Menu, MenuItem},
        };
        let menu = Menu::new();
        let show = MenuItem::new("Show Radio Crash", true, None);
        let quit = MenuItem::new("Quit Radio Crash", true, None);
        let updates = MenuItem::new("Check for updates", true, None);
        menu.append(&show)?;
        menu.append(&updates)?;
        menu.append(&quit)?;
        let frame = Frame::render([-60.; 2], [-60.; 2], Self::layout());
        let icon = TrayIconBuilder::new()
            .with_tooltip("Radio Crash · L/R audio meters · click to restore")
            .with_icon(Icon::from_rgba(
                frame.rgba.clone(),
                frame.width,
                frame.height,
            )?)
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(false)
            .with_menu_on_right_click(true)
            .build()?;
        Ok(Self {
            icon,
            show: show.id().clone(),
            quit: quit.id().clone(),
            updates,
            update_label: "Check for updates".into(),
            last: Instant::now(),
            levels: [-60.; 2],
            frame,
            ticks: 0,
            uploads: 0,
        })
    }
    fn layout() -> Layout {
        if cfg!(target_os = "macos") {
            Layout::MenuBar
        } else {
            Layout::NotificationArea
        }
    }
    pub fn poll(&self) -> TrayAction {
        let mut action = TrayAction::None;
        while let Ok(event) = tray_icon::menu::MenuEvent::receiver().try_recv() {
            if event.id == *self.updates.id() {
                return TrayAction::Updates;
            }
            if event.id == self.show {
                action = TrayAction::Restore;
            }
            if event.id == self.quit {
                return TrayAction::Quit;
            }
        }
        while let Ok(event) = tray_icon::TrayIconEvent::receiver().try_recv() {
            use tray_icon::{MouseButton, MouseButtonState, TrayIconEvent};
            match event {
                TrayIconEvent::Click {
                    id,
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
                | TrayIconEvent::DoubleClick {
                    id,
                    button: MouseButton::Left,
                    ..
                } if id == *self.icon.id() => action = TrayAction::Restore,
                _ => {}
            }
        }
        action
    }
    pub fn set_update(&mut self, version: Option<&str>) {
        let label = version
            .map(|v| format!("Update available: {v}"))
            .unwrap_or_else(|| "Check for updates".into());
        if label != self.update_label {
            self.updates.set_text(&label);
            self.update_label = label;
        }
    }
    pub fn update(
        &mut self,
        meters: &Meters,
        running: bool,
        reference: f32,
        fast_vu: bool,
        response: crate::dsp::VuResponse,
    ) {
        let elapsed = self.last.elapsed();
        if elapsed < Duration::from_millis(30) {
            return;
        }
        self.last = Instant::now();
        self.ticks += 1;
        let selected = meters.vu(response);
        for c in 0..2 {
            self.levels[c] = crate::dsp::follow(
                self.levels[c],
                if running { selected.level[c] } else { -60. },
                elapsed.as_secs_f32(),
                if fast_vu { 180. } else { 120. },
            );
        }
        let frame = Frame::render_calibrated(
            self.levels,
            if running { selected.held } else { [-60.; 2] },
            Self::layout(),
            reference,
        );
        if frame.rgba == self.frame.rgba {
            return;
        }
        match tray_icon::Icon::from_rgba(frame.rgba.clone(), frame.width, frame.height)
            .map_err(anyhow::Error::from)
            .and_then(|i| self.icon.set_icon(Some(i)).map_err(anyhow::Error::from))
        {
            Ok(()) => {
                self.uploads += 1;
                self.frame = frame;
            }
            Err(e) => log::warn!("Tray meter: {e}"),
        }
    }
    pub fn export(&self, path: &std::path::Path) -> anyhow::Result<()> {
        image::save_buffer(
            path,
            &self.frame.rgba,
            self.frame.width,
            self.frame.height,
            image::ColorType::Rgba8,
        )?;
        Ok(())
    }
}

/// A tray-only receiver should not leave a running-app button in the Dock.
pub fn set_tray_mode(hidden: bool) {
    #[cfg(target_os = "macos")]
    {
        if let Some(mtm) = objc2::MainThreadMarker::new() {
            use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
            let app = NSApplication::sharedApplication(mtm);
            if !app.setActivationPolicy(if hidden {
                NSApplicationActivationPolicy::Accessory
            } else {
                NSApplicationActivationPolicy::Regular
            }) {
                log::warn!("Could not switch Dock visibility");
            }
        }
    }
    #[cfg(not(target_os = "macos"))]
    let _ = hidden;
}
pub fn is_tray_mode() -> bool {
    #[cfg(target_os = "macos")]
    {
        use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
        objc2::MainThreadMarker::new().is_some_and(|mtm| {
            NSApplication::sharedApplication(mtm).activationPolicy()
                == NSApplicationActivationPolicy::Accessory
        })
    }
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

// egui-winit enables an undecorated DWM shadow automatically. winit implements
// that shadow with a one-physical-pixel non-client top inset. Our photographed
// chassis already has its own perimeter, so retain a fully client-area window.
#[cfg(target_os = "windows")]
fn configure_borderless_receiver(window: &winit::window::Window) {
    use winit::platform::windows::WindowExtWindows;
    window.set_undecorated_shadow(false);
}

#[cfg(target_os = "windows")]
pub fn configure_receiver_window(cc: &eframe::CreationContext<'_>) {
    if let Some(window) = cc.winit_window() {
        configure_borderless_receiver(window);
        log::info!(
            "Windows receiver: borderless, no native shadow inset, client {:?}, scale {}",
            window.inner_size(),
            window.scale_factor()
        );
    }
}

#[cfg(all(test, target_os = "windows"))]
mod windows_window_tests {
    use super::*;
    use winit::{
        dpi::LogicalSize,
        event_loop::EventLoop,
        platform::windows::{EventLoopBuilderExtWindows, WindowAttributesExtWindows},
        window::Window,
    };

    #[test]
    #[allow(deprecated)] // Hidden windows need no render/event loop in this Win32 geometry test.
    fn borderless_receiver_client_origin_matches_outer_origin() {
        let event_loop = EventLoop::builder()
            .with_any_thread(true)
            .build()
            .expect("Windows event loop");
        for size in [(780., 286.), (1200., 440.)] {
            let window = event_loop
                .create_window(
                    Window::default_attributes()
                        .with_visible(false)
                        .with_decorations(false)
                        .with_transparent(true)
                        .with_resizable(true)
                        .with_inner_size(LogicalSize::new(size.0, size.1))
                        // Reproduce egui-winit's automatic starting shadow before our fix.
                        .with_undecorated_shadow(true),
                )
                .expect("hidden native Windows receiver");
            configure_borderless_receiver(&window);
            assert!(!window.is_decorated());
            assert_eq!(
                window.inner_position().unwrap(),
                window.outer_position().unwrap(),
                "client and pointer origin must not include a hidden shadow inset"
            );
            assert_eq!(
                window.inner_size(),
                window.outer_size(),
                "no native frame around the client area"
            );
        }
    }
}
