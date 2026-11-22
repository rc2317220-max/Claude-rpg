# RustPG 4.0.2 — Skill Graph (experimental)

**Objetivo:** aproximar a organização da árvore à referência enviada (Pufferfish Skills + Default Skill Trees): categorias independentes, nós conectados por pré-requisitos e indicação de estado, sem importar arquivos Java nem arte de terceiros.

## Mudanças reais

- Motor de topologia `src/rustpg_skillgraph.rs`, independente da API de desenho, para agrupar os nós por conexões reais e subdividir ramificações grandes sem perder talentos.
- Tela **Árvores** agora exibe ramificações interligadas com ícones, molduras por estado, níveis, conexão e prévia ao posicionar o mouse; o catálogo antigo deixa de ser a visualização padrão.
- Navegador de categorias, com seleção direta das árvores (em vez de passar por dezenas de setas).
- Paginação por conjunto de ramificações, tamanho mínimo de interação, painel lateral de descrição com rolagem e botão de aquisição validado pelo servidor.
- Painéis posicionados de forma responsiva: lado a lado para telas largas, empilhados nas estreitas.

## O que não foi feito

- Não importei as árvores ou os efeitos originais do pacote Java; o RustPG mantém suas próprias habilidades e progressão.
- Não há motor JSON/plugin externo já disponível para carregar árvores de terceiros; somente a biblioteca Rust de topologia foi isolada.
- Não confirmei compilação dentro do Starling nem execução no launcher. É obrigatório testar antes de usar no mundo principal.

## Teste rápido

1. Faça backup do mundo e substitua a versão 4.0.1 pelo .hmod novo.
2. Abra Árvores, pressione CATEGORIAS e toque em qualquer categoria.
3. Navegue entre páginas com as setas ou scroll e confira se todos os nós são alcançáveis.
4. Clique num nó; confirme descrição, status e requisitos. Teste adquirir um nó permitido.
5. Teste a mesma tela numa resolução/escala de GUI menor.

## Referências estudadas

- `puffish_skills-0.19.2-26.1-fabric.jar`: estrutura de categorias, estados, renderização de conexões, cliques e definições separadas.
- `default_skill_trees-1.1-1.20.4.zip`: esquema data-driven `category.json`, `skills.json`, `connections.json`, `definitions.json`, `experience.json`.

A implementação é original, com as texturas que já pertenciam ao RustPG.
