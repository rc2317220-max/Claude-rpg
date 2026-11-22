# RustPG 4.1.0-rc — CANDIDATA DE INTEGRAÇÃO (NÃO HOMOLOGADA)

## Identificação
- Motor alvo: Starling 0.9.0 / Minecraft Rust 26.3 (revisão nativa conforme manifesto).
- Formato: `.hmod` com fontes Rust, arquivos de dados e recursos visuais.
- Código original baseado no RustPG 4.0.5. Compatibilidade de saves 3.x e 4.x depende da migração no codec.
- AVISO: não há Rust (`rustc`/`cargo`) nem launcher em execução no ambiente que produziu este pacote. Nenhuma alegação de build real, execução, FPS ou aprovação multiplayer.

## Etapa 1 — correções e compatibilidade
- Corrigido parser de compras com `#1/#2/#3` em IDs. A compra continua autenticada e validada no servidor.
- Redefinição de habilidades: livro-razão registra o custo por tipo de ponto (geral/profissional); pontos retornam à origem. Compras antigas sem registro usam migração única sem gerar restituições futuras por repetição.
- Perfis codec v5, versão do mod independente (4.1.0-rc).
- Corrigidos tipos da interface de equipamentos e uma movimentação indevida de String na chave dimensional.

## Etapa 2 — biblioteca de combate
- `rustpg_combat_pipeline.rs`: estágios tipados Fatigue, AttackRelics, GuardRelics, ABI 1 e limites contra saída NaN/Inf/negativa de extensões. Intercepta dano REAL no servidor.
- `rustpg_combat_training.rs`: escolha explícita de profissões por tipo de morte observável. Setas incrementam Arcos; tridentes, Tridentes; criaturas passivas, Caça. Tipo da arma corpo a corpo não é inferido com precisão pelo hook de morte (fallback Espadas herdado).
- V4 anterior preservada: eventos de dano final com dimensão e posição, histórico de combo e barras de HP próximas (sem confundir observação de vida com dano confirmado).

## Etapa 3 — relíquias (ativas + passivas)
- `rustpg_relic_active.rs`: seis poderes ativáveis por família de relíquia, 18 SP, 12s de duração, 65s de recarga; configuração de slot, tecla H, botão ATIVAR e `/rpg power <slot>`.
- Cooldowns persistidos em segundos UTC no perfil, não resetados por desconexão; efeitos existem apenas enquanto relíquia compatível está equipada.
- Efeitos ativos: Vigia +14% guarda; Eclipse +9% roubo de vida; Abismo +12% dano; Aurora +2.4 SP/s; Caçador +18% mineração; Monarca +20% redução de custo de stamina. A combinação com bônus passivos respeita caps globais.
- HUD exibe status da relíquia ativa, se houver espaço.

## Etapa 4 — Skill Tree e interface
- Nós de convergência com DOIS pré-requisitos de atributos distintos, preservando as IDs antigas, exigindo nível geral 20 e proficiência 12.
- O catálogo/constelação existente foi preservado; painel de detalhes e interação de seleção permanecem associados aos nós reais.
- Equipamento usa paginação adaptada à altura, separa botão ATIVAR de REMOVER e inclui status de duração/cooldown.
- HUD emprega barra verde para HP e azul para stamina; sprites com recortes reais, painel em nove partes, valores reais de HP, oxigênio/fome e informações de recarga. A substituição vanilla é seletiva e configurável.

## Etapa 5 — gameplay/profissões e antiexploit
- Extração: somente minérios reconhecidos rendem XP geral de mineração. Não há XP geral por terra, flores ou pedra.
- Lenhador: troncos explicitamente autorizados; não rende XP quando o bloco foi recolocado.
- Escavação: areia, cascalho e argila, somente XP de profissão; materiais recolocados não contam.
- Agricultura: somente plantações explicitamente reconhecidas E maduras; recompensas profissionais, nunca XP geral trivial.
- Exploração: primeira visita a uma dimensão gera recompensa uma única vez por perfil salvo.
- Classificação de criaturas comuns mantida para bestiário e escala de XP reduzida; mortes de jogadores não geram XP PvE.

## Etapa 6 — revisão independente (estática)
- Executar `python qa/review_v4_full.py`: 21 testes de contratos, continuidade, dados e integridade visual, incluindo conferência contra o ZIP de fontes Starling 0.9.0 fornecido.
- Vários módulos possuem testes Rust `#[cfg(test)]` de regras de compra, respec, integridade de nós, relíquias, cooldown, XP e registradores. **Esses testes ainda não foram executados.**
- Validar também o ZIP: código e sprites incluídos, manifest consistente.

## Para instalar/testar no launcher Android
1. Fazer backup de TODOS os mundos, não apenas o perfil do RPG. Não atualizar diretamente um mundo insubstituível.
2. Remover a versão antiga do RustPG e manter uma única versão instalada.
3. Compilar `.hmod` no Starling 0.9.0. Se surgir erro, coletar o log com caminho/linha e código (E0xxx); não assumir que integridade ZIP garante build.
4. Entrar em mundo de teste e executar `/rpg debug`, abrir R, acessar Skill Tree. Testar compra `#1`, reatribuição de pontos e recarga.
5. Equipar relíquia e ativar com H ou `/rpg power ring` (escolher slot em Opções > Mods).
6. Testar dano de espada, flecha, ataques contra vaca e zumbi; números flutuantes, barras de HP, stamina até exaustão e persistência após reiniciar.
7. Quebrar terra/flor/pedra, minério natural e minério recolocado, tronco natural/recolocado, trigo imaturo/maduro.
8. Verificar HP 20/20 -> 21/21, morte e renascimento, mudança de dimensão, conectividade em multiplayer e uso da RAM/CPU do Android.

## O que AINDA NÃO é um lançamento FULL validado
- Nenhum resultado de compilação Rust, execução no Starling, multiplayer ou teste Android neste ambiente.
- Registro de extensões internas com ABI tipada, MAS **não** carregador de plug-ins executáveis externos em tempo de jogo.
- Existem 25 profissões listadas, mas NÃO há eventos de progressão específicos implementados para todas (ex.: comércio e alquimia).
- Sem framework geral de animações de mobs, pipeline 3D de chefes ou editor visual de datapacks.
- Não foi provado que todos os efeitos gráficos/atalhos funcionam na revisão exata do seu launcher.
- Não afirmar que testes estáticos comprovam segurança, desempenho ou perfeição da interface.

## Estrutura principal
- `src/rustpg_relic_active.rs` — poderes ativos, cooldown, persistência.
- `src/rustpg_relic_engine.rs` — efeitos passivos extensíveis.
- `src/rustpg_combat_pipeline.rs` — motor de dano modular.
- `src/rustpg_gathering_rules.rs` — regras de coleta e colheita.
- `src/rustpg_combat_training.rs` — classificação de treinamento.
- `src/rustpg_nodes.rs` — nós da árvore com ramificações de convergência.
- `src/rustpg_profile.rs` — codec e livro-razão de pontos.
- `src/lib.rs` — ponte autenticada com Starling e eventos; nenhuma extensão recebe acesso livre a ServerHost.
- `qa/review_v4_full.py` — auditoria de contratos e recursos (não compilação).
