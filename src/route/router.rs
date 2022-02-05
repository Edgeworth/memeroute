use std::sync::Mutex;

use eyre::Result;
use memega::cfg::{
    Cfg, Crossover, Duplicates, Mutation, Niching, Replacement, Stagnation, Survival,
};
use memega::eval::Evaluator;
use memega::ops::crossover::{crossover_cycle, crossover_order, crossover_pmx};
use memega::ops::distance::kendall_tau;
use memega::ops::mutation::{mutate_insert, mutate_inversion, mutate_scramble, mutate_swap};
use memega::run_evolve_debug;
use memega::runner::Runner;
use rand::prelude::SliceRandom;
use rand::Rng;

use crate::model::pcb::{Pcb, Via, Wire};
use crate::model::primitive::rect::Rt;
use crate::name::Id;
use crate::route::grid::GridRouter;

pub trait RouteStrategy {
    fn route(&mut self) -> Result<RouteResult>;
}

#[derive(Debug, Default, Clone)]
pub struct RouteResult {
    pub wires: Vec<Wire>,
    pub vias: Vec<Via>,
    pub debug_rts: Vec<Rt>,
    pub failed: bool,
}

impl RouteResult {
    pub fn merge(&mut self, r: RouteResult) {
        self.wires.extend(r.wires);
        self.vias.extend(r.vias);
        self.debug_rts.extend(r.debug_rts);
        self.failed |= r.failed;
    }
}

#[derive(Debug)]
pub struct Router {
    pcb: Mutex<Pcb>,
}

impl Clone for Router {
    fn clone(&self) -> Self {
        Self::new(self.pcb.lock().unwrap().clone())
    }
}

impl Router {
    pub fn new(pcb: Pcb) -> Self {
        Self { pcb: Mutex::new(pcb) }
    }

    pub fn rand_net_order(&self) -> Vec<Id> {
        let mut net_order: Vec<_> = self.pcb.lock().unwrap().nets().map(|v| v.id).collect();
        //net_order.shuffle(rand::thread_rng());
        net_order.sort_unstable();
        net_order
    }

    pub fn route(&self, net_order: Vec<Id>) -> Result<RouteResult> {
        let mut grid = GridRouter::new(self.pcb.lock().unwrap().clone(), net_order);
        grid.route()
    }

    pub fn run_ga(&self) -> Result<RouteResult> {
        let cfg = Cfg::new(32)
            .with_mutation(Mutation::Adaptive)
            .with_crossover(Crossover::Adaptive)
            .with_survival(Survival::SpeciesTopProportion(0.1))
            // .with_species(Species::TargetNumber(100))
            .with_niching(Niching::None)
            .with_stagnation(Stagnation::ContinuousAfter(200))
            .with_replacement(Replacement::ReplaceChildren(0.5))
            .with_duplicates(Duplicates::DisallowDuplicates)
            .with_par_fitness(true)
            .with_par_dist(true);

        let net_order: Vec<_> = self.pcb.lock().unwrap().nets().map(|v| v.id).collect();
        let genfn = move || {
            let mut rand_order = net_order.clone();
            rand_order.shuffle(&mut rand::thread_rng());
            rand_order
        };

        let runner = Runner::new(self.clone(), cfg, genfn);
        let order = run_evolve_debug(runner, 1, 1, 1)?.nth(0).genome.clone();
        self.route(order)
    }
}

impl Evaluator for Router {
    type Genome = Vec<Id>;
    const NUM_CROSSOVER: usize = 4;
    const NUM_MUTATION: usize = 4;


    fn crossover(&self, s1: &mut Self::Genome, s2: &mut Self::Genome, idx: usize) {
        match idx {
            0 => {} // Do nothing.
            1 => crossover_pmx(s1, s2),
            2 => crossover_order(s1, s2),
            3 => crossover_cycle(s1, s2),
            _ => panic!("unknown crossover strategy"),
        };
    }

    fn mutate(&self, s: &mut Self::Genome, rate: f64, idx: usize) {
        let mut r = rand::thread_rng();
        if r.gen::<f64>() > rate {
            return;
        }
        match idx {
            0 => mutate_swap(s),
            1 => mutate_insert(s),
            2 => mutate_scramble(s),
            3 => mutate_inversion(s),
            _ => panic!("unknown mutation strategy"),
        }
    }

    fn fitness(&self, s: &Self::Genome, _: usize) -> f64 {
        let res = self.route(s.clone()).unwrap();
        let mut cost = 0.0;
        if res.failed {
            cost += 1000.0;
        }
        cost += res.vias.len() as f64 * 10.0;
        // TODO: Count wire lengths
        1.0 / (1.0 + cost)
    }

    fn distance(&self, s1: &Self::Genome, s2: &Self::Genome) -> f64 {
        kendall_tau(s1, s2).unwrap() as f64
    }
}

pub fn apply_route_result(pcb: &mut Pcb, r: &RouteResult) {
    for wire in r.wires.iter() {
        pcb.add_wire(wire.clone());
    }
    for via in r.vias.iter() {
        pcb.add_via(via.clone());
    }
    for rt in r.debug_rts.iter() {
        pcb.add_debug_rt(*rt);
    }
}
