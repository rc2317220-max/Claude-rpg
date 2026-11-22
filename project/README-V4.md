# Claude RPG 4.0.0 — Fundação, etapa 1 / revisão de arquitetura

**ESTA NÃO É UMA V4 FINAL HOMOLOGADA.** O pacote contém código-fonte Rust e
assets para o compilador do Starling 0.9.0, mas não foi compilado com `cargo`/
`rustc` nem executado no launcher neste ambiente.

## Duas passadas de cada etapa

1. **Implementador:** desenvolve a funcionalidade em módulo isolado.
2. **Revisor:** verifica a integração no servidor/cliente, a presença dos
   símbolos na API do Starling 0.9.0, a integridade de assets e os testes de
   domínio. O verificador independente está em `qa/audit_static.py`.
3. **Gate obrigatório para homologação:** compilar no Starling 0.9.0, executar
   em mundo de backup e verificar os cenários abaixo. Essa etapa requer o
   próprio launcher; verificações lexicais não a substituem.

## Implementado nesta fundação

- Biblioteca de domínio isolada em `src/crpg_v4_core.rs` com protocolo de
  extensões ABI v4. Módulos compilados no pacote registram regras
  determinísticas, limites de XP e perfis de recompensa.
- Dano de criaturas ligado a `living_damage_post`, valor final do servidor;
  mensagem `claude_rpg:hit`, limitada por alcance, com posição do golpe
  capturada em `living_incoming_damage`. Número de dano é projetado em 2D a
  partir da posição 3D e não depende da mira. Crits não são classificados.
- Mineração autorizada no servidor apenas para minério reconhecido. Índice
  **persistente por nível/dimensão** de posições de minérios recolocados, via
  attachment + codec binário `crpg_v4_provenance.rs`, impede XP por blocos
  colocados. Com lotação do índice, a proteção nega XP (fail-closed).
- Todo ser vivo elegível em `living_drops` entra no bestiário (incluindo
  animais e entidades de outros mods) e recompensa balanceada. PvP excluído.
- HUD compacto acima da hotbar, com HP real, stamina, XP, fome e oxigênio;
  opção de ocultar determinadas camadas vanilla sem esconder a hotbar.
- Árvore substituída, na visualização principal, por catálogo paginado com
  linhas de tamanho fixo, sprites de talento, requisitos, barra de descrição
  rolável e compra validada no servidor. Sem zoom nem arrastar.
- Tecla padrão R e `/rpg` preservados. O cliente tem caminhos via evento físico,
  mapeamento `consume_click` e mensagem remota, com configurações em
  `Opções > Mods` para HUD e atalho.
- 176 PNGs originais preservados em atlas GUI; correção das dimensões UV de
  ícones 128x128 e tooltip 512x256.

## Limitações e riscos abertos (NÃO escondidos)

- **Sem compilação real:** assinatura gerada dos hooks, integração de macros,
  tipos, borrows e compatibilidade do launcher precisam ser confirmados.
- O canal de dano guarda posições no mundo; existe risco de broadcast cruzado
  entre dimensões com coordenadas coincidentes. Ainda requer o identificador
  de dimensão no evento e filtro de espectadores adequado.
- HP bar de alvo é exibida ao mirar. Barras 3D individuais de TODAS as
  criaturas visíveis ainda requerem enumeração de entidades e render especializado.
- Oxygen é representado textualmente; HUD para montarias, efeitos e outras
  camadas vanilla requer testes e refinamento.
- A indexação de minérios ocorre antes de outros mods poderem cancelar o
  posicionamento. Isso pode gerar falso positivo conservador (menos XP).
  Substituição de blocos por pistões/comandos/outros mods requer rastreamento.
- O sistema de extensões ABI v4 é um registro de módulos **compilados junto ao
  mod**, NÃO um loader externo de `.dll`/scripts arbitrários.
- Os 176 sprites foram preservados e vários estão integrados; nem todos são
  usados em tela atualmente. A árvore tem novo formato, porém ainda merece
  design, animações e iconografia detalhados por habilidade.
- Não há certificação de equilíbrio, progressão de longo prazo, encantamentos
  ou integração multiplayer; versões 3.x já apresentaram erros reais.

## Gates para as próximas etapas

- **Etapa 2:** compilar e corrigir erros E0xxx, confirmar hooks/payloads,
  reproduzir saves pré-existentes e fazer o atalho abrir pelo launcher.
- **Etapa 3:** combate completo (HP de todos os mobs, indicadores 3D,
  críticos e estados, dimensionamento dos pacotes, multiplayer).
- **Etapa 4:** interface e assets (UX responsiva e acessível, HUD completo,
  árvore por categorias, descrições sem cortes, animações eficientes).
- **Etapa 5:** sistema de combate/progressão integrado às missões, profissões,
  encantamentos e extensão por módulos, com testes de balanceamento.
- **Etapa 6:** homologação com cenário de regressão, testes de performance e
  validação de persistência real no Android.

Faça **backup do mundo** e remova outra versão com `id = "claude_rpg"`
antes de instalar. Após compilação, teste: `/rpg`, `R`, `ESC`/`R`, minerar
terra e minério natural, colher minério recolocado antes e após reiniciar
mundo, bater em vaca e zumbi sem mirar, HUD debaixo d'água, menus e save.
