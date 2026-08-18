# 当前学习机制与技术定位

## 1. 一句话定位

当前控制器是一个**具有连续内部状态、使用奖励调制资格迹在线训练动作读出的具身强化学习器**。

它可以近似描述为：

```text
固定随机感觉投影 + 泄漏状态单元 + softmax 动作策略 + 在线策略梯度资格迹
```

这不是新的机器学习门类，也不是完整的大脑或生物神经元模拟。它将储备池计算、策略梯度、三因素可塑性和具身交互中的已有思想组合成一个小型、透明、可做因果消融的实验系统。

## 2. 每一步如何学习

### 2.1 感觉与内部状态

主体产生 12 个感觉量：偏置、能量、当前位置食物、食物相对方向、前方边界、危险相对方向和上一步后果。感觉先经过固定随机权重，再进入 24 个内部单元。

对内部单元 `i`：

```text
drive_i  = Σ_j input_weight[i,j] × sensor_j - adaptation_strength × adaptation_i
target_i = tanh(drive_i)
hidden_i = leak × previous_hidden_i + (1 - leak) × target_i
```

泄漏项让过去输入在状态中逐渐衰减；适应项抑制长期持续激活。当前内部单元之间**没有相互连接**，因此它还不是标准的循环储备池，也不是脉冲网络。

### 2.2 随机动作策略

24 个内部状态通过 96 条可塑连接映射到 4 个动作：前进、左转、右转和进食。

```text
logit_a = Σ_i policy_weight[a,i] × hidden_i / temperature
probability = softmax(logits)
action ~ probability
```

按概率采样动作提供探索。控制器不能直接读取感觉通道，所有动作必须经过内部状态。

### 2.3 后果信号

当前即时后果为：

```text
reward = energy_change / food_energy + 0.035 × distance_progress
```

吃到食物会增加能量；被动生存、移动、撞墙和危险接触会消耗能量；接近最近食物提供小额塑形奖励。距离塑形是明确的人为辅助，不应被误认为系统自行发现了目标。

### 2.4 资格迹与权重更新

每条动作连接保存一个逐渐衰减的资格迹，用来记录近期内部活动对动作选择的责任：

```text
action_error = selected(action) - probability(action)
eligibility  = 0.88 × old_eligibility + action_error × hidden
advantage    = reward - moving_reward_baseline
weight      += 0.035 × advantage × eligibility
```

这与在线策略梯度的分数函数更新相符，也可以从神经启发角度理解为三个因素共同改变连接：内部单元活动、动作选择误差和全局后果信号。

每个回合开始时清空内部状态和资格迹，但保留动作权重。v1 训练 1,200 个回合，只修改 96 条内部到动作的权重；感觉投影不学习。

## 3. 与已有方向的关系

| 当前组成 | 对应方向 | 关键差异 |
|---|---|---|
| 固定随机感觉投影、只训练读出 | Reservoir Computing / Echo State Network | 当前没有隐藏单元间的循环连接 |
| 连续泄漏状态 | Leaky RNN / 连续状态网络 | 不使用反向传播穿越时间 |
| softmax 动作采样与奖励更新 | REINFORCE / Policy Gradient | 使用逐步在线更新和简单移动基线 |
| 衰减的动作责任记录 | Eligibility Trace | 当前只追踪动作读出连接 |
| 活动 × 动作责任 × 后果 | 三因素学习规则 | 后果是全局标量，不是局部神经调制物质 |
| 身体在环境中采样经验 | Embodied Reinforcement Learning | 世界和感觉仍高度人工设计 |
| 未来的脉冲替代实现 | Liquid State Machine / SNN | v1 不是脉冲网络 |

市面和研究界已经存在强化学习框架、储备池工具、脉冲网络模拟器以及 Intel Loihi 一类神经形态研究硬件。项目当前价值不在宣称算法原创，而在于建立一个可以同时观察行为、内部状态和因果干预的受控实验台。

## 4. 与常见机器学习的区别

若“传统机器学习”指监督学习，区别主要在训练信号和数据产生方式：

| 监督学习 | 当前系统 |
|---|---|
| 给定样本与正确标签 | 只给环境后果 |
| 优化预测误差 | 优化长期行为回报 |
| 数据集通常预先收集 | 行动会改变下一份数据 |
| 常用批训练和全局反向传播 | 每步在线修改少量连接 |
| 输出是分类或数值 | 输出是影响环境的动作 |

若与现代深度强化学习比较，目标和基本数学并无本质区别。当前系统主要选择了更小、更透明、局部更新且显式有状态的实现，代价是容量、稳定性和样本效率远弱于成熟算法。

“模拟神经元状态”不是唯一也不是决定性区别。RNN、LSTM、GRU 同样维护状态。状态只提供历史依赖；学习来自探索、后果、责任分配和持久权重变化的共同作用。

## 5. 当前证据能说明什么

v1 已证明：

- 训练会持久改变动作权重；
- 学习后行为在未见地图上优于关闭学习；
- 打乱权重或消融重要内部单元会破坏行为；
- 相同种子可以重现实验。

v1 尚未证明：

- 连续内部状态比无状态控制器更好；
- 主体在没有距离塑形时仍能学习；
- 资格迹比仅奖励最后一步更必要；
- 内部表示是自然形成而非随机投影的偶然可分性；
- 脉冲、电位或更复杂生物机制带来功能收益；
- 系统具有概念、理解、开放式学习或生命特征。

因此下一阶段的首要问题不是增加神经元，而是逐一确定：**状态、时间信用分配和神经动力学分别贡献了什么。**

## 6. 参考入口

- [OpenAI Spinning Up：Vanilla Policy Gradient](https://spinningup.openai.com/en/latest/algorithms/vpg.html)：随机策略、优势和策略梯度；
- [Maass 等：Liquid State Machine](https://proceedings.neurips.cc/paper_files/paper/2002/file/6211080fa89981f66b1a0c9d55c61d0f-Paper.pdf)：动态储备池与可训练读出；
- [Gerstner 等：Neuromodulated STDP and Three-Factor Learning Rules](https://pmc.ncbi.nlm.nih.gov/articles/PMC4717313/)：资格迹和三因素可塑性；
- [Intel Loihi 2 技术说明](https://www.intel.com/content/dam/www/central-libraries/us/en/documents/neuromorphic-computing-loihi-2-brief.pdf)：支持神经状态、事件和第三因子轨迹的神经形态研究硬件。
