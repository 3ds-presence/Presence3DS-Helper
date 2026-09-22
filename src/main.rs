use ctru::prelude::*;
use ctru::services::ps::Ps;
use ctru::services::soc::Soc;

mod app;
mod model;
mod utils;
mod views;

use app::App;
fn main() {
    let mut apt = Apt::new().unwrap();
    let gfx = Gfx::new().unwrap();
    let _soc = Soc::new().unwrap();
    let _ps = Ps::new().unwrap();
    let mut app = App::new(&gfx);

    while apt.main_loop() {
        gfx.wait_for_vblank();
        apt.set_home_allowed(!app.blocks_home());

        if app.app_loop() {
            break;
        }
    }

    drop(app);
    drop(gfx);
}
