use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};
pub mod graph;

#[macro_use]
pub mod logger;

use arboretum_td::tree_decomposition::TreeDecomposition;
use clap::Parser;
use graph::approximated_td::{ApproximatedTD, SubTDBuilder, TDBuilder};

use graph::dcel::vertex::VertexId;
use graph::dcel_file_writer::JsDataWriter;

use graph::mis_finder::find_mis;
use graph::nice_tree_decomp::NiceTreeDecomposition;

use graph::quick_graph::QuickGraph;
use graph::{Dcel, DcelBuilder};

use crate::graph::node_relations::NodeRelations;
use crate::graph::tree_decomposition::td_write_to_dot;

fn read_graph_file_into_quick_graph(filename: &str) -> Result<QuickGraph, String> {
    return if let Ok(mut lines) = read_lines(filename) {
        /* create datastructure for graph */
        let mut graph: QuickGraph;
        let vertex_count: usize = lines.next().unwrap().unwrap().parse().unwrap();
        graph = QuickGraph::new(vertex_count);
        graph.edge_count = lines.next().unwrap().unwrap().parse().unwrap();

        /* read in edges */
        for _ in 0..(2 * graph.edge_count) {
            let edge = lines.next().unwrap().unwrap();
            let mut edge = edge.split(" ");
            let u: usize = edge.next().unwrap().parse().unwrap();
            let v: usize = edge.next().unwrap().parse().unwrap();
            if let Some(ref mut adjacency_u) = &mut graph.adjacency[u] {
                adjacency_u.push(v);
            } else {
                return Err(String::from("Could not push neighbor to adjacency list. "));
            }
        }

        Ok(graph)
    } else {
        Err(format!("Could not open file {}", filename))
    };
}

fn read_graph_file_into_dcel_builder(filename: &str) -> Result<DcelBuilder, String> {
    return if let Ok(mut lines) = read_lines(filename) {
        /* build datastructure for DECL */
        let mut dcel_builder: DcelBuilder;
        if let Some(Ok(_)) = lines.next() {
            dcel_builder = DcelBuilder::new();
            let edge_count: usize = lines.next().unwrap().unwrap().parse().unwrap();

            /* read edges into DECL */
            for _ in 0..(2 * edge_count) {
                let edge = lines.next().unwrap().unwrap();
                let mut edge = edge.split(" ");
                let u: usize = edge.next().unwrap().parse().unwrap();
                let v: usize = edge.next().unwrap().parse().unwrap();
                dcel_builder.push_arc(u, v);
            }
        } else {
            return Err(String::from("Error: Could not read line. "));
        }

        /* return DCEL-Builder */
        Ok(dcel_builder)
    } else {
        Err(format!("Could not open file {}", filename))
    };
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn write_web_file(filename: &str, dcel: &Dcel) {
    let mut writer = JsDataWriter::new(filename, dcel);
    writer.write_data()
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum Reduction {
    Twin,
    IsolatedClique,
    NodalFold,
}

struct PTASConfig {
    k: usize,
    exact_donut_tree_decomposition: bool,
    reduce_input: Vec<Reduction>,
    reduce_donuts: Vec<Reduction>,
}

enum Scheme {
    PTAS { config: PTASConfig },
    Exhaustive { reduce_input: Vec<Reduction> },
}

#[derive(Debug)]
struct MISResult {
    timings: Vec<(String, Duration)>,
    total_time: Duration,
    result: Vec<VertexId>,
}

struct Stopwatch {
    current: String,
    current_start: Instant,
    timings: Vec<(String, Duration)>,
}

impl Stopwatch {
    fn new() -> Self {
        Self {
            timings: vec![],
            current: String::from(""),
            current_start: Instant::now(),
        }
    }

    fn start(&mut self, period: &str) {
        self.current = String::from(period);
        self.current_start = Instant::now();
    }

    fn stop(&mut self) {
        let stopping_time = Instant::now();
        let duration = stopping_time.duration_since(self.current_start);
        self.timings.push((self.current.clone(), duration));
    }
}

fn find_max_independent_set(graph: &Dcel, scheme: Scheme) -> Result<MISResult, Box<dyn Error>> {
    let mut watch = Stopwatch::new();
    let start_time = Instant::now();

    let _result = match scheme {
        Scheme::PTAS {
            config: ptas_config,
        } => {
            watch.start("Applying approximations");
            for input_reduction in ptas_config.reduce_input {
                // TODO: apply input reduction
            }
            watch.stop();

            watch.start("Find Rings");
            // let _rings = graph.find_rings();
            watch.stop();

            let root = 0;
            // build spanning tree
            watch.start("Spanning Tree");
            let spanning_tree = graph.spanning_tree(root);
            watch.stop();
            for i in 0..ptas_config.k {
                println!("Approximation: i: {i}");
                watch.start(format!("Approximation: i={i:?}").as_str());
                // TODO use spanning tree to find donuts

                let donuts = graph.find_donuts_for_k(ptas_config.k, i, &spanning_tree)?;
                for donut_reductions in ptas_config.reduce_donuts.clone() {
                    // TODO: apply donut reduction on DCEL builders
                }

                for (i, donut) in donuts.iter().enumerate() {
                    // continue;
                    println!("Donut {i}: ");
                    // donut
                    //     .vertex_mapping
                    //     .iter()
                    //     .for_each(|&v| println!("global v{v}"));
                    let mut td_b =
                        SubTDBuilder::new(&donut, &spanning_tree, donut.min_lvl.unwrap());
                    let td = ApproximatedTD::from(&mut td_b);
                    if td.bags().len() == 0 {
                        println!("bags of donut are {i} empty");
                        continue;
                        //todo add all nodes of donut to MIS
                    }

                    let decomp = TreeDecomposition::from(&td);
                    let td_rels = NodeRelations::new(&decomp);
                    let td_path = format!("./td_{i}.dot");

                    let mut td_out = File::create(td_path.as_str()).unwrap();
                    td_write_to_dot("td", &mut td_out, &decomp, &td_rels).unwrap();
                    Command::new("dot")
                        .args([
                            "-Tpdf",
                            td_path.as_str(),
                            "-o",
                            format!("./td_{i}.pdf").as_str(),
                        ])
                        .spawn()
                        .expect("dot command did not work.");

                    // TODO: generate MIS for this donut and add to list
                    let ntd = NiceTreeDecomposition::from(&decomp);
                    let ntd_rels = NodeRelations::new(&ntd.td);
                    assert!(ntd.validate(&decomp, &ntd_rels));

                    let ntd_path = format!("./ntd_{i}.dot");

                    let mut ntd_out = File::create(ntd_path.as_str()).unwrap();
                    td_write_to_dot("ntd", &mut ntd_out, &ntd.td, &ntd_rels).unwrap();
                    Command::new("dot")
                        .args([
                            "-Tpdf",
                            ntd_path.as_str(),
                            "-o",
                            format!("./ntd_{i}.pdf").as_str(),
                        ])
                        .spawn()
                        .expect("dot command did not work.");
                    match find_mis(&graph.adjacency_matrix(), &ntd) {
                        Ok((mis, size)) => {
                            println!("mis: {mis:?}, size: {size}");
                        }
                        Err(e) => {
                            println!("Error: {e}")
                        }
                    };
                    // panic!("fuuuu u");
                }

                // TODO:
                watch.stop();
            }

            // Choose best MIS and return that
        }
        Scheme::Exhaustive {
            reduce_input: input_reductions,
        } => {}
    };

    let end_time = Instant::now();
    let total_time = end_time.duration_since(start_time);

    Ok(MISResult {
        timings: watch.timings,
        total_time,
        result: vec![],
    })
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum CliScheme {
    PTAS,
    Exhaustive,
}

#[derive(Debug, Parser)]
struct CliArguments {
    #[arg(value_enum)]
    scheme: CliScheme,

    #[arg(long, default_value_t = 1)]
    k: usize,

    #[arg(short = 'E')]
    exact_donut_tree_decomposition: bool,

    #[arg(short = 'R')]
    input_reductions: Vec<Reduction>,

    #[arg(short = 'D')]
    donut_reductions: Vec<Reduction>,

    #[arg(value_hint = clap::ValueHint::DirPath)]
    input: PathBuf,

    #[arg(default_value_t = String::from("data/test.js"))]
    output: String,
}

fn main() {
    let args = CliArguments::parse();
    println!("{args:?}");

    let scheme = match args.scheme {
        CliScheme::Exhaustive => Scheme::Exhaustive {
            reduce_input: args.input_reductions,
        },
        CliScheme::PTAS => Scheme::PTAS {
            config: PTASConfig {
                k: args.k,
                exact_donut_tree_decomposition: args.exact_donut_tree_decomposition,
                reduce_input: args.input_reductions,
                reduce_donuts: args.donut_reductions,
            },
        },
    };

    let mut dcel_b = match read_graph_file_into_dcel_builder(args.input.to_str().unwrap()) {
        Ok(result) => result,
        Err(error) => panic!("Failed to read graph file into DCEL: {error:?}"),
    };

    let dcel = dcel_b.build();

    // write_web_file(&args.output, &dcel);
    let mis_result = match find_max_independent_set(&dcel, scheme) {
        Ok(result) => result,
        Err(error) => panic!("Failed computing maximum independent set: {error:?}"),
    };

    println!("Result: {mis_result:?}");

    //    let args: Vec<String> = env::args().collect();

    //    for a in BfsIter::new(&dcel, 0) {
    //        print!("{:?}", a);
    //    }

    //    //dcel.triangulate();

    // write_web_file(&args.output, &dcel);
    //    // let mut dg = DualGraph::new(&st);
    //    // dg.build();

    //    println!("{:?}", dcel);
}
