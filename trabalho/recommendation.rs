use crate::graph::Graph;
use crate::models::{EdgeType, NodeId};
use std::collections::{HashMap, HashSet};

// pesos escolhidos na mão, dá pra ajustar depois
const PESO_COMPRA_EM_COMUM: f64 = 1.0;
const PESO_MESMA_CATEGORIA: f64 = 0.5;
const PROFUNDIDADE_SIMILARES: usize = 2;

#[derive(Debug, Clone, PartialEq)]
pub struct Recommendation {
    pub product_id: NodeId,
    pub name: String,
    pub score: f64,
}

pub fn recommend_for_customer(graph: &Graph, customer_id: NodeId, top_n: usize) -> Vec<Recommendation> {
    let comprados: HashSet<NodeId> = graph
        .neighbors_of_type(customer_id, EdgeType::Purchased)
        .map(|e| e.target)
        .collect();

    let mut scores: HashMap<NodeId, f64> = HashMap::new();

    for &prod in &comprados {
        // quem comprou isso também levou o quê?
        for comprador in graph.neighbors_of_type(prod, EdgeType::BoughtBy) {
            if comprador.target == customer_id {
                continue;
            }
            for compra in graph.neighbors_of_type(comprador.target, EdgeType::Purchased) {
                if !comprados.contains(&compra.target) {
                    *scores.entry(compra.target).or_default() += PESO_COMPRA_EM_COMUM;
                }
            }
        }

        // outros produtos da mesma categoria
        for cat in graph.neighbors_of_type(prod, EdgeType::BelongsTo) {
            for item in graph.neighbors_of_type(cat.target, EdgeType::Contains) {
                if item.target != prod && !comprados.contains(&item.target) {
                    *scores.entry(item.target).or_default() += PESO_MESMA_CATEGORIA;
                }
            }
        }

        // similaridade cadastrada na mão
        for sim in graph.neighbors_of_type(prod, EdgeType::SimilarTo) {
            if !comprados.contains(&sim.target) {
                *scores.entry(sim.target).or_default() += sim.weight;
            }
        }
    }

    montar_ranking(graph, scores, top_n)
}

/// Recomendação a partir de um produto: usa BFS pelas arestas de similaridade
/// (quanto mais longe, menos pontos) e soma quem comprou este produto e levou outros.
pub fn recommend_for_product(graph: &Graph, product_id: NodeId, top_n: usize) -> Vec<Recommendation> {
    let mut scores: HashMap<NodeId, f64> = HashMap::new();

    for (id, dist) in graph.bfs_tipo(product_id, PROFUNDIDADE_SIMILARES, EdgeType::SimilarTo) {
        *scores.entry(id).or_default() += 1.0 / dist as f64;
    }

    for comprador in graph.neighbors_of_type(product_id, EdgeType::BoughtBy) {
        for compra in graph.neighbors_of_type(comprador.target, EdgeType::Purchased) {
            if compra.target != product_id {
                *scores.entry(compra.target).or_default() += PESO_COMPRA_EM_COMUM;
            }
        }
    }

    montar_ranking(graph, scores, top_n)
}

fn montar_ranking(graph: &Graph, scores: HashMap<NodeId, f64>, top_n: usize) -> Vec<Recommendation> {
    let mut ranking: Vec<Recommendation> = scores
        .into_iter()
        .filter_map(|(id, score)| {
            graph.get_node(id).map(|d| Recommendation {
                product_id: id,
                name: d.name().to_string(),
                score,
            })
        })
        .collect();

    // desempata por id pra saída ficar estável entre execuções
    ranking.sort_by(|a, b| b.score.total_cmp(&a.score).then(a.product_id.cmp(&b.product_id)));
    ranking.truncate(top_n);
    ranking
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::MegaStore;

    fn cenario() -> (MegaStore, NodeId, NodeId, NodeId) {
        let mut store = MegaStore::new();
        let cat = store.cadastrar_categoria("Geral");

        let p1 = store.cadastrar_produto("P1", cat, 100.0, 4.5);
        let p2 = store.cadastrar_produto("P2", cat, 200.0, 4.7);
        let p3 = store.cadastrar_produto("P3", cat, 50.0, 4.2);

        let u1 = store.cadastrar_cliente("U1");
        let u2 = store.cadastrar_cliente("U2");

        store.registrar_compra(u1, p1);
        store.registrar_compra(u2, p1);
        store.registrar_compra(u2, p2);

        store.definir_similaridade(p1, p3, 0.8);

        (store, u1, p1, p2)
    }

    #[test]
    fn nao_recomenda_o_que_ja_comprou() {
        let (store, u1, p1, p2) = cenario();
        let recs = store.recomendar(u1, 5);

        assert!(recs.iter().any(|r| r.product_id == p2));
        assert!(!recs.iter().any(|r| r.product_id == p1));
    }

    #[test]
    fn sem_repetidos() {
        let (store, u1, _, _) = cenario();
        let mut ids: Vec<NodeId> = store.recomendar(u1, 10).iter().map(|r| r.product_id).collect();
        let antes = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), antes);
    }

    #[test]
    fn por_produto_junta_similares_e_coprados() {
        let (store, _, p1, p2) = cenario();
        let p3 = store.buscar_por_nome("P3").unwrap();
        let recs = store.recomendar_por_produto(p1, 10);

        assert!(recs.iter().any(|r| r.product_id == p2), "co-compra");
        assert!(recs.iter().any(|r| r.product_id == p3), "similar");
        assert!(!recs.iter().any(|r| r.product_id == p1));
    }

    #[test]
    fn respeita_top_n() {
        let (store, u1, _, _) = cenario();
        assert!(store.recomendar(u1, 1).len() <= 1);
    }

    #[test]
    fn compra_em_comum_pesa_mais_que_categoria() {
        let mut store = MegaStore::new();
        let cat = store.cadastrar_categoria("Cat");

        let base = store.cadastrar_produto("Base", cat, 10.0, 4.0);
        let forte = store.cadastrar_produto("Forte", cat, 10.0, 4.0);
        let fraco = store.cadastrar_produto("Fraco", cat, 10.0, 4.0);

        let u1 = store.cadastrar_cliente("U1");
        let u2 = store.cadastrar_cliente("U2");

        store.registrar_compra(u1, base);
        store.registrar_compra(u2, base);
        store.registrar_compra(u2, forte);

        let recs = store.recomendar(u1, 10);
        let score = |id| recs.iter().find(|r| r.product_id == id).unwrap().score;

        assert!(score(forte) > score(fraco));
    }
}
