# 第一版 LIF 模型

## 1. 模型目的

第一版采用 LIF（Leaky Integrate-and-Fire，漏积分放电）模型，目的是验证网络级动力学和模拟基础设施，不是复制真实神经元的全部生理过程。

本文件中的公式是计算规范。参数值在首轮实验前仍属暂定，必须携带单位并写入实验记录。

## 2. 状态与单位

每个神经元至少包含：

```text
id                  u32
polarity            excitatory | inhibitory
modelPosition       [x, y, z]
membranePotential   mV
restPotential       mV
resetPotential      mV
threshold           mV
membraneTimeConstant ms
lastUpdateTime      integer simulation time
refractoryUntil     integer simulation time
pacemakerConfig     optional
```

模型坐标参与距离和延迟计算。界面为了整理画面而使用的 `viewPosition` 不得改变模型状态。

起搏是一种可选驱动配置，不是第三种神经元极性。突触至少包含：

```text
id                  u32
sourceNeuronId      u32
targetNeuronId      u32
weight              mV
pathLength          model distance unit
conductionVelocity  model distance unit / ms
synapticDelay       ms
```

## 3. 两个事件之间的演化

在没有输入事件时，膜电位解析衰减：

```text
V(t) = Vrest + (Vlast - Vrest) × exp(-(t - tlast) / tauM)
```

因此第一版不需要全局固定模拟 tick。系统只在相关事件发生时把神经元状态推进到事件时刻。

## 4. 突触输入

第一版使用瞬时电压型突触：

```text
V ← decayTo(arrivalTime)
V ← V + sum(arrivingWeights)
```

- 兴奋性突触权重为正；
- 抑制性突触权重为负；
- 同一时刻抵达同一神经元的输入先按固定顺序聚合，再判断一次阈值；
- 权重单位暂定为 mV，不能与未来的电流型突触混用。

第一版不模拟连续突触电流。引入连续电流时，需要新的模型版本及阈值穿越求解策略。

## 5. 放电、复位与不应期

聚合输入后：

```text
if arrivalTime >= refractoryUntil and V >= threshold:
    emit SpikeEvent
    V = resetPotential
    refractoryUntil = arrivalTime + refractoryDuration
```

第一版暂定规则：不应期内抵达的突触事件仍记录，但不改变膜电位，也不累积到不应期结束后。该规则必须有测试；若修改，应创建新的模型版本。

所有连接必须具有严格大于零的总传播延迟，避免同一时刻形成无限级联。

## 6. 传播延迟

```text
totalDelay = pathLength / conductionVelocity + synapticDelay
arrivalTime = spikeTime + quantize(totalDelay)
```

`pathLength` 可以由模型空间中的连接路径决定，不要求等于两点直线距离。量化精度和舍入规则属于模型版本的一部分。

## 7. 起搏事件

起搏不是普通 LIF 神经元凭空越过阈值，而是明确记录的驱动事件。每次起搏都必须出现在输入事件流中，以便区分：

- 无外部驱动的循环活动；
- 明确的周期性起搏驱动；
- 未来可能加入的随机噪声驱动。

## 8. 确定性要求

已决定：

- 模拟时间使用整数；
- 同时事件具有全序；
- 随机种子和随机数算法版本化；
- 所有参数携带单位并序列化；
- 显示和统计采样不参与动力学。

开放问题：

- 时间基础单位使用微秒还是更粗粒度；
- 膜电位使用 `float64`、定点数还是显式量化浮点；
- 指数衰减是否使用版本化查找表；
- 是否要求 Windows x86-64 与 macOS ARM64 重新计算后事件逐项一致。

在这些问题解决前，确定性承诺限定为“同一平台、同一构建和同一配置”。跨平台必须能够无损播放同一份已保存事件日志。

## 9. 第一组测试

- 静息电位保持不变；
- 高于或低于静息电位时正确衰减；
- 单个不足阈值的输入不放电；
- 同时输入先求和再判断阈值；
- 抑制输入降低膜电位；
- 放电后正确复位；
- 不应期边界行为固定；
- 延迟和到达时间量化正确；
- 零延迟连接被拒绝；
- 重放相同事件产生相同输出摘要。
