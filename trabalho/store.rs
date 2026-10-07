use crate::graph::Graph;
use crate::models::{EdgeType, NodeData, NodeId};
use crate::recommendation::{recommend_for_customer, recommend_for_product, Recommendation};

pub struct MegaStore {
    pub graph: Graph,
}

impl MegaStore {
    pub fn new() -> Self {
        MegaStore { graph: Graph::new() }
    }

    pub fn cadastrar_categoria(&mut self, nome: &str) -> NodeId {
        self.graph.add_node(NodeData::Category { name: nome.to_string() })
    }

    pub fn cadastrar_cliente(&mut self, nome: &str) -> NodeId {
        self.graph.add_node(NodeData::Customer { name: nome.to_string() })
    }

    pub fn cadastrar_produto(&mut self, nome: &str, categoria_id: NodeId, preco: f64, rating: f64) -> NodeId {
        let id = self.graph.add_node(NodeData::Product {
            name: nome.to_string(),
            category_id: categoria_id,
            price: preco,
            rating,
        });
        self.graph.add_edge(id, categoria_id, EdgeType::BelongsTo, 1.0);
        self.graph.add_edge(categoria_id, id, EdgeType::Contains, 1.0);
        id
    }

    /// Devolve false se a compra já existia (não cria aresta repetida,
    /// senão o score de recomendação seria inflado).
    pub fn registrar_compra(&mut self, cliente_id: NodeId, produto_id: NodeId) -> bool {
        if self.graph.has_edge(cliente_id, produto_id, EdgeType::Purchased) {
            return false;
        }
        self.graph.add_edge(cliente_id, produto_id, EdgeType::Purchased, 1.0);
        self.graph.add_edge(produto_id, cliente_id, EdgeType::BoughtBy, 1.0);
        true
    }

    pub fn definir_similaridade(&mut self, a: NodeId, b: NodeId, peso: f64) {
        self.graph.add_undirected_edge(a, b, EdgeType::SimilarTo, peso);
    }

    pub fn produto(&self, id: NodeId) -> Option<&NodeData> {
        self.graph.get_node(id)
    }

    pub fn buscar_por_nome(&self, nome: &str) -> Option<NodeId> {
        self.graph.id_by_name(nome)
    }

    fn listar_se(&self, filtro: impl Fn(&NodeData) -> bool) -> Vec<(NodeId, &NodeData)> {
        (0..self.graph.node_count())
            .filter_map(|id| self.graph.get_node(id).map(|d| (id, d)))
            .filter(|(_, d)| filtro(*d))
            .collect()
    }

    pub fn listar_produtos(&self) -> Vec<(NodeId, &NodeData)> {
        self.listar_se(NodeData::is_product)
    }

    pub fn listar_categorias(&self) -> Vec<(NodeId, &NodeData)> {
        self.listar_se(NodeData::is_category)
    }

    pub fn listar_clientes(&self) -> Vec<(NodeId, &NodeData)> {
        self.listar_se(NodeData::is_customer)
    }

    pub fn recomendar(&self, cliente_id: NodeId, top_n: usize) -> Vec<Recommendation> {
        recommend_for_customer(&self.graph, cliente_id, top_n)
    }

    pub fn recomendar_por_produto(&self, produto_id: NodeId, top_n: usize) -> Vec<Recommendation> {
        recommend_for_product(&self.graph, produto_id, top_n)
    }
}

impl Default for MegaStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn produto_fica_ligado_a_categoria() {
        let mut store = MegaStore::new();
        let cat = store.cadastrar_categoria("Livros");
        let prod = store.cadastrar_produto("Livro A", cat, 25.0, 4.8);

        match store.produto(prod).unwrap() {
            NodeData::Product { category_id, .. } => assert_eq!(*category_id, cat),
            outro => panic!("esperava produto, veio {:?}", outro),
        }
    }

    #[test]
    fn compra_repetida_nao_duplica_aresta() {
        let mut store = MegaStore::new();
        let cat = store.cadastrar_categoria("Cat");
        let prod = store.cadastrar_produto("Prod", cat, 10.0, 4.0);
        let cli = store.cadastrar_cliente("Cli");

        assert!(store.registrar_compra(cli, prod));
        assert!(!store.registrar_compra(cli, prod));

        assert_eq!(store.graph.neighbors_of_type(cli, EdgeType::Purchased).count(), 1);
        assert_eq!(store.graph.neighbors_of_type(prod, EdgeType::BoughtBy).count(), 1);
    }

    #[test]
    fn listar_produtos_ignora_o_resto() {
        let mut store = MegaStore::new();
        let cat = store.cadastrar_categoria("Cat");
        store.cadastrar_produto("Prod", cat, 10.0, 4.0);
        store.cadastrar_cliente("Cli");

        assert_eq!(store.listar_produtos().len(), 1);
    }
}
