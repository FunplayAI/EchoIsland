# 灵动岛窗口行为规则

## 目的

这份文档记录当前灵动岛窗口在“自动弹出”和“用户主动唤起”两种场景下的行为边界。

最近复查时间：`2026-04-15`

## 背景

之前灵动岛在自动弹出卡片时，可能会打断用户正在终端或 IDE 中的输入。

根因不是前端动画本身，而是窗口 surface 更新过程中，Windows 侧存在可能触发窗口激活的置顶刷新路径。

所以现在要明确区分两类行为：

- `passive`：系统自动更新窗口形态，不应打断当前输入焦点
- `interactive`：用户主动唤起窗口，可以允许窗口获得焦点

## 当前规则

### 1. `passive`

适用场景：

- 状态队列自动弹出
- 授权 / 提问卡片驱动的大胶囊展开
- 自动收起 / 自动高度调整
- 普通窗口 surface 同步

行为要求：

- 可以更新窗口大小、命中区域、置顶状态
- 不应该主动抢占当前输入焦点
- 不应该把外部正在输入的终端 / IDE 中断

当前对应调用：

- `set_island_bar_stage_passive`
- `set_island_panel_stage_passive`
- `set_island_expanded_passive`

### 2. `interactive`

适用场景：

- 用户明确点击托盘，希望显示主窗口
- 后续如果增加显式“打开主窗口”入口

行为要求：

- 允许显示并激活主窗口
- 允许把焦点切回 EchoIsland

当前对应调用：

- `show_main_window_interactive`

## 当前实现位置

Rust 侧：

- `apps/desktop/src-tauri/src/window_surface_service.rs`
- `apps/desktop/src-tauri/src/commands.rs`
- `apps/desktop/src-tauri/src/island_window.rs`

前端调用侧：

- `apps/desktop/web/api.js`
- `apps/desktop/web/panel-controller.js`

## Windows 关键约束

在 Windows 下，自动弹卡片路径不能依赖“先取消置顶、再重新置顶”的方式刷新窗口层级。

原因是这类调用可能带来额外的窗口激活副作用，进而打断当前输入状态。

当前策略是：

- `passive` 路径只走不激活窗口的置顶刷新方式
- 主动交互才允许使用显式显示 / 聚焦逻辑

## 当前收益

这套拆分之后，语义上已经更清楚：

- 自动弹卡片 = 纯 surface 变化
- 主动打开窗口 = 允许交互激活

这样后续继续做：

- 权限卡片
- 消息卡片
- hover 展开
- 托盘唤起

时，不容易再把“被动刷新”和“主动激活”混到一起。

## 后续约定

如果以后新增窗口相关命令，优先先判断它属于哪一类：

- 是否只是 UI 自动变化
- 是否是用户明确主动操作

只有第二类，才应该进入 `interactive` 语义。
