# RustPG v4 FULL — Etapa 1 / Correções e revisão de integração (4.0.4)

Base: 4.0.3 HUD & Fadiga. Esta é uma **revisão de desenvolvimento**, não o lançamento final nem uma compilação confirmada.

## Entregas implementadas no código-fonte

1. Protocolo do combate atualizado de v2 para v3 com dimensão explícita e validação de entradas.
2. Servidor entrega eventos de dano apenas a jogadores no mesmo `LevelRef`, próximos à posição do impacto.
3. Cliente descarta eventos de outra dimensão e possíveis IDs reaproveitados em posição incompatível.
4. Índice de posições de impacto agora é indexado por `(dimensão, id da entidade)`.
5. Saída do mundo limpa a caixa de entrada de combate e o estado visual, prevenindo dados antigos.
6. Regras anti-farm não dependem da cooperação de plugins: o registro central barra XP de mineração por blocos triviais ou recolocados.
7. Recuperação da velocidade após fadiga respeita mudanças realizadas por outros mods: só restaura se a velocidade vigente ainda for a aplicada pelo RustPG.
8. HUD com indicadores persistentes de fome/armadura e ar quando necessário, inclusive durante exaustão.
9. Barras de HP de todas as entidades vivas próximas na camada da interface, com dados reais sincronizados do cliente. Amostragem a cada 8 ticks, até 20 barras, culling por frustum simples.
10. Configuração nativa `mob_world_bars` e traduções PT-BR/EN-US.

## Pendências e restrições reais

- **Não foi possível executar `cargo test`/`cargo check` nem o Starling no container.** API/hook verificados em referência de código, mas ainda não homologados em runtime.
- A posição vertical das barras sobre mobs usa altura genérica (+1,85 blocos); a API de dimensão de caixa do alvo é **somente servidor**. Para escala precisa de todos os modelos, é necessária leitura render-side no estágio de cada entidade, ou ponte nativa.
- A detecção de velocidade modificada por outros mods é conservadora: evita sobrescrita, mas não garante que todos os buffs mantenham a penalidade percentual durante o conflito. Exige atributo modificador com ownership ou hook de viagem controlado.
- HUD completo ainda precisará de implementação visual com barras/símbolos próprios para fome, armadura, ar, absorção e montaria, além de perfil de escala no Android.
- A proteção anti-farm não detecta blocos recolocados por geração de estruturas/outros mods sem evento de colocação; o servidor não pode inventar essa proveniência.
- API de relíquias e animações, combate avançado, progressão reestruturada e efeitos em partículas continuam pendentes.

## Homologação obrigatória no Starling 0.9.0

1. Fazer backup do mundo e remover versões anteriores.
2. Compilar: registrar todos os erros com módulo/linha; **qualquer E0308/E0716 bloqueia avanço**.
3. Entrar no mundo, `/rpg debug`, abrir/fechar com R.
4. Andar perto de vaca/ovelha/zumbi: barras aparecem sobre o mundo, exibem HP real, não sobre flores/blocos.
5. Trocar de dimensão e atacar mobs em ambas: não ver popups de outra dimensão nem combos fantasma.
6. Perder vida, ficar sem fome, mergulhar, vestir armadura: verificar indicadores presentes quando os ícones vanilla estiverem ocultos.
7. Exaurir a stamina: correr/andar ×0,60, quebrar ×1/3, dano ×0,75; liberar fadiga aos 50%.
8. Conferir que quebrar terra e flores rende 0 XP e minério natural válido rende pontos.
9. Usar plugin/modificador de velocidade: não perder permanentemente o bônus após exaustão.
10. Comparar FPS e uso de memória com barras ativas/desativadas; medir cena com 20+ mobs.

Qualquer resultado ausente impede rotular esta etapa como aprovada ou a v4 como FULL.
