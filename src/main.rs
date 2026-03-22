mod backend;
mod components;
mod services;

use components::battery::BatteryModel;
use components::display::OledCareModel;
use components::display::SplendidModel;
use components::fan::FanModel;
use relm4::adw;
use relm4::adw::prelude::*;
use relm4::prelude::*;

struct AppModel {
    battery: Controller<BatteryModel>,
    fan: Controller<FanModel>,
    oled_care: Controller<OledCareModel>,
    splendid: Controller<SplendidModel>,
}

#[relm4::component]
impl SimpleComponent for AppModel {
    type Init = ();
    type Input = ();
    type Output = ();

    view! {
        adw::ApplicationWindow {
            set_title: Some("MyAsus Control Center"),
            set_default_size: (1200, 800),

            #[wrap(Some)]
            set_content = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {},

                #[wrap(Some)]
                set_content = &adw::PreferencesPage {
                    #[local_ref]
                    add = battery_widget -> adw::PreferencesGroup {},
                    #[local_ref]
                    add = fan_widget -> adw::PreferencesGroup {},
                    #[local_ref]
                    add = oled_care_widget -> adw::PreferencesGroup {},
                    #[local_ref]
                    add = splendid_widget -> adw::PreferencesGroup {},
                },
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let battery = BatteryModel::builder().launch(()).detach();
        let fan = FanModel::builder().launch(()).detach();
        let oled_care = OledCareModel::builder().launch(()).detach();
        let splendid = SplendidModel::builder().launch(()).detach();

        let model = AppModel {
            battery,
            fan,
            oled_care,
            splendid,
        };
        let battery_widget = model.battery.widget();
        let fan_widget = model.fan.widget();
        let oled_care_widget = model.oled_care.widget();
        let splendid_widget = model.splendid.widget();
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }
}

fn main() {
    let app = RelmApp::new("de.guido.myasus-linux");
    adw::StyleManager::default().set_color_scheme(adw::ColorScheme::PreferDark);
    app.run::<AppModel>(());
}
