use std::path::{Path, PathBuf};
use std::time::Instant;

use eframe::egui::Widget;
use eframe::{egui, epi};
use memeroute::dsn::pcb_to_session::PcbToSession;
use memeroute::model::pcb::Pcb;
use memeroute::route::router::{apply_route_result, Router};
use serde::{Deserialize, Serialize};

use crate::pcb::pcb_view::PcbView;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default)]
struct State {
    filename: String,
}

impl Default for State {
    fn default() -> Self {
        Self { filename: "data/left.dsn".to_string() }
    }
}

#[derive(Debug, Clone)]
pub struct MemerouteGui {
    s: State,
    pcb: Pcb,
    pcb_view: PcbView,
    data_path: PathBuf,
}

impl MemerouteGui {
    pub fn new<P: AsRef<Path>>(pcb: Pcb, data_path: P) -> Self {
        let pcb_view = PcbView::new(pcb.clone(), pcb.bounds());
        Self { s: State::default(), pcb, pcb_view, data_path: data_path.as_ref().into() }
    }
}

impl epi::App for MemerouteGui {
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, &self.s);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");


            if ui.button("Route").clicked() {
                let router = Router::new(self.pcb.clone());
                let start = Instant::now();
                let resp = router.route(router.rand_net_order()).unwrap();
                // let resp = router.run_ga().unwrap();
                println!(
                    "Route result succeeded: {}, {} wires {} vias, time: {:?}",
                    !resp.failed,
                    resp.wires.len(),
                    resp.vias.len(),
                    Instant::now().duration_since(start)
                );
                apply_route_result(&mut self.pcb, &resp);

                let output_path = self.data_path.with_extension("ses");
                let ses = PcbToSession::new(self.pcb.clone()).convert().unwrap();
                std::fs::write(output_path, ses).unwrap();

                // Update pcb view.
                self.pcb_view.set_pcb(self.pcb.clone());
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.pcb_view.ui(ui);
        });
    }

    fn name(&self) -> &str {
        "memeroute gui"
    }

    fn setup(
        &mut self,
        _ctx: &egui::Context,
        _frame: &epi::Frame,
        storage: Option<&dyn epi::Storage>,
    ) {
        if let Some(storage) = storage {
            self.s = epi::get_value(storage, epi::APP_KEY).unwrap_or_default();
        }
    }
}
