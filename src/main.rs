#![warn(clippy::all, rust_2018_idioms)]
#![allow(warnings)]
/*
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
*/
// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
//#[tokio::main]
fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Unable to create Runtime");

    let _enter = rt.enter();

    std::thread::spawn(move || {
        rt.block_on(async {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
            }
        });
    });

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Whiteboard",
        native_options,
        Box::new(|cc| Ok(Box::new(online_whiteboard_frontend::WhiteboardApp::new(cc)))),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(Online_Whiteboard_Frontend::WhiteboardApp::new(cc)))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
