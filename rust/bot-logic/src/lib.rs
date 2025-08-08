use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    console_log!("WASM module initialized.");
    Ok(())
}

#[wasm_bindgen]
pub struct Bot {
    state_controller: StateController,
}

#[wasm_bindgen]
impl Bot {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Bot {
        console_log!("New bot created from Rust.");
        Bot {
            state_controller: StateController::new(),
        }
    }

    pub fn start(&self) {
        self.state_controller.start();
    }

    pub fn stop(&self) {
        self.state_controller.stop();
    }
}

pub struct StateController {
    is_activated: bool,
}

impl StateController {
    pub fn new() -> StateController {
        StateController {
            is_activated: false,
        }
    }

    pub fn start(&self) {
        console_log!("StateController started.");
        // In the future, this will start the state machine loop.
    }

    pub fn stop(&self) {
        console_log!("StateController stopped.");
    }
}
