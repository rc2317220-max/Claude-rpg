# Starling 0.9.0 — laboratório Android real e RustPG

Atualizado em 2026-11-23. **Este documento distingue jogo real, importação do pacote e carregamento nativo.** Não marcar RustPG como testado no jogo apenas porque um `.hmod` foi importado ou um comando TCP respondeu.

## Base confirmada

- Starling 0.9.0 x86_64 oficial roda no emulador Android 35 (KVM, SwiftShader) usando o perfil **Play offline**, sem autenticação Microsoft.
- O launcher obtém/prepara os recursos 26.3 (cerca de 1,4 GiB no armazenamento privado).
- O Agent Control é habilitado com `/data/user/0/io.github.juanluci.starling/files/horizon-args.txt`, **um argumento por linha**: `--control-socket`, caminho absoluto para `horizon-control.json`, `--no-pause`.
- O endpoint JSON informa porta e token. A conexão TCP é encaminhada ao runner via `adb forward tcp:0 tcp:PORTA`. O protocolo exige terminador real `\\n`: enviar `auth TOKEN\\n` e então `hello\\n`, `status\\n` etc. **Não enviar barra-invertida e letra n literalmente**.
- [Ação 45167281922](https://github.com/rc2317220-max/Claude-rpg/actions/runs/45167281922): sucesso. Agent Control autenticou, criou `RustPG_Agent_Lab` em criativo (seed `424242`), carregou 213 chunks, listou mobs, aplicou tecla V e salvou screenshot real (2400×1080). Esta execução **não prova RustPG ativo**: o manifesto antigo era recusado pelo mod loader.
- A correção do `project/horizon-mod.toml` acrescentou `[native].mixins = "5a64af938ec86b00d53d16a408aabc0038fb3e57"`, exigido pela revisão pinada do Starling, além do `horizon = "ff303502a4f7982551a0e0d3622d9268efe9e829"`. Agora `scripts/package_native.py` verifica ambos os pins.
- [Ação 45170015474](https://github.com/rc2317220-max/Claude-rpg/actions/runs/45170015474): compilação e testes do pacote corrigido OK (**157/157**), publicação automática em `releases/RustPG.hmod` e `versão atual/RustPG.hmod`. O arquivo é um **pacote de código-fonte do mod**, não prova de código nativo executado.
- [Ação 45171170991](https://github.com/rc2317220-max/Claude-rpg/actions/runs/45171170991): o jogo executou o protocolo e o pacote corrigido não reproduziu o erro de manifesto anterior; o job falhou *depois* de imprimir COMPLETE por uma duplicação acidental do runner. Duplicação removida em `tooling/starling-android-runtime-chatgpt`.

## Sessão nativa efetivamente confirmada — 2026-11-23

- **[GitHub Actions #45173780745](https://github.com/rc2317220-max/Claude-rpg/actions/runs/45173780745)** executou o APK Starling 0.9.0 construído com o SDK nativo + `librustpg.so`, SHA-256 `b396194928d859bb0858be79da968dc9bd41bff287bea18d0aa6aeb21c0f11ea`.
- Autenticação Agent Control, `hello`, mundo criativo `RustPG_Native_Lab`, 213 chunks, entrada `key v tap` e `screenshot` (2400×1080) confirmadas.
- A resposta `ui` após V foi `screen:"mod"`. A captura real mostra a interface RustPG dentro do Minecraft: Resumo, Status, Habilidades, Bestiário e Profissões, HUD HP/SP/XP e painel lateral.
- O **teste A/B [#45174708784](https://github.com/rc2317220-max/Claude-rpg/actions/runs/45174708784)** abriu o mesmo APK SDK sem `native-libs`; nele V não abriu `screen:"mod"`. Isso confirma que a UI é da biblioteca RustPG, e não do Starling sem mod.
- O **Starling oficial [#45173981281](https://github.com/rc2317220-max/Claude-rpg/actions/runs/45173981281)** abriu mundo e fez screenshot sem conta Microsoft, mas o `.hmod` de código-fonte por si só não ativa hooks compilados.
- **Defeito identificado em #45173780745:** o gerador emitia `minecraft:spider`, `minecraft:phantom`, `minecraft:ravager`, etc. como `template` de criaturas. A API de conteúdo do Starling pinado admite apenas `cat`, `parrot`, `villager`, `wandering_trader`, `wolf` e `zombie` como modelos-base vivos. Uma entrada inválida recusava **todo** o conteúdo de entidades e os 12 renderizadores eram ignorados. O gerador foi corrigido para usar `zombie` como base hostil suportada; modelos e texturas 3D próprias permanecem. O empacotador agora valida as 12 definições.
- Novo teste `tooling/starling-native-sdk-rustpg`: injeta as 12 definições corrigidas no `native-libs/horizon-native-mods/rustpg/data/rustpg/horizon/entity_type` do build já compilado (sem alterar a biblioteca), inicia o mesmo mundo e executa `summon` de quatro criaturas; exige `mobs` com os quatro tipos antes de considerar o teste aprovado. Verificar o resultado do GitHub Actions antes de afirmar que as criaturas funcionam.

## Como continuar testando

1. A branch `tooling/starling-native-sdk-rustpg` contém um runner real em `.agent/run.sh`. A modificação desse arquivo dispara `.github/workflows/agent-terminal.yml` com Android emulado.
2. O runner pode reutilizar o APK e `librustpg.so` do artefato `14816419970` com `GH_TOKEN` do próprio GitHub Actions (requer `actions:read`), instala em Android 35 e segue o perfil offline.
3. Após os recursos ficarem prontos, o launcher tem **dois botões em duas telas diferentes**: primeiro **Go fishing**, depois **Game**. A ausência do segundo toque impediu tentativas anteriores.
4. O endpoint Agent Control nasce em `/data/user/0/io.github.juanluci.starling/files/horizon-control.json` quando `horizon-args.txt` contém (uma linha por argumento) `--control-socket`, caminho do JSON e `--no-pause`. Usar `adb forward` e cada comando terminado em quebra de linha real.
5. Depois de abrir um mundo, executar comandos `status`, `summon rustpg:...`, `mobs 64`, `key v tap`, `ui`, `screenshot`; guardar logs e prints originais. Não marcar critério de spawn concluído até o retorno `ok` e a presença do tipo no `mobs`.
6. A execução nativa do Minecraft é muito lenta com GPU software SwiftShader (menos de 1 FPS ao gerar terreno em 2400×1080). Isso **não** mede desempenho no Galaxy A55 ARM64 nem substitui testes ARM64.

## Artefatos e publicação

- O pacote está na branch `main`, caminhos `releases/RustPG.hmod` e `versão atual/RustPG.hmod` após o workflow de publicação.
- A execução comprovada da UI é [#45173780745](https://github.com/rc2317220-max/Claude-rpg/actions/runs/45173780745), com capturas e `control-session.jsonl` em seus artefatos.
- Testes adicionais devem evitar reaproveitar evidências de uma versão de código diferente sem dizer qual revisão executou.
