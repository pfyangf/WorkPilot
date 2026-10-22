# WorkPilot

共享 Rust 业务核心。Windows、Android 和 HarmonyOS 的界面尚未创建。

## 要求

- Rust stable
- 本阶段不需要 Node、Android SDK、DevEco Studio 或模型密钥

## 测试

```powershell
cargo test -p workpilot-domain
cargo test -p workpilot-contracts
cargo test --workspace
```

Phase 0 的实际结果见 `docs/compatibility/phase0.md`。
