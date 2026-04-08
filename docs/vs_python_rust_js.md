# FARD vs Python vs Rust vs JS

Same problem. Four languages.

## List pipeline

Python:  sum(x*x for x in range(1,11) if x % 2 == 0)
FARD:    list.range(1,11) |> list.filter(x => x%2==0) |> list.map(x => x*x) |> list.fold(0, (a,x) => a+x)

## Safe field access

JS:    const host = config?.db?.host ?? 'localhost'
FARD:  let host = config?.db?.host ?? 'localhost'

Identical syntax. Every FARD execution is also a cryptographic artifact.

## Error propagation

Rust:  fn pipeline(x: i64) -> Result<i64,String> { let a = div(x,2)?; let b = div(a,2)?; Ok(b) }
FARD:  fn pipeline(x) { let a = div(x,2)? let b = div(a,2)? { t:'ok', v:b } }

No type annotations. No lifetimes. Same safety.

## Record update

JS:    const updated = { ...defaults, color: 'red' }
FARD:  let updated = { ...defaults, color: 'red' }

## The difference

Every FARD run produces: fard_run_digest=sha256:...
What ran. On what inputs. With what code. Verifiable by anyone.
Python, Rust, JS cannot do this. FARD does it by default.
