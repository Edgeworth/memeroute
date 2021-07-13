use eframe::egui::Widget;
use eframe::{egui, epi};
use memeroute::model::pcb::Pcb;
use memeroute::model::shape::rt::Rt;
use memeroute::route::router::Router;
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
}

impl MemerouteGui {
    pub fn new(pcb: Pcb) -> Self {
        let pcb_view = PcbView::new(pcb.clone(), pcb.bounds());
        Self { s: Default::default(), pcb, pcb_view }
    }
}

impl epi::App for MemerouteGui {
    fn name(&self) -> &str {
        "Memeroute"
    }

    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &mut epi::Frame<'_>,
        storage: Option<&dyn epi::Storage>,
    ) {
        if let Some(storage) = storage {
            self.s = epi::get_value(storage, epi::APP_KEY).unwrap_or_default();
        }
    }

    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, &self.s);
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            if ui.button("Route").clicked() {
                let mut router = Router::new(self.pcb.clone());
                let resp = router.route().unwrap();

                for wire in resp.wires.into_iter() {
                    self.pcb.add_wire(wire);
                }

                for via in resp.vias.into_iter() {
                    self.pcb.add_via(via);
                }

                // Update pcb view.
                self.pcb_view.set_pcb(self.pcb.clone());
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.pcb_view.ui(ui);
        });
    }
}
