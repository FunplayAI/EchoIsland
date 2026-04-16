# Mac / Win UI 重构计划

## 背景

当前桌面端 UI 分成两条渲染路径：

- Windows / WebView：`apps/desktop/web/*`
- macOS 原生岛：`apps/desktop/src-tauri/src/macos_native_test_panel.rs`

这保证了 macOS 灵动岛区域的可控性，但也带来了内容 UI 双份维护的问题。后续如果继续直接迁样式和动画，维护成本会持续上升。

## 目标

重构为：

- **共享内容层**：会话卡片、待审批、待提问、空态、live tool、状态文案、排序逻辑
- **平台壳体层**：macOS 灵动岛原生壳体、Windows 窗口壳体
- **平台布局层**：不同平台使用不同尺寸、间距、动画参数与局部布局

核心原则：

1. 不强行让 mac 和 win 完全一样
2. 共享“内容结构与逻辑”，区分“壳体与布局”
3. mac 继续保留原生能力处理刘海、安全区、命中区域、层级

## 非目标

以下内容不作为本次重构第一阶段目标：

- 一次性移除全部 mac 原生 UI
- 一次性重写所有动画
- 改动运行时、扫描器、会话数据结构
- 为了统一而牺牲 mac 灵动岛定位稳定性

## 当前架构判断

### 共享部分

- 会话数据：`crates/runtime`
- 状态聚合：`crates/core`
- Tauri 启动与命令：`apps/desktop/src-tauri/src/main.rs`

### 平台专属部分

- macOS 原生壳体与内容：`apps/desktop/src-tauri/src/macos_native_test_panel.rs`
- Windows / WebView 内容：`apps/desktop/web/*`
- Windows 形状与窗口控制：`apps/desktop/src-tauri/src/island_window.rs`

问题不在数据层，而在 **内容 UI 实现重复**。

## 目标架构

### 1. 原生壳体层

负责平台必须原生处理的部分：

- 顶部位置与安全区适配
- 命中区域 / 非矩形区域
- 置顶层级 / 空间切换行为
- 紧凑态到展开态的外壳骨架动画

### 2. 共享内容层

尽量统一到前端渲染：

- session card
- pending permission / pending question
- empty state
- live tool
- 标题、副标题、状态 pill、badge
- 列表排序、状态映射、时间格式

### 3. 平台布局层

允许同一份内容在不同平台使用不同 token：

- 宽度
- 高度
- 圆角
- padding / gap
- compact 顶栏布局
- 展开动画参数
- 是否显示某些局部元素

## 推荐方案

采用：

- **mac：原生壳体 + 统一内容层**
- **win：现有 WebView 内容层 + 现有窗口壳体**

即：

- mac 原生继续负责灵动岛容器
- 容器内部内容尽量回归共享前端
- 平台差异通过 token 和少量结构分支控制

## 文件拆分建议

### 保留原生

- `apps/desktop/src-tauri/src/macos_native_test_panel.rs`
- `apps/desktop/src-tauri/src/island_window.rs`
- `apps/desktop/src-tauri/src/main.rs`

### 共享内容

- `apps/desktop/web/renderers/session-list-renderer.js`
- `apps/desktop/web/renderers/panel-measure.js`
- `apps/desktop/web/utils.js`

### 样式分层

建议新增：

- `apps/desktop/web/styles.shared.css`
- `apps/desktop/web/styles.platform.macos.css`
- `apps/desktop/web/styles.platform.windows.css`

如果短期不想拆太多文件，也可以先保留一个 `styles.css`，按 section 分出：

- shared tokens
- mac tokens
- windows tokens
- platform overrides

## 平台区分方式

在前端根节点注入平台标识：

- `data-platform="macos"`
- `data-platform="windows"`

以及平台能力标识：

- `data-shell="native"`
- `data-shell="webview"`

这样可以用统一组件 + 平台样式覆盖，而不是大量 JS 条件分支。

## 第一阶段任务

### Phase 1：整理平台 token

目标：

- 不改现有可用视觉结果
- 先把平台差异从“散落实现”变成“显式配置”

任务：

- 定义 shared / mac / windows 三组 token
- 给前端根节点加 `data-platform`
- 给动画参数、宽高、间距建立平台映射

产出：

- 统一的平台样式入口
- 更容易迁移 compact / expanded 内容

### Phase 2：expanded 内容统一

目标：

- 把 expanded 内部卡片内容统一为共享前端实现

任务：

- session card 改为共享 renderer
- pending card 改为共享 renderer
- live tool / empty state 改为共享 renderer
- 保留 mac 原生外壳和展开骨架动画

产出：

- Win / mac expanded 内容基本共用
- 后续样式调整改一处即可

### Phase 3：compact 内容收敛

目标：

- 评估 compact 顶栏中哪些可以共享，哪些继续保留原生

建议：

- 数字滚动、刘海贴边布局、原生命中区继续保留 mac 专属
- 文案、状态颜色、badge 语义尽量共享

### Phase 4：动画参数统一管理

目标：

- 统一动画参数来源，但保留平台差异

任务：

- 提取 Win / mac 的 easing、duration、drop distance
- 保证“内容动画可共享，壳体动画可分平台”

## 实施顺序

建议按以下顺序推进：

1. 先建立平台 token 与平台标识
2. 再统一 expanded 内容
3. 然后处理 compact 内容收敛
4. 最后再整理动画体系

不要反过来先统一动画，否则会被现有双份内容实现拖住。

## 风险

### 风险 1：mac 原生壳体和 Web 内容同步不稳定

应对：

- 原生只负责容器尺寸与位置
- 内容高度测量由共享层统一输出

### 风险 2：平台差异被“统一”过度抹平

应对：

- 允许 mac / win 使用不同布局 token
- 仅共享内容逻辑，不强制共享所有尺寸

### 风险 3：迁移过程中视觉回退

应对：

- 每次只迁一个区域
- 保留当前可用的 mac 原生实现作为回退基线

## 验收标准

达到以下标准视为重构方向正确：

- 修改一处卡片内容结构，Win / mac 同步生效
- 修改平台 token，不影响另一平台内容逻辑
- mac 仍正确贴合刘海与安全区
- Win 保持现有窗口表现
- 动画与布局差异被限制在平台层，而不是整套 UI 分叉

## 下一步

建议立即执行：

1. 建立平台标识注入
2. 抽出平台 token
3. 先把 expanded 内容区迁回共享前端

---

如果后续继续实施，这份文档可以作为总任务清单持续更新。
