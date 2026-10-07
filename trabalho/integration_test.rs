use megastore::bench::gerar_dados_sinteticos;
use megastore::store::MegaStore;

#[test]
fn fluxo_completo() {
    let mut store = MegaStore::new();

    let games = store.cadastrar_categoria("Games");
    let casa = store.cadastrar_categoria("Casa");

    let controle = store.cadastrar_produto("Controle sem fio", games, 250.0, 4.6);
    let headset = store.cadastrar_produto("Headset gamer", games, 300.0, 4.4);
    let luminaria = store.cadastrar_produto("Luminária de mesa", casa, 80.0, 4.0);

    let joao = store.cadastrar_cliente("João");
    let maria = store.cadastrar_cliente("Maria");

    store.registrar_compra(joao, controle);
    store.registrar_compra(maria, controle);
    store.registrar_compra(maria, headset);

    let recs = store.recomendar(joao, 5);

    assert!(recs.iter().any(|r| r.product_id == headset), "headset devia aparecer pro João");
    assert!(!recs.iter().any(|r| r.product_id == controle), "já comprado, não devia aparecer");
    assert!(!recs.iter().any(|r| r.product_id == luminaria));
}

#[test]
fn grafo_sintetico_tem_o_tamanho_esperado() {
    let dados = gerar_dados_sinteticos(5, 200, 50, 4, 7);

    assert_eq!(dados.store.graph.node_count(), 5 + 200 + 50);
    assert_eq!(dados.produtos.len(), 200);
    assert_eq!(dados.clientes.len(), 50);
}

#[test]
fn top_n_em_dataset_grande() {
    let dados = gerar_dados_sinteticos(10, 2_000, 500, 6, 123);
    let recs = dados.store.recomendar(dados.clientes[0], 10);
    assert!(recs.len() <= 10);
}

#[test]
fn compras_repetidas_nao_inflam_o_score() {
    let mut store = MegaStore::new();
    let cat = store.cadastrar_categoria("Cat");
    let a = store.cadastrar_produto("A", cat, 10.0, 4.0);
    let b = store.cadastrar_produto("B", cat, 10.0, 4.0);
    let u1 = store.cadastrar_cliente("U1");
    let u2 = store.cadastrar_cliente("U2");

    store.registrar_compra(u1, a);
    store.registrar_compra(u2, a);
    store.registrar_compra(u2, b);
    let antes = store.recomendar(u1, 5)[0].score;

    // repetir a mesma compra não pode mudar nada
    store.registrar_compra(u2, b);
    store.registrar_compra(u2, b);
    let depois = store.recomendar(u1, 5)[0].score;

    assert_eq!(antes, depois);
}

#[test]
fn recomendacao_por_produto() {
    let mut store = MegaStore::new();
    let cat = store.cadastrar_categoria("Cat");
    let mouse = store.cadastrar_produto("Mouse", cat, 100.0, 4.5);
    let pad = store.cadastrar_produto("Mousepad", cat, 40.0, 4.2);
    store.definir_similaridade(mouse, pad, 0.8);

    let recs = store.recomendar_por_produto(mouse, 5);
    assert_eq!(recs[0].product_id, pad);
}
