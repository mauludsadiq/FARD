# FARD Package Manager

FARD has a built-in package manager. Packages are content-addressed, date-versioned, and declared in a `fard.toml` manifest at the root of your project.

-----

## fard.toml

Every FARD project has a `fard.toml` at its root:

```toml
name = "myapp"
version = "2026-03-11"
entry = "main.fard"

[deps]
greet = "greet@2026-03-11"
math  = "fard-math@2026-02-15"
```

Fields:

- `name` — package name (lowercase, no spaces)
- `version` — date-based version string (`YYYY-MM-DD`)
- `entry` — entry point `.fard` file
- `[deps]` — map of short import names to `"packagename@version"` strings

-----

## Importing Packages

Once declared in `[deps]`, import by short name:

```
import("pkg/greet") as greet

greet.hello("world")   // → "Hello, world!"
```

The runtime resolves `pkg/greet` to `greet@2026-03-11` via `fard.toml`, then fetches or loads the package.

You can also import with an explicit version (no `fard.toml` needed):

```
import("pkg:greet@2026-03-11") as greet
```

And import a specific module within a package:

```
import("pkg:greet@2026-03-11/utils") as greet_utils
```

-----

## Writing a Package

A package is a directory containing a `fard.toml` and one or more `.fard` files:

```
greet/
  fard.toml
  main.fard
```

`fard.toml`:

```toml
name = "greet"
version = "2026-03-11"
entry = "main.fard"
```

`main.fard`:

```
fn hello(name) {
  "Hello, ${name}!"
}

fn farewell(name) {
  "Goodbye, ${name}."
}

export { hello, farewell }
```

Any name in the `export { ... }` block is available to importers. Everything else is private.

-----

## Publishing a Package

```bash
fardrun publish --package ./greet --token <github-token>
```

This:

1. Reads `fard.toml` for name and version
1. Creates a tarball `greet@2026-03-11.tar.gz`
1. Uploads it to a GitHub release tagged `pkg-greet-2026-03-11`
1. Updates `registry.json` with the new entry

The registry is a static JSON file hosted as a GitHub release asset. No central server. No registry authority. Anyone can host a registry.

-----

## Installing Packages

```bash
fardrun install
```

Reads `fard.toml`, resolves all `[deps]`, and downloads them into the local cache at `~/.fard/cache/`. Subsequent runs use the cached version — no network required.

```bash
fardrun install --dep greet@2026-03-11
```

Install a specific package without a `fard.toml`.

-----

## Local Registry

For development or air-gapped environments, point `fardrun` at a local directory instead of the network:

```bash
fardrun run --program main.fard --registry ./local_registry --out ./out
```

Local registry layout:

```
local_registry/
  pkgs/
    greet/
      2026-03-11/
        fard.toml
        main.fard
    fard-math/
      2026-02-15/
        fard.toml
        main.fard
```

-----

## Version Policy

FARD uses date-based versions: `YYYY-MM-DD`.

There are no floating versions, no `^1.2.3`, no `latest`. Every dependency in `fard.toml` is a frozen version string. This is intentional — it is the same principle as content-addressed RunIDs. A dependency either resolves to exactly what you declared or it fails. There is no ambiguity.

If you want to upgrade a dependency, change the version string in `fard.toml` explicitly.

-----

## Content Addressing

Packages are verified by SHA-256 digest on download. The registry entry for each package includes its source digest:

```json
{
  "packages": {
    "greet@2026-03-11": {
      "url": "https://github.com/.../greet@2026-03-11.tar.gz",
      "sha256": "sha256:4dda9ce7..."
    }
  }
}
```

`fardrun install` verifies the digest before extracting. A package whose bytes do not match its declared digest is rejected.

This means the package manager shares the same structural guarantee as the language itself: same name + same version → same bytes → same digest → same behavior. A FARD dependency is not a moving target.

-----

## Package Witness

When a program imports a package, the package’s source digest is recorded in `module_graph.json`:

```json
{
  "nodes": [
    { "spec": "pkg/greet", "kind": "pkg", "digest": "sha256:4dda9ce7..." }
  ]
}
```

This means the `fard_run_digest` of any program that imports a package commits to the exact source of every package it used. The computation is fully reproducible — not just in theory but verifiably, by digest.

-----

## fard.toml for This Repo

The FARD v0.5 repository itself uses the following manifest:

```toml
name = "fard"
version = "1.0.0"
entry = "examples/hello/main.fard"

[deps]
# No external deps — FARD's stdlib is built in
```

-----

## Summary

|Feature                              |Status                 |
|-------------------------------------|-----------------------|
|`fard.toml` manifest parsing         |✓                      |
|`import("pkg/name")` short resolution|✓                      |
|`import("pkg:name@version")` explicit|✓                      |
|Local registry (`--registry`)        |✓                      |
|`fardrun publish` to GitHub releases |✓                      |
|Package digest verification          |✓                      |
|Package witness in module graph      |✓                      |
|`fardrun install`                    |Roadmap v1.2           |
|Central registry                     |Not planned — by design|