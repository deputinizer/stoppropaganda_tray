  
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use trayicon::{MenuBuilder, TrayIconBuilder};

#[derive(Clone, Eq, PartialEq, Debug)]
enum Events {
    ClickTrayIcon,
    DoubleClickTrayIcon,
    Exit,
    ShowConsole,
    HideConsole,

}

pub fn tray_main() {
    let event_loop = EventLoop::<Events>::with_user_event();
    //let your_app_window = WindowBuilder::new().build(&event_loop).unwrap();
    let proxy = event_loop.create_proxy();
    let icon = include_bytes!("img/splogo16.ico");


    // Needlessly complicated tray icon with all the whistles and bells
    let _tray_icon = TrayIconBuilder::new()
        .sender_winit(proxy)
        .icon_from_buffer(icon)
        .tooltip("Sad Putin")
        .on_click(Events::ClickTrayIcon)
        .on_double_click(Events::DoubleClickTrayIcon)
        .menu(
            MenuBuilder::new()
                .item("Show console", Events::ShowConsole)
                .item("Hide console", Events::HideConsole)
                
                .separator()
                .item("E&xit", Events::Exit),
        )
        .build()
        .unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            // Main window events
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } => {
                let _ = window_id;
                //if window_id == your_app_window.id() => *control_flow = ControlFlow::Exit,
            }

            // User events
            Event::UserEvent(e) => match e {
                Events::Exit => *control_flow = ControlFlow::Exit,
                Events::ShowConsole => {
                    crate::console::show_console();
                }
                Events::HideConsole => {
                    crate::console::hide_console();
                }

                _e => {
                    //println!("Got event {:?}", _e)
                },
            },
            _ => (),
        }
    });
}