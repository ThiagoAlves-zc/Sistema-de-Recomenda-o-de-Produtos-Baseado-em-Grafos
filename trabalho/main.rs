use megastore::bench::gerar_dados_sinteticos;
use megastore::models::NodeData;
use megastore::store::MegaStore;
use std::io::{self, Write};
use std::time::Instant;

fn ler_linha(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    buf.trim().to_string()
}

fn ler_numero(prompt: &str) -> f64 {
    loop {
        match ler_linha(prompt).parse() {
            Ok(v) => return v,
            Err(_) => println!("Número inválido."),
        }
    }
}

fn ler_inteiro(prompt: &str) -> usize {
    loop {
        match ler_linha(prompt).parse() {
            Ok(v) => return v,
            Err(_) => println!("Inteiro inválido."),
        }
    }
}

fn pausar() {
    ler_linha("\n[ENTER]");
}

fn eh_categoria(store: &MegaStore, id: usize) -> bool {
    store.produto(id).map_or(false, |d| d.is_category())
}

fn eh_cliente(store: &MegaStore, id: usize) -> bool {
    store.produto(id).map_or(false, |d| d.is_customer())
}

fn eh_produto(store: &MegaStore, id: usize) -> bool {
    store.produto(id).map_or(false, |d| d.is_product())
}

fn cadastrar_categoria(store: &mut MegaStore) {
    let nome = ler_linha("\nNome da categoria: ");
    if nome.is_empty() {
        return;
    }
    let id = store.cadastrar_categoria(&nome);
    println!("Categoria criada (ID {})", id);
}

fn cadastrar_produto(store: &mut MegaStore) {
    let categorias = store.listar_categorias();
    if categorias.is_empty() {
        println!("\nCadastre uma categoria antes.");
        return;
    }
    for (id, cat) in &categorias {
        println!("[{}] {}", id, cat.name());
    }

    let nome = ler_linha("\nNome do produto: ");
    let categoria_id = ler_inteiro("ID da categoria: ");
    if !eh_categoria(store, categoria_id) {
        println!("Categoria inválida.");
        return;
    }

    let preco = ler_numero("Preço: ");
    let rating = ler_numero("Rating (0-5): ");

    let id = store.cadastrar_produto(&nome, categoria_id, preco, rating);
    println!("Produto criado (ID {})", id);
}

fn cadastrar_cliente(store: &mut MegaStore) {
    let nome = ler_linha("\nNome do cliente: ");
    if nome.is_empty() {
        return;
    }
    let id = store.cadastrar_cliente(&nome);
    println!("Cliente criado (ID {})", id);
}

fn listar_produtos(store: &MegaStore) {
    let produtos = store.listar_produtos();
    if produtos.is_empty() {
        println!("\nNenhum produto ainda.");
        return;
    }
    println!("\n-- Produtos --");
    for (id, dado) in produtos {
        if let NodeData::Product { name, price, rating, .. } = dado {
            println!("[{}] {} - R$ {:.2} ({:.1})", id, name, price, rating);
        }
    }
}

fn registrar_compra(store: &mut MegaStore) {
    if store.listar_clientes().is_empty() || store.listar_produtos().is_empty() {
        println!("\nPrecisa ter cliente e produto cadastrados.");
        return;
    }

    let cliente = ler_inteiro("\nID do cliente: ");
    if !eh_cliente(store, cliente) {
        println!("Cliente inválido.");
        return;
    }

    let produto = ler_inteiro("ID do produto: ");
    if !eh_produto(store, produto) {
        println!("Produto inválido.");
        return;
    }

    if store.registrar_compra(cliente, produto) {
        println!("Compra registrada.");
    } else {
        println!("Essa compra já estava registrada.");
    }
}

fn definir_similaridade(store: &mut MegaStore) {
    let a = ler_inteiro("\nProduto A: ");
    let b = ler_inteiro("Produto B: ");

    if !eh_produto(store, a) || !eh_produto(store, b) {
        println!("ID inválido.");
        return;
    }

    let peso = ler_numero("Peso (0.0 a 1.0): ");
    store.definir_similaridade(a, b, peso);
    println!("Feito.");
}

fn recomendar_para_cliente(store: &MegaStore) {
    let cliente = ler_inteiro("\nID do cliente: ");
    if !eh_cliente(store, cliente) {
        println!("Cliente inválido.");
        return;
    }

    let recs = store.recomendar(cliente, 5);
    if recs.is_empty() {
        println!("Ainda não dá pra recomendar nada pra esse cliente.");
        return;
    }

    println!("\n-- Recomendações --");
    for r in recs {
        println!("[{}] {} (score {:.2})", r.product_id, r.name, r.score);
    }
}

fn recomendar_para_produto(store: &MegaStore) {
    let produto = ler_inteiro("\nID do produto: ");
    if !eh_produto(store, produto) {
        println!("Produto inválido.");
        return;
    }

    let recs = store.recomendar_por_produto(produto, 5);
    if recs.is_empty() {
        println!("Nada relacionado a esse produto ainda.");
        return;
    }

    println!("\n-- Quem viu/comprou este também leva --");
    for r in recs {
        println!("[{}] {} (score {:.2})", r.product_id, r.name, r.score);
    }
}

fn explorar_conexoes(store: &MegaStore) {
    let inicio = ler_inteiro("\nID inicial: ");
    let prof = ler_inteiro("Profundidade: ");

    let nome_de = |id| store.produto(id).map(|d| d.name()).unwrap_or("?");

    println!("\n-- BFS --");
    for (no, dist) in store.graph.bfs(inicio, prof) {
        println!("d={}: [{}] {}", dist, no, nome_de(no));
    }

    println!("\n-- DFS --");
    for no in store.graph.dfs(inicio, prof) {
        println!("[{}] {}", no, nome_de(no));
    }
}

fn carregar_mock() -> MegaStore {
    let mut store = MegaStore::new();

    let eletronicos = store.cadastrar_categoria("Eletrônicos");
    let livros = store.cadastrar_categoria("Livros");

    let mouse = store.cadastrar_produto("Mouse", eletronicos, 120.0, 4.5);
    let teclado = store.cadastrar_produto("Teclado", eletronicos, 350.0, 4.7);
    let mousepad = store.cadastrar_produto("Mousepad", eletronicos, 40.0, 4.2);
    let _livro = store.cadastrar_produto("Livro A", livros, 25.0, 4.8);

    let ana = store.cadastrar_cliente("Ana");
    let bruno = store.cadastrar_cliente("Bruno");

    store.registrar_compra(ana, mouse);
    store.registrar_compra(bruno, mouse);
    store.registrar_compra(bruno, teclado);

    store.definir_similaridade(mouse, mousepad, 0.8);

    println!("Dados de exemplo carregados.");
    store
}

fn rodar_benchmark() {
    // (categorias, produtos, clientes, compras por cliente)
    let cenarios = [
        (10, 100, 50, 5),
        (20, 1_000, 300, 8),
        (30, 10_000, 2_000, 10),
        (40, 50_000, 5_000, 10),
    ];

    println!("\n-- Benchmark --");
    for (cats, prods, clis, compras) in cenarios {
        let t = Instant::now();
        let dados = gerar_dados_sinteticos(cats, prods, clis, compras, 42);
        let t_gerar = t.elapsed();

        let t = Instant::now();
        let _ = dados.store.recomendar(dados.clientes[0], 10);
        let t_rec = t.elapsed();

        println!(
            "prod: {:>5} | cli: {:>5} | nós: {:>6} | gerar: {:>8.2?} | recomendar: {:>8.2?}",
            prods,
            clis,
            dados.store.graph.node_count(),
            t_gerar,
            t_rec
        );
    }
}

fn main() {
    let mut store = MegaStore::new();

    loop {
        println!("\n--- MENU ---");
        println!(" 1. Nova categoria");
        println!(" 2. Novo produto");
        println!(" 3. Novo cliente");
        println!(" 4. Registrar compra");
        println!(" 5. Definir similaridade");
        println!(" 6. Listar produtos");
        println!(" 7. Recomendar");
        println!(" 8. Explorar grafo (BFS/DFS)");
        println!(" 9. Benchmark");
        println!("10. Carregar exemplo");
        println!("11. Recomendar a partir de um produto");
        println!(" 0. Sair");

        match ler_linha("> ").as_str() {
            "1" => cadastrar_categoria(&mut store),
            "2" => cadastrar_produto(&mut store),
            "3" => cadastrar_cliente(&mut store),
            "4" => registrar_compra(&mut store),
            "5" => definir_similaridade(&mut store),
            "6" => listar_produtos(&store),
            "7" => recomendar_para_cliente(&store),
            "8" => explorar_conexoes(&store),
            "9" => rodar_benchmark(),
            "10" => store = carregar_mock(),
            "11" => recomendar_para_produto(&store),
            "0" => break,
            _ => println!("Opção inválida."),
        }

        pausar();
    }
}
