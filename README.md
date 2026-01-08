# harness-barn

Rust workspace for AI coding harness detection and skill discovery.

## Crates

| Crate | Purpose |
|-------|---------|
| [`harness-locate`](crates/harness-locate) | Detect installed harnesses, resolve config paths, parse MCP configs |
| [`skills-locate`](crates/skills-locate) | Fetch remote skills/plugins from GitHub, parse marketplace registries |

## harness-locate

Cross-platform detection for Claude Code, OpenCode, Goose, AMP Code, and Crush.

```rust
use harness_locate::{Harness, HarnessKind, Scope};

// Find installed harnesses
for harness in Harness::installed()? {
    println!("{} at {:?}", harness.kind(), harness.config(&Scope::Global)?);
}
```

## skills-locate

Discover and fetch plugins from remote sources.

```rust
use skills_locate::{GitHubRef, discover_plugins};

let source = GitHubRef::parse("github:owner/repo")?;
let plugins = discover_plugins(&source).await?;

for plugin in plugins {
    println!("{}: {}", plugin.name, plugin.description);
}
```

## Installation

```toml
[dependencies]
harness-locate = "0.4"
skills-locate = "0.2"
```

## License

MIT
