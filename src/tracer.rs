use wasm_bindgen::prelude::*;
use visioncortex::PathSimplifyMode;
use visioncortex::color_clusters::{IncrementalBuilder, Clusters, Runner, RunnerConfig, HIERARCHICAL_MAX};


use serde::Deserialize;
use super::util;

#[derive(Debug, Deserialize)]
pub struct Parameters {
  pub mode: String,
  pub hierarchical: String,
  pub corner_threshold: f64,
  pub length_threshold: f64,
  pub max_iterations: usize,
  pub splice_threshold: f64,
  pub filter_speckle: usize,
  pub color_precision: i32,
  pub layer_difference: i32,
  pub path_precision: u32,
}

#[wasm_bindgen]
pub struct Tracer {
  stage: Stage,
  counter: usize,
  mode: PathSimplifyMode,
  params: Parameters,
}

pub enum Stage {
  New,
  Clustering(IncrementalBuilder),
  Reclustering(IncrementalBuilder),
  Vectorize(Clusters),
}

impl Tracer {
  pub fn new(params: Parameters) -> Self {
    Self {
      width,
      height,
      stage: Stage::New,
      counter: 0,
      mode: util::path_simplify_mode(&params.mode),
      params,
    }
  }
}

#[wasm_bindgen]
impl Tracer {

  pub fn init(&mut self) {
    let width = self.width;
    let height = self.height;
    let image = self.image;
    let runner = Runner::new(RunnerConfig {
      diagonal: self.params.layer_difference == 0,
      hierarchical: HIERARCHICAL_MAX,
      batch_size: 25600,
      good_min_area: self.params.filter_speckle,
      good_max_area: (width * height) as usize,
      is_same_color_a: self.params.color_precision,
      is_same_color_b: 1,
      deepen_diff: self.params.layer_difference,
      hollow_neighbours: 1,
    }, image);
    self.stage = Stage::Clustering(runner.start());
  }

  pub fn tick(&mut self) -> bool {
    match &mut self.stage {
      Stage::New => {
        panic!("uninitialized");
      },
      Stage::Clustering(builder) => {
        println!("Clustering tick");
        if builder.tick() {
          match self.params.hierarchical.as_str() {
            "stacked" => {
              self.stage = Stage::Vectorize(builder.result());
            },
            "cutout" => {
              let clusters = builder.result();
              let view = clusters.view();
              let image = view.to_color_image();
              let runner = Runner::new(RunnerConfig {
                diagonal: false,
                hierarchical: 64,
                batch_size: 25600,
                good_min_area: 0,
                good_max_area: (image.width * image.height) as usize,
                is_same_color_a: 0,
                is_same_color_b: 1,
                deepen_diff: 0,
                hollow_neighbours: 0,
              }, image);
              self.stage = Stage::Reclustering(runner.start());
            },
            _ => panic!("unknown hierarchical `{}`", self.params.hierarchical)
          }
        }
        false
      },
      Stage::Reclustering(builder) => {
        println!("Reclustering tick");
        if builder.tick() {
          self.stage = Stage::Vectorize(builder.result())
        }
        false
      },
      Stage::Vectorize(clusters) => {
        let view = clusters.view();
        if self.counter < view.clusters_output.len() {
          println!("Vectorize tick");
          let cluster = view.get_cluster(view.clusters_output[self.counter]);
          let paths = cluster.to_compound_path(
            &view, false, self.mode,
            self.params.corner_threshold,
            self.params.length_threshold,
            self.params.max_iterations,
            self.params.splice_threshold
          );
          println!("Paths $1", paths);
          self.counter += 1;
          false
        } else {
          println!("Vectorize done");
          true
        }
      }
    }
  }

  pub fn progress(&self) -> i32 {
    (match &self.stage {
      Stage::New => {
        0
      },
      Stage::Clustering(builder) => {
        builder.progress() / 2
      },
      Stage::Reclustering(_builder) => {
        50
      },
      Stage::Vectorize(clusters) => {
        50 + 50 * self.counter as u32 / clusters.view().clusters_output.len() as u32
      }
    }) as i32
  }

}
