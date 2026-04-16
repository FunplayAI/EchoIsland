# 当前实现架构

## 目的

这份文档只描述仓库当前已经落地的结构，方便快速定位职责边界。

最近复查时间：`2026-04-15`

## Workspace 组成

当前 workspace 成员：

- `apps/desktop/src-tauri`：Tauri 桌面宿主，负责窗口、托盘、命令入口、扫描循环与平台服务接线
- `apps/desktop-host`：控制台宿主，主要用于早期联调与命令行验证
- `apps/hook-bridge`：把外部 Hook 事件转换为 EchoIsland IPC 事件
- `crates/core`：事件协议、状态归一化、会话数据模型
- `crates/runtime`：运行时编排、快照聚合、pending 队列、持久化同步
- `crates/ipc`：本地 TCP IPC、鉴权、请求收发
- `crates/persistence`：会话状态保存与恢复
- `crates/paths`：统一的配置、状态、日志与 Hook 路径规则
- `crates/adapters`：工具适配器状态检测、安装逻辑、fallback 扫描逻辑

## 当前主链路

当前实现不是单一入口，而是三条链路汇总到 `runtime`：

```text
外部工具 Hook
    -> hook-bridge
    -> ipc
    -> runtime

本地 fallback 扫描
    -> adapters
    -> desktop scan runner
    -> runtime

前端 UI / Tauri command
    -> desktop commands/services
    -> runtime / platform services
```

## 已经明确的职责边界

### `core`

- 定义事件协议与核心状态模型
- 不处理平台细节
- 不持有 UI 逻辑

### `runtime`

- 汇总不同来源的 session
- 生成前端快照
- 维护权限 / 提问等 pending 队列
- 驱动持久化保存

### `ipc`

- 提供默认本地地址 `127.0.0.1:37891`
- 做 payload 限制与 token 鉴权
- 把外部输入送入运行时

### `adapters`

- 提供 Codex / Claude / OpenClaw 等工具侧能力检测入口
- 承载 Hook 安装、状态探测、fallback 扫描等工具相关逻辑
- 不直接承担桌面 UI 状态管理

### `apps/desktop/src-tauri`

当前已拆出几类桌面服务：

- `commands.rs`：Tauri 命令入口
- `app_runtime.rs`：应用级运行时持有与接线
- `session_scan_runner.rs`：后台扫描循环与 watcher 退抖
- `terminal_focus_service.rs`：终端跳转与绑定
- `window_surface_service.rs`：灵动岛窗口尺寸 / stage / 显隐控制
- `platform.rs` / `platform_stub.rs`：平台能力与 stub 实现

### `apps/desktop/web`

前端当前已经不是单文件脚本，主要拆分为：

- `snapshot/`：快照刷新、状态队列、完成态跟踪
- `renderers/`：卡片、标题、面板渲染
- `actions/`：用户操作
- `mascot/`：角色状态机与绘制
- `panel-controller.js` / `snapshot-controller.js`：面板与快照协同

## 当前架构判断

这套结构已经比早期实现更适合继续迭代，但仍然是“Windows 先行、跨平台预留”的状态。

当前最重要的事实：

- 运行时核心已经相对平台无关
- 平台相关点主要集中在窗口、终端跳转、输入捕获与工具接线
- 前端也已经具备继续拆分的基础

## 过时内容检查

本次已去掉旧文档里不够准确的旧表述，例如：

- 笼统的 `desktop-ui` 命名
- 没体现平台能力 / 平台 stub / 服务拆分的旧架构描述

当前这份说明与代码目录基本一致，可继续作为近期参考。
