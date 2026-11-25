# RustPG Compile — infraestrutura independente (Starling 0.9.0)

O projeto RustPG permanece privado. O servidor deve usar um fork **público** separado de https://github.com/starling-client/mod-builds, porque o Starling baixa as bibliotecas sem autenticação. Não mover o código privado do RustPG ao fork. Os binários compilados e metadados do store são públicos.

## Configuração
1. Criar o fork público em https://github.com/starling-client/mod-builds/fork, nome mod-builds, proprietário rc2317220-max. Ativar Actions.
2. Copiar setup-workflow.yml para .github/workflows/rustpg-compile-setup.yml na branch main do fork (não no repositório privado do RustPG).
3. Na conta GitHub, criar token fine-grained restrito ao fork mod-builds, Actions e Contents Read and write. Guardar somente no secret de Actions BUILD_REPO_TOKEN.
4. Na Cloudflare, criar conta gratuita, ativar workers.dev e criar API token com permissão de editar Workers no próprio account. Guardar CLOUDFLARE_API_TOKEN e CLOUDFLARE_ACCOUNT_ID nos secrets de Actions do fork. Não publicar nem enviar tokens em chats.
5. Executar RustPG Compile server setup. A tarefa prepara a release requests, dispara o store-base para ARM64 e publica o Worker oficial após npm test. A base deve passar na comparação binária com o APK oficial.
6. Somente depois de store-base aprovado, copiar o endereço HTTPS efetivamente retornado por wrangler deploy no log para Mods > Compile > Build server... no Starling 0.9.0.

## Versão exata
- Starling v0.9.0: ff303502a4f7982551a0e0d3622d9268efe9e829 (tag GitHub)
- ABI: arm64-v8a
- Cargo profile: fast
- APK oficial: https://github.com/starling-client/starling/releases/download/v0.9.0/starling-0.9.0-arm64-v8a.apk
- Upstream consultado: starling-client/mod-builds@29a71250d91b70786c7d724b2eaab699cfdd61f7

## Validação
A primeira base é demorada; o manual upstream estima 45 minutos e builds subsequentes em Actions de 5–10 minutos. Não há garantia de duração. O cache pode expirar; reconstruir a base quando necessário.
Não desativar as verificações de identidade ABI/SDK. Não desativar o sandbox. Não oferecer compilação de mods que exijam whole-game Compile como se fossem Compile v2. O serviço público tem limites e pode receber solicitações de terceiros; usar as quotas oficiais e acompanhar Actions.
O Preview 4 é outro build; adicionar apenas após comprovar suporte ao protocolo, formato e identidade das bibliotecas.
O catálogo oficial é independente. Se a consulta ao store oficial impedir o fallback, é necessário investigar o cliente; este servidor não reativa o catálogo Horizon.

Fontes:
- https://github.com/starling-client/mod-builds/blob/main/SELF-HOSTING.md
- https://github.com/starling-client/mod-builds/blob/main/.github/workflows/store-base.yml
- https://github.com/starling-client/starling/releases/tag/v0.9.0

Estado: configuração preparada, não implantada; sem fork público ou credenciais Cloudflare disponíveis nesta sessão.
