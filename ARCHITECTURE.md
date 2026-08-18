# 架构

## 1. 系统数据流

```text
Arena
  │  food / hazard / energy / body state
  ▼
Sensor Encoder [12]
  │
  ▼
Leaky Adaptive Hidden State [24]
  │
  ▼
Stochastic Action Policy [4]
  │
  ▼
forward / turn-left / turn-right / eat
  │
  ├──────────────► Arena transition
  │
  └── eligibility × consequence ──► persistent policy weight change
```

环境和控制器在 Rust 中运行。Web 不重新计算训练，只读取版本化结果并回放行为，因此界面帧率不会改变实验。

## 2. 具身核心

`src/embodied.rs` 包含七个边界：

1. `Arena`：位置、方向、食物、危险、能量消耗和终止条件；
2. `RewardConfig`：分别控制能量变化和距离进展的奖励权重，每一步输出可核对的 `RewardBreakdown`；
3. `SensorConfig`：可关闭食物方向，并独立控制方向精度、噪声和随机遮挡；
4. 感觉编码：能量、当前位置食物、相对食物方向、边界、相对危险方向和最近后果；
5. `AdaptiveController`：固定感觉投影、带泄漏状态与适应项的内部单元；
6. 动作策略：内部状态到四个动作的 softmax 概率；
7. 三因素可塑性：动作误差资格迹乘以后果优势，更新动作突触。

动作策略不能直接读取感觉通道；所有感觉必须经过 24 个内部单元。这个约束让内部单元消融成为真实因果测试，而不是装饰层。

当前 24 个内部单元之间没有相互连接，各自只保留自身泄漏状态和适应状态。因而 v1 应称为“有状态随机特征层”，不能直接称为完整的 Echo State Network、循环神经网络或神经微回路。详细学习公式和技术定位见 [LEARNING.md](LEARNING.md)。

## 3. 训练和测试隔离

训练与评估使用不重叠的确定性地图种子。训练只修改动作突触；评估期间关闭可塑性。正式实验固定比较：

- 相同初始权重的未经训练控制器；
- 经历相同训练地图但关闭可塑性的控制器；
- 学习后控制器；
- 精确保留权重分布但打乱位置的控制器；
- 将最重要 25% 内部单元从动作层断开的控制器。

所有分支使用相同评估地图和动作随机种子。

## 4. 可复现数据

`examples/embodied_lab.rs` 输出 `app/public/embodied-v1.json`：

```text
EmbodiedExperimentResult
├── config
├── trainingCurve
├── evaluations
├── plasticity
├── traces
└── acceptance
```

JSON 只包含数个代表行为轨迹和聚合指标，不保存每个训练回合的所有内部状态。权威数值由 Rust 重新生成，Web 文件是可审查的发布快照。

`gate_a.rs` 在核心实验之外运行 9 个固定干预条件。每个条件聚合独立模型种子，保留逐种子结果，并对学习、关闭学习和配对差报告 95% Student-t 置信区间。`examples/gate_a_lab.rs` 输出 `app/public/gate-a-v1.1.json`。

## 5. Web 与桌面

`app/` 使用原生 TypeScript、DOM 和 Canvas 2D，不再依赖 Three.js。主界面展示：

- 主体、食物、危险与实际路径；
- 能量、动作和即时后果；
- 感觉输入、24 个内部状态和动作概率；
- 未训练、关闭学习、学习后、置乱和消融对照；
- 训练曲线和验收项。

Tauri 只是同一 Web 应用的桌面壳。核心 crate 不依赖浏览器、Tauri 或 UI 类型。

## 6. 保留的事件网络

`lif.rs`、`network.rs`、`metrics.rs` 和 Gate 0～2 测试仍是经过验证的底层资产，但当前具身控制器不会假装它们已经足以模拟真实神经系统。未来只有在行为闭环中能定义明确对照时，才将内部循环连接、LIF、电导突触、AdEx 或脉冲可塑性逐项接入。接入顺序和验收标准见 [ROADMAP.md](ROADMAP.md)。
