# Neuro Engine

一个以行为证据为中心的最小具身学习实验室。

项目不再试图从随机点神经网络的复杂放电中猜测“意义”。当前目标是让一个具有能量需求、感觉、内部状态和动作能力的主体，通过环境后果改变突触，并在未见环境中表现出可重复、可消融的学习。

## 当前结果

`embodied-learning/v1` 使用 15×15 二维环境、12 个感觉通道、24 个带泄漏与适应的内部单元和 4 个动作单元。动作突触采用奖励调制资格迹。

在 80 张未见地图上：

| 条件 | 平均食物 / 7 | 完成率 | 最终能量 |
|---|---:|---:|---:|
| 未经训练 | 0.50 | 0% | 3.4 |
| 关闭学习 | 0.50 | 0% | 3.4 |
| 学习后 | 7.00 | 100% | 39.6 |
| 打乱已学习突触 | 0.00 | 0% | 8.4 |
| 消融重要内部单元 | 0.35 | 0% | 8.1 |

这证明当前版本形成了可由行为直接观察、由连接置乱和内部消融验证的学习闭环；不宣称它是生命、意识或真实人脑模型。

Gate A 又使用 9 个辅助条件、每项 12 个独立模型种子进行了审计：移除距离塑形后，学习相对关闭学习的平均食物增益仍为 `5.218`，95% CI `[4.544, 5.891]`；完全移除食物方向后增益降为 `0.676 [0.255, 1.097]`。因此学习不依赖距离塑形，但精确方向感觉仍是主要人工辅助。

Gate B 使用早期线索消失后的 T 型岔路证明了连续状态的功能贡献：`leaky-state` 在 12 个未参与机制选择的模型种子上达到 `81.8% [73.6%, 90.0%]`，相同预算的 `state-reset` 为 `51.2%`；配对增益是 `30.6 [22.6, 38.6]` 个百分点，历史置乱后降至 `48.8%`。这证明短期状态有用，但不代表形成了概念或类脑记忆。

随后的独立鲁棒性扫描把当前固定状态机制的可靠记忆边界定在 **4 个无信息步骤**：4 步为 `86.2% [83.3%, 89.2%]`，6 步为 `60.0% [54.1%, 65.9%]`，8 步接近机会水平。持续感觉投影增大也会覆盖早期状态，说明这是一种边界明确的短期机制。

Gate C 隔离了延迟信用分配：当前线索在岔路可见，分支动作之后等待 8 步才获得真实能量。无状态控制器使用资格迹 0.88 时达到 `99.3% [98.7%, 99.9%]`，关闭资格迹时为 `47.0% [41.3%, 52.6%]`，配对增益 `52.3 [46.5, 58.1]` 个百分点；随机线索对照保持机会水平。该结果证明资格迹在受控任务中有用，不代表一般长期学习已经解决。

Gate D 加入 144 条固定稀疏跨单元连接。在 8 步无信息延迟中，与线索投影对齐的结构化循环达到 `80.6% [70.4%, 90.9%]`，拥有相同权重集合、连接数、正负边和谱半径的置乱循环为 `56.2% [49.7%, 62.6%]`，配对增益 `24.5 [14.3, 34.6]` 个百分点；可靠记忆边界从 4 步延长到 8 步。结构是设计者固定的，不宣称系统已经自行形成脑区。

## 运行

```powershell
cargo test --all-targets --release
cargo run --release --example embodied_lab
cargo run --release --example gate_a_lab
cargo run --release --example gate_b_lab
cargo run --release --example gate_b_robustness_lab
cargo run --release --example gate_c_lab
cargo run --release --example gate_d_lab
npm install
npm run dev
```

浏览器打开 [http://127.0.0.1:1420](http://127.0.0.1:1420)。Web 仪表盘可以同步查看主体轨迹、能量、感觉输入、内部活动、动作概率、训练曲线和未见地图对照。

桌面壳：

```powershell
npm run tauri dev
```

## 代码结构

```text
src/embodied.rs             二维世界、主体、控制器、可塑性与实验
src/gate_b.rs               延迟线索环境、等预算控制器和配对统计
src/gate_c.rs               延迟能量环境、资格迹矩阵和因果对照
src/gate_d.rs               稀疏循环动力学、置乱控制和稳定性审计
examples/embodied_lab.rs    生成版本化实验数据
tests/embodied.rs           确定性、闭环、可塑性和行为验收
app/                        具身行为仪表盘
src/lif.rs                  保留的单神经元 LIF 基础
src/network.rs              保留的确定性事件网络内核
src/experiment.rs           保留的早期网络生成与基线实验
experiments/                版本化结果与失败记录
```

## 文档

- [EMBODIED_LEARNING.md](EMBODIED_LEARNING.md)：当前研究问题、系统边界和验收条件；
- [LEARNING.md](LEARNING.md)：实际学习公式、现有技术定位、与常见机器学习的区别和证据边界；
- [ARCHITECTURE.md](ARCHITECTURE.md)：实现边界和数据流；
- [ROADMAP.md](ROADMAP.md)：从奖励审计、状态必要性到脉冲对照的递进实验路线；
- [GATE_B.md](GATE_B.md)：已完成的延迟线索任务和防泄漏验收冻结规格；
- [GATE_B_ROBUSTNESS.md](GATE_B_ROBUSTNESS.md)：已完成的状态延迟、线索带宽与持续干扰边界扫描；
- [GATE_C.md](GATE_C.md)：已完成的资格迹 × 延迟能量冻结规格；
- [GATE_D.md](GATE_D.md)：已完成的稀疏内部循环、等权重置乱与稳定性冻结规格；
- [experiments/embodied-v1.md](experiments/embodied-v1.md)：正式结果、对照和限制；
- [experiments/gate-a-v1.1.md](experiments/gate-a-v1.1.md)：奖励塑形与方向感觉的多种子审计；
- [experiments/gate-b-v1.2.md](experiments/gate-b-v1.2.md)：连续状态必要性的多种子配对实验；
- [experiments/gate-b-robustness-v1.2b.md](experiments/gate-b-robustness-v1.2b.md)：固定状态机制的能力边界；
- [experiments/gate-c-v1.3.md](experiments/gate-c-v1.3.md)：资格迹解决受控延迟信用分配的 4×4 实验；
- [experiments/gate-d-v1.4.md](experiments/gate-d-v1.4.md)：结构化循环延长可靠记忆的多种子实验；
- [experiments/phase2-v1.md](experiments/phase2-v1.md)：被否定的随机网络分类路线，作为负结果保留。

## 原则

- 行为是输出，不用外部分类器替主体解释意义；
- 学习必须在未见环境中优于关闭学习对照；
- 每个新机制必须可关闭并拥有独立实验；
- 画面用于理解已经定义的行为，不替代统计与因果证据；
- 当前闭环没有稳定前，不扩大规模、不模拟完整人脑。
