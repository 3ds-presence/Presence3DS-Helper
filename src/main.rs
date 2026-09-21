use ctru::prelude::*;

mod app;
mod model;
mod utils;
mod views;

use app::App;
fn main() {
    let apt = Apt::new().unwrap();
    let gfx = Gfx::new().unwrap();
    let mut app = App::new(&gfx);

    while apt.main_loop() {
        gfx.wait_for_vblank();

        if app.app_loop() {
            break;
        }
    }

    drop(app);
    drop(gfx);
}
