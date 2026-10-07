use crate::models::{Edge, EdgeType, NodeData, NodeId};
use std::collections::{HashMap, HashSet, VecDeque};

pub struct Graph {
    nodes: HashMap<NodeId, NodeData>,
    adjacency: HashMap<NodeId, Vec<Edge>>,
    name_index: HashMap<String, NodeId>,
    next_id: NodeId,
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            nodes: HashMap::new(),
            adjacency: HashMap::new(),
            name_index: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn add_node(&mut self, data: NodeData) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;

        // nomes repetidos: fica o primeiro
        self.name_index.entry(data.name().to_string()).or_insert(id);
        self.nodes.insert(id, data);
        self.adjacency.insert(id, Vec::new());
        id
    }

    pub fn add_edge(&mut self, from: NodeId, to: NodeId, edge_type: EdgeType, weight: f64) {
        self.adjacency
            .entry(from)
            .or_default()
            .push(Edge { target: to, edge_type, weight });
    }

    pub fn add_undirected_edge(&mut self, a: NodeId, b: NodeId, edge_type: EdgeType, weight: f64) {
        self.add_edge(a, b, edge_type, weight);
        self.add_edge(b, a, edge_type, weight);
    }

    pub fn has_edge(&self, from: NodeId, to: NodeId, tipo: EdgeType) -> bool {
        self.neighbors_of_type(from, tipo).any(|e| e.target == to)
    }

    pub fn get_node(&self, id: NodeId) -> Option<&NodeData> {
        self.nodes.get(&id)
    }

    pub fn id_by_name(&self, name: &str) -> Option<NodeId> {
        self.name_index.get(name).copied()
    }

    pub fn neighbors(&self, id: NodeId) -> &[Edge] {
        self.adjacency.get(&id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn neighbors_of_type(&self, id: NodeId, tipo: EdgeType) -> impl Iterator<Item = &Edge> + '_ {
        self.neighbors(id).iter().filter(move |e| e.edge_type == tipo)
    }

    /// Retorna (nó, distância), sem incluir o nó inicial.
    pub fn bfs(&self, start: NodeId, max_depth: usize) -> Vec<(NodeId, usize)> {
        self.bfs_impl(start, max_depth, None)
    }

    /// Igual ao `bfs`, mas só atravessa arestas do tipo informado.
    pub fn bfs_tipo(&self, start: NodeId, max_depth: usize, tipo: EdgeType) -> Vec<(NodeId, usize)> {
        self.bfs_impl(start, max_depth, Some(tipo))
    }

    fn bfs_impl(&self, start: NodeId, max_depth: usize, tipo: Option<EdgeType>) -> Vec<(NodeId, usize)> {
        let mut visited = HashSet::new();
        let mut fila = VecDeque::new();
        let mut result = Vec::new();

        visited.insert(start);
        fila.push_back((start, 0));

        while let Some((atual, depth)) = fila.pop_front() {
            if depth > 0 {
                result.push((atual, depth));
            }
            if depth == max_depth {
                continue;
            }
            for e in self.neighbors(atual) {
                if tipo.map_or(false, |t| e.edge_type != t) {
                    continue;
                }
                if visited.insert(e.target) {
                    fila.push_back((e.target, depth + 1));
                }
            }
        }
        result
    }

    pub fn dfs(&self, start: NodeId, max_depth: usize) -> Vec<NodeId> {
        let mut visited = HashSet::new();
        let mut result = Vec::new();
        self.dfs_rec(start, 0, max_depth, &mut visited, &mut result);
        result
    }

    fn dfs_rec(
        &self,
        atual: NodeId,
        depth: usize,
        max_depth: usize,
        visited: &mut HashSet<NodeId>,
        result: &mut Vec<NodeId>,
    ) {
        if !visited.insert(atual) {
            return;
        }
        if depth > 0 {
            result.push(atual);
        }
        if depth == max_depth {
            return;
        }
        for e in self.neighbors(atual) {
            self.dfs_rec(e.target, depth + 1, max_depth, visited, result);
        }
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.adjacency.values().map(Vec::len).sum()
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn produto(nome: &str) -> NodeData {
        NodeData::Product {
            name: nome.to_string(),
            category_id: 0,
            price: 10.0,
            rating: 4.0,
        }
    }

    #[test]
    fn insere_e_busca() {
        let mut g = Graph::new();
        let id = g.add_node(produto("Mouse"));
        assert_eq!(g.get_node(id).unwrap().name(), "Mouse");
        assert_eq!(g.node_count(), 1);
    }

    #[test]
    fn aresta_so_vai_num_sentido() {
        let mut g = Graph::new();
        let a = g.add_node(produto("A"));
        let b = g.add_node(produto("B"));
        g.add_edge(a, b, EdgeType::SimilarTo, 1.0);

        assert_eq!(g.neighbors(a).len(), 1);
        assert!(g.neighbors(b).is_empty());
    }

    #[test]
    fn bfs_respeita_profundidade() {
        let mut g = Graph::new();
        let a = g.add_node(produto("A"));
        let b = g.add_node(produto("B"));
        let c = g.add_node(produto("C"));
        g.add_edge(a, b, EdgeType::SimilarTo, 1.0);
        g.add_edge(b, c, EdgeType::SimilarTo, 1.0);

        assert_eq!(g.bfs(a, 1), vec![(b, 1)]);
        let r = g.bfs(a, 2);
        assert_eq!(r.len(), 2);
        assert!(r.contains(&(c, 2)));
    }

    #[test]
    fn bfs_tipo_ignora_outras_arestas() {
        let mut g = Graph::new();
        let a = g.add_node(produto("A"));
        let b = g.add_node(produto("B"));
        let c = g.add_node(produto("C"));
        g.add_edge(a, b, EdgeType::SimilarTo, 1.0);
        g.add_edge(a, c, EdgeType::Purchased, 1.0);

        assert_eq!(g.bfs_tipo(a, 3, EdgeType::SimilarTo), vec![(b, 1)]);
    }

    #[test]
    fn has_edge_olha_o_tipo() {
        let mut g = Graph::new();
        let a = g.add_node(produto("A"));
        let b = g.add_node(produto("B"));
        g.add_edge(a, b, EdgeType::SimilarTo, 1.0);

        assert!(g.has_edge(a, b, EdgeType::SimilarTo));
        assert!(!g.has_edge(a, b, EdgeType::Purchased));
        assert!(!g.has_edge(b, a, EdgeType::SimilarTo));
    }

    #[test]
    fn bfs_nao_entra_em_loop() {
        let mut g = Graph::new();
        let a = g.add_node(produto("A"));
        let b = g.add_node(produto("B"));
        g.add_undirected_edge(a, b, EdgeType::SimilarTo, 1.0);

        assert_eq!(g.bfs(a, 5), vec![(b, 1)]);
    }

    #[test]
    fn dfs_chega_nos_dois() {
        let mut g = Graph::new();
        let a = g.add_node(produto("A"));
        let b = g.add_node(produto("B"));
        let c = g.add_node(produto("C"));
        g.add_edge(a, b, EdgeType::SimilarTo, 1.0);
        g.add_edge(b, c, EdgeType::SimilarTo, 1.0);

        let r = g.dfs(a, 2);
        assert!(r.contains(&b) && r.contains(&c));
    }
}
