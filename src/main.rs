use std::collections::{HashMap, HashSet};
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

use graph::dcel::spanning_tree::SpanningTree;
use graph::dcel::vertex::VertexId;
use graph::dcel_file_writer::JsDataWriter;

use graph::mis_finder::{find_mis, find_mis_exhaustive};
use graph::nice_tree_decomp::NiceTreeDecomposition;

use graph::quick_graph::QuickGraph;
use graph::sub_dcel::SubDcel;
use graph::{Dcel, DcelBuilder};

use crate::graph::mis_finder::find_connected_vertices;
use crate::graph::node_relations::NodeRelations;
use crate::graph::reductions::isolated_clique_reduction::{
    do_isolated_clique_reductions, transfer_isolated_clique, IsolatedClique,
};
use crate::graph::reductions::nodal_fold_reduction::{
    do_nodal_fold_reductions, transfer_nodal_fold_reductions, NodalFold,
};
use crate::graph::reductions::twin_reduction::{
    do_twin_reductions, transfer_twin_reductions, TwinReduction,
};
use crate::graph::reductions::{ApplicableReduction, Reductions};
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

fn write_web_file(filename: &str, dcel: &Dcel, result: MISResult) {
    let mut writer = JsDataWriter::new(filename, dcel, result);
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
    AllWithTD,
    Exhaustive { reduce_input: Vec<Reduction> },
}

#[derive(Debug)]
pub struct MISResult {
    timings: Vec<(String, Duration)>,
    total_time: Duration,
    result: Vec<VertexId>,
    k: usize,
    i: usize,
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

fn mis_for_whole_graph(
    graph: &SubDcel,
    spanning_tree: &SpanningTree,
    watch: &mut Stopwatch,
) -> Result<Vec<VertexId>, Box<dyn Error>> {
    println!("Solving whole graph");
    watch.start("WholeGraph");
    let mut builder = SubTDBuilder::new(&graph, &spanning_tree, 0);
    let td = ApproximatedTD::from(&mut builder);
    let td = TreeDecomposition::from(&td);
    let ntd = NiceTreeDecomposition::from(&td);

    // find_mis(&graph.adjacency_matrix(), &ntd).map(|(set, size)| set.into_iter().collect())
    let result = find_mis(&graph.dcel.adjacency_matrix(), &ntd);
    watch.stop();

    match result {
        Ok((mis, size)) => {
            println!("mis: {mis:?}, size: {size}");
            Ok(mis.into_iter().collect::<Vec<VertexId>>())
        }
        Err(e) => {
            println!("Error: {e}");
            Err(Box::new(e))
        }
    }
}

fn mis_with_donut(
    graph: &Dcel,
    spanning_tree: &SpanningTree,
    ptas_config: &PTASConfig,
    watch: &mut Stopwatch,
) -> Result<(usize, Vec<VertexId>), Box<dyn Error>> {
    let mut best_i = 0;
    let mut best_mis = vec![];
    for i in 0..=ptas_config.k {
        println!("Approximation: i: {i}");
        watch.start(format!("Approximation: i={i:?}").as_str());
        // TODO use spanning tree to find donuts

        let donuts = graph.find_donuts_for_k(ptas_config.k, i, &spanning_tree)?;
        for donut_reductions in ptas_config.reduce_donuts.clone() {
            // TODO: apply donut reduction on DCEL builders
        }

        let mut mis_for_i = vec![];
        for (i, donut) in donuts.iter().enumerate() {
            // continue;
            println!("Donut {i}: ");
            // donut
            //     .vertex_mapping
            //     .iter()
            //     .for_each(|&v| println!("global v{v}"));
            let mut td_b = SubTDBuilder::new(&donut, &spanning_tree, donut.min_lvl.unwrap());
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
                    mis.into_iter().for_each(|v| mis_for_i.push(v));
                }
                Err(e) => {
                    println!("Error: {e}")
                }
            };
            // panic!("fuuuu u");
        }

        println!("mis: {mis_for_i:?}, size: {}", mis_for_i.len());
        assert!(find_connected_vertices(
            &HashSet::from_iter(mis_for_i.iter().copied()),
            &graph.adjacency_matrix(),
        )
        .is_empty());

        if mis_for_i.len() > best_mis.len() {
            best_mis.clear();
            mis_for_i.iter().for_each(|i| best_mis.push(*i));
            best_i = i;
        }

        watch.stop();
    }
    Ok((best_i, best_mis))
}

fn reduce_input_graph(
    mut dcel_builder: &mut DcelBuilder,
    mut quick_graph: &mut QuickGraph,
    reductions: &Vec<Reduction>,
    mut vertex_ids: &mut HashMap<VertexId, VertexId>,
) -> Reductions {
    let mut found_reductions: Reductions = Reductions::default();

    for input_reduction in reductions.iter() {
        match input_reduction {
            Reduction::NodalFold => {
                found_reductions.nodal_folds = do_nodal_fold_reductions(&mut quick_graph);
                found_reductions.nodal_folds.iter().for_each(|nodal_fold| {
                    nodal_fold.reduce_dcel_builder(&mut dcel_builder, &mut vertex_ids)
                });
            }
            Reduction::IsolatedClique => {
                found_reductions.isolated_cliques = do_isolated_clique_reductions(&mut quick_graph);
                found_reductions
                    .isolated_cliques
                    .iter()
                    .for_each(|isolated_clique| {
                        isolated_clique.reduce_dcel_builder(&mut dcel_builder, &mut vertex_ids)
                    });
            }
            Reduction::Twin => {
                found_reductions.twins = do_twin_reductions(&mut quick_graph);
                found_reductions.twins.iter().for_each(|twin_reduction| {
                    twin_reduction.reduce_dcel_builder(&mut dcel_builder, &mut vertex_ids)
                });
            }
        };
    }
    found_reductions
}

fn transfer_reductions(
    reduce_input: Vec<Reduction>,
    mut reductions: &mut Reductions,
    mut independence_set: &mut Vec<VertexId>,
    vertex_ids: &HashMap<VertexId, VertexId>,
) {
    /* reconstruct original vertex indices */
    let mut inverted_ids: HashMap<VertexId, VertexId> = HashMap::new();
    vertex_ids
        .iter()
        .for_each(|(&original_index, &recent_index)| {
            inverted_ids.insert(recent_index, original_index);
        });

    for i in 0..independence_set.len() {
        independence_set[i] = inverted_ids[&independence_set[i]];
    }

    for i in 0..reduce_input.len() {
        let input_reduction = &reduce_input[reduce_input.len() - i - 1];
        match input_reduction {
            Reduction::NodalFold => {
                transfer_nodal_fold_reductions(&mut independence_set, &mut reductions.nodal_folds);
            }
            Reduction::IsolatedClique => {
                transfer_isolated_clique(&mut independence_set, &mut reductions.isolated_cliques);
            }
            Reduction::Twin => {
                transfer_twin_reductions(&mut independence_set, &mut reductions.twins)
            }
        }
    }
}

fn find_max_independent_set(
    mut dcel_builder: &mut DcelBuilder,
    mut quick_graph: &mut QuickGraph,
    scheme: Scheme,
) -> Result<MISResult, Box<dyn Error>> {
    let mut watch = Stopwatch::new();
    let start_time = Instant::now();

    /* initialize table with vertex indices */
    let mut vertex_ids: HashMap<VertexId, VertexId> = HashMap::new();
    (0..quick_graph.adjacency.len()).for_each(|vertex| {
        vertex_ids.insert(vertex, vertex);
    });

    let graph: Dcel = dcel_builder.build();

    let mut k = 0;
    let mut best_i = 0;

    let result = match scheme {
        Scheme::PTAS {
            config: ptas_config,
        } => {
            k = ptas_config.k;
            watch.start("Applying approximations");

            let mut input_reductions: Reductions = reduce_input_graph(
                &mut dcel_builder,
                &mut quick_graph,
                &ptas_config.reduce_input,
                &mut vertex_ids,
            );

            watch.stop();

            watch.start("Find Rings");
            // let _rings = graph.find_rings();
            watch.stop();

            let root = 0;
            // build spanning tree
            watch.start("Spanning Tree");
            let spanning_tree = graph.spanning_tree(root);
            watch.stop();

            let mut result = if ptas_config.k > spanning_tree.max_level() {
                let subdcel =
                    &graph.find_donuts_for_k(usize::MAX - 1, usize::MAX - 1, &spanning_tree)?[0];
                mis_for_whole_graph(&subdcel, &spanning_tree, &mut watch).unwrap()
            } else {
                let (i, best_mis) =
                    mis_with_donut(&graph, &spanning_tree, &ptas_config, &mut watch).unwrap();
                best_i = i;
                best_mis
            };

            transfer_reductions(
                ptas_config.reduce_input,
                &mut input_reductions,
                &mut result,
                &vertex_ids,
            );

            result
        }

        Scheme::Exhaustive {
            reduce_input: input_reductions,
        } => {
            let mut found_reductions: Reductions = reduce_input_graph(
                &mut dcel_builder,
                &mut quick_graph,
                &input_reductions,
                &mut vertex_ids,
            );
            let root = 0;
            let spanning_tree = graph.spanning_tree(root);
            k = spanning_tree.max_level();
            let mut result: Vec<VertexId> = find_mis_exhaustive(&graph.adjacency_matrix())
                .map(|(mis, _)| mis.into_iter().collect::<Vec<_>>())?;
            transfer_reductions(
                input_reductions,
                &mut found_reductions,
                &mut result,
                &vertex_ids,
            );
            result
        }

        Scheme::AllWithTD => {
            let root = 0;
            watch.start("Spanning Tree");
            let spanning_tree = graph.spanning_tree(root);
            k = spanning_tree.max_level();
            let subdcel =
                &graph.find_donuts_for_k(usize::MAX - 1, usize::MAX - 1, &spanning_tree)?[0];
            println!("{:?}", subdcel.vertex_mapping);
            println!("{}", subdcel.vertex_mapping.len());
            watch.stop();
            mis_for_whole_graph(&subdcel, &spanning_tree, &mut watch)?
        }
    };

    let end_time = Instant::now();
    let total_time = end_time.duration_since(start_time);

    Ok(MISResult {
        timings: watch.timings,
        total_time,
        result,
        k,
        i: best_i,
    })
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum CliScheme {
    PTAS,
    AllWithTD,
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
        CliScheme::AllWithTD => Scheme::AllWithTD {},
    };

    let mut dcel_b = match read_graph_file_into_dcel_builder(args.input.to_str().unwrap()) {
        Ok(result) => result,
        Err(error) => panic!("Failed to read graph file into DCEL: {:?}", error),
    };

    let mut dcel_b2 = dcel_b.clone();

    let mut quick_graph: QuickGraph =
        match read_graph_file_into_quick_graph(args.input.to_str().unwrap()) {
            Ok(result) => result,
            Err(error) => panic!("Failed to read graph file into quick graph: {:?}", error),
        };

    // write_web_file(&args.output, &dcel);
    let mis_result = match find_max_independent_set(&mut dcel_b, &mut quick_graph, scheme) {
        Ok(result) => result,
        Err(error) => panic!("Failed computing maximum independent set: {error:?}"),
    };

    println!("Result: {mis_result:?}");
    println!("Size of MIS: {:?}", mis_result.result.len());

    //    let args: Vec<String> = env::args().collect();

    //    for a in BfsIter::new(&dcel, 0) {
    //        print!("{:?}", a);
    //    }

    //    //dcel.triangulate();

    let dcel2 = &dcel_b2.build();
    // println!("FACES: {:?}", dcel.faces().len());
    // println!("FACES 2: {:?}", dcel.faces().len());

    write_web_file(&args.output, &dcel2, mis_result);
    //    // let mut dg = DualGraph::new(&st);
    //    // dg.build();

    //    println!("{:?}", dcel);
}
