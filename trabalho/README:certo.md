# MegaStore / ConectaStore

Sistema de recomendação de produtos com grafo, feito em Rust para a disciplina Data Structure
Strategy and Implementation (UniFECAF). Não tem dependências externas: o grafo, a BFS/DFS e o
gerador de dados de teste usam só a biblioteca padrão.


## Como funciona

Clientes, produtos e categorias são vértices. Compras, pertencimento a categoria e
similaridade entre produtos são arestas. Para recomendar algo a um cliente, o programa olha
para cada produto que ele já comprou e soma pontos para os candidatos que aparecem por três caminhos:

- quem comprou o mesmo produto também levou outro: +1,0 por comprador em comum
- outro produto da mesma categoria: +0,5
- produto marcado como similar: o peso da aresta (de 0 a 1)

O que o cliente já comprou fica de fora, o resultado sai do maior para o menor score e cada
produto aparece uma vez só. Também dá para pedir recomendações a partir de um produto: aí o
programa faz uma BFS pelas arestas de similaridade e soma quem comprou aquele produto e levou outros.

## Estruturas usadas

- `HashMap<NodeId, Vec<Edge>>`: lista de adjacência do grafo
- `HashMap<NodeId, NodeData>`: dados de cada vértice, consulta por ID
- `HashMap<String, NodeId>`: índice para achar por nome
- `HashMap<NodeId, f64>`: scores da recomendação (também evita candidato repetido)
- `HashSet<NodeId>`: visitados na BFS/DFS e produtos já comprados
- `VecDeque`: fila da BFS

O porquê de cada uma, e a comparação com matriz de adjacência, está no relatório.

## Rodando

Precisa do Rust instalado (<https://rustup.rs>).

```bash
git clone <url-do-repositorio>
cd megastore
cargo run --release
```

Testes (unitários e de integração):

```bash
cargo test
```

## Exemplo

O jeito mais rápido de ver funcionando é carregar os dados de exemplo (opção 10) e pedir
recomendação para a Ana (opção 7, ID 6):

```
> 10
Dados de exemplo carregados.
> 7
ID do cliente: 6

-- Recomendações --
[3] Teclado (score 1.50)
[4] Mousepad (score 1.30)
```

A Ana comprou o Mouse. O Bruno comprou Mouse e Teclado, então o Teclado ganha 1,0. Teclado e
Mousepad são da mesma categoria do Mouse e ganham 0,5 cada, e o Mousepad ainda ganha 0,8 da
similaridade cadastrada com o Mouse. Dá 1,5 e 1,3.

A opção 11 faz o mesmo a partir de um produto (ID 2, o Mouse):

```
> 11
ID do produto: 2

-- Quem viu/comprou este também leva --
[3] Teclado (score 1.00)
[4] Mousepad (score 1.00)
```

Menu:

```
 1. Nova categoria              7. Recomendar (por cliente)
 2. Novo produto                8. Explorar grafo (BFS/DFS)
 3. Novo cliente                9. Benchmark
 4. Registrar compra           10. Carregar exemplo
 5. Definir similaridade       11. Recomendar a partir de um produto
 6. Listar produtos             0. Sair
```

## Organização do código

```
megastore/
├── src/
│   ├── main.rs            menu de linha de comando
│   ├── lib.rs
│   ├── models.rs          NodeData, EdgeType, Edge
│   ├── graph.rs           grafo, BFS e DFS
│   ├── store.rs           cadastros, compras, consultas e recomendar
│   ├── recommendation.rs  cálculo das recomendações
│   └── bench.rs           gerador de dados sintéticos
├── tests/
│   └── integration_test.rs
├── Cargo.toml
└── README.md
```

O `main.rs` só fala com o `MegaStore`. Ele monta o grafo e chama o módulo de recomendação,
que apenas lê o `Graph`. O gerador de dados ficou na biblioteca, e não no `main`, para os
testes de integração conseguirem usar também.

Cada compra vira duas arestas (`Purchased` e `BoughtBy`). `registrar_compra` ignora compra
repetida; sem isso, quem comprasse o mesmo produto duas vezes inflaria o score dos candidatos.

## Desempenho

Rodei com `cargo run --release`, opção 9. Cada linha gera os dados com seed 42 e mede o
tempo de montar o grafo e de uma recomendação (top 10) para o primeiro cliente.

| Categorias | Produtos | Clientes | Compras/cliente | Nós | Geração | Recomendação |
|---:|---:|---:|---:|---:|---:|---:|
| 10 | 100 | 50 | 5 | a preencher | a preencher | a preencher |
| 20 | 1.000 | 300 | 8 | a preencher | a preencher | a preencher |
| 30 | 10.000 | 2.000 | 10 | a preencher | a preencher | a preencher |
| 40 | 50.000 | 5.000 | 10 | a preencher | a preencher | a preencher |

Máquina: [processador, RAM, sistema]

O tempo de uma recomendação depende de quantas compras o cliente tem, de quantos compradores
cada produto tem e do tamanho das categorias, e não do total de nós do grafo. A discussão
completa está no relatório.
