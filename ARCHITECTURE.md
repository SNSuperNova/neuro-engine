# 架构

本文记录已经实现的系统。总研究问题见 [RESEARCH_PROGRAM.md](RESEARCH_PROGRAM.md)；统一编排、参数测绘和相图界面已由 Map 0 实现，Map 1 又加入单机制局部重绘和同种子配对，冻结边界见 [MAP_0.md](MAP_0.md) 与 [MAP_1.md](MAP_1.md)。

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

`gate_b.rs` 实现 `delayed-cue-fork/v1` 和三个等预算控制器。三者共享 12 个感觉、24 个状态单元、4 个动作、288 个固定输入权重和 96 个可训练动作权重：`stateless` 不保留状态，`state-reset` 在每步读入前清空状态，`leaky-state` 保留固定自递归状态。输入投影对短暂线索使用较强尺度、对持续感觉使用较弱尺度，但三者逐权重相同。`examples/gate_b_lab.rs` 输出 `app/public/gate-b-v1.2.json`。

Gate B 的走廊是确定性的强制延迟段，不让稀疏奖励导航掩盖状态实验。控制器在岔路才通过左转或右转提交分支，在正确终点通过进食取得能量。正式 JSON 保存逐种子指标、配对置信区间、训练曲线、等预算元数据和代表轨迹，但不保存墙钟时间。

同一模块还提供 Gate B 单因素鲁棒性扫描。它只运行 `leaky-state`，改变无信息延迟、线索投影尺度或持续输入尺度，并输出 `app/public/gate-b-robustness-v1.2b.json`。扫描使用独立种子，不修改 Gate B 的机制或结论。

`gate_c.rs` 实现 `delayed-energy-fork/v1`。控制器在当前线索可见时提交左右分支，环境随后关闭线索并强制等待；只有等待结束时才根据先前选择交付真实能量。控制器在所有条件中无状态，只改变 96 条动作连接的资格迹衰减，因此延迟表现差异不会由状态记忆解释。

```text
current cue ──► fixed random features ──► stochastic branch action
                                              │
                                              ▼
                                   decaying action eligibility
                                              │
real energy after forced delay ───────────────┘──► policy update
```

`examples/gate_c_lab.rs` 输出完整 4×4 因子矩阵、逐种子结果、配对效应、训练曲线、代表时间轨迹与验收项到 `app/public/gate-c-v1.3.json`。随机线索对照使用相同预算和时间结构，只破坏线索与目标的关系。

`gate_d.rs` 在 Gate B 延迟线索接口上增加固定稀疏跨单元矩阵。每个状态单元接收 6 条非自身边；结构化矩阵按左右线索投影偏好设置同组正边、异组负边，然后缩放到谱半径 0.15。状态同步更新，只读取上一时刻的完整隐藏向量。

```text
                           ┌── sparse recurrent matrix ──┐
                           ▼                              │
sense [12] ──► fixed projection ──► state [24] ──► action [4]
                                          │              │
                                          └─ stability   └─ 96 trainable weights
```

`no-recurrence` 将跨单元矩阵置零；`shuffled-recurrence` 对结构化矩阵做隐藏身份相似变换，保留全部权重、稀疏度、正负边和谱但破坏其与输入投影的对应。三者只训练相同 96 条状态到动作连接。`examples/gate_d_lab.rs` 输出 `app/public/gate-d-v1.4.json`，其中同时保存延迟扫描、配对效应、稳定性指标、预算摘要和代表轨迹。

`gate_e.rs` 在同一个延迟线索接口上比较无状态、连续状态和事件式 LIF。三者共享固定输入投影、动作读出、训练序列和动作随机流；连续状态与 LIF 还共享 48 个概念动态标量和 0.86 的时间衰减。LIF 每环境步向 24 个 `LifNeuron` 注入带整数微秒时间戳的聚合电压事件，动作层读取低通脉冲迹。

```text
sense [12] ─► shared fixed projection [288]
                     ├─ stateless tanh
                     ├─ continuous state + adaptation [24 + 24]
                     └─ LIF membrane + spike trace [24 + 24]
                                      │
                                      ▼
                         shared trainable readout [96] ─► action [4]
```

`examples/gate_e_lab.rs` 输出 `app/public/gate-e-v1.5.json`。确定性结果保存行为、样本效率、活动、事件计数、概念状态内存、25% 损伤和代表轨迹；平台墙钟时间单独保存，避免破坏科学报告的精确重现。

`gate_f.rs` 实现 `reversal-cue-fork/v1`。原规则预训练只修改 48 个内部状态到左右分支的动作权重，之后将动作读出逐位冻结。四个分支从同一控制器克隆，只改变内部可塑边界：关闭全部内部更新、开放 48 个线索感觉投影、开放 48 个结构化循环入边，或开放相同循环入边但关闭逐单元 L2 归一化。

```text
cue [left/right] ─► state [24] ─► frozen branch readout [48]
        │                 ▲
        ├─ sensory eligibility [48]      (one condition only)
        └─ recurrent eligibility [48]    (one condition only)
                         │
reward × frozen action feedback ─────────┘
```

每个回合的资格量只累积突触前活动与突触后局部敏感度，结果奖励和冻结动作反馈作为第三因子。适应阶段使用共享的 16% 动作探索，评估关闭探索。`examples/gate_f_lab.rs` 输出检查点曲线、逐种子配对指标、读出与模型摘要、权重漂移及代表内部轨迹到 `app/public/gate-f-v1.6.json`。

## 5. Web 与桌面

`app/` 使用原生 TypeScript、DOM 和 Canvas 2D，不再依赖 Three.js。主界面展示：

- 主体、食物、危险与实际路径；
- 能量、动作和即时后果；
- 感觉输入、24 个内部状态和动作概率；
- 未训练、关闭学习、学习后、置乱和消融对照；
- 训练曲线和验收项。
- Gate B 的线索写入、无信息延迟、岔路选择、内部状态和配对证据。
- Gate B 的延迟、线索带宽和持续干扰边界曲线；
- Gate C 的资格迹 × 能量延迟矩阵、因果效应与能量到账时间轨迹。
- Gate D 的三控制器记忆边界、等权重因果比较和状态稳定性指标。
- Gate E 的连续/LIF 记忆边界、脉冲轨迹、损伤结果和成本对照。
- Gate F 的规则反转/恢复曲线、冻结读出证据、内部轨迹和权重范数漂移。
- Map 0 的四轴相图、开发/确认切换、配置探针分解和因果对照。
- Map 1 的局部确认相图、双时间尺度/旧机制同种子配对和漂移对比。

Tauri 只是同一 Web 应用的桌面壳。核心 crate 不依赖浏览器、Tauri 或 UI 类型。

## 6. 保留的事件网络

`lif.rs` 已通过 Gate E 接入受控具身闭环；`network.rs`、`metrics.rs` 和 Gate 0～2 测试继续作为底层资产。Gate E 只使用单元 LIF、瞬时电压输入和低通脉冲迹，不代表完整事件网络、电导突触、AdEx 或脉冲可塑性已经获得行为证据。后续机制仍须逐项接入并独立验收。

## 7. Map 0 实验编排层

`src/learnability_map.rs` 在现有 Gate 之上增加统一实验编排层，不重写旧 Gate 的冻结结果：

```text
版本化底层配置
        │
        ├──► 固定探针组 ──► 行为与学习指标
        ├──► 稳定性审计 ──► 活动、权重与资源指标
        └──► 因果干预   ──► 形成概率与边界变化
                              │
                              ▼
                         参数区域相图
```

编排层保证同一底层参数跨任务复用，开发种子和确认种子隔离，旧 Gate 可以独立回归。Web 读取版本化结果，展示区域和置信度；它不参与参数选择，也不在浏览器中重新训练。

## 8. Map 1 单机制层

`src/map1.rs` 复用 Map 0 的控制器、探针、分类与统计，只注入一个可选择的内稳态机制。双时间尺度分支保存每单元活动 EMA 与兴奋性增益，并对开放循环权重组做对数范数慢回拉；参考范数分支保留 Map 0 的精确投影路径。

确认阶段的每个新机制运行都有同参数、同种子的旧机制运行，因此差值不包含随机世界变化。Map 1 还复用五项因果干预。Web 只读取 `map1-v0.2.json`，展示局部相图和配对差值。
