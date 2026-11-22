# RustPG 4.3.0-rc — novas integrações (candidata, não homologada)

Base: 4.2.0-rc, Starling 0.9.0. Fontes Rust preservadas no .hmod.
**Não compilado ou executado no Starling neste ambiente.** A prévia `qa/QA-PREVIA-V430.png` é um desenho esquemático para inspeção visual offline, não uma captura do jogo.

## 1. Precisão de XP das armas
- Adicionado hook `loot_drops` (com assinatura `()` confirmada em `death_weapon_tests.rs` do Starling fornecido).
- Seu `context.weapon_item(host)` identifica a arma gravada no disparo do projétil (arco, besta ou tridente) mesmo após o jogador trocar de item. A identidade do causador é verificada contra a do jogador.
- O registro temporário é indexado por `(dimensão, entidade morta, atacante)`, expira em até 20 ticks e é consumido em `living_drops`. Nenhum XP é concedido apenas por disparar flecha.
- Espadas, machados, maças, lanças e desarmado continuam usando o registro do ataque corpo a corpo. Mobs passivos permanecem no bestiário/hunting.
- **Não atribuí XP de Saqueador ao simples ato de abrir baús:** falta um evento que comprove a retirada de loot. Outros casos sem autoria verificável ficam sem XP, evitando exploits.

## 2. Extensões executáveis externas, mas confinadas
- `rustpg_script_vm.rs`: interpretador próprio de instruções `.rustpgx` com ABI 1 (não DLL e não WASM nativo).
- As instruções `units`, `const`, `add`, `mul`, `min`, `max`, `award` são executadas no servidor apenas após um evento confirmado, sem loops/jumps, acesso ao sistema de arquivos ou chamadas ao jogo.
- Cada programa tem até 32 instruções e profundidade de pilha até 4. Até 16 programas podem ser carregados; os resultados compartilham o orçamento global de 64 XP profissional por evento com as regras internas.
- Programas são carregados pelo datapack: `data/<namespace>/rustpg/behaviors/<arquivo>.rustpgx`. Recarregar com `/reload`. Validação e publicação atômica; falhas visíveis em `/rpg debug`.
- Exemplo embarcado: `data/rustpg/plugins_vm/bonus.rustpgx` (bônus de 1 XP em comércio concluído).
- **Não permite execução de código nativo Rust externo nem scripts com acesso geral ao mundo**. Isso requer outro isolamento, assinatura/permissões e suporte do motor nativo; não é algo que um loader de texto possa substituir.

### Exemplo de script externo
```
id=meupack:comercio_treinamento
api=1
on=trade
units
const 2
mul
min 8
award trading
```
Um jogador recebe no máximo 8 XP profissionais desse script por evento de comércio concluído, respeitando o orçamento compartilhado.

## 3. HUD
- Novo módulo puro `rustpg_hud_geometry.rs` para localizar o HUD próximo à hotbar, com reserva de espaço para os controles.
- Em Opções > Mods: `hud_scale` e `hud_y_offset` (0–56 px em unidades GUI).
- O status de poder ativo passou a ocupar o cabeçalho do HUD, sem uma segunda barra independente.
- Usa os recortes `hud_health_fill_trim` verde, `hud_stamina_fill_trim` azul, `hud_experience_fill_trim` dourado e moldura compacta em nove partes.
- Mantém HP real, bônus de absorção, stamina, pontos, fome, defesa e ar; substituição de camadas vanilla permanece opcional.
- O layout foi revisado no modelo para resoluções 260×150, 320×180, 400×240, 800×480 e 1920×1080. **Essa verificação não comprova a aparência dentro do launcher.**

## 4. Skill Tree
- Em tela compacta, mapa de caminhos e detalhes são duas visualizações completas, trocadas pelo botão `DETALHES >` / `< CAMINHOS`.
- Tocar num nó abre automaticamente seus detalhes em largura total, com descrição quebrada em linhas, rolagem e botão de compra. Não exige arrastar nem reduzir o mapa para ler o texto.
- Em telas grandes, mantém a apresentação lado a lado. Os cliques são gerados pelas mesmas regiões geométricas usadas para desenhar.
- As 182 texturas e os caminhos antigos permanecem, sem trocar IDs dos personagens nem alterar o perfil.

## QA executado
- 15 testes de integração da 4.2.0: OK.
- 21 testes de regressão do pacote anterior: OK.
- 14 testes novos de VM, hook nativo, identidade de arma, geometria, estados e PNG: OK.
- 182 PNGs abrem e coincidem com as referências do manifesto: OK.
- `Cargo.toml`, `horizon-mod.toml` e `assets-manifest.json` alinhados para `4.3.0-rc`.
- Rust `cargo check`, `cargo test`, compilação Starling, multiplayer, FPS Android e teste real do HUD: NÃO executados. Esses são critérios obrigatórios para declarar a FULL.

## Teste obrigatório no Starling
1. Backup de mundo e instalação isolada (somente uma versão do RustPG).
2. Compilar: recolher erro `E0xxx` com contexto e linha se falhar.
3. `/rpg debug`: verificar runtime VM, contagem de pacotes e erros de carregamento.
4. Disparar arco e besta contra criaturas e matar cada uma, mudando de arma antes da morte; comparar XP de Arcos/Bestas.
5. Criar datapack externo com script, `/reload`, concluir uma troca, conferir XP profissional e limite por evento. Tentar script inválido e conferir relatório em debug.
6. Conferir GUI em 400×240 e 800×480: detalhes, compra de nós, retorno, texto longo, rolagem e ações tocáveis.
7. Conferir HP 20/20→21/21, stamina, XP, hotbar, ar submerso, substituição vanilla e offset; observar colisões com botões do launcher.
8. Reiniciar mundo, trocar de dimensão, usar multiplayer e medir FPS antes de homologar.
