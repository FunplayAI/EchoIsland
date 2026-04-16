# Windows Terminal 标签页跳转说明

## 目的

这份文档记录当前 Windows Terminal 标签页跳转逻辑是怎么工作的。

最近复查时间：`2026-04-15`

目标：

- 点击灵动岛卡片
- 聚焦正确的终端窗口
- 在可能的情况下切到正确的 `Windows Terminal` 标签页

这是一份当前实现说明，不代表最终跨平台设计。

## 入口

前端入口：

- `apps/desktop/web/main.js`

Tauri 命令入口：

- `apps/desktop/src-tauri/src/commands.rs`
- 命令名：`focus_session_terminal`
- 绑定命令：`bind_session_terminal`

Rust 主要实现区域：

- `apps/desktop/src-tauri/src/terminal_focus_service.rs`
- `apps/desktop/src-tauri/src/terminal_focus/`
- `apps/desktop/src-tauri/src/focus_store.rs`

## 当前数据模型

### `SessionFocusTarget`

一次“我要跳到哪个会话”的目标信息，当前会带上：

- `source`
- `project_name`
- `cwd`
- `terminal_app`
- `host_app`
- `window_title`
- `terminal_pid`

### `SessionTabCache`

当前缓存的 Windows Terminal 标签页信息：

- `terminal_pid`
- `window_hwnd`
- `runtime_id`
- `title`

它会被保存到本地 focus store，供后续点击跳转复用。

## 当前工作方式

### 1. 显式绑定

用户可以对某个会话执行 `bind_session_terminal`：

- 读取当前前台 `Windows Terminal` 标签页
- 把该标签页信息写入会话绑定缓存

这条链路最直接，也最稳定。

### 2. 自动学习

`terminal_focus/learning.rs` 里当前会做两类观察：

- `observe_foreground_terminal_tab`：记录最近前台标签页
- `learn_newly_active_session_tabs`：当快照里只出现一个新的活跃候选时，尝试把它和前台标签页学习绑定

自动学习当前依赖：

- 会话是否进入活跃状态
- `last_user_prompt` 是否变化
- `last_activity` 是否推进

如果同时出现多个候选，会主动放弃学习，避免误绑。

### 3. 点击聚焦

点击卡片时，`TerminalFocusService::focus_session` 会：

- 读取 session 当前信息
- 组合 `SessionFocusTarget`
- 优先使用已缓存标签页
- 再走 token 匹配与窗口聚焦逻辑

Windows 下的具体实现位于：

- `apps/desktop/src-tauri/src/terminal_focus/windows.rs`

## 当前限制

当前方案仍然带有明显 Windows 特征：

- 主要针对 `Windows Terminal`
- 自动学习依赖前台窗口观察
- 标签页匹配本质上仍然是“缓存 + 令牌匹配 + 最近观察”的组合策略

这也是为什么它更像“当前稳定实现”，而不是最终跨平台抽象。

## 过时内容检查

本次复查已修正旧说明中不够准确的点：

- 旧文档只提到零散函数，未覆盖当前 `terminal_focus` 子模块结构
- 旧文档没有体现“显式绑定 + 自动学习 + 最近前台兜底”这三层逻辑

当前这份说明与现有代码结构一致。
