# RustPG 4.0.5 — Etapa 3 · Fundamentos nativos e relíquias funcionais

## Implementado no código

- `src/rustpg_relic_engine.rs`: motor original, independente do Starling, com registro de seis habilidades tipadas, caps globais, rejeição de IDs/ABI repetidos e sanitização de não-finitos. O efeito vem do `motif` já salvo no `Gear` antigo. Não há alteração na estrutura dos itens.
- Vigia → redução de dano de entrada (até 24%); Eclipse → roubo de vida sobre dano FINAL efetivo (até 16%, máximo 4 HP por golpe); Abismo → bônus de dano (até 18%); Aurora → recuperação de stamina (até +4 SP/s); Caçador → mineração mais rápida (até +30%); Monarca → redução do gasto de stamina (até 35%). Intensidades dependem de nível e raridade e de quantas relíquias equipadas contribuem.
- `lib.rs` injeta os efeitos nos hooks existentes (`living_damage_pre/post`, `player_break_speed`, `attack_entity`, `server_tick`) e mantém decisões autoritativas no servidor. O cliente só desenha e prevê velocidade de mineração com o perfil sincronizado.
- `src/rustpg_native_modifiers.rs`: usa `living_replace_attribute_modifier` e `living_remove_attribute_modifier` da API nativa em vez de sobrescrever bases globais. Exaustão multiplica a velocidade de movimento por 0,60 e deixa buffs de outros mods intactos. HP adicional é um modificador de vida com ID exclusivo, somado aos demais.
- Migração de antigos valores-base de HP: somente quando o valor atual, o valor-base e o esperado do RPG coincidem. Primeiro instala o novo modificador; só depois restaura a base quando tecnicamente seguro, e desfaz o modificador se a migração falhar. Salva flag `hp.modifier_migrated` no perfil. Casos indistinguíveis de mods externos exigem verificação manual.
- Curas por roubo de vida são enfileiradas para o `server_tick`, pois o hook de dano recebe `LevelRandomHost`, não `ServerHost`; nenhuma alteração de HP ocorre no callback inadequado. Eventos ficam limitados a 128 por tick, e a cura não revive jogadores mortos.
- Estados de combate são indexados por `(dimensão, entity id)` e associados ao jogador por UUID para limpeza de desconexão. Ataques ambientais continuam gerando números de dano mesmo sem atacante.
- Tooltip das relíquias com texto completo em várias linhas quando o mouse passa sobre o item. Compatível com as regras de botões clicáveis da tela.
- `/rpg debug` mostra quantidade de módulos de relíquias e ABI.

## O que NÃO está aprovado

Este arquivo **não foi compilado** pelo Rust nem testado dentro do Starling nesta sessão, porque `cargo`/`rustc` estão indisponíveis e não há acesso de rede para instalar a toolchain. Checagens estáticas e integridade de ZIP não provam que os hooks executem nem que a UI seja visualmente boa.

Ainda pendentes da v4 FULL: loader de plug-ins externos (com ABI e confiança), indicadores 3D render-side com oclusão e altura real, substituição profissional do HUD, remodelação profunda das árvores, relíquias ativas com cooldown persistente, reescrita completa do motor de atributos e testes multiplayer/Android.

## Roteiro de teste no Starling (backup do mundo obrigatório)

1. Remova 4.0.4 e instale somente 4.0.5. Compile. Se houver erro, envie mensagem **completa** com linha e tipo Rust, não apenas a linha final.
2. Execute `/rpg debug` e confira ABI 1 e seis famílias.
3. Equipe anel/peça do Eclipse, ataque uma vaca/zumbi ferido com vida do jogador abaixo do máximo: a cura ocorre apenas quando tira HP real, não quando o golpe é absorvido; teto 4 HP.
4. Equipe peças Vigia / Abismo, compare dano recebido e causado antes/depois, mantendo o mesmo alvo e condições.
5. Equipe Aurora / Monarca e compare a taxa de regeneração de stamina e o gasto de corrida/ataque. Exaustão mantém penalidade até 50%, e buff externo de movimento não deve desaparecer ao recuperar.
6. Equipe Caçador, teste mineração com stamina cheia e depois exausto. Blocos triviais não dão XP.
7. Ganhe +1 HP máximo estando em 20/20 → HUD 21/21. Teste ferido, equipa/desequipa e use outros mods de atributos. Reinicie o mundo e confira os perfis.
8. Confira tooltips de relíquias, tecla R, perfil salvo e barras de criaturas em mais de uma dimensão.

A v4 FULL continua dependente de compilação, regressões e execução real no launcher.
