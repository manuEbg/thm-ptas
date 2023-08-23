# thm-ptas
This repository is used for our GFSP SS23 Project.

The ptas approximation scheme only works correctly on graphs that are generated via the python script with the circular option.

To find independent sets on other graphs you need to use one of the other schemes ( EXHAUSTIVE or ALL-WITH-TD )

If you are interested in how this algorithm works, please have a look at the [wiki](https://github.com/manuEbg/thm-ptas/wiki) or the [presentation](https://github.com/manuEbg/thm-ptas/blob/main/THM%20PTAS-2.pdf). 

## How to use?

To execute the Program with a given example graph use 

```
cargo run -- ptas data/exp.graph 
```

## The input data


The input of this program is the planar embedding of a (planar) undirected graph.

The embedding should be provided as a text-file and follow this specification:


```
<Number of vertices>
<Number of edges>
<Source vertex> <Target vertex>
<Source vertex> <Target vertex>
<Source vertex> <Target vertex>
...
```

It is important, that the embedding contains both directions for each arc of the graph.
Also the arcs have to be in counterclockwise order for each source vertex.

If you want more information about this format or more graphs, you can find both [here](http://www.inf.udec.cl/~jfuentess/datasets/graphs.php).

### Generating Input Data

To generate random planar graphs and their embedding, you can use the python script located in this repository.
For example: 
```
 python3 ./generate.py --nodes 25 --rings 2 --nprob 0.8 --eprob 0.7 data/exp.graph --type random
```

## The Visualization

After successfully running the program you can inspect your results by opening `web/index.html` with a browser.
