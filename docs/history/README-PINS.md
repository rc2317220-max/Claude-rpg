# Claude RPG 3.2.6 — Compilação Starling 0.9

Este pacote e um projeto nativo .hmod com Cargo.toml, src/, assets/ e horizon-mod.toml.

Os commits horizon/mixins no manifesto foram preservados da versao recebida pelo usuario.
Nao foi possivel atestar esses commits contra a build remota; o proprio Starling ja compilou
uma versao anterior nesta mesma origem. O launcher pode alinhar os pins com o SDK local.

Novidades e verificacoes: consulte README-DEBUG.md e
RELATORIO-AUDITORIA-CLAUDE-RPG-3.2.6.txt (relatorio externo).

Este ambiente nao tem rustc/cargo nem o runtime Starling, portanto a compilacao da nova
versao nao foi testada. Aprovados testes estaticos do pacote/PNG/API/acoes.

Nao instale duas versoes do id claude_rpg simultaneamente. Faca backup dos mundos.
