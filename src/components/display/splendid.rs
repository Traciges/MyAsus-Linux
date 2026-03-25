use std::path::PathBuf;

use gtk4 as gtk;
use relm4::adw;
use relm4::adw::prelude::*;
use relm4::prelude::*;

use super::helpers::{icc_profil_anwenden, setup_icc_profiles};
use crate::services::config::AppConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum SplendidProfil {
    #[default]
    Vivid,
    VividMovie,
    Dimmed,
}

impl SplendidProfil {
    pub fn dateiname(&self) -> &'static str {
        match self {
            Self::Vivid => "vivid.icm",
            Self::VividMovie => "vivid-movie.icm",
            Self::Dimmed => "dimmed.icm",
        }
    }
}

pub struct SplendidModel {
    aktuelles_profil: SplendidProfil,
    icc_basis_pfad: Option<PathBuf>,
    check_vivid: gtk::CheckButton,
    check_vivid_movie: gtk::CheckButton,
    check_dimmed: gtk::CheckButton,
}

#[derive(Debug)]
pub enum SplendidMsg {
    ProfilWechseln(SplendidProfil),
}

#[derive(Debug)]
pub enum SplendidCommandOutput {
    IccBereit(PathBuf),
    ProfilAngewendet(SplendidProfil),
    Fehler(String),
}

#[relm4::component(pub)]
impl Component for SplendidModel {
    type Init = ();
    type Input = SplendidMsg;
    type Output = ();
    type CommandOutput = SplendidCommandOutput;

    view! {
        adw::PreferencesGroup {
            set_title: "Splendid",

            add = &adw::ActionRow {
                set_title: "Lebendig",
                add_prefix = &model.check_vivid.clone(),
                set_activatable_widget: Some(&model.check_vivid),
            },

            add = &adw::ActionRow {
                set_title: "Kino/Film",
                add_prefix = &model.check_vivid_movie.clone(),
                set_activatable_widget: Some(&model.check_vivid_movie),
            },

            add = &adw::ActionRow {
                set_title: "Gedimmt",
                add_prefix = &model.check_dimmed.clone(),
                set_activatable_widget: Some(&model.check_dimmed),
            },
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let check_vivid = gtk::CheckButton::new();
        let check_vivid_movie = gtk::CheckButton::new();
        let check_dimmed = gtk::CheckButton::new();

        check_vivid_movie.set_group(Some(&check_vivid));
        check_dimmed.set_group(Some(&check_vivid));
        check_vivid.set_active(true);

        for (btn, profil) in [
            (&check_vivid, SplendidProfil::Vivid),
            (&check_vivid_movie, SplendidProfil::VividMovie),
            (&check_dimmed, SplendidProfil::Dimmed),
        ] {
            let sender = sender.clone();
            btn.connect_toggled(move |b| {
                if b.is_active() {
                    sender.input(SplendidMsg::ProfilWechseln(profil));
                }
            });
        }

        sender.command(|out, shutdown| {
            shutdown
                .register(async move {
                    match setup_icc_profiles().await {
                        Ok(pfad) => out.emit(SplendidCommandOutput::IccBereit(pfad)),
                        Err(e) => out.emit(SplendidCommandOutput::Fehler(e)),
                    }
                })
                .drop_on_shutdown()
        });

        let config = AppConfig::load();
        let gespeichertes_profil = config.splendid_profil;

        match gespeichertes_profil {
            SplendidProfil::Vivid => {}
            SplendidProfil::VividMovie => check_vivid_movie.set_active(true),
            SplendidProfil::Dimmed => check_dimmed.set_active(true),
        }

        let model = SplendidModel {
            aktuelles_profil: gespeichertes_profil,
            icc_basis_pfad: None,
            check_vivid,
            check_vivid_movie,
            check_dimmed,
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: SplendidMsg, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            SplendidMsg::ProfilWechseln(profil) => {
                if profil == self.aktuelles_profil {
                    return;
                }
                self.aktuelles_profil = profil;
                AppConfig::update(|c| c.splendid_profil = profil);

                if let Some(basis) = self.icc_basis_pfad.clone() {
                    profil_anwenden(profil, basis, &sender);
                } else {
                    eprintln!("ICC-Basispfad noch nicht bereit");
                }
            }
        }
    }

    fn update_cmd(
        &mut self,
        msg: SplendidCommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            SplendidCommandOutput::IccBereit(pfad) => {
                eprintln!("ICC-Profile bereit unter {}", pfad.display());
                profil_anwenden(self.aktuelles_profil, pfad.clone(), &sender);
                self.icc_basis_pfad = Some(pfad);
            }
            SplendidCommandOutput::ProfilAngewendet(profil) => {
                eprintln!("Splendid: Profil {:?} angewendet", profil);
            }
            SplendidCommandOutput::Fehler(e) => {
                eprintln!("Fehler: {e}");
            }
        }
    }
}

fn profil_anwenden(
    profil: SplendidProfil,
    basis: PathBuf,
    sender: &ComponentSender<SplendidModel>,
) {
    let dateiname = profil.dateiname().to_string();
    sender.command(move |out, shutdown| {
        shutdown
            .register(async move {
                match icc_profil_anwenden(&dateiname, &basis).await {
                    Ok(()) => out.emit(SplendidCommandOutput::ProfilAngewendet(profil)),
                    Err(e) => out.emit(SplendidCommandOutput::Fehler(e)),
                }
            })
            .drop_on_shutdown()
    });
}
