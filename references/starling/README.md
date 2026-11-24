# Starling Preview 4 — referência de código-fonte do RustPG

Snapshot: `preview-2026.11.23.4`, upstream commit `cac7bf64590fb38b4badee90f1f09848ce7f8daa`.
[Upstream fixado](https://github.com/starling-client/starling/tree/cac7bf64590fb38b4badee90f1f09848ce7f8daa) | [0.9.0 (Horizon)](https://github.com/starling-client/starling/tree/v0.9.0)

O ZIP fornecido pelo mantenedor contém 14.300 arquivos, 16.488 entradas incluindo diretórios, 176.114.522 bytes descomprimidos. SHA256 ZIP: `9d8b79c6bf62bf19ca2d185f63751226932e46d4d7d9e1311986440077661b98`. Hash canônico do conteúdo (caminho relativo UTF-8 + NUL + SHA256 binário de cada arquivo, em ordem de caminho): `1a6783938f3c7966b29e7130710d2b94c63c64eb71ddc6dac646c4a3671c404d`.

## Referências para todas as IAs
Use a revisão fixada, não main. Se a cópia local extraída estiver presente, consulte-a primeiro; caso contrário, consulte os links do upstream.
- [Guia para agentes](https://github.com/starling-client/starling/blob/cac7bf64590fb38b4badee90f1f09848ce7f8daa/docs/modding/src/for-agents.md)
- [Native route: mixins, loops, bridges e modules](https://github.com/starling-client/starling/blob/cac7bf64590fb38b4badee90f1f09848ce7f8daa/docs/modding/src/concepts/native-route.md)
- [Compilação Android](https://github.com/starling-client/starling/blob/cac7bf64590fb38b4badee90f1f09848ce7f8daa/docs/starling-native-mods.md)
- [Adaptação de mixins](https://github.com/starling-client/starling/blob/cac7bf64590fb38b4badee90f1f09848ce7f8daa/packaging/android/native_mixins_source.py)
- [Lacunas da API](https://github.com/starling-client/starling/blob/cac7bf64590fb38b4badee90f1f09848ce7f8daa/docs/design/mod-api/host-helpers-and-remaining-gaps.md)
- [Licença](https://github.com/starling-client/starling/blob/cac7bf64590fb38b4badee90f1f09848ce7f8daa/LICENSE)

Não confundir os alvos de Horizon com os de Sunrise. Preferir API pública onde suficiente; mixins para lacunas concretas. SDK de desktop e compilação Android têm rotas diferentes. Não afirmar suporte de runtime sem compilar e testar. Preservar versão 0.9.0 funcional. Nunca incluir recursos proprietários de Mojang, credenciais ou caches de build no espelho. Manter o snapshot sem modificações e preservar todas as licenças.
