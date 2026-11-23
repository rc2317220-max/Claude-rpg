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

## Compilação nativa distinta da importação

- `tools/android_native_sdk.py prepare` + build do Starling 0.9.0 x86_64 com `RUSTFLAGS="-C prefer-dynamic"` + `tools/android_native_sdk.py mods OUT RustPG.hmod --profile quick --abi x86_64` produzem `librustpg.so`.
- [Ação 45168700240](https://github.com/rc2317220-max/Claude-rpg/actions/runs/45168700240): o build nativo de `librustpg.so` e do APK foi bem-sucedido; arquivos `native-libs/native-mods.json`, `librustpg.so`, `native-libs/horizon-native-mods/rustpg` foram copiados para HORIZON_PRIVATE. SHA-256 da biblioteca: `b396194928d859bb0858be79da968dc9bd41bff287bea18d0aa6aeb21c0f11ea`. O **teste de jogo nativo falhou** ao não publicar Agent Control; NÃO considerá-lo validado.
- O APK SDK do Starling é diferente do APK oficial; ambos têm versão 0.9.0, mas a variante SDK usa bibliotecas Rust dinâmicas e precisa de validação própria.
- O carregador lê `$HORIZON_PRIVATE/native-libs/native-mods.json`, confere SHA-256 e identidade exata `<horizon commit> rustpg 6.1.0-alpha <rustc>`, então registra os hooks. Fonte: `crates/horizon-client/src/android_lib.rs` e `crates/horizon-mod-host/src/native_libraries.rs` da revisão `ff303502`.
- Diagnósticos da ação 45168700240: `GameActivity` foi iniciada; log confirmou argumentos `--control-socket`; nenhuma `FATAL EXCEPTION` Java; endpoint não apareceu. A screenshot após reiniciar ainda mostrava a escolha do launcher `Starling is ready` (a sequência exige pressionar `Go fishing` e depois `Game`).
- Testes corretivos: [runner com APK/biblioteca pré-compilados e coleta de backtrace](https://github.com/rc2317220-max/Claude-rpg/tree/tooling/starling-native-sdk-rustpg), [A/B sem RustPG no mesmo APK](https://github.com/rc2317220-max/Claude-rpg/tree/tooling/starling-native-sdk-baseline), e [runner oficial](https://github.com/rc2317220-max/Claude-rpg/tree/tooling/starling-android-runtime-chatgpt).

## Critério para considerar RustPG validado

1. `auth`, `hello`, `status` respondem dentro da GameActivity do **APK native-sdk** com `native-libs` presente.
2. Evidência de carregamento efetivo de `rustpg` (não apenas entrada no arquivo `native-mods.json`).
3. Criar/abrir mundo em modo criativo, aguardar `server.loaded = true`, verificar interações reais do mod e abrir a interface RustPG.
4. Screenshot real do jogo com HUD/UI RustPG; analisar visualmente, sem substituir por mockups.
5. Examinar logs de carregamento, erros e estabilidade; repetir com configurações corrigidas e publicar evidência.

A resolução padrão do emulador é 2400×1080 e o renderer usa CPU SwiftShader (poucos FPS). Para isolar travas de renderização, experimentar `adb shell wm size 1280x720` **depois** do onboarding, antes de reiniciar o GameActivity. A comparação A/B com e sem `native-libs` evita atribuir ao RustPG uma falha do APK SDK de base.
