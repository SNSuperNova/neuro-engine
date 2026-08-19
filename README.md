# Neuro Engine

一个以行为证据研究动态调整机制如何形成宏观功能的实验室。

项目不再试图从随机点神经网络的复杂放电中猜测“意义”，也不以补齐功能完备的真实神经元为近期目标。当前 24 单元系统是可替换的参考载体；主要研究对象是局部、有界、在线的调整机制如何让系统通过环境后果形成、保留和迁移能力，并用未见条件、独立种子和内部干预检验这些能力是否可重复。

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

Gate E 把事件式 LIF 接入同一延迟线索闭环。在连续泄漏与脉冲迹衰减同为 0.86、动态状态和可训练连接预算匹配时，延迟 8 的 LIF 达到 `88.5% [85.3%, 91.6%]`，连续状态为 `67.3% [60.5%, 74.1%]`，配对增益 `21.2 [13.6, 28.7]` 个百分点；可靠记忆边界分别为 12 和 4 步。代价同样公开：LIF 当前 CPU 实现慢约 6.3 倍，概念状态预算相同但实现动态状态为 2,496 B（连续状态 384 B），25% 损伤下降也更大，因此仍是实验分支而非默认控制器。

Gate F 冻结动作读出后反转线索—目标规则。冻结内部网络保持 `0.0%`，感觉投影可塑性在 800 回合后达到 `100.0%`，局部循环可塑性达到 `91.3% [79.0%, 103.6%]`，并都能在规则恢复后重新达到 100%。关闭内稳态的循环控制行为相近，但开放权重组范数漂移约 658%。因此内部突触变化可以承担这个受控任务的持续适应，而归一化目前只证明能控制参数尺度；该结果仍依赖二元规则、强制探索和预对齐通路。

Map 0 随后在统一 24 单元底层上扫描 64 组循环增益、内部可塑率、内稳态和探索参数。512 个开发运行之后，6 个候选配置又完成 72 个独立确认和 360 个因果对照。确认阶段没有系统同时通过记忆、延迟信用、重复切换、损伤恢复和稳定性标准：71/72 为任务特化，1/72 因权重漂移不稳定。协议完整通过，但没有找到稳定可学习区域，因此当前不进入 Gate G。

Map 1 只把参考范数投影替换为活动目标与慢速权重回拉，并在 Map 0 候选区局部重绘。正式确认中，新机制 72 次运行有 59 次因权重漂移不稳定；相对旧机制平均探针得分仅 `+1.2 pp`，权重漂移却增加 `1.766`，重复反转和损伤恢复总体下降。该机制已作为负结果淘汰。当前瓶颈仍指向局部可塑性与稳定性的冲突，而不是节点数量。

Map 2 已完成最小动态基座审计。软边界可塑性把相对权重漂移平均降低 `0.687`，持续供能在断供时产生 `13.9 pp` 探针下降；冻结两者后的 720 试次无重置连续流达到 `66.1%`，逐试次重置为 `67.6%`，断供为 `55.7%`，冻结可塑性为 `53.0%`。6 个相邻确认配置形成跨种子稳定区域。M0 随后把它冻结为 `reference-substrate/v1`：候选机制只能通过局部、有界动作接口改变载体，不能读取目标或直接写入动作读出；Map 2A/B/C 完整 JSON 已逐字段回归，原发布哈希保持不变。M1 已在这一冻结基座上完成，Scale 0 继续后移。

M1 已完成四规则连续保留诊断。A 离开前为 `83.4%`，经过 B/C/D 后回归初始为 `60.7%`；但 B/C/D 连续学习最终只有 `55.4%`，四个规则单独训练时的最低准确率也只有 `51.3%`。因此六个参考参数点全部归为容量不足，不能把 A 的下降单独解释成覆盖性遗忘。冻结可塑性和随机后果对照分别为 `45.9%`、`47.6%`，说明现有调整有效但不足，并由此触发了固定总连接预算的 M2C 结构调整分支。

M2C 首个候选已经执行。它保持 24 个节点和每节点 6 条循环入边，只允许两个可塑来源槽位按奖励调制的局部资格证据移动。开发集选择每 64 试次重连一次；独立确认中，局部重连的单规则最低准确率为 `47.9%`，仅权重可塑为 `48.7%`，B/C/D 连续学习为 `59.5%` 对 `59.6%`，且没有可靠优于等次数随机重连。候选机制因此淘汰，不进入 M3；下一步先诊断局部结构证据能否预测反事实换边收益。

## 运行

```powershell
cargo test --all-targets --release
cargo run --release --example embodied_lab
cargo run --release --example gate_a_lab
cargo run --release --example gate_b_lab
cargo run --release --example gate_b_robustness_lab
cargo run --release --example gate_c_lab
cargo run --release --example gate_d_lab
cargo run --release --example gate_e_lab
cargo run --release --example gate_f_lab
cargo run --release --example map0_lab
cargo run --release --example map1_lab
cargo run --release --example map2a_lab
cargo run --release --example map2b_lab
cargo run --release --example map2c_lab
cargo run --release --example mechanism_m0_lab
cargo run --release --example mechanism_m1_lab
cargo run --release --example mechanism_m2c_lab
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
src/gate_e.rs               连续状态与 LIF 的等预算行为、损伤和成本比较
src/gate_f.rs               冻结读出后的内部可塑性、规则反转与内稳态对照
src/learnability_map.rs     统一探针、四轴扫描、独立确认和因果对照
src/map1.rs                 双时间尺度内稳态、局部重绘和旧机制配对
src/adaptive_mechanism.rs   参考载体清单与受约束候选机制接口
src/mechanism_m0.rs         M0 冻结发布清单与能力边界
src/mechanism_m1.rs         M1 多规则保留协议、失败分类与配对对照
src/mechanism_m2c.rs        M2C 固定预算结构扫描、配对效应与机制决策
examples/embodied_lab.rs    生成版本化实验数据
tests/embodied.rs           确定性、闭环、可塑性和行为验收
app/                        具身行为仪表盘
src/lif.rs                  保留的单神经元 LIF 基础
src/network.rs              保留的确定性事件网络内核
src/experiment.rs           保留的早期网络生成与基线实验
experiments/                版本化结果与失败记录
```

## 文档

- [RESEARCH_PROGRAM.md](RESEARCH_PROGRAM.md)：总研究假设、灰盒方法、系统自由度和功能形成标准；
- [LEARNABILITY_MAP.md](LEARNABILITY_MAP.md)：稳定可学习性地图的阶段设计背景；
- [MAP_0.md](MAP_0.md)：已完成的统一探针、参数扫描和区域判定冻结规格；
- [MAP_1.md](MAP_1.md)：已完成的双时间尺度内稳态单机制修订规格；
- [FOUNDATION_AUDIT.md](FOUNDATION_AUDIT.md)：后续最小动态基座、持续供能和规模扫描的执行边界；
- [ADAPTIVE_MECHANISMS.md](ADAPTIVE_MECHANISMS.md)：M0～M4 调整机制研究主线与条件分支；
- [experiments/mechanism-m0-v0.1.md](experiments/mechanism-m0-v0.1.md)：参考载体、权限接口、Map 2 回归和能力边界冻结记录；
- [experiments/mechanism-m1-v0.2.md](experiments/mechanism-m1-v0.2.md)：四规则连续保留、单规则容量和条件分支诊断；
- [experiments/mechanism-m2c-v0.3.md](experiments/mechanism-m2c-v0.3.md)：固定预算局部重连、随机配对和首个结构候选负结果；
- [EMBODIED_LEARNING.md](EMBODIED_LEARNING.md)：当前研究问题、系统边界和验收条件；
- [LEARNING.md](LEARNING.md)：实际学习公式、现有技术定位、与常见机器学习的区别和证据边界；
- [ARCHITECTURE.md](ARCHITECTURE.md)：实现边界和数据流；
- [ROADMAP.md](ROADMAP.md)：已完成 Gate、稳定可学习性地图与后续条件路线；
- [GATE_B.md](GATE_B.md)：已完成的延迟线索任务和防泄漏验收冻结规格；
- [GATE_B_ROBUSTNESS.md](GATE_B_ROBUSTNESS.md)：已完成的状态延迟、线索带宽与持续干扰边界扫描；
- [GATE_C.md](GATE_C.md)：已完成的资格迹 × 延迟能量冻结规格；
- [GATE_D.md](GATE_D.md)：已完成的稀疏内部循环、等权重置乱与稳定性冻结规格；
- [GATE_E.md](GATE_E.md)：已完成的连续状态与 LIF 脉冲状态等预算冻结规格；
- [GATE_F.md](GATE_F.md)：已完成的内部可塑性、规则反转与内稳态冻结规格；
- [experiments/embodied-v1.md](experiments/embodied-v1.md)：正式结果、对照和限制；
- [experiments/gate-a-v1.1.md](experiments/gate-a-v1.1.md)：奖励塑形与方向感觉的多种子审计；
- [experiments/gate-b-v1.2.md](experiments/gate-b-v1.2.md)：连续状态必要性的多种子配对实验；
- [experiments/gate-b-robustness-v1.2b.md](experiments/gate-b-robustness-v1.2b.md)：固定状态机制的能力边界；
- [experiments/gate-c-v1.3.md](experiments/gate-c-v1.3.md)：资格迹解决受控延迟信用分配的 4×4 实验；
- [experiments/gate-d-v1.4.md](experiments/gate-d-v1.4.md)：结构化循环延长可靠记忆的多种子实验；
- [experiments/gate-e-v1.5.md](experiments/gate-e-v1.5.md)：LIF 的记忆收益、CPU 代价和损伤边界；
- [experiments/gate-f-v1.6.md](experiments/gate-f-v1.6.md)：冻结动作读出后的规则反转、恢复和权重稳定性；
- [experiments/map0-v0.1.md](experiments/map0-v0.1.md)：稳定可学习性地图的正式负结果与机制边界；
- [experiments/map1-v0.2.md](experiments/map1-v0.2.md)：双时间尺度内稳态的配对负结果；
- [experiments/phase2-v1.md](experiments/phase2-v1.md)：被否定的随机网络分类路线，作为负结果保留。

## 原则

- 行为是输出，不用外部分类器替主体解释意义；
- 学习必须在未见环境中优于关闭学习对照；
- 每个新机制必须可关闭并拥有独立实验；
- 画面用于理解已经定义的行为，不替代统计与因果证据；
- 当前闭环没有稳定前，不扩大规模、不模拟完整人脑。
