use std::path::PathBuf;

use gtk4 as gtk;
use relm4::adw;
use relm4::adw::prelude::*;
use relm4::prelude::*;

use super::helpers::{icm_profil_anwenden, icm_profil_reset, setup_icm_profiles};
use crate::services::config::AppConfig;

fn dateiname_fuer_index(index: u32) -> Option<&'static str> {
    match index {
        1 => Some("ASUS_sRGB.icm"),
        2 => Some("ASUS_DCIP3.icm"),
        3 => Some("ASUS_DisplayP3.icm"),
        _ => None,
    }
}

// ── Farbskala (ComboRow) ─────────────────────────────────────────────────────

pub struct FarbskalaModel {
    farbskala_index: u32,
    icm_basis_pfad: Option<PathBuf>,
}

impl FarbskalaModel {
    fn farbskala_beschreibung(&self) -> &'static str {
        match self.farbskala_index {
            1 => "Standardfarben für Webinhalte und sRGB-Geräte.",
            2 => "Standardfarben für digitales Kino (DCI-P3).",
            3 => "Umfangreichere, naturgetreue Farben für kreative Arbeit.",
            _ => "Standardmäßig lebendige, optimierte Farben.",
        }
    }
}

#[derive(Debug)]
pub enum FarbskalaMsg {
    FarbskalaWechseln(u32),
}

#[derive(Debug)]
pub enum FarbskalaCommandOutput {
    IcmBereit(PathBuf),
    ProfilAngewendet(u32),
    Fehler(String),
}

#[relm4::component(pub)]
impl Component for FarbskalaModel {
    type Init = ();
    type Input = FarbskalaMsg;
    type Output = ();
    type CommandOutput = FarbskalaCommandOutput;

    view! {
        adw::PreferencesGroup {
            add = &adw::ComboRow {
                set_title: "Farbskala",
                #[watch]
                set_subtitle: model.farbskala_beschreibung(),
                set_model: Some(&gtk::StringList::new(&[
                    "Nativ",
                    "sRGB",
                    "DCI-P3",
                    "Display P3",
                ])),
                #[watch]
                set_selected: model.farbskala_index,
                connect_selected_notify[sender] => move |row| {
                    sender.input(FarbskalaMsg::FarbskalaWechseln(row.selected()));
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let config = AppConfig::load();

        let model = FarbskalaModel {
            farbskala_index: config.farbskala_index,
            icm_basis_pfad: None,
        };

        let widgets = view_output!();

        sender.command(|out, shutdown| {
            shutdown
                .register(async move {
                    match setup_icm_profiles().await {
                        Ok(pfad) => out.emit(FarbskalaCommandOutput::IcmBereit(pfad)),
                        Err(e) => out.emit(FarbskalaCommandOutput::Fehler(e)),
                    }
                })
                .drop_on_shutdown()
        });

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: FarbskalaMsg, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            FarbskalaMsg::FarbskalaWechseln(index) => {
                if index == self.farbskala_index {
                    return;
                }
                self.farbskala_index = index;
                AppConfig::update(|c| c.farbskala_index = index);

                if let Some(basis) = self.icm_basis_pfad.clone() {
                    profil_anwenden(index, basis, &sender);
                } else {
                    eprintln!("ICM-Basispfad noch nicht bereit");
                }
            }
        }
    }

    fn update_cmd(
        &mut self,
        msg: FarbskalaCommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            FarbskalaCommandOutput::IcmBereit(pfad) => {
                eprintln!("ICM-Profile bereit unter {}", pfad.display());
                profil_anwenden(self.farbskala_index, pfad.clone(), &sender);
                self.icm_basis_pfad = Some(pfad);
            }
            FarbskalaCommandOutput::ProfilAngewendet(index) => {
                eprintln!("Farbskala: Profil-Index {} angewendet", index);
            }
            FarbskalaCommandOutput::Fehler(e) => {
                eprintln!("Fehler: {e}");
            }
        }
    }
}

fn profil_anwenden(index: u32, basis: PathBuf, sender: &ComponentSender<FarbskalaModel>) {
    sender.command(move |out, shutdown| {
        shutdown
            .register(async move {
                let ergebnis = match dateiname_fuer_index(index) {
                    None => icm_profil_reset().await,
                    Some(dateiname) => icm_profil_anwenden(dateiname, &basis).await,
                };
                match ergebnis {
                    Ok(()) => out.emit(FarbskalaCommandOutput::ProfilAngewendet(index)),
                    Err(e) => out.emit(FarbskalaCommandOutput::Fehler(e)),
                }
            })
            .drop_on_shutdown()
    });
}

// ── Zielmodus (SwitchRow) ────────────────────────────────────────────────────

pub struct ZielmodusModel {
    zielmodus_aktiv: bool,
}

#[derive(Debug)]
pub enum ZielmodusMsg {
    ZielmodusWechseln(bool),
}

#[relm4::component(pub)]
impl SimpleComponent for ZielmodusModel {
    type Init = ();
    type Input = ZielmodusMsg;
    type Output = ();

    view! {
        adw::PreferencesGroup {
            add = &adw::SwitchRow {
                set_title: "Zielmodus",
                set_subtitle: "Bei Aktivierung bleibt die Helligkeit des aktiven Fensters unverändert, während nicht aktive Bereiche abgedunkelt werden, um Strom zu sparen.\n\nHinweis: Wenn Sie sowohl \"Zielmodus\" als auch \"Windows-Taskleiste im Desktop-Modus automatisch ausblenden\" aktivieren, aktiviert Windows automatisch die Funktion \"Nicht stören\".",
                #[watch]
                set_active: model.zielmodus_aktiv,
                connect_active_notify[sender] => move |switch| {
                    sender.input(ZielmodusMsg::ZielmodusWechseln(switch.is_active()));
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let config = AppConfig::load();

        let model = ZielmodusModel {
            zielmodus_aktiv: config.zielmodus_aktiv,
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: ZielmodusMsg, _sender: ComponentSender<Self>) {
        match msg {
            ZielmodusMsg::ZielmodusWechseln(aktiv) => {
                if aktiv == self.zielmodus_aktiv {
                    return;
                }
                self.zielmodus_aktiv = aktiv;
                AppConfig::update(|c| c.zielmodus_aktiv = aktiv);
            }
        }
    }
}
