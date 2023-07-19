/* decrease all vertices with a bigger index than a removed or merged vertex */
pub fn decrease_vertices(vertices: &Vec<usize>, vertex: usize) -> Vec<usize> {
    vertices.iter().
        map(|&vector_vertex| if vector_vertex > vertex { vector_vertex - 1} else { vector_vertex } ).
        collect()
}

/* increase all vertices with a bigger index than a restored vertex */
pub fn increase_vertices(vertices: &Vec<usize>, vertex: usize) -> Vec<usize> {
    vertices.iter().
        map(|&vector_vertex| if vector_vertex >= vertex { vector_vertex + 1} else { vector_vertex } ).
        collect()
}

/* update independence set when restoring removed or merged vertices */
pub fn restore_independence_set(
    independence_set: Vec<usize>,
    removed_vertices: Vec<usize>
) -> Vec<usize>
{
    /* prepare vertices for updating */
    let mut result = independence_set.clone();
    let mut current_removed_vertices: Vec<usize> = removed_vertices.clone();
    current_removed_vertices.reverse();

    /* update result and removed vertices for each removed vertex */
    for index in 0..current_removed_vertices.len() {
        result = increase_vertices(&result, current_removed_vertices[index]);
    }

    /* return result */
    result
}