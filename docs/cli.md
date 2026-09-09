# BrotaParser CLI

## Compile a contract

```bash
brotaparser contract.brota
```

The command writes the normalized contract as canonical JSON to stdout.

## Read from stdin

```bash
cat contract.brota | brotaparser
```

## Exit codes

| Code | Meaning |
|------|---------|
| `0` | Contract compiled successfully. |
| `1` | Syntax or semantic diagnostics were emitted. |
| `2` | Input could not be read. |

Diagnostics are written as JSON to stderr. Consumers should use stable `BRT-*`
codes instead of matching human-readable messages.
