# RustPG: Starling 0.9.0, mixins e pacotes nativos

## Duas rotas de compilação (não são intercambiáveis)

O RustPG contém 19 mixins e 1 bridge, declarados em `project/horizon-mod.toml`. Esses patches alteram funções do motor. **Eles não são aplicados pelo Compile v2 da biblioteca nativa separada**: o próprio Starling, em `tools/android_native_sdk.py`, recusa `[native.mixin]`, `[native.bridge]` e `[native.module]`. Só a recompilação de todo o jogo, com aplicação dos mixins, poderia executá-los.

As versões novas que usam apenas a loja de bibliotecas também avisam que o formato legado `.hnmod` não compila mais; trocar a extensão de um `.hmod` por `.hnmod` não resolve. O empacotamento distingue **conteúdo/fonte** de **build realmente executado**.

1. **`RustPG-core.hmod`**: manifesto nativo sem bridge/mixins, com a feature `whole_game_mixins` desligada. Preparado para o Compile v2. Os nodes que dependem exclusivamente de mixins ficam bloqueados para compra, mas permanecem visíveis e os dados previamente salvos não são apagados. Hooks já suportados no SDK continuam sendo compilados.
2. **`RustPG-dev.hmod`**: código-fonte completo, com 19 mixins, bridge e feature `whole_game_mixins` ativada. **Não é instalável pela rota Compile v2**. Seu funcionamento real exige um pipeline de rebuild integral do Starling com patches aplicados, seguido de teste do APK/biblioteca gerada.

Os dois ZIPs são gerados por `python3 scripts/package_native.py`, chamado por `bash scripts/package_hmod.sh`. O gerador inclui somente arquivos permitidos pelo importador nativo (`Cargo.toml`, `horizon-mod.toml`, `src/*.rs`, `assets/`, `data/` etc.), sem incluir `editor/` e `models/` originais na raiz. A versão anterior zippava o diretório inteiro e podia ser rejeitada como pacote inválido.

## Integrações incluídas no fonte

- Combate: `attack_entity` não efetiva um golpe se o servidor não conseguir cobrar a estamina. Só depois de cobrar um golpe o cache registra arma/alvo para combo, perícia e efeitos. O hook pode cancelar a ação no servidor.
- Movimentação: `movement_input` impede sprint pedido pelo cliente quando o perfil sincronizado informa exaustão; o servidor continua drenando estamina e aplicando o modificador de velocidade.
- Sangramento: o dano periódico usa `EntityRef::hurt` na thread/host autoritativo, preservando as resistências e os hooks existentes.
- Encantamentos: o nível virtual de Afiação acima dos 255 do leitor nativo é amostrado da arma equipada no ataque. Somente um golpe validado pelo servidor recebe +0,5 dano por nível excedente; o extra efetivo é limitado a 4.096 níveis para impedir overflow, mas o rank armazenado no item continua crescendo.

## Verificação e limitações

O GitHub Actions roda testes Rust usando o SDK fixado no commit `ff303502a4f7982551a0e0d3622d9268efe9e829` e verifica também `--no-default-features`. Isso valida assinaturas/tipos e testes unitários, **não prova que os patches foram aplicados no binário Android nem que o cliente já suporta esse pin**.

O arquivo `RustPG-core.hmod` deve ser testado primeiro em mundo descartável; mantenha cópia de segurança do mundo. Os elementos dependentes de mixins ainda precisam da compilação integrada do motor. **Nenhuma das duas variantes deve ser anunciada como versão final 100% funcional.**

Referências no código do Starling:
- `crates/horizon-client/src/game/starling/native_builds.rs`: `compile_from_store`, `WHOLE_GAME` e recusa do formato legado
- `tools/android_native_sdk.py`: recusa de tabelas `[native.*]` no Compile v2
- `crates/horizon-mod-package/src/manifest.rs`: arquivos permitidos nos pacotes de fonte nativa
