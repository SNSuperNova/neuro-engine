# Neuro Engine

一个可观察、可干预、可回放的脉冲神经网络动力学沙盒。

项目当前不以复制真实大脑、图片识别或训练通用 AI 为目标。第一阶段只研究一个问题：

> 包含传播延迟、兴奋和抑制的简化循环网络，能否形成可复现、不过度爆发的持续活动，并对局部刺激表现出可测量的状态变化？

## 当前阶段

Gate 0～2 已实现：仓库包含一个无第三方依赖的 Rust 事件驱动 LIF 内核、确定性网络传播、空间延迟、指标计算和版本化对照实验。当前冻结基线以持续起搏支持的稳定活动为正常运行条件；撤除起搏只是依赖性诊断。下一阶段优先实现可视化，以观察结果决定后续模型方向。开发范围由 [MVP.md](MVP.md) 约束。

运行验证：

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets --release
cargo run --example gate0
cargo run --release --example gate2
cargo run --release --example gate2_scan
```

Gate 2 的参数、验收区间和实测结果见 [experiments/experiment-001-v1.md](experiments/experiment-001-v1.md)。

文档阅读顺序：

1. [MVP.md](MVP.md)：现在做什么、不做什么；
2. [MODEL_LIF.md](MODEL_LIF.md)：第一版计算模型；
3. [ARCHITECTURE.md](ARCHITECTURE.md)：模块和数据边界；
4. [EXPERIMENT_001.md](EXPERIMENT_001.md)：如何证明第一版有效；
5. [VISUALIZATION.md](VISUALIZATION.md)：下一阶段的 3D 游览、时间回放和性能规格；
6. [DESIGN.md](DESIGN.md)：长期愿景和后续研究方向；
7. [decisions](decisions/)：已经作出的重要工程决策。

## 项目原则

- 模型是用于探索假设的简化计算模型，不宣称具有完整生物真实性；
- 模拟结果由数值指标和可重复实验验证，不能只凭动画判断；
- 模拟内核不依赖图形界面，并能以无界面方式运行；
- 显示帧率、播放速度和相机操作不能影响模拟结果；
- 新机制必须能单独关闭，以便进行对照实验；
- 完成 Gate 3 可视化观察工具前，不扩展到学习、生长、视觉输入和复杂神经元。

## 文档中的结论等级

- **已决定**：当前实现必须遵守，改变时需要增加决策记录；
- **暂定**：可以用于第一轮实现，但必须通过实验校准；
- **开放问题**：尚未决定，不应被代码暗中固化。
