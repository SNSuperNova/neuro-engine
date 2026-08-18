# Neuro Engine

一个可观察、可干预、可回放的脉冲神经网络动力学沙盒。

项目当前不以复制真实大脑、图片识别或训练通用 AI 为目标。第一阶段只研究一个问题：

> 包含传播延迟、兴奋和抑制的简化循环网络，能否形成可复现、不过度爆发的持续活动，并对局部刺激表现出可测量的状态变化？

## 当前阶段

Gate 0～3 已实现：仓库包含 Rust 事件驱动 LIF 内核、确定性网络传播、空间延迟、指标计算、版本化对照实验，以及同时显示 3D 网络与二维神经状态平面的 Tauri + Three.js“神经显微镜”。当前冻结基线以持续起搏支持的稳定活动为正常运行条件；撤除起搏只是依赖性诊断。Phase 2 v1 已完成模式输入、统计读出、置乱对照和真实网络消融的实现与跨种子运行，但 H1/H3 未通过，因此暂停扩大规模，详见 [Phase 2 v1 报告](experiments/phase2-v1.md)。第一阶段边界仍由 [MVP.md](MVP.md) 约束。

运行验证：

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets --release
cargo run --example gate0
cargo run --release --example gate2
cargo run --release --example gate2_scan
cargo run --release --example phase2
cargo run --release --example phase2_scan
cargo run --release --example export_playback
npm install
npm run dev
npm run tauri dev
```

Gate 2 的参数、验收区间和实测结果见 [experiments/experiment-001-v1.md](experiments/experiment-001-v1.md)。
Phase 2 最小 `3×3` 成对试验及首次输入扫描见 [experiments/phase2-p2-0.md](experiments/phase2-p2-0.md)；完整统计实验、复现命令和负结果见 [experiments/phase2-v1.md](experiments/phase2-v1.md)。

文档阅读顺序：

1. [MVP.md](MVP.md)：现在做什么、不做什么；
2. [MODEL_LIF.md](MODEL_LIF.md)：第一版计算模型；
3. [ARCHITECTURE.md](ARCHITECTURE.md)：模块和数据边界；
4. [EXPERIMENT_001.md](EXPERIMENT_001.md)：如何证明第一版有效；
5. [VISUALIZATION.md](VISUALIZATION.md)：已实现的 3D 游览、时间回放和性能规格；
6. [POPULATION_VIEW.md](POPULATION_VIEW.md)：群体统计、区域信息流与功能证据门槛；
7. [PHASE_2.md](PHASE_2.md)：`3×3` 模式刺激、自然功能群、简单读出和因果验证；
8. [DESIGN.md](DESIGN.md)：长期愿景和后续研究方向；
9. [decisions](decisions/)：已经作出的重要工程决策。

## 项目原则

- 模型是用于探索假设的简化计算模型，不宣称具有完整生物真实性；
- 模拟结果由数值指标和可重复实验验证，不能只凭动画判断；
- 模拟内核不依赖图形界面，并能以无界面方式运行；
- 显示帧率、播放速度和相机操作不能影响模拟结果；
- 新机制必须能单独关闭，以便进行对照实验；
- Gate 4 仍以可关闭、可比较的干预分支为边界，不直接扩展到学习、生长、视觉输入和复杂神经元。

## Gate 3 操作

- `WASD`：在 XY 平面移动；
- `Q / E`：降低或升高 Z 轴位置；
- 拖动鼠标：旋转视角；滚轮：沿视线前后移动；
- 单击神经元或输入 ID：选择并检查；
- 右侧 `10×10` 固定空间展开：切换膜电位、5 ms 放电痕迹和剩余不应期；二维选择与 3D、检查器同步；
- 时间轴、播放按钮和倍速：查看同一份确定性实验结果；
- 运行分支：切换持续起搏对照、局部刺激和撤除起搏诊断。

## 文档中的结论等级

- **已决定**：当前实现必须遵守，改变时需要增加决策记录；
- **暂定**：可以用于第一轮实现，但必须通过实验校准；
- **开放问题**：尚未决定，不应被代码暗中固化。
