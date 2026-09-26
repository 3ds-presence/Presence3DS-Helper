// Presence3DS Helper — Helper Homebrew for Presence3DS
// Copyright (C) 2026 3DS Presence - LeonLeBreton
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

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
