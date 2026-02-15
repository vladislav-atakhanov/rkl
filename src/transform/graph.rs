use std::{
    collections::{BinaryHeap, HashMap},
    fmt::Debug,
};

#[derive(Debug)]
pub struct Node<'a> {
    pub weight: usize,
    pub deps: Vec<&'a str>,
}
pub fn priority_topo_sort<'a>(graph: &HashMap<&'a str, Node<'a>>) -> Result<Vec<&'a str>, String> {
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut reverse_graph: HashMap<&str, Vec<&str>> = HashMap::new();

    // Инициализация
    for (&name, node) in graph {
        in_degree.entry(name).or_insert(0);

        for &dep in &node.deps {
            reverse_graph.entry(dep).or_default().push(name);
            *in_degree.entry(name).or_insert(0) += 1;
        }
    }

    // Min-heap по weight
    let mut heap = BinaryHeap::new();

    for (&name, &deg) in &in_degree {
        if deg == 0 {
            let weight = graph[name].weight;
            heap.push((weight, name));
        }
    }

    let mut result = Vec::new();

    while let Some((_, node)) = heap.pop() {
        result.push(node);

        if let Some(dependents) = reverse_graph.get(node) {
            for &dep in dependents {
                let deg = in_degree
                    .get_mut(dep)
                    .ok_or(format!("Unknown dependency {:?}", dep))?;
                *deg -= 1;

                if *deg == 0 {
                    let weight = graph[dep].weight;
                    heap.push((weight, dep));
                }
            }
        }
    }

    if result.len() != graph.len() {
        return Err("Cycle detected".into());
    }

    Ok(result)
}
