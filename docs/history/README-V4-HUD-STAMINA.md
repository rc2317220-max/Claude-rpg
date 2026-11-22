# RustPG 4.0.3 — HUD & Fadiga

Baseada no arquivo v4.0.2 (Skill Graph). Esta etapa implementa regras próprias de exaustão no servidor e substitui o desenho das barras.

- Vida verde; stamina azul; XP dourado.
- 6 texturas novas recortadas às áreas visíveis dos PNGs (sem margens transparentes).
- Aumento de HP real: se HP máximo passa 20→21, HP atual também sobe em +1 (20/20→21/21); ferimentos não são anulados.
- Ações consomem stamina; sprint custa 0,8 por tick, ataque comum custa 2,5.
- Regen base 6 pontos/seg após 1s sem drenar.
- Exaustão começa no zero e só termina em >=50% da stamina máxima (mesmo após reiniciar).
- Exausto: movimento ×0,60; mineração ×1/3 (duração triplicada); dano final ×0,75.
- Penalidade de mineração calculada no player_break_speed dos DOIS lados e combate no living_damage_pre do servidor.
- Sistema puro de regras `rustpg_v4_fatigue.rs`, codec de perfil atualizado com retrocompatibilidade.

## Limitações dos testes

Sem rustc/cargo no ambiente. Testes estruturais e validação dos PNGs não provam que o arquivo compila no Starling 0.9.0. Testar com backup de mundo no launcher.
