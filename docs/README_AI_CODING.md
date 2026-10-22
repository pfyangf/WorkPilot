# WorkPilot 文档包

本目录是产品和实现基线。技术栈以共享 Rust 业务核心为准。早期 Python + PySide6 方案已废弃，见 `adr/0001-shared-rust-core.md`。

- `PRD.md`：产品目标、功能范围、用户流程、验收标准。
- `TECHNICAL_DESIGN.md`：技术选型、架构、数据模型、接口、流程、测试和开发阶段。
- `adr/0001-shared-rust-core.md`：已接受的架构决策。

阅读顺序：ADR、PRD、技术设计。三者冲突时，安全、可追溯、人工确认和 MVP 范围优先；技术栈以 ADR 为准；产品行为以 PRD 为准。

## 开发计划

- `superpowers/plans/2026-10-22-roadmap.md`：阶段顺序和完成标准。
- `superpowers/plans/2026-10-22-phase0-rust-workspace.md`：当前可执行的实施计划。

## 当前阶段

Phase 0 的实施计划见 `superpowers/plans/2026-10-22-phase0-rust-workspace.md`。兼容性结果以 `compatibility/phase0.md` 为准。不要直接生成 Windows、Android、HarmonyOS 或同步服务。

## 给实现者的约束

1. 先按阶段交付可构建、可测试的垂直切片。
2. Domain 不调用数据库、HTTP 或平台 API。
3. 跨语言边界只传递普通 DTO，不传递复杂对象所有权。
4. 错误必须显式返回。禁止吞掉异常后假装成功。
5. 密钥不得写入源码、普通配置、SQLite 明文或日志。
6. 数据库结构变更必须有迁移。
7. 日报结论必须能回溯来源。未验证的结果不得写成已完成。
8. 文件操作必须先预览，并由用户确认；手机端不做 Windows 文件整理。
9. 未实现的入口必须禁用或标明规划中。
10. 构建和测试结论只能来自实际执行的命令。兼容性未验证时写明「未验证」。
