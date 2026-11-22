CLAUDE RPG 3.3.0 — REWORK DE INTERFACE E HUD
Compatibilidade-alvo: Starling 0.9.0 / Minecraft Rust

INSTALAÇÃO
- Faça backup do mundo. Remova a versão anterior de claude_rpg.
- Importe este .hmod no Starling 0.9.0; compile e ative.
- Abra o mundo. Mensagem inicial: [Claude RPG 3.3.0] Cliente ativo.
- Controles: R (mapeamento + evento físico), /rpg como alternativa.

O QUE MUDOU
- R: evento físico e mapeamento processados separadamente. O pedido abre no tick
  seguinte, evita duas aberturas e evita reabrir o menu após ESC/Back.
- Nova HUD de jogo: HP vanilla real, vigor RPG, experiência e pontos disponíveis,
  com base/fill de sprites existentes no pacote.
- Mobs: o alvo sob a mira (crosshair) mostra nome, HP do mundo cliente,
  nível/tier caso a attachment de escala já tenha sido sincronizada.
- Números de dano: delta de HP observado em um alvo que continua sob a mira;
  flutua em 2D perto da mira, não acima de todos os mobs em 3D.
- Árvore: abre com zoom para leitura; mini-mapa e modo ÍNDICE para navegar,
  GERAL para visão completa, CENTRAL para nó selecionado, arrastar e rolar.
- Detalhes: painel lateral em telas largas, painel inferior em telas estreitas;
  efeito e requisitos em múltiplas linhas, rolagem para descrições longas;
  prévia ao passar o mouse e clique para selecionar.
- Assets: 176 PNGs originais validados e 18 referências GUI estáticas
  verificadas; desenho de texturas sem esconder os textos. O chat informa
  quantos dos três sprites de diagnóstico estão disponíveis no atlas: x/3.
- Nodos: texturas de estado bloqueado, disponível, adquirido e maximizado.
- Performance: topologia da árvore em cache, popups de dano limitados.

TESTE DE CAMPO
1. Entre no mundo e observe mensagem 'Cliente ativo' e 'Assets: x/3 sprites'.
2. Pressione R, feche com ESC, pressione R novamente: não deve reabrir sozinho.
3. Se R falhar, teste /rpg e o mapeamento nos controles.
4. Abra a guia ÁRVORES: passe o mouse num nó, leia o painel, role a descrição.
5. Teste GERAL, CENTRO, ÍNDICE, arrastar e zoom da árvore.
6. Mire um mob: confira nome/HP; ataque sem desviar a mira para ver delta.
7. Saia e volte ao mundo; confira permanência das habilidades.

LIMITAÇÕES E HONESTIDADE DOS TESTES
- Este arquivo teve auditoria estática de fontes, assinatura e recursos, não
  compilação Rust nem execução dentro do Starling neste ambiente.
- A tela de jogo NÃO substitui o HUD vanilla nem esconde suas barras; é um
  overlay RPG complementar, evitando interferir em outros mods/controles.
- Números de dano são aproximados pelo HP observado enquanto mantém a mira.
  Não capturam críticos/absorção, nem golpes em mobs fora do crosshair.
- Sprites dentro do pacote não garantem que o atlas do launcher publique todos;
  o contador 'Assets: x/3 sprites' foi adicionado justamente para diagnosticar.
- Não há promessa de que todos os sistemas de combate/fazenda/pesca estejam
  ativados por hooks nativos; melhorias estão concentradas em input/UI/HUD.
