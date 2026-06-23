# Braille transport — design + validated BrlAPI protocol notes

intone separates **braille translation** (pure, in `intone-core::braille`) from **braille
transport** (how cells/text reach an output). Transport is a pluggable port so the dev/remote
text sink and a physical-display sink can coexist.

## The seam (implemented)

```rust
trait BrailleSink { fn show(&mut self, text: &str); }
```

- **`TextBrailleSink`** (implemented): translates the announcement to uncontracted (Grade 1)
  Unicode braille via `intone_core::braille::to_braille` and prints `[braille] ⠓⠑⠇⠇⠕`. This is the
  channel used for headless/remote dev, and it works today.
- **`BrlApiSink`** (planned, see below): sends to a physical display via BRLTTY's BrlAPI.

The `Speaker` holds an `Option<Box<dyn BrailleSink>>`, selected at startup from `Settings.braille`.

## Translation ownership (important)

**BRLTTY translates text → braille itself** (its own tables, contracted/Grade 2, per language).
So the device adapter must send the **raw announcement text** via BrlAPI `writeText` and let
BRLTTY render it — it must **not** send intone's pre-translated Grade-1 cells. intone's `to_braille`
is therefore the *text-sink* rendering (for sighted devs / terminals), not the device path.
(Contracted braille for the text sink, if ever wanted, would use liblouis behind `to_braille`.)

## BrlAPI protocol — validated against a live `brltty` BrlAPI server (release 0.8.6, protocol v8)

Validated empirically (connecting to a real `brltty -A auth=none,host=127.0.0.1:0` server):

- **Framing:** every packet is `[u32 size][u32 type][payload]`, all **big-endian**. `type` is the
  ASCII code of the packet letter as a `u32` (e.g. `'v'` = `0x76`). Confirmed: server's first
  packet was `size=00000008 type='v' payload=00000008`.
- **Version exchange:** server sends `VERSION('v')` with `u32` protocol version (8); the client
  **must reply** with its own `VERSION('v')` packet (without it, the server stalls waiting).
- **Auth:** server then sends `AUTH('a')` listing offered methods as `u32`s; `auth=none` is
  `0x4e` (`'N'`). For **NONE the client sends no auth reply** — it proceeds directly to the next
  instruction. (Sending an `'a'` reply for NONE drew an `ERROR('E')` with code 4,
  "unknown instruction"; sending nothing, the server waits for the next instruction.) `authClientPacket_t`
  is `{ u32 type; u8 key }` — `key` bytes only for `AUTH_KEY`.

### Not yet validated (needs a real display or a non-degenerate virtual brltty)

The only available test server used the `no` braille + `no` screen drivers (display size 0, no
controlling tty), which cannot validate these:

- **`ENTERTTYMODE('t')`** — payload is a tty path: `u32 count`, then `count × u32` tty numbers,
  then the driver-name bytes (length = remainder). With NoScreen the server has no tty, so this
  likely errors `UNKNOWNTTY(12)` there.
- **`WRITE('w')`** — `writeArgumentsPacket_t` = `u32 flags` then fields in flag-weight order:
  `WF_DISPLAYNUMBER(0x01)`→`u32`; `WF_REGION(0x02)`→`u32 begin,u32 size`; `WF_TEXT(0x04)`→`u32
  len` + UTF-8 text; `WF_ATTR_AND(0x08)`/`WF_ATTR_OR(0x10)`→`size` bytes; `WF_CURSOR(0x20)`→`u32`;
  `WF_CHARSET(0x40)`→`u8 len` + charset bytes. `writeText` uses `REGION(1,displaySize) | TEXT |
  CURSOR | CHARSET("UTF-8")`.
- Server replies `ACK('A')` on success, `ERROR('e')`/`EXCEPTION('E')` (`{u32 code; u32 type; …}`)
  on failure.

## `BrlApiSink` adapter design (follow-up)

1. Connect to BrlAPI: `BRLAPI_HOST` env (`host:port`; port = base **4101** + instance) or the local
   abstract socket; TCP to `127.0.0.1:4101` for the default local instance.
2. Handshake: read `VERSION` → send `VERSION(8)` → read `AUTH` offer → (NONE: no reply) →
   `ENTERTTYMODE` → expect `ACK`.
3. `show(text)` → `WRITE` with `writeText` flags carrying the **raw text**; expect `ACK`.
4. **Fail closed to the text sink:** any connect/handshake/write error makes the sink unavailable
   and intone falls back to `TextBrailleSink` — braille output never breaks the reader.
5. Keep `libbrlapi`/`liblouis` **out of the Cargo graph** (LGPL): speak the socket protocol
   directly (the BRLTTY daemon is the arm's-length boundary), consistent with the project's
   permissive-licensing strategy.

Pure packet **builders** (version/enter-tty/write → bytes) should be unit-tested against the
framing above before the live adapter lands.
