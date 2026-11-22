Claude RPG 3.3.0 — Guia de depuração

Leia README-3.3.0.md para todas as alterações, instalação, limitações e testes.

Testes rápidos:
1) Ao entrar no mundo aparece 'Cliente ativo ... Assets: x/3 sprites'.
2) R abre menu; ESC fecha; R abre novamente.
3) /rpg abre; /rpg status e /rpg debug funcionam.
4) Em ÁRVORES: hover mostra descrição; rolar sobre detalhes exibe
   as demais linhas; comprar depende de pontos e pré-requisitos.
5) No mundo: HUD RPG aparece; mirando mob vivo deve exibir HP.
6) Após atacar sem tirar a mira, pode aparecer delta de HP em 2D.

Se 'Assets: 0/3 sprites', os PNGs existem, mas o atlas GUI do launcher
não está resolvendo os recursos, e os gráficos cairão no fallback vetorial.

Sem compilador Rust neste ambiente; o pacote precisa ser compilado no Starling.
