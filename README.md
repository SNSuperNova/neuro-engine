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

## 运行

```powershell
cargo test --all-targets --release
cargo run --release --example embodied_lab
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
- [ARCHITECTURE.md](ARCHITECTURE.md)：实现边界和数据流；
- [ROADMAP.md](ROADMAP.md)：下一阶段只允许逐项加入的机制；
- [experiments/embodied-v1.md](experiments/embodied-v1.md)：正式结果、对照和限制；
- [experiments/phase2-v1.md](experiments/phase2-v1.md)：被否定的随机网络分类路线，作为负结果保留。

## 原则

- 行为是输出，不用外部分类器替主体解释意义；
- 学习必须在未见环境中优于关闭学习对照；
- 每个新机制必须可关闭并拥有独立实验；
- 画面用于理解已经定义的行为，不替代统计与因果证据；
- 当前闭环没有稳定前，不扩大规模、不模拟完整人脑。
