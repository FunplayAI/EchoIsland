# 当前低开销扫描实现

## 目的

这份文档只记录当前 Windows 下 Codex fallback 扫描是怎样做到“低开销”的。

最近复查时间：`2026-04-15`

## 当前状态

当前实现已经从“UI 每次刷新顺便扫盘”改成了“后台独立扫描 + 前端只读快照”。

也就是说：

- 前端拉快照不会主动触发磁盘全量扫描
- 扫描频率和 UI 刷新频率已经解耦
- 文件变更优先由 watcher 驱动，轮询只做保底

## 当前链路

```text
Codex 本地文件变化 / 保底轮询
        │
        ▼
desktop 后台 scan loop
        │
        ▼
CodexSessionScanner
  ├─ history.jsonl 增量读取
  ├─ session 文件 mtime + size 判断
  ├─ 最近 session 限量扫描
  └─ SessionRecord 结果缓存比对
        │
        ▼
runtime.sync_source_sessions("codex", ...)
        │
        ▼
前端 get_snapshot 只读取 runtime 快照
```

## 低开销来自哪里

### 1. 扫描循环独立运行

当前后台扫描由 `apps/desktop/src-tauri/src/session_scan_runner.rs` 驱动：

- watcher 事件来了尽快扫
- 没事件时按推荐间隔保底轮询
- watcher 事件会做退抖合并

### 2. `history.jsonl` 做增量读取

当前 `crates/adapters/src/codex/scan.rs` 会维护 history 读取状态：

- `size`
- `modified_at`
- `offset`
- `latest_prompt_by_session`

只有文件变化时才继续往后读，而不是每次从头读取。

### 3. session 文件按变化判断是否重解析

session 文件不是每轮都全量重读，而是看：

- 文件大小
- 修改时间

只有发生变化时才重新解析对应 session 文件。

### 4. 活跃 / 空闲轮询分档

当前推荐间隔分为两档：

- 活跃期：约 `3s`
- 空闲期：约 `15s`

因此活跃时响应更快，空闲时磁盘和 CPU 开销更低。

### 5. UI 只读 runtime 快照

`get_snapshot` 当前职责是读 `runtime.snapshot()` 的结果，不再顺手触发 fallback 扫描。

这一步很关键，因为它避免了：

- 鼠标悬浮导致的额外扫盘
- UI 动画频繁刷新带来的连锁扫描

## 当前边界

这套方案当前主要描述的是：

- Windows 下的 Codex fallback 扫描

它不是所有工具统一的最终方案，也不是未来 Hook 方案的替代品。

## 过时内容检查

本次复查后，这份说明没有发现明显过时点；但要注意两件事：

- 如果 Codex Windows 原生 Hook 能力后续真正可用，这份文档需要补充“Hook 优先、扫描兜底”的新链路
- 如果扫描源扩展到 Claude / OpenClaw 的本地文件扫描，这份文档也需要升格为通用扫描说明
