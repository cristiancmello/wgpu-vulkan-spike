# wgpu-vulkan-spike — Guia de Desenvolvimento TDD (Freeman/GOOS)

## Visão Geral

Este projeto é um servidor de renderização que aceita comandos de desenho via Unix socket de um processo externo (ex: Scheme). O desenvolvimento segue o estilo **Freeman/GOOS** (Growing Object-Oriented Software, Guided by Tests):

1. **Teste de aceitação E2E define o contrato** — especifica o comportamento observável do sistema
2. **Ciclo duplo** — testes de aceitação envolvem ciclos de testes unitários
3. **Implementação guiada pelos testes** — código mínimo para passar nos testes

---

## Fases de Desenvolvimento

### Fase 1 (ATUAL): Draw Command Buffer

**Objetivo**: Estabelecer um canal de entrada de comandos via socket e acumulá-los em um buffer compartilhado.

**Escopo**:
- [ ] Unix socket listener em `/tmp/wgpu-draw.sock`
- [ ] Parser de S-expressions para comandos de desenho
- [ ] `DrawCommand` enum com tipos: `Clear`, `DrawTriangle`, `DrawRect`, `Present`
- [ ] `Arc<Mutex<Vec<DrawCommand>>>` como buffer compartilhado
- [ ] Thread separada para socket (não bloqueante para o event loop principal)

**Teste de Aceitação E2E**:
```rust
// tests/acceptance_buffer.rs
#[test]
fn client_sends_draw_triangle_command_appears_in_buffer() {
    // 1. Sobe socket listener
    // 2. Conecta via cliente
    // 3. Envia: "(draw-triangle 0.0 0.5 -0.5 -0.5 0.5 -0.5 1.0 0.0 0.0 1.0)\n"
    // 4. Verifica que buffer contém DrawCommand::DrawTriangle com valores corretos
}
```

Este teste define o contrato externo.

#### Checklist de Implementação — Fase 1

**RED (Testes falhando)**
- [ ] Escrever teste de aceitação em `tests/acceptance_buffer.rs`

**GREEN (Testes passando)**
- [ ] Implementar `DrawCommand` enum + `parse_command()` em `src/draw_command.rs`
- [ ] Implementar `spawn_socket_listener()` em `src/socket_listener.rs`
- [ ] Integrar socket listener em `src/main.rs`

**VERIFY**
- [ ] Teste E2E passa: socket → parse → buffer contém comandos corretos
- [ ] `cargo test` — todos os testes passam

**REFACTOR**
- [ ] Revisar nomes, estrutura, simplicidade

---

### Fase 2 (FUTURA): Renderizador consome o buffer

**Objetivo**: Fazer o renderizador wgpu consumir os comandos do buffer e apresentar cada frame.

**Teste de Aceitação E2E**:
```rust
// tests/acceptance_render.rs
#[test]
fn buffer_with_draw_triangle_renders_to_surface() {
    // 1. Sobe servidor
    // 2. Envia draw-triangle
    // 3. Captura frame (mock/offscreen buffer)
    // 4. Verifica que pixels têm a cor esperada
}
```

**Escopo**:
- [ ] RenderPipeline com shader WGSL (triângulos coloridos por vértice)
- [ ] Consumir buffer a cada frame
- [ ] Converter DrawCommand → Vertex
- [ ] Criar vertex buffer dinâmico
- [ ] Render pass com clear color dinâmico

---

## Ciclo Duplo Freeman/GOOS

```
┌────────────────────────────────────────────┐
│  Teste de Aceitação (RED — falhando)       │
│  "enviar S-expr via socket →                │
│   buffer contém DrawCommand correto"       │
│                                            │
│  ┌──────────────────────────────────────┐  │
│  │  Ciclo Unitário (RED → GREEN)        │  │
│  │  - parse_command unit tests          │  │
│  │  - DrawCommand está bem formado?     │  │
│  │  - Acumulação no buffer funciona?    │  │
│  └──────────────────────────────────────┘  │
│                                            │
│  Teste de Aceitação (GREEN — passando) ✓  │
└────────────────────────────────────────────┘
```

**O teste de aceitação NUNCA testa internals** — ele apenas verifica o comportamento externo observável. Os testes unitários guiam a implementação interna.

---

## Protocolo de Comandos (S-expressions)

Cada comando é uma S-expression em uma linha, terminado com `\n`.

| Comando | Formato | Exemplo |
|---|---|---|
| Clear | `(clear r g b a)` | `(clear 1.0 0.0 0.0 1.0)` |
| Draw Triangle | `(draw-triangle x1 y1 x2 y2 x3 y3 r g b a)` | `(draw-triangle 0.0 0.5 -0.5 -0.5 0.5 -0.5 1.0 0.0 0.0 1.0)` |
| Draw Rectangle | `(draw-rect x y w h r g b a)` | `(draw-rect -0.5 -0.5 1.0 1.0 0.0 1.0 0.0 0.5)` |
| Present | `(present)` | `(present)` |

**Coordenadas**: NDC (Normalized Device Coordinates), X e Y em `[-1.0, 1.0]`. Y cresce para cima.
**Cores**: RGBA, cada componente em `[0.0, 1.0]`.

---

## Estrutura de Tipos

### `DrawCommand` enum

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum DrawCommand {
    Clear {
        r: f32, g: f32, b: f32, a: f32,
    },
    DrawTriangle {
        x1: f32, y1: f32,
        x2: f32, y2: f32,
        x3: f32, y3: f32,
        r: f32, g: f32, b: f32, a: f32,
    },
    DrawRect {
        x: f32, y: f32, w: f32, h: f32,
        r: f32, g: f32, b: f32, a: f32,
    },
    Present,
}
```

### `parse_command` function

```rust
/// Parseia uma S-expression em um DrawCommand.
/// Retorna None se a string não for válida.
pub fn parse_command(input: &str) -> Option<DrawCommand> {
    // Tokeniza, mapeia nome do comando + floats
}
```

---

## Executando os Testes

```bash
# Todos os testes (unitários + aceitação)
cargo test

# Apenas testes de aceitação (E2E)
cargo test acceptance

# Testes com output (println)
cargo test -- --nocapture

# Um teste específico
cargo test client_sends_draw_triangle_command_appears_in_buffer
```

---

## Regras Freeman/GOOS

1. **Nunca escrever código de produção sem um teste falhando**
   - Escrever teste → Red → Implementação → Green → Refactor

2. **Teste de aceitação é a especificação externo**
   - Define o comportamento observável do sistema completo
   - Não testa detalhes internos (isso são testes unitários)

3. **Testes unitários guiam o design interno**
   - Uma unidade = uma responsabilidade
   - Prefere objetos colaboradores com interfaces claras

4. **Separar testes de aceitação de testes unitários**
   - Aceitação em `tests/` (fora de `src/`)
   - Unitários em `#[cfg(test)]` módulos em `src/`

5. **Proibições**
   - `unwrap()` em código de produção (OK em testes)
   - Muita abstração cedo (YAGNI)
   - Implementar Fase 2 antes de Fase 1 estar passando

---

## Arquivos Principais

| Arquivo | Responsabilidade | Status |
|---|---|---|
| `src/main.rs` | Event loop principal, State, integração do socket | ⏳ |
| `src/draw_command.rs` | `DrawCommand` enum, `parse_command()`, testes unitários | ⏳ |
| `src/socket_listener.rs` | `spawn_socket_listener()`, lógica de thread do socket | ⏳ |
| `tests/acceptance_buffer.rs` | Teste E2E do buffer | ⏳ |
| `Cargo.toml` | Dependências | ✅ |

---


---

## Documentação do Protocolo (para cliente Scheme)

```scheme
;; Cliente Scheme se conecta a /tmp/wgpu-draw.sock
;; Envia uma S-expression por vez, terminada com newline

;; Exemplos de uso:
(write-to-socket "(clear 0.0 1.0 0.0 1.0)\n")   ;; fundo verde
(write-to-socket "(draw-triangle 0.0 0.5 -0.5 -0.5 0.5 -0.5 1.0 0.0 0.0 1.0)\n")
(write-to-socket "(present)\n")                  ;; opcionalmente flush
```
