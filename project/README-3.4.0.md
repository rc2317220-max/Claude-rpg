CLAUDE RPG 3.4.0 — REVISÃO DE ARQUITETURA (TESTE)
Alvo: Starling 0.9.0, SDK gerado pelo próprio launcher.

ALTERAÇÕES IMPLEMENTADAS EM CÓDIGO
1. Progressão de mineração server-side com whitelist explícita de minérios; quebrar terra, flores, pedra, madeira etc. não rende XP nem avança a quest Mine. Valores por minério, com testes de regras independentes no módulo crpg_xp_rules.rs.
2. Árvore em catálogo de talentos. Layout responsivo de lista + detalhes; sem mover mapa/zoom; botões para navegar páginas; clique escolhe uma habilidade e compra somente após validação pelo servidor; hover apresenta prévia. Descrições e requisitos quebram em linhas e têm rolagem.
3. Menu de configurações nativo de Starling: Opções > Mods > Claude RPG. Liga/desliga HUD, substituição de barras vanilla, painel do mob, números de dano; muda tamanho do HUD e escolhe a tecla de abertura (R/G/V/T/B/K). O mapeamento de Opções > Controles é respeitado se o jogador o tiver remapeado explicitamente.
4. Tecla física envia pedido de abertura diretamente ao sistema de telas; mantém como fallback a fila no client tick e comando /rpg.
5. HUD movido do bloco grande superior-esquerdo para um painel horizontal compacto acima da hotbar, com barras reais HP/SP/XP, fome, armadura, pontos e efeitos gráficos do pacote. Quando substituição vanilla está ligada, cancela apenas as layers player_health e contextual_info_bar, sem cancelar hotbar, chat ou controles.
6. Camadas do menu: o background desenha antes dos cards/textos, não por cima.

LIMITAÇÕES (IMPORTANTES)
- Não testado em Rust/Starling neste ambiente; a API foi confrontada com os arquivos da 0.9.0, mas testes de compilação e jogo só são possíveis no launcher.
- A layer 'player_health' também contém as bolhas de oxigênio; quando a substituição está ativada, a 3.4.0 não desenha um medidor de ar customizado. Desative 'Substituir barras vanilla' ao jogar em ambientes aquáticos até que a interface completa de oxigênio seja adicionada.
- A layer contextual_info_bar inclui também o medidor de salto de montaria/locator; a opção de substituição a oculta junto ao XP vanilla.
- Não há carregador dinâmico de plug-ins externos nesta versão: o código foi separado por responsabilidades. Um sistema de extensões só deve ser ativado com versionamento de API e permissões explícitas.
- Regras de XP não impedem quebrar e recolocar minério obtido com Toque Suave; a próxima camada anti-farm deve reconhecer blocos colocados pelo jogador e persistir esse estado.
- Damage numbers ainda medem delta de HP do alvo sob a mira. Para números precisos no mundo 3D será necessário conectar o hook de dano real do servidor e seu canal de sincronização.

TESTE
A) Faça backup do mundo; remova .hmod antigo. Instale 3.4.0.
B) Compile. Se E0308/E0xxx, copie o primeiro erro COMPLETO com arquivo e linha, não só a mensagem de resumo.
C) Vá a Opções > Mods > Claude RPG e confirme os ajustes. Abra por R; teste G após mudar atalho extra.
D) Quebre terra e flor: XP RPG não deve subir. Minere ferro: deve subir (apenas o XP RPG).
E) Abra Árvores: explore o catálogo sem usar zoom nem arrastar, confira seleção, descrição e compra.
F) Confira HUD dentro e fora d'água; use opção para preservar vanilla se necessário.
