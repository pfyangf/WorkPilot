# WorkPilot 技术设计文档

- 文档版本：v2.0
- 日期：2026-10-22
- 对应 PRD：WorkPilot PRD v2.0
- 决策：`docs/adr/0001-shared-rust-core.md`
- 技术原则：模块化单体、本地优先、业务逻辑与 UI 分离、接口适配、易测试、不过度设计

v1.0 中的 Python 3.12、PySide6、SQLAlchemy、Alembic、httpx、keyring、PyInstaller 和 pytest 方案已废弃。本文不再保留该实现路径。

## 1. 技术选型

### 1.1 技术栈

| 层次 | 方案 | 说明 |
|---|---|---|
| 共享业务核心 | Rust stable、Cargo workspace、edition 2021 | 领域规则、用例和跨端契约 |
| 序列化 | serde、serde_json | DTO 和跨语言 JSON |
| 标识与时间 | UUID v7 字符串、UTC RFC 3339 | `work_date` 使用 `YYYY-MM-DD` |
| Windows UI | Tauri 2、React、TypeScript | 主工作台 |
| Windows 数据 | SQLite、SQLx | Rust 负责数据访问；迁移用 SQLx migrate |
| Windows 集成 | Tauri 与 Win32 | 托盘、文件对话框、凭据存储、后续 Coding 采集 |
| Android | Kotlin、Jetpack Compose | JNI 或已验证 FFI 调用 Rust |
| HarmonyOS | ArkTS、ArkUI | 优先验证 NAPI；失败则走 HTTPS 客户端 |
| 同步 | Phase 4 的 HTTPS API | 不在 Phase 0–3 实现 |
| 测试 | Rust 单元测试；UI 按平台补少量测试 | 领域规则不依赖 UI |
| 日志 | 各平台本地滚动日志 | 写入前过滤密钥和令牌 |

具体 crate 版本在建立 workspace 时锁定，并在 `docs/compatibility/` 记录实际 `rustc` 版本。未执行的工具链检查一律写「未验证」。

### 1.2 模块边界

应用以个人和小团队为主。使用一个 Cargo workspace 中的少量 crate，不拆微服务，不为每个函数建 trait。

只在这些位置定义接口：数据库、文件系统、LLM Provider、Coding 工具、操作系统凭据和通知。

Phase 4 的同步 API 是一个独立进程，只服务已授权设备。它不是微服务平台。Windows 桌面进程不默认监听公网。

### 1.3 尚未验证的能力

以下能力在写本文时都没有构建记录，不得在 README、界面或后续文档中写成已支持：

- 本机 Rust stable 能否完成 workspace 构建
- Kotlin 通过 JNI 调用 Rust
- ArkTS 通过 NAPI 调用 Rust，包括 DevEco Studio、目标 HarmonyOS SDK、链接、打包和真机
- Tauri 2 托盘、SQLx 迁移和 React 界面

鸿蒙 NAPI 验证失败时，鸿蒙端只保留 HTTPS 客户端，等待 Phase 4。该失败不阻塞 Windows MVP。

## 2. 系统架构

```text
┌────────────────────────────────────────────────────────────┐
│ UI（各平台独立）                                            │
│ Windows: React + Tauri                                     │
│ Android: Jetpack Compose                                   │
│ HarmonyOS: ArkUI                                           │
└────────────────────────────┬───────────────────────────────┘
                             │ DTO / 命令
┌────────────────────────────▼───────────────────────────────┐
│ Platform adapters                                          │
│ Windows 托盘、凭据、文件                                    │
│ Android JNI、权限、通知                                     │
│ HarmonyOS NAPI 或 HTTPS 客户端                              │
└────────────────────────────┬───────────────────────────────┘
                             │
┌────────────────────────────▼───────────────────────────────┐
│ Application                                                │
│ TaskService / WorkLogService / ReportService               │
│ LlmConfigService / DesktopService / CodingIntegrationSvc   │
└───────────────┬────────────────────────┬───────────────────┘
                │                        │
┌───────────────▼──────────────┐  ┌──────▼───────────────────┐
│ Domain                       │  │ Contracts                │
│ 实体、状态机、校验、错误     │  │ DTO、错误码、同步字段    │
└───────────────┬──────────────┘  └──────────────────────────┘
                │
┌───────────────▼────────────────────────────────────────────┐
│ Infrastructure                                             │
│ SQLite / SQLx / 文件系统 / LLM Provider / 凭据引用         │
└────────────────────────────────────────────────────────────┘
```

分层规则：

- UI 负责展示、输入和交互，不写业务规则，不直接执行 SQL。
- Application 编排用例、事务和外部调用。
- Domain 只包含实体、状态流转、校验和领域错误。不依赖数据库、HTTP、Tauri、JNI、NAPI。
- Contracts 是跨端稳定数据契约。
- Infrastructure 实现存储、网络、模型和文件系统适配器。
- 平台差异留在对应 adapter。不要为了复用把托盘、通知或鸿蒙生命周期塞进 Rust 核心。
- 跨语言只传递拥有完整数据的 DTO：字符串、数字、布尔、枚举和这些字段组成的结构。不跨 FFI 传递 Rust 对象指针或闭包。

## 3. 目录

Phase 0 只创建能编译和测试的部分：

```text
WorkPilot/
├─ Cargo.toml
├─ rust-toolchain.toml
├─ README.md
├─ docs/
│  ├─ PRD.md
│  ├─ TECHNICAL_DESIGN.md
│  ├─ README_AI_CODING.md
│  ├─ adr/0001-shared-rust-core.md
│  └─ compatibility/
└─ crates/
   ├─ workpilot-domain/
   └─ workpilot-contracts/
```

后续阶段再添加，Phase 0 不预先生成空工程：

| 路径 | 阶段 | 职责 |
|---|---|---|
| `crates/workpilot-application` | Phase 1 | 用例编排 |
| `crates/workpilot-storage-sqlite` | Phase 1 | SQLx 与迁移 |
| `crates/workpilot-report` | Phase 1 | 来源收集、脱敏、版本 |
| `crates/workpilot-ai` | Phase 1 | LLM Provider |
| `apps/windows` | Phase 1 | Tauri 2 与 React |
| `apps/android` | Phase 2 | Kotlin 与 Compose |
| `apps/harmony` | Phase 3 | ArkTS；桥接失败时仅为 API 客户端 |
| `crates/workpilot-sync`、`services/sync-api` | Phase 4 | 认证、增量同步、冲突处理 |

## 4. 核心数据模型

需要同步的实体使用 UUID v7 字符串作为主键，并包含 `updated_at`、`version` 和 `deleted`。`version` 从 1 起，每次业务更新加 1。`deleted` 为软删除标记。禁止用 SQLite `INTEGER PRIMARY KEY` 充当全局 ID。

时间戳存储 UTC。展示时转换为本地时区。日报和日志的 `work_date` 是用户选择的本地日期。

### 4.1 Project

| 字段 | 类型 | 约束 |
|---|---|---|
| id | UUID v7 字符串 | 主键 |
| name | string | 必填 |
| description | text | 可空 |
| status | string | `active` / `archived` |
| created_at / updated_at | datetime | 必填 |
| version | integer | 必填 |
| deleted | bool | 默认 false |

任务可以不关联项目。Phase 1 可以先不做独立项目管理页。

### 4.2 Task

| 字段 | 类型 | 约束 |
|---|---|---|
| id | UUID v7 字符串 | 主键 |
| project_id | UUID nullable | 可不关联项目 |
| parent_task_id | UUID nullable | 最多一层子任务 |
| title | string | 必填，有长度上限 |
| description | text | 可空 |
| status | enum | `todo` / `in_progress` / `blocked` / `done` / `cancelled` |
| priority | enum | `low` / `medium` / `high` |
| start_date / due_date | date nullable | 可空 |
| is_continuous | bool | 持续任务不因日报生成而关闭 |
| created_at / updated_at | datetime | 必填 |
| completed_at | datetime nullable | 仅 `done` 有值 |
| version | integer | 必填 |
| deleted | bool | 默认 false |

状态流转：

| 当前 | 可转到 |
|---|---|
| `todo` | `in_progress`、`blocked`、`done`、`cancelled` |
| `in_progress` | `todo`、`blocked`、`done`、`cancelled` |
| `blocked` | `todo`、`in_progress`、`cancelled` |
| `done` | `todo`、`in_progress` |
| `cancelled` | `todo` |

进入 `done` 时写入 `completed_at`。离开 `done` 时清空 `completed_at`。非法流转返回 `invalid_transition`。

父任务转为 `done` 时，若仍有未完成子任务，返回 `needs_confirmation`，不自动完成子任务。调用方显式确认后才允许父任务完成。

### 4.3 WorkLog

| 字段 | 类型 | 说明 |
|---|---|---|
| id | UUID v7 字符串 | 主键 |
| task_id | UUID nullable | 可关联任务 |
| work_date | date | 用户工作日期 |
| content | text | 必填 |
| source_type | enum | `manual` / `coding_tool` / `system` / `ai_generated` |
| source_ref | string nullable | 外部事件引用 |
| created_at / updated_at | datetime | 必填 |
| version | integer | 必填 |
| deleted | bool | 默认 false |

新日志追加，不覆盖旧日志，也不修改任务状态。关联任务时只更新该任务的 `updated_at`。日报按 `work_date` 取日志。

手工日志允许修改和删除，并写入审计记录。`ai_generated` 不能改写成 `manual`。

### 4.4 TaskEvent

| 字段 | 类型 | 说明 |
|---|---|---|
| id | UUID v7 字符串 | 主键 |
| task_id | UUID | 必填 |
| event_type | string | `created`、`status_changed`、`edited`、`completed`、`reopened` 等 |
| old_value / new_value | JSON nullable | 不存密钥和令牌 |
| summary | string | 可读说明 |
| source_type | string | `manual` / `system` / `integration` |
| created_at | datetime | 必填 |

任务有日志或事件时，默认软删除或归档，不级联删除历史。

### 4.5 DailyReport

| 字段 | 类型 | 说明 |
|---|---|---|
| id | UUID v7 字符串 | 主键 |
| work_date | date | 用户工作日期 |
| version_no | integer | 同一日期内递增，展示用版本号 |
| title | string | 默认「日期 + 日报」 |
| content_markdown | text | 当前正文 |
| content_origin | enum | `generated` / `user_edited` |
| status | enum | `draft` / `reviewed` / `exported` |
| model_name | string nullable | 生成时的模型名 |
| created_at / updated_at | datetime | 必填 |
| version | integer | 同步版本，从 1 起 |
| deleted | bool | 默认 false |

同一日期可有多条记录。`(work_date, version_no)` 唯一。默认展示该日期未删除记录中 `version_no` 最大的一条。

用户保存编辑后，`content_origin` 变为 `user_edited`。对这条记录重新生成时返回 `report_edit_conflict`，正文保持不变。要留下新的生成结果，必须插入更大的 `version_no`，不得更新已编辑行。

### 4.6 ReportSource

| 字段 | 类型 | 说明 |
|---|---|---|
| id | UUID v7 字符串 | 主键 |
| report_id | UUID | 日报 |
| source_type | string | `task` / `work_log` / `task_event` / `coding_event` |
| source_id | string | 来源 ID |
| excerpt | text nullable | 短摘要，避免敏感内容 |
| created_at | datetime | 快照时间 |

编辑日报正文不改 ReportSource。

### 4.7 CodingSessionSnapshot

Phase 1 之后使用。字段：`id`、`tool_name`、`external_session_id`、`task_id` nullable、`started_at`、`ended_at` nullable、`status`、`summary`、`updated_at`、`version`、`deleted`。

只保存摘要。不默认保存完整提示词、终端输出、源代码、完整本地路径或文件内容。重复导入按外部 ID 或稳定哈希去重。事件不自动改变任务状态。

### 4.8 SyncMetadata

Phase 4 使用。字段至少包括：`device_id`、`entity_type`、`entity_id`、`version`、`updated_at`、`deleted`、`last_synced_at` nullable、`pending`。

默认同步排除：AI 密钥、访问令牌、凭据引用所指向的秘密、原始 Coding 会话、完整文件路径、本地文件内容。

### 4.9 DesktopRule 与 OperationHistory

仅 Windows 后续阶段。手机端没有这些用例。

DesktopRule：`id`、`name`、`enabled`、`priority`、`match_type`、`match_value`、`target_folder`、`created_at`、`updated_at`。

OperationHistory：`id`、`batch_id`、`operation_type`、`source_path`、`target_path`、`status`、`error_message` nullable、`created_at`、`undo_status`。

执行前检查源存在、目标可写、名称冲突，以及路径是否越出用户允许的根目录。默认冲突策略是跳过。删除不是整理规则的默认动作。

### 4.10 数据库

- 开启外键。
- 索引：`Task.status`、`Task.due_date`、`WorkLog.work_date`、`TaskEvent.task_id`、`DailyReport.work_date`。
- 结构变更使用 SQLx 迁移。禁止每次启动删除并重建表。
- 写操作使用事务，失败回滚。
- Phase 0 不创建数据库。

## 5. 关键流程

### 5.1 创建或更新任务

1. UI 检查必填项和日期。
2. 领域模型执行状态和子任务规则。
3. 仓储在一个事务内保存任务。
4. 新建或状态变化时写入 TaskEvent。
5. 返回 DTO。
6. 失败写入日志，并向用户说明下一步。日志不得包含密钥。

### 5.2 追加工作日志

1. 用户提供内容和 `work_date`，任务可选。
2. 新增 WorkLog。
3. 不修改旧日志，不改变任务状态。
4. 更新关联任务的 `updated_at`。

### 5.3 生成日报

1. 收集指定 `work_date` 的任务事件、工作日志，以及用户勾选的 Coding 摘要。
2. 按来源分组、去重并限制长度。
3. 向用户展示来源摘要。
4. 做基础脱敏。用户确认后才发送到云端模型。
5. Provider 返回空内容或调用失败时，不写入新正文。
6. 若目标日报已是 `user_edited`，返回 `report_edit_conflict`。
7. 成功时插入新的草稿版本和 ReportSource。
8. 生成过程不修改任务状态。
9. 导出 Markdown 前处理重名。导出失败时保留原日报。

提示词要求模型只根据来源写作，把事实和建议分开。未提供的完成情况写成「待确认」。命令启动、会话结束和 Git 变更都不是任务完成。

### 5.4 文件整理

仅 Windows 后续阶段，并使用临时目录做自动化测试。

1. 扫描只读取用户选定目录的直接子项，除非用户开启递归。
2. 规则引擎只生成计划。
3. 预览显示源路径、目标路径、命中规则和冲突。
4. 用户确认后逐项执行，每项单独记录结果。
5. 撤销前检查目标仍存在、源路径空闲，且文件仍符合恢复条件。

桌面路径通过 Windows Known Folder 获取。不能假定桌面一定位于用户目录下的 `Desktop`。

## 6. LLM Provider

调用方依赖 trait，不依赖厂商 SDK。UI 不直接请求模型。

```rust
pub struct LlmRequest {
    pub system_prompt: String,
    pub user_prompt: String,
    pub model: String,
    pub temperature: f32,
    pub max_output_tokens: u32,
    pub timeout_seconds: u64,
}

pub struct LlmResponse {
    pub text: String,
    pub model: Option<String>,
}

pub trait LlmProvider {
    async fn generate(&self, request: LlmRequest) -> Result<LlmResponse, LlmError>;
}
```

Phase 1 至少实现一个 OpenAI-compatible Chat Completions Provider，并提供测试用的假 Provider。Provider 处理鉴权、超时、限流、错误码和响应解析。

密钥存放：

- Windows：系统凭据存储。数据库只存凭据引用。
- Android：Keystore。
- HarmonyOS：系统安全存储。

设置页提供保存、测试、更新和删除。异常、提示词、日报和导出文件不包含密钥。模型失败不阻塞任务和日志功能。

## 7. Coding 工具适配器

Phase 1 不接入真实工具。后续每个工具一个 adapter，可以单独关闭。工具只支持文件导入时，不显示实时状态。

采集在后台运行。自动任务关联只给建议，不直接改任务状态。adapter 故障不影响任务和日报。

禁止用未授权的注入或抓取方式读取工具数据。

## 8. 平台能力

### 8.1 Windows

- 托盘提供显示或隐藏窗口、快速新增任务、打开今日工作台、退出。
- 关闭窗口默认留在托盘。退出时先停止后台任务，再关闭数据库。
- 托盘初始化失败时，主窗口仍可使用，并提示原因。
- 大量文件操作不阻塞 UI，支持进度和取消。
- 窗口布局是独立的后续验证项。恢复失败的窗口必须逐项报告。桌面图标位置和应用窗口位置是两项能力。

### 8.2 Android

原生层负责权限、生命周期、通知和后台。Rust 负责业务规则和 DTO。

配套功能：查看任务和详情、快速新增任务、记录日志、查看进度、查看并轻量编辑日报、查看最近同步时间和待同步状态、查看用户授权后的 Coding 摘要。

### 8.3 HarmonyOS

功能范围与 Android 配套端相同。

NAPI 必须在目标 HarmonyOS SDK、DevEco Studio 和真机上完成构建、链接、调用和打包后，才能记为可用。失败时客户端调用 Phase 4 HTTPS API。Phase 4 完成前，鸿蒙端可以保留未连接的同步状态，不得伪造同步成功。

## 9. 同步

Phase 4 实现。要求：身份认证、设备授权、增量同步、幂等提交、失败重试、冲突检测、设备撤销、敏感字段过滤。

冲突至少识别 `version` 不一致。策略在实现该阶段前单独写清，不在 Phase 0 选一种假同步。

手机客户端不打开 Windows 的 SQLite 文件。

## 10. 错误

领域错误码：

| 代码 | 含义 |
|---|---|
| `validation` | 字段不合法 |
| `invalid_transition` | 状态不允许这样变化 |
| `needs_confirmation` | 父任务仍有未完成子任务 |
| `report_edit_conflict` | 已编辑日报不允许被重新生成覆盖 |
| `not_found` | 实体不存在 |

基础设施另有外部服务、文件和数据库错误，它们不属于 Domain。用户可见文案说明发生了什么以及下一步。技术细节进滚动日志。外部调用必须有超时。

## 11. 测试

领域测试覆盖：

- 合法与非法状态流转、完成时间的写入和清空。
- 未完成子任务阻止父任务静默完成。
- 持续任务按日期追加多条日志且不改任务状态。
- 已编辑日报重新生成返回冲突且正文不变。
- DTO 含稳定 ID、`updated_at`、`version`、`deleted`。

Phase 1 再增加 SQLite 迁移、事务回滚、假 LLM 的成功、超时、鉴权失败和空响应，以及临时目录中的导出。文件测试不操作真实桌面。

Phase 0 验收命令：

```powershell
cargo test -p workpilot-domain
cargo test -p workpilot-contracts
cargo test --workspace
```

只有命令实际退出码为 0，才能在兼容性记录里写成通过。

## 12. 开发阶段

### Phase 0：技术验证

建立 workspace、Task、WorkLog、DailyReport 规则、契约 DTO 和单元测试。记录实际工具链版本。JNI 和 NAPI 另列验证任务，工具链或真机不可用时写明未验证。

### Phase 1：Windows MVP

SQLite 迁移、任务、工作日志、日报生成与编辑导出、LLM Provider、基本 UI、托盘和日志。Coding 采集和桌面整理保持禁用。

### Phase 2：Android

Kotlin 与 Compose、Rust bridge、任务查看和新增、日志、日报轻量编辑、同步状态占位。

### Phase 3：HarmonyOS

ArkTS 工程、NAPI 真机验证、同等配套功能。桥接失败则只保留 HTTPS 客户端边界。

### Phase 4：同步

认证、设备管理、增量、幂等、冲突和同步状态。按需要再加实时通知。

Windows 的桌面整理和 Coding 采集在 MVP 之后单独实施，不插入 Phase 0。

## 13. 实现约束

1. 每次只完成一个可构建、可测试的切片。
2. Domain 不访问数据库、HTTP 和平台 API。
3. 不把平台功能塞进 Rust 核心。
4. 密钥不进源码、普通配置、数据库明文和日志。
5. 未接入的 Coding 工具不显示实时状态。
6. 未实现的按钮禁用或标明规划中。
7. 不把命令启动、会话结束或 Git diff 当成任务完成。
8. 不把假接口描述成已经实现的同步。
9. 不假设鸿蒙原生桥接已经成功。
10. 每个阶段结束时列出文件、实际命令、结果、未验证项和下一阶段。
