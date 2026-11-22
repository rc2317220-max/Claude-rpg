# RustPG 4.8.0 — estudo e porte parcial de 17 referências

Atualização **experimental**, baseada no Starling 0.9.0. O código foi alterado; não confundir com 17 mods inteiros convertidos para Rust.

- `src/rustpg_campaign.rs`: 18 missões em grafo de dependências, progresso persistido e recompensas de ações verificadas no servidor. Visível na página Quests e em `/rpg campaign`.
- `src/rustpg_combo.rs`: combos de 1–5 golpes reais, janela máxima de 30 ticks, dano concedido **somente** pela skill de combo da arma, com limite total de +20%.
- `src/rustpg_boss_ui.rs`: barras de chefes com 3 fases visuais pela vida atual, 5 divisórias e pulsação.
- `src/rustpg_timeline.rs`: keyframes para brilho, impactos e pulso das barras, usados no renderer 2D existente.
- `src/rustpg_profile.rs`: campanha salva junto com o perfil; dados da 4.7 são preservados. Não altera ou apaga saves automaticamente.
- `src/rustpg_attributes.rs`: novo atributo independente `weapon.<class>.combo_damage`, atualizado pelo registrador da versão 0.9.0.
- `src/lib.rs`: integra essas mecânicas aos callbacks nativos; nenhuma classe Java é executada nem copiada.

**Validação**: somente verificações estáticas e integridade do pacote. Não compilado no SDK do Starling 0.9.0 e não executado em mundo. Não use em seu único mundo sem backup.

Relatório completo com os 17 arquivos e as lacunas: `AUDITORIA-17-MODS-RUSTPG-RPG-480.md` fornecido separadamente.
