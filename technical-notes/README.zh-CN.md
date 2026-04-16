# 技术说明笔记

这个目录只放“解释类 / 实现类”文档，方便回看当前代码是怎么工作的。

和 `docs/` 的分工：

- `technical-notes/`：实现说明、运行机制、参数说明、模块关系
- `docs/`：路线图、方案、任务清单、风险、产品与执行文档

## 当前文档

- `architecture-current.zh-CN.md`
- `architecture-current.en.md`
- `current-low-overhead-scan.zh-CN.md`
- `current-low-overhead-scan.en.md`
- `module-dependency-map.zh-CN.md`
- `module-dependency-map.en.md`
- `windows-terminal-tab-focus.zh-CN.md`
- `windows-terminal-tab-focus.en.md`
- `window-surface-behavior.zh-CN.md`
- `window-surface-behavior.en.md`
- `status-queue-timing.zh-CN.md`
- `status-queue-timing.en.md`

## 命名规范

后续新增说明文档，统一按下面的规则：

- 双语正文文件：`<topic>.zh-CN.md` + `<topic>.en.md`
- `topic` 统一使用英文 kebab-case
- 主题名尽量描述“实现内容”，不要写成任务名或会议纪要名
- 目录入口保留 `README.md`，语言索引分别放在 `README.zh-CN.md` 与 `README.en.md`

推荐示例：

- `session-lifecycle.zh-CN.md`
- `session-lifecycle.en.md`
- `platform-capabilities.zh-CN.md`
- `platform-capabilities.en.md`

不推荐示例：

- `会话生命周期说明.md`
- `平台能力说明英文版.md`
- `关于扫描的一些记录.md`

补充约定：

- 如果某份文档暂时只有中文或英文，建议尽快补齐配对版本
- 在补齐前，文件名也尽量保留语言后缀，避免后续重命名成本

## 新鲜度检查

最近一次人工复查时间：`2026-04-15`

本次已确认并修正的点：

- 旧的单语文档已补成中英文双份
- 架构描述已按当前 workspace 与服务拆分重写
- 低开销扫描说明已按当前 watcher + 增量扫描链路更新
- Windows Terminal 跳转说明已按当前 `terminal_focus` 模块更新

当前未发现这批 `technical-notes/` 中有明显过时的实现描述。

需要注意：

- 这些文档描述的是“当前实现”，不是对未来结构的承诺
- 如果后续继续推进跨平台抽象、Hook 原生接入或 UI 架构重组，需要再复查一次
