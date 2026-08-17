# 第一阶段架构

## 1. 总体边界

```text
实验定义
   │
   ▼
无界面模拟内核 ──→ 事件日志 / 检查点 / 指标
                         │
                         ▼
                 回放与 3D 可视化
```

模拟内核是唯一的动力学事实来源。界面只能提交实验命令和读取结果，不能通过渲染循环推进模拟。

## 2. 技术方向

当前建议：

```text
模拟内核        Rust
桌面容器        Tauri
界面            TypeScript
3D              Three.js + WebGL2
分析辅助        Python（可选，不进入正式模拟路径）
```

理由和约束记录在 [decisions/0001-initial-technical-direction.md](decisions/0001-initial-technical-direction.md)。

## 3. Rust 模块

```text
sim-core        时间、事件队列、状态推进和确定性规则
model-lif       LIF 与瞬时电压型突触
experiment      配置、刺激、消融和运行控制
event-store     事件块、检查点、版本和恢复
metrics         放电率、同步性、静默和轨迹指标
app-bridge      批量快照与前端命令，不包含模型逻辑
```

模块名是职责说明，不要求第一天拆成六个独立包。只有边界稳定或需要独立测试时才拆包。

当前 Gate 2 仍使用一个 Rust crate，并按源码模块分离：

```text
src/lif.rs          单神经元解析演化
src/network.rs      突触、空间传播、事件队列和日志
src/metrics.rs      网络指标和轨迹差异
src/experiment.rs   固定种子生成器与对照实验
```

## 4. 时间与调度

内核维护确定性优先队列：

```text
(eventTime, eventPriority, targetId, eventId)
```

事件类型至少包括：

```text
ExternalStimulus
PacemakerInput
SpikeArrival
Intervention
HomeostasisUpdate   // MVP 默认关闭
CheckpointRequest
```

同一目标、同一时刻的 `SpikeArrival` 先进行确定性聚合，再更新一次神经元。具体优先级必须在编码前形成独立表格和测试。

Gate 1 已固定实际排序为：外部事件先按 `(time, targetId, externalEventId)` 规范化；运行中同刻事件按 `(targetId, internalSequence)` 处理；同一目标只形成一个输入批次。新产生的突触事件只能到达未来时刻。

## 5. 模拟与界面通信

禁止逐脉冲发送 JSON。模拟线程按时间窗口或数量阈值生成批次：

```text
SimulationBatch
├── timeRange
├── spikeEvents[]
├── arrivalEvents[]
├── sampledNeuronState[]
└── metricSamples[]
```

密集数组采用紧凑二进制表示。JSON 只用于低频命令、配置和可读元数据。

界面以 30/60 FPS 绘制，但只读取模拟时间。播放速度和掉帧不得改变事件日志。

## 6. 3D 查看器

- 神经元使用 GPU 实例化绘制；
- 连接使用批量线段或曲线缓冲区；
- 默认不显示全部连接；
- 支持选择、搜索、局部展开和连接追踪；
- 在途脉冲的位置由 `sendTime` 与 `arrivalTime` 插值；
- `modelPosition` 与只影响布局的 `viewPosition` 分离；
- 相机状态不写入模拟检查点。

## 7. 存储与回放

实验目录的逻辑结构：

```text
experiment/
├── manifest.json
├── model.json
├── inputs/
├── events/
├── checkpoints/
└── metrics/
```

要求：

- `manifest` 记录格式版本、模型版本、程序版本、平台、参数和种子；
- 密集事件按块存储，不保存每个 tick 的完整状态；
- 检查点必须包含继续推演所需的全部状态，包括随机数状态和在途事件；
- 未知或不兼容版本必须明确拒绝，不做静默猜测；
- 分支引用父实验和分叉时间，父实验保持只读。

具体二进制格式在出现第一批真实事件数据后决定，避免过早固定字段。

## 8. 性能约束

第一阶段从结构上保证：

- 模拟与界面位于不同线程；
- 神经元热状态使用连续数组；
- 突触使用连续邻接存储；
- 事件和前端更新批量处理；
- 可视化采用实例化和可见性筛选；
- 基准测试覆盖事件吞吐、内存占用、检查点恢复和播放帧率。

模拟器默认限制最多处理 10,000,000 个输入、最多排队 1,000,000 个输入。限制可以显式调整，但不能被静默绕过；其作用是让错误参数以明确错误终止，而不是耗尽内存。

第一阶段不使用 GPU 计算神经动力学。只有 CPU 基准确认模拟是瓶颈，并且目标规模无法通过数据布局、批处理和并行实验解决时，才单独评估 GPU。

## 9. 架构守则

- 没有第二种实际模型时，不构建通用模型插件系统；
- 模型代码不能引用 Tauri、Three.js 或 UI 类型；
- 指标和归因不能反向修改真实模型状态；
- 新机制必须有开关、对照实验和版本标识；
- 优化前先建立可重复基准，优化后验证事件结果没有变化。
