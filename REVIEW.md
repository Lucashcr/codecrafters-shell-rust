# Relatório de revisão: codecrafters-shell (Rust)

Ordenado do mais relevante ao menos relevante.

Estado atual: 39 testes passam (27 unitários + 12 de integração), mas `cargo fmt --check` falha em 2 arquivos e `cargo clippy -- -W clippy::pedantic` gera 5 avisos.

---

## Bugs de comportamento

- [x] **1. `println!` em `path_loader.rs` corrompe o stdout**
  - **Onde:** `src/path_loader.rs:7`, `println!("failed to read dir: {entry}")`.
  - **Motivo:** se o `PATH` tiver um diretório inexistente (comum), o shell imprime essa mensagem no stdout antes do primeiro prompt. Isso quebra a saída que os testes do CodeCrafters comparam.
  - **O que fazer:** remover a mensagem e ignorar silenciosamente o diretório que não puder ser lido, como o bash faz. Se quiser manter algum log, use `eprintln!`.

- [x] **2. Builtins devem ter prioridade sobre executáveis do PATH**
  - **Onde:** `src/prompt_parser.rs:21-50`, função `parse`.
  - **Motivo:** `exit` e `echo` são checados antes do PATH, mas `type` é checado depois. Um executável chamado `type` no PATH sobrescreveria o builtin, e o comportamento fica inconsistente.
  - **O que fazer:** trocar a cadeia de `if` por um único `match command { "exit" => …, "echo" => …, "type" => …, _ => busca no PATH }`. Assim todos os builtins vêm primeiro.

- [x] **3. `read_entry` varre subdiretórios do PATH recursivamente**
  - **Onde:** `src/path_loader.rs:27-29` e `:38-40`.
  - **Motivo:** shells olham apenas o primeiro nível de cada diretório do PATH. A recursão é lenta e pode gerar falsos positivos. Também exigiu o hack `p.ends_with("/X11")`, que existe por causa do symlink `/usr/bin/X11 -> .`, que cria um loop.
  - **O que fazer:** remover a recursão e o hack do `/X11`. Para cada entrada, só verificar se é arquivo executável.

- [x] **4. Permissão do symlink verificada no objeto errado**
  - **Onde:** `src/path_loader.rs:41`. Em `target_metadata.is_file() && metadata.permissions()...`, `metadata` é a do **link**, não a do alvo.
  - **Motivo:** a permissão do alvo é a que importa. O ramo de symlink também duplica a lógica do ramo de arquivo.
  - **O que fazer:** chamar `std::fs::metadata(entry.path())` uma única vez, que segue symlinks, e aplicar uma única checagem. Extrair `fn is_executable(&Metadata) -> bool` (`is_file() && mode() & 0o111 != 0`). Isso elimina o ramo duplicado.

- [x] **5. Mensagem de erro sem quebra de linha**
  - **Onde:** `src/command_interpreter.rs:62`, `write!(stderr, "failed to execute {name}: {error}")`.
  - **Motivo:** sem `\n`, o próximo prompt aparece colado na mensagem.
  - **O que fazer:** trocar por `writeln!`.

---

## Modelagem de tipos e design

- [x] **6. Substituir o `Box<Command>` recursivo em `Command::Type`**
  - **Onde:** `src/command_interpreter.rs:4-11` e `src/prompt_parser.rs:9-11` (`make_type` chama `parse` recursivamente).
  - **Motivo:** o `type` só precisa saber *o que é* um nome (builtin, executável ou não encontrado), não de um `Command` completo. Hoje `type type` funciona por acaso: o `parse` devolve um `Echo("type requires an argument")`, que cai no braço `_` e imprime "builtin". Isso é frágil e confuso.
  - **O que fazer:**
    ```rust
    pub enum Command {
        Echo(String),
        Exit,
        Type { name: String, kind: CommandKind },
        Executable { name: String, path: PathBuf, args: Vec<String> },
        Unknown(String),
        Nothing,
    }
    pub enum CommandKind { Builtin, Executable(PathBuf), NotFound }
    ```
    Criar `const BUILTINS: &[&str] = &["echo", "exit", "type"]` e `fn resolve(name, &executables) -> CommandKind`, usada pelo `type`. Os campos nomeados também evitam confundir `Executable(String, String, Vec<String>)`, onde duas `String` seguidas são fáceis de trocar.

- [x] **7. Mover `Command` para um módulo próprio (`src/command.rs`)**
  - **Motivo:** hoje o tipo vive em `command_interpreter`, mas o `prompt_parser` o produz e depende dele. Um módulo neutro deixa o fluxo parser → command → interpreter claro e evita dependência circular.
  - **O que fazer:** criar `src/command.rs` com `Command` e `CommandKind`, e declarar `mod command;` no `main.rs`.

- [x] **8. Usar `PathBuf` em vez de `String` para caminhos**
  - **Onde:** `HashMap<String, String>` em `path_loader.rs` e `prompt_parser.rs`, e a chamada `to_str()` em `path_loader.rs:19`.
  - **Motivo:** caminhos no Unix não são necessariamente UTF-8. O `to_str()` descarta silenciosamente entradas válidas, e `PathBuf` é o tipo idiomático.
  - **O que fazer:** retornar `HashMap<String, PathBuf>` de `load`. O nome do comando pode continuar `String`.

- [x] **9. Refatorar `interpret` para um único ponto de retorno**
  - **Onde:** `src/command_interpreter.rs:18-83`. Há vários `return Ok(InterpreterResult {...})` repetidos, mais um `Ok(...)` final.
  - **Motivo:** repetição que dificulta ler e alterar o fluxo.
  - **O que fazer:** derivar `Debug, PartialEq, Eq` em `InterpreterResult` e criar construtores (`InterpreterResult::ok()`, `::exit()`, `::code(n)`). Fazer o `match` devolver o resultado como expressão.

---

## Qualidade do código e do loop principal

- [x] **10. Simplificar `repl` e `main`**
  - **Onde:** `src/main.rs`.
  - **Motivo:**
    - `result` é inicializado mutável e depois sobrescrito.
    - `prompt = String::from(prompt.trim())` realoca à toa.
    - A flag `finish_loop` pode ser um `loop` com `break`.
    - O `println!` comentado é lixo.
    - `std::result::Result<(), std::io::Error>` é verboso.
  - **O que fazer:**
    - Usar `let line = prompt.trim();`.
    - Fazer o fluxo linear, tratando o erro de leitura com `?` ou um `return` antecipado.
    - Declarar `fn main() -> io::Result<()>`.
    - Obter `stdin.lock()` e `stdout.lock()` uma vez em `main`, e fazer `repl` receber `&mut impl BufRead` e `&mut impl Write`. Isso permite testar o `repl` sem subprocesso.
    - Remover o comentário morto.

- [x] **11. Simplificar `path_loader::load`**
  - **Onde:** `src/path_loader.rs:58-74`.
  - **Motivo:**
    - `split(":")` aciona o aviso do clippy (padrão de um caractere).
    - O `reduce(|a, b| b.into_iter().chain(a).collect())` é obscuro e refaz o `HashMap` a cada diretório.
    - Receber `&mut Stderr` só para avisar que o PATH não existe não compensa.
  - **O que fazer:**
    - Usar `std::env::var_os("PATH")` e `std::env::split_paths`.
    - Iterar os diretórios na ordem do PATH, inserindo com `map.entry(name).or_insert(path)`, para que o primeiro do PATH vença.
    - Remover o parâmetro `err`. PATH ausente resulta em mapa vazio.

---

## Dependências, lints e formatação

- [-] **12. Remover dependências não utilizadas**
  - **Onde:** `Cargo.toml`: `anyhow`, `bytes`, `thiserror`.
  - **Motivo:** nenhuma é usada no código. Elas só aumentam o tempo de build e a superfície de dependências.
  - **O que fazer:** apagar as três linhas de `[dependencies]` e rodar `cargo build` para atualizar o `Cargo.lock`.

- [x] **13. Rodar `cargo fmt`**
  - **Onde:** `src/path_loader.rs` (linha ~38) e `tests/shell.rs` (linha ~86).
  - **Motivo:** `cargo fmt --check` falha, o que é um problema em qualquer CI.
  - **O que fazer:** `cargo fmt`. Se fizer a refatoração do item 11 antes, a formatação de `path_loader.rs` muda de qualquer forma, então rode o `fmt` por último.

- [x] **14. Corrigir os avisos do clippy pedantic e fixar os lints no `Cargo.toml`**
  - **Avisos atuais:**
    - 3 × `continue` redundante (`path_loader.rs`).
    - 1 × padrão de string de um caractere (`split(":")`).
    - 1 × `format!` com variável que pode ser inline (`command_interpreter.rs:46`, `"{cmd} is {}", path`).
  - **Motivo:** a maioria desaparece com os itens 3, 4 e 11. Fixar os lints no manifesto mantém o padrão para o futuro.
  - **O que fazer:** adicionar ao `Cargo.toml`:
    ```toml
    [lints.clippy]
    pedantic = { level = "warn", priority = -1 }
    ```
    Se aparecer ruído, como `module_name_repetitions`, liberar com `allow` caso a caso. Validar com `cargo clippy --all-targets -- -D warnings`.

---

## Testes

- [x] **15. Limpar os testes unitários**
  - **Onde:** `src/prompt_parser.rs` e `src/command_interpreter.rs`.
  - **Motivo:**
    - `pub mod tests` não precisa ser público.
    - `blank_input_does_not_panic` usa `catch_unwind` à toa, e o teste não verifica o resultado.
    - O helper `run` chama `.as_ref().unwrap()` duas vezes.
    - `mock_executables()` está duplicado nos dois módulos.
  - **O que fazer:**
    - Trocar por `mod tests`.
    - Trocar o teste de entrada em branco por `assert_eq!(parse(input, &mock), Command::Nothing)`.
    - Simplificar `run` com `let r = interpret(...).unwrap();`.
    - Se criar o `src/command.rs` (item 7), colocar o mock em um helper compartilhado `#[cfg(test)]`.

- [x] **16. Remover comentários obsoletos em `tests/shell.rs`**
  - **Onde:** `tests/shell.rs:120` e `:126`, "Known bug: EOF on stdin makes the shell loop forever."
  - **Motivo:** o bug já foi corrigido (o `repl` retorna `true` em EOF) e os testes passam. Os comentários enganam quem lê.
  - **O que fazer:** apagar os dois comentários.

- [x] **17. Adicionar testes para os comportamentos corrigidos**
  - **O que fazer:**
    - Parser: `type` de executável retorna `CommandKind::Executable`.
    - Parser: builtin tem prioridade sobre um executável de mesmo nome no mock.
    - `path_loader`: criar um diretório temporário (em `std::env::temp_dir()`) com um arquivo executável, um não executável, um symlink para executável e um subdiretório. Verificar que só o executável e o symlink entram, e que o subdiretório não é varrido.
    - Integração: `PATH` com diretório inexistente não imprime nada extra no stdout.

---

## Fora de escopo por enquanto (decisão consciente)

- [x] **18. "command not found" vai para o stdout e `exit`/linha vazia imprimem `\n` extra**
  - **Motivo:** o bash manda "command not found" para o stderr. Mas os testes de integração capturam só o stdout e esperam o formato atual (`"$ foo: command not found\n$ \n"`). Mudar isso exige alterar os testes junto, e pode afetar a validação do CodeCrafters.
  - **O que fazer, se quiser depois:** primeiro confirmar o que os testes do CodeCrafters esperam. Depois mover a mensagem para o stderr e atualizar `tests/shell.rs` e os testes unitários (`unknown_command_not_found`).

---

## Ordem sugerida de implementação

1. Itens 12, 1, 5 (mudanças pequenas e seguras).
2. Itens 3, 4, 8, 11 (reescrita do `path_loader`).
3. Itens 7, 6, 2, 9 (modelagem de tipos, parser e interpreter).
4. Item 10 (`main`/`repl`).
5. Itens 15, 16, 17 (testes).
6. Itens 14 e 13 por último (lints e `cargo fmt`).

## Verificação final

```
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Teste manual com `./your_program.sh`:
- `type ls`
- `ls -la`
- `PATH=/nao/existe:$PATH ./your_program.sh` não deve imprimir nada extra.
