# ADR-0001：采用共享 Rust 业务核心

- 状态：已接受
- 日期：2026-10-22
- 相关文档：`docs/PRD.md`、`docs/TECHNICAL_DESIGN.md`

## 背景

WorkPilot 最初按 Windows 单机应用起草，技术选型是 Python 3.12、PySide6、SQLAlchemy 和 Alembic。产品要长期维护，并增加 Android 与 HarmonyOS 配套端。继续使用该桌面方案会让业务规则写在 Python UI 进程里，手机端无法复用。

## 决策

1. 业务规则放在 Rust workspace 中。Windows、Android、HarmonyOS 各自使用原生 UI。
2. Windows 使用 Tauri 2、React、TypeScript、SQLite 和 SQLx。Rust 负责业务逻辑、系统集成和数据访问。
3. Android 使用 Kotlin 与 Jetpack Compose，通过 JNI 或已验证的 FFI 调用 Rust。权限、生命周期、通知和后台由 Android 原生层负责。
4. HarmonyOS 使用 ArkTS 与 ArkUI。优先在目标 SDK、DevEco Studio 和真机上验证 NAPI。验证失败时，鸿蒙客户端改为调用后续的 HTTPS API，不因此阻塞 Windows 和 Android。
5. 跨设备同步属于 Phase 4。Phase 4 之前不实现同步服务。手机不访问 Windows 本地 SQLite。Windows 桌面服务不默认暴露到公网。
6. 稳定标识使用 UUID v7 字符串。数据库自增 ID 不得作为全局标识。
7. 用户编辑过的日报正文被重新生成时，返回冲突，不覆盖原文。保留生成结果时新建版本，旧版本保留。
8. Phase 0 只建立 workspace、领域模型、契约和测试。不生成三端应用，不建数据库，不调用模型。

## 废弃

以下方案不再使用，后续设计和代码都不要引用：

- Python 作为业务或 UI 实现语言
- PySide6、Qt、pywin32
- SQLAlchemy、Alembic
- httpx、keyring、PyInstaller、pytest、pytest-qt
- `src/workpilot/` 这一 Python 包布局

Windows 凭据仍使用操作系统凭据存储。这是平台能力，不是 Python keyring 方案。

## 后果

- 领域层不依赖数据库、HTTP、Tauri、JNI 和 NAPI。
- 鸿蒙原生桥接在真机验证通过之前，文档和界面都不得写成已支持。
- 桌面文件整理和 Coding 工具采集仍是 Windows 后续能力，手机端不执行。
- 现有 PRD 的任务、日志、日报、安全和可追溯规则继续有效。平台范围和技术栈以本决策及《技术设计文档》为准。
