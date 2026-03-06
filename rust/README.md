# Rust Kernel And Services

## Workspace Members

- `qala-shared`
- `qala-kernel`
- `ai-agents-service`
- `security-sem-service`

## Build

```powershell
cd rust
cargo check --workspace
```

## Run

```powershell
cargo run -p qala-kernel
cargo run -p ai-agents-service
cargo run -p security-sem-service
```
