# 模块依赖与职责图

## 目的

这份文档帮助快速理解“模块依赖”和“职责落点”。

最近复查时间：`2026-04-15`

## 总览

当前仓库可以概括成：

- Rust 内核
- Tauri 桌面宿主
- 工具适配与 Hook 桥接
- Web 前端展示层

更贴近当前代码的关系如下：

```text
外部工具 / hook / 本地扫描
        │
        ├─ hook -> hook-bridge -> ipc -> runtime
        └─ fallback scan -> adapters -> desktop -> runtime

runtime -> persistence

desktop -> runtime
desktop -> adapters
desktop -> ipc
desktop -> paths

desktop web -> tauri commands -> runtime / platform services
```

## Rust 侧模块

### `crates/core`

- 协议类型
- session 数据结构
- 事件归一化与状态基础

### `crates/runtime`

- 运行时总编排
- 快照聚合
- pending 权限 / 提问队列
- 与持久化层同步

### `crates/persistence`

- 会话状态保存
- 启动恢复

### `crates/ipc`

- 本地 TCP 监听与发送
- token 鉴权
- payload 大小限制

### `crates/paths`

- 用户目录、应用状态目录
- Hook 配置路径
- bridge 日志路径

### `crates/adapters`

- 工具状态查询
- Hook 安装入口
- fallback 扫描实现

## 应用层模块

### `apps/hook-bridge`

- 接收外部工具 Hook 上下文
- 转换为统一 IPC 事件
- 不承载 UI 逻辑

### `apps/desktop/src-tauri`

- 应用启动
- 命令入口
- 扫描循环
- 终端跳转
- 窗口 surface 控制
- 平台能力暴露

### `apps/desktop/web`

当前前端已经按职责拆开：

- 快照调度
- 状态队列
- 面板渲染
- 操作绑定
- 角色动画

## 当前依赖判断

从跨平台角度看，目前最值得保留的分层是：

- `core/runtime/persistence` 作为平台无关核心
- `desktop/src-tauri` 作为平台宿主与桥接层
- `desktop/web` 作为相对独立的 UI 层

平台差异最大的部分仍然是：

- 窗口行为
- 终端聚焦 / 标签页跳转
- 工具输入捕获与本地集成

## 过时内容检查

本次复查修正了旧版依赖图里容易让人误解的点：

- 不是所有输入都先经过 `adapters`
- `hook-bridge` 是独立入口，不是 `adapters` 的附属
- 当前前端已明显细分，不再适合用单一 `desktop-ui` 概括
