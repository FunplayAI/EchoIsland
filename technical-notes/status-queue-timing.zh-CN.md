# 状态队列时序参数说明

## 目的

这份文档只回答一件事：

- `apps/desktop/web/ui-context.js` 里的状态队列相关时序参数，各自控制什么

目标是让后续调参时，不需要再回头顺代码猜语义。

---

## 参数位置

当前参数集中在：

- `apps/desktop/web/ui-context.js`

主要分为三组：

- `timings.statusQueue`
- `timings.pendingCard`
- `timings.interaction`

---

## `timings.statusQueue`

### `completionMs`

- 含义：完成卡片在状态队列中的基础停留时长
- 当前作用：决定 `completion` 从出现到开始进入退场阶段的时间
- 调大效果：完成卡片停留更久
- 调小效果：完成卡片更快开始消失

### `approvalMs`

- 含义：授权卡片在状态队列中的基础停留时长
- 当前作用：决定 `approval` 从出现到开始进入退场阶段的时间
- 调大效果：授权卡片更久不自动过期
- 调小效果：授权卡片更快自动退出

### `exitMinMs`

- 含义：单张状态卡片退场动画的最小时长基线
- 当前作用：和 `cardExitDurationMs` 一起决定 `removing` 阶段持续多久
- 调大效果：单卡退场更慢、更明显
- 调小效果：单卡退场更快

### `exitExtraMs`

- 含义：在通用卡片退出动画基础上额外补的缓冲时长
- 当前作用：避免卡片刚开始退场就被真正移出队列
- 调大效果：退场结束后更稳，但节奏更慢
- 调小效果：节奏更利落，但过小可能显得生硬

### `refreshLeadMs`

- 含义：精确定时刷新时，相对目标时间点补偿的毫秒数
- 当前作用：让状态队列在动画结束点附近更稳定地触发下一次刷新
- 调大效果：刷新更保守，避免错过时点
- 调小效果：更贴近真实到点时间

### `refreshMinDelayMs`

- 含义：精确定时刷新允许的最小延迟
- 当前作用：避免出现过密、近似同步的连续刷新
- 调大效果：更稳，但更不敏捷
- 调小效果：更灵敏，但可能更频繁

### `autoCloseHoverSuppressMs`

- 含义：状态队列清空并自动收起后，对 hover 自动展开的短暂抑制时间
- 当前作用：避免刚收起又因为鼠标仍在附近立刻重新展开
- 调大效果：更不容易反弹展开
- 调小效果：用户更快可以靠 hover 再次拉起

---

## `timings.pendingCard`

### `minVisibleMs`

- 含义：pending 卡片的最短显示时间
- 当前作用：避免真实 pending 刚出现又因短暂状态波动立刻消失
- 调大效果：卡片更稳，不容易闪
- 调小效果：响应更快，但更容易闪烁

### `releaseGraceMs`

- 含义：pending 卡片在真实源消失后的额外保留时间
- 当前作用：给 UI 一个平滑释放窗口
- 调大效果：pending 更稳，退场更柔和
- 调小效果：pending 更快离场

---

## `timings.interaction`

### `compactActionHoverSuppressMs`

- 含义：在紧凑态点击灵动岛触发跳转后，对 hover 自动展开的抑制时间
- 当前作用：避免点击跳转后立即又被 hover 展开大胶囊
- 调大效果：点击后的界面更稳
- 调小效果：更快恢复 hover 触发能力

---

## 调参建议

### 1. 调“状态卡片停留多久”

优先改：

- `statusQueue.completionMs`
- `statusQueue.approvalMs`

### 2. 调“退场是不是顺滑”

优先改：

- `statusQueue.exitMinMs`
- `statusQueue.exitExtraMs`

### 3. 调“收起后会不会误弹回”

优先改：

- `statusQueue.autoCloseHoverSuppressMs`
- `interaction.compactActionHoverSuppressMs`

### 4. 调“pending 会不会闪”

优先改：

- `pendingCard.minVisibleMs`
- `pendingCard.releaseGraceMs`

---

## 一句话版本

- `statusQueue` 管“状态卡片怎么活、怎么退、什么时候补刷新”
- `pendingCard` 管“pending 卡片稳不稳”
- `interaction` 管“用户操作后 hover 会不会误触发”
