# FARD Null/Optional/Access Semantics — Frozen Contract

## Field Access

| Receiver | Field | Result |
|---|---|---|
| record with field | any | field value |
| record without field | any | **ERROR** — "no member 'x'" |
| null | any | **ERROR** — "no methods on type: unit" |
| non-record | any | **ERROR** — "no methods on type: T" |

Rationale: absent field is an ERROR, not null.
Use `rec.has` before access, or `rec.getOr(r, key, default)` for safe access.
This is the stable substrate for future `?.` sugar.

## Null Coalescing (??)

| Left | Right | Result |
|---|---|---|
| null | any | right |
| non-null | any | left (right not evaluated) |

Precedence: same as `||` (prec 1, lowest infix).
Left-associative: `a ?? b ?? c` = `(a ?? b) ?? c`.

## Error Propagation (?)

| Receiver | Result |
|---|---|
| `{t:"ok", v:x}` | unwraps to `x` |
| `{t:"err", e:x}` | propagates — fn returns `{t:"err", e:x}` |
| any other value | **ERROR** — "? requires result record" |

Rationale: `?` is explicitly for result records only.
Non-result values are an error — this catches misuse early.

## Pipe (|>)

| Left | Right | Result |
|---|---|---|
| any value | fn | `fn(value)` |
| null | fn | `fn(null)` — pipe does NOT short-circuit |

Rationale: pipe is pure function application.
Use `?? default` before pipe if null should stop the chain.

## Computed Key Access

| Key type | Result |
|---|---|
| Text | uses as-is |
| Int | converts to string: `123` → `"123"` |
| other | format as debug string |

## Interaction Matrix

| Expression | Result |
|---|---|
| `null ?? false \|\| true` | `true` |
| `false \|\| null ?? true` | `true` |
| `null ?? true && true` | `true` |
| `null \|> type.of` | `"unit"` |
| `{t:"ok",v:1}?` | `1` |
| `{t:"err",e:"x"}?` inside fn | fn returns `{t:"err",e:"x"}` |
| `42?` | **ERROR** |
| `null.x` | **ERROR** |
| `{y:1}.x` | **ERROR** |
| `rec.getOr(r, "x", null)` | `null` (safe) |

## Summary: null vs absent vs error

- **null** — valid Unit value. Stored, compared, passed normally.
- **absent field** — accessing missing key in record is an ERROR.
- **error result** — `{t:"err",e:...}` is a value, propagated by `?`.

These three are distinct. `?.` sugar (future) maps absent+null → null.
Until then: use `rec.has`, `rec.getOr`, `??` explicitly.
