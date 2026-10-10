# Relatório de revisão 2: codecrafters-shell (Rust)

Segunda passada, feita depois que o `REVIEW.md` foi implementado. Ordenado do mais relevante ao menos relevante dentro de cada seção.

Estado verificado:
- `cargo test`: 36 unitários + 13 de integração, todos passam.
- `cargo fmt --check`: ok.
- `cargo clippy --all-targets -- -D warnings`: **falha** (`items_after_test_module` em `src/path_loader.rs:37`).
- `cargo clippy -- -W clippy::pedantic`: mais 2 avisos (`continue` redundante em `path_loader.rs:29`; `;` faltando em `prompt_parser.rs:224`).

Decisão registrada: o item 12 do `REVIEW.md` (remover `anyhow`, `bytes`, `thiserror`) **não será feito**. As dependências vêm do template do CodeCrafters e removê-las pode afetar o build e os testes da plataforma.

---

## Seção A — Itens do REVIEW.md marcados como feitos, mas incompletos

- [x] **A1. Item 14: o `clippy -D warnings` ainda falha e os lints não estão fixados**
  - **Onde:** `src/path_loader.rs:37` (`mod tests` vem antes de `pub fn load`), `Cargo.toml` (sem `[lints.clippy]`).
  - **Motivo:** o clippy padrão acusa `items_after_test_module`, então a verificação final do `REVIEW.md` não passa. Sem o bloco no manifesto, o padrão pedantic não é mantido no futuro.
  - **O que fazer:**
    - Mover o `mod tests` de `path_loader.rs` para o fim do arquivo, depois de `load`.
    - Corrigir os 2 avisos pedantic:
      - `path_loader.rs:29`: remover `Err(_) => continue,` (ou trocar o `match` por `if let Ok(entry) = file_result`).
      - `prompt_parser.rs:224`: adicionar `;` ao `assert_eq!` dentro do `for`.
    - Adicionar ao `Cargo.toml`:
      ```toml
      [lints.clippy]
      pedantic = { level = "warn", priority = -1 }
      ```
    - Validar com `cargo clippy --all-targets -- -D warnings`.

- [x] **A2. Item 6: o `type` ainda depende de um `Echo` de erro para reconhecer builtins**
  - **Onde:** `src/prompt_parser.rs:10-13` (`make_type`), `src/command.rs:99-105` (`resolve_type`).
  - **Motivo:** `make_type` chama `parse(s, ...)` e `resolve_type` trata `Command::Echo(_)` como builtin. Por isso `type type` funciona só porque `parse("type")` devolve o `Echo("type requires an argument")`. É o mesmo "por acaso" que o item 6 queria eliminar. `resolve_type` também trata `Nothing` e `Unknown` como "não encontrado" no `_ =>`, o que esconde casos.
  - **O que fazer:**
    - Em `prompt_parser.rs`, criar `const BUILTINS: &[&str] = &["echo", "exit", "type"];`.
    - Criar `fn resolve(name: &str, executables: &HashMap<String, PathBuf>) -> CommandType`: builtin se estiver em `BUILTINS`, executável se estiver no mapa, senão `NotFound`.
    - `make_type` passa a chamar `resolve`, sem passar por `parse`.
    - Remover `Command::resolve_type`.
    - Opcional: trocar as variantes-tupla por campos nomeados (`Executable { name, path, args }`, `Type { name, kind }`), para evitar confusão entre as duas `String` em `Executable(String, PathBuf, Vec<String>)`.

- [x] **A3. Item 10: `repl` continua acoplado a `Stdin`/`Stdout`/`Stderr` concretos**
  - **Onde:** `src/main.rs`, assinatura de `repl` e `main`.
  - **Motivo:** o `REVIEW.md` pedia `lock()` uma vez em `main` e `repl` genérico (`impl BufRead`, `impl Write`). Hoje o `repl` só é testável via subprocesso, e cada `read_line` adquire o lock do stdin.
  - **O que fazer:**
    - Em `main`, usar `let mut stdin = io::stdin().lock();` e `let mut stdout = io::stdout().lock();`.
    - Declarar `fn repl(stdin: &mut impl BufRead, stdout: &mut impl Write, stderr: &mut impl Write, executables: &HashMap<String, PathBuf>) -> io::Result<bool>`.
    - Depois disso, dá para testar o `repl` com `Cursor<&[u8]>` e `Vec<u8>`, sem subprocesso.

- [x] **A4. Item 15: mock duplicado e `assert_ne!` fraco**
  - **Onde:** `mock_executables()` em `src/command.rs` e `src/prompt_parser.rs`; `ls_is_not_executable_file` em `src/prompt_parser.rs:~146`.
  - **Motivo:**
    - O mock continua duplicado, que era parte do item.
    - `assert_ne!(parse("ls", ...), make_executable("ls", ...))` passa com qualquer resultado diferente de um executável `ls`, inclusive um `panic`-free `Nothing` ou `Echo` errado. Não testa o que o nome promete.
  - **O que fazer:**
    - Trocar por `assert_eq!(parse("ls", &mock_executables()), make_unknown("ls"));`.
    - Criar um módulo `#[cfg(test)] pub(crate) mod test_support` (por exemplo em `src/main.rs` ou `src/command.rs`) com `mock_executables()` e usá-lo nos dois lugares.

- [ ] **A5. Item 18 só foi feito pela metade**
  - **Onde:** `src/command.rs` (`Command::Nothing` e `Command::Exit` fazem `writeln!(stdout)`), `src/main.rs` (EOF e linha vazia também fazem `writeln!(stdout)`).
  - **Motivo:** o "command not found" já vai para o stderr, mas `exit`, linha vazia e EOF ainda imprimem um `\n` extra. Os testes de integração esperam esse formato (`"$ \n"`). Pode ser intencional para os testes do CodeCrafters, que não costumam exigir isso.
  - **O que fazer:** rodar `codecrafters test` e conferir se a saída exige o `\n`. Se não exigir, remover os `writeln!` e ajustar `tests/shell.rs` e os testes unitários. Se exigir, deixar um comentário curto explicando o motivo.

- [ ] **A6. Atualizar o próprio `REVIEW.md`**
  - **Onde:** `REVIEW.md`.
  - **Motivo:** o texto ficou desatualizado em relação ao código:
    - Cabeçalho "Estado atual" ainda descreve o estado inicial (39 testes, fmt falhando).
    - Vários itens citam `command_interpreter.rs` (agora `command.rs`) e `CommandKind` (agora `CommandType`).
    - O item 18 está na seção "Fora de escopo", mas foi feito (parcialmente, ver A5).
    - O item 12 aparece como `[-]` sem explicação.
  - **O que fazer:** atualizar o cabeçalho, mover o item 18 para a seção certa e anotar no item 12: "ignorado de propósito; as dependências vêm do template do CodeCrafters".

---

## Seção B — Pontos novos

- [ ] **B1. O executável não usa o caminho resolvido**
  - **Onde:** `src/command.rs:82`, `std::process::Command::new(name).args(args).status()`.
  - **Motivo:** o `PathBuf` guardado em `Command::Executable(_, path, _)` nunca é usado. `Command::new(name)` busca o nome no PATH de novo, então pode executar um binário diferente do que `type` reportou (por exemplo, se o PATH mudar, ou se houver `.` no PATH). Também é trabalho duplicado.
  - **O que fazer:**
    ```rust
    use std::os::unix::process::CommandExt;

    std::process::Command::new(path).arg0(name).args(args).status()
    ```
    Assim executa exatamente o caminho indexado e mantém `argv[0]` igual ao nome digitado. Adicionar um teste de integração com um script temporário no PATH.

- [ ] **B2. Erros de uso do `type` vão para o stdout com código 0**
  - **Onde:** `src/prompt_parser.rs:37-38` (`make_echo("too many arguments")`, `make_echo("type requires an argument")`), testes `type_without_argument` e `type_with_too_many_arguments` em `tests/shell.rs`.
  - **Motivo:** depois de mover o "command not found" para o stderr, esses erros ficaram inconsistentes: são erros, mas saem no stdout e retornam sucesso. Usar `Echo` para isso também mistura "mensagem de erro" com "comando echo".
  - **O que fazer:**
    - Criar uma variante `Command::UsageError(String)` (ou `Error { message, code }`) que escreve no stderr e retorna `InterpResult::error(1)`.
    - Fazer `parse` retornar essa variante nesses dois casos.
    - Atualizar `tests/shell.rs` para usar `assert_clean_run_with_stderr` e ajustar os testes unitários.

- [ ] **B3. `InterpResult::error(code)` é usado também para sucesso**
  - **Onde:** `src/command.rs:83`, `Ok(s) => InterpResult::error(s.code().unwrap_or(1))`.
  - **Motivo:** quando o programa termina bem, o código é 0 e o resultado vem de uma função chamada `error`. Confunde a leitura.
  - **O que fazer:** renomear para `InterpResult::with_code(code: i32)` e manter `ok()`/`exit()` como atalhos.

- [ ] **B4. Erro de leitura persistente causa loop infinito**
  - **Onde:** `src/main.rs`, braço `Err(error)` do `read_line`, que escreve a mensagem e retorna `Ok(false)`.
  - **Motivo:** se o erro se repetir (por exemplo, stdin fechado ou erro de I/O), o `repl` imprime o erro e o prompt sem parar.
  - **O que fazer:** tratar `io::ErrorKind::Interrupted` como "tentar de novo" e, para qualquer outro erro, retornar `Err(error)` (ou `Ok(true)` depois de escrever a mensagem), para o shell terminar.

- [ ] **B5. Código de saída é ignorado**
  - **Onde:** `src/main.rs` (usa só `should_finish`), `src/command.rs` (`Command::Exit` sem argumento).
  - **Motivo:**
    - `return_code` nunca é lido. `exit 3` sai com código 0, e `exit foo` é aceito em silêncio.
    - Um processo morto por sinal vira código 1 (`unwrap_or(1)`); o bash usa `128 + sinal`.
  - **O que fazer:**
    - Guardar o último `return_code` em `main` e sair com ele (`std::process::exit`).
    - Aceitar `exit [n]` no parser (`Command::Exit(i32)`), com erro para argumento inválido.
    - Para sinais, usar `std::os::unix::process::ExitStatusExt::signal` e retornar `128 + sig`.

- [ ] **B6. Pequenas redundâncias em `path_loader.rs`**
  - **Onde:** `src/path_loader.rs:21-26` e `:36-51`.
  - **Motivo:**
    - `!programs.contains_key(filename)` é inútil: dentro de um único diretório os nomes são únicos.
    - `entry.file_name().to_str()` seguido de `String::from` pode ser `into_string()`.
    - `load` com `match` e um braço `None => HashMap::new()` fica mais simples com `let ... else`.
  - **O que fazer:**
    ```rust
    let Ok(name) = entry.file_name().into_string() else { continue };
    if is_executable(&metadata) {
        programs.insert(name, entry.path());
    }
    ```
    e, em `load`, `let Some(path_var) = std::env::var_os("PATH") else { return HashMap::new() };`.

- [ ] **B7. Código comentado**
  - **Onde:** `src/prompt_parser.rs:44`, `// Command::Executable(String::from(command), path.to_path_buf(), args)`.
  - **O que fazer:** apagar.

- [ ] **B8. `to_string_lossy()` em saída formatada**
  - **Onde:** `src/command.rs:73`, `writeln!(stdout, "{name} is {}", path.to_string_lossy())`.
  - **Motivo:** `Path::display()` é o idiomático para mostrar um caminho e dispensa a conversão explícita.
  - **O que fazer:** trocar por `path.display()`.

- [ ] **B9. Arquivos não versionados**
  - **Onde:** `REVIEW.md`, `REVIEW-2.md` e `mise.toml` aparecem como untracked no `git status`.
  - **O que fazer:** decidir caso a caso. `mise.toml` (fixa `rust = "1.96"`) faz sentido versionar. Os `REVIEW*.md` podem ir para o repositório ou para o `.gitignore`.

---

## Ordem sugerida de implementação

1. A1 (desbloqueia `clippy -D warnings`).
2. B1, B2, A2 (comportamento e modelagem).
3. B3, B4, B5.
4. A3, A4.
5. B6, B7, B8.
6. A5 (depois de rodar `codecrafters test`), A6 e B9.

## Verificação final

```
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
codecrafters test
```
