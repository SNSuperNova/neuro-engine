import "./style.css";

type Heading = "north" | "east" | "south" | "west";
type Action = "forward" | "turn-left" | "turn-right" | "eat";
interface Position { x: number; y: number }
interface RewardBreakdown { energyDelta: number; distanceProgress: number; total: number }
interface Frame {
  step: number; position: Position; heading: Heading; energy: number; foodsEaten: number;
  action: Action; reward: number; actionProbabilities: number[]; sensors: number[];
  rewardBreakdown: RewardBreakdown; hiddenActivity: number[]; remainingFood: Position[];
}
interface Summary { stepsSurvived: number; foodsEaten: number; finalEnergy: number; collisions: number; hazardContacts: number; completed: boolean; totalReward: number; rewardBreakdown: RewardBreakdown }
interface Trace { label: string; mapSeed: number; hazards: Position[]; initialFood: Position[]; frames: Frame[]; summary: Summary }
interface Evaluation { label: string; episodeCount: number; meanStepsSurvived: number; meanFoodsEaten: number; meanFinalEnergy: number; meanCollisions: number; meanHazardContacts: number; completionFraction: number }
interface CurvePoint { episode: number; meanFoodsEaten: number; meanStepsSurvived: number; meanFinalEnergy: number; meanReward: number }
interface Dataset {
  version: string;
  config: { arena: { width: number; height: number; foodCount: number; maximumEnergy: number }; trainingEpisodes: number; evaluationEpisodes: number };
  sensorLabels: string[]; actionLabels: string[]; trainingCurve: CurvePoint[]; evaluations: Evaluation[];
  plasticity: { changedWeightCount: number; totalWeightCount: number; rootMeanSquareChange: number; maximumAbsoluteChange: number; lesionedHiddenUnits: number[] };
  traces: Trace[];
  acceptance: Record<string, boolean> & { passed: boolean };
}
interface Interval { mean: number; lower95: number; upper95: number }
interface AuditMetrics {
  foodsEaten: Interval; finalEnergy: Interval; completionFraction: Interval; collisions: Interval;
  hazardContacts: Interval; totalReward: Interval; energyReward: Interval; distanceReward: Interval;
}
interface AuditCurvePoint { episode: number; foodsEaten: Interval }
interface AuditVariant {
  id: string; label: string;
  reward: { energyDeltaWeight: number; distanceProgressWeight: number };
  sensors: { foodDirectionEnabled: boolean; foodDirectionPrecision: number; foodDirectionNoise: number; foodDirectionDropout: number };
  learned: AuditMetrics; learningDisabled: AuditMetrics; pairedEffect: AuditMetrics;
  sampleEfficiency: { foodThreshold: number; reachedSeedCount: number; reachedSeedFraction: number; meanEpisodesWhenReached: Interval | null };
  trainingCurve: AuditCurvePoint[];
}
interface GateADataset {
  version: string;
  config: { modelSeedCount: number; trainingEpisodes: number; evaluationEpisodes: number; arena: { foodCount: number } };
  variants: AuditVariant[]; conclusions: string[];
  acceptance: Record<string, boolean> & { passed: boolean };
}
type ForkSide = "left" | "right";
type GateBCondition = "delayed-cue" | "cue-visible-at-fork" | "cue-randomized" | "history-shuffled";
type GateBController = "stateless" | "state-reset" | "leaky-state";
interface GateBMetricPoint { correctChoiceFraction: number; branchChoiceFraction: number; foodFraction: number; meanFinalEnergy: number; meanSteps: number; rightChoiceFraction: number; leftTargetAccuracy: number; rightTargetAccuracy: number }
interface GateBMetricIntervals { correctChoiceFraction: Interval; branchChoiceFraction: Interval; foodFraction: Interval; meanFinalEnergy: Interval; meanSteps: Interval; rightChoiceFraction: Interval; leftTargetAccuracy: Interval; rightTargetAccuracy: Interval }
interface GateBFrame { step: number; positionBefore: Position; headingBefore: Heading; position: Position; heading: Heading; visibleCue: ForkSide | null; action: Action; reward: number; energy: number; branchChoice: ForkSide | null; hiddenActivity: number[]; actionProbabilities: number[] }
interface GateBTrace { label: string; condition: GateBCondition; controller: GateBController; target: ForkSide; presentedCue: ForkSide; frames: GateBFrame[]; summary: { steps: number; branchChoice: ForkSide | null; correctChoice: boolean; foodEaten: boolean; finalEnergy: number; totalReward: number } }
interface GateBReport { condition: GateBCondition; controller: GateBController; metrics: GateBMetricIntervals; sampleEfficiency: { accuracyThreshold: number; reachedSeedCount: number; reachedSeedFraction: number; meanEpisodesWhenReached: Interval | null }; trainingCurve: { episode: number; correctChoiceFraction: Interval }[] }
interface GateBDataset {
  version: string; config: { modelSeedCount: number; trainingEpisodes: number; evaluationEpisodes: number; maximumSteps: number; cueSteps: number; controller: { hiddenLeak: number } };
  reports: GateBReport[]; pairedEffects: { id: string; leftLabel: string; rightLabel: string; effect: GateBMetricIntervals }[];
  traces: GateBTrace[]; conclusions: string[]; acceptance: Record<string, boolean> & { passed: boolean };
}
type RobustnessFactor = "delay-steps" | "cue-input-scale" | "persistent-input-scale";
interface RobustnessPoint { id: string; factor: RobustnessFactor; value: number; delaySteps: number; cueInputScale: number; persistentInputScale: number; report: GateBReport }
interface RobustnessDataset { version: string; config: { modelSeedCount: number; evaluationEpisodes: number }; baselineDelaySteps: number; baselineCueInputScale: number; baselinePersistentInputScale: number; reliableMemoryBoundarySteps: number | null; points: RobustnessPoint[] }
interface GateCReport { rewardDelaySteps: number; eligibilityDecay: number; cueRandomized: boolean; metrics: { correctChoiceFraction: Interval; branchChoiceFraction: Interval; leftTargetAccuracy: Interval; rightTargetAccuracy: Interval }; sampleEfficiency: { reachedSeedCount: number; reachedSeedFraction: number; meanEpisodesWhenReached: Interval | null } }
interface GateCFrame { step: number; phaseBefore: "junction" | "waiting" | "outcome"; waitingStepsRemainingBefore: number; visibleCue: ForkSide | null; action: Action; reward: number; energy: number; branchChoice: ForkSide | null; energyPaidOut: boolean; hiddenActivity: number[]; actionProbabilities: number[] }
interface GateCTrace { label: string; rewardDelaySteps: number; eligibilityDecay: number; cueRandomized: boolean; target: ForkSide; presentedCue: ForkSide; frames: GateCFrame[]; summary: { correctChoice: boolean; energyPaidOut: boolean; finalEnergy: number; totalReward: number } }
interface GateCDataset { version: string; config: { modelSeedCount: number; trainingEpisodes: number; evaluationEpisodes: number; rewardDelays: number[]; eligibilityDecays: number[] }; reports: GateCReport[]; pairedEffects: { id: string; effect: { correctChoiceFraction: Interval } }[]; traces: GateCTrace[]; acceptance: Record<string, boolean> & { passed: boolean } }
type GateDController = "no-recurrence" | "shuffled-recurrence" | "structured-recurrence";
interface GateDStability { meanAbsoluteActivity: Interval; saturatedUnitFraction: Interval; silentUnitFraction: Interval; populationSynchrony: Interval; finiteStateFraction: Interval }
interface GateDReport { delaySteps: number; controller: GateDController; metrics: { correctChoiceFraction: Interval; branchChoiceFraction: Interval; foodFraction: Interval; meanFinalEnergy: Interval; leftTargetAccuracy: Interval; rightTargetAccuracy: Interval }; stability: GateDStability; sampleEfficiency: { reachedSeedCount: number; reachedSeedFraction: number; meanEpisodesWhenReached: Interval | null } }
interface GateDFrame { step: number; visibleCue: ForkSide | null; delayStepsRemaining: number; action: Action; reward: number; energy: number; branchChoice: ForkSide | null; hiddenActivity: number[]; actionProbabilities: number[] }
interface GateDTrace { label: string; delaySteps: number; controller: GateDController; target: ForkSide; presentedCue: ForkSide; frames: GateDFrame[]; correctChoice: boolean; foodEaten: boolean; finalEnergy: number }
interface GateDDataset { version: string; config: { modelSeedCount: number; evaluationEpisodes: number; memoryDelays: number[]; recurrentInDegree: number; recurrentSpectralRadius: number }; controllerBudgets: { controller: GateDController; activeRecurrentWeightCount: number; excitatoryEdgeCount: number; inhibitoryEdgeCount: number; estimatedSpectralRadius: number; recurrentWeightDigest: number }[]; reports: GateDReport[]; pairedEffects: { id: string; effect: { correctChoiceFraction: Interval } }[]; reliableMemoryBoundarySteps: [GateDController, number | null][]; traces: GateDTrace[]; acceptance: Record<string, boolean> & { passed: boolean } }
type GateEController = "stateless" | "continuous-state" | "lif-spiking";
interface GateEActivity { meanAbsoluteFeature: Interval; activeUnitFraction: Interval; silentUnitFraction: Interval; emittedSpikesPerStep: Interval; finiteStateFraction: Interval }
interface GateEReport { delaySteps: number; controller: GateEController; metrics: { correctChoiceFraction: Interval; branchChoiceFraction: Interval; foodFraction: Interval; meanFinalEnergy: Interval; leftTargetAccuracy: Interval; rightTargetAccuracy: Interval }; activity: GateEActivity; damagedMetrics: { correctChoiceFraction: Interval } | null; damageAccuracyDrop: Interval | null; sampleEfficiency: { reachedSeedCount: number; reachedSeedFraction: number; meanEpisodesWhenReached: Interval | null } }
interface GateEFrame { step: number; visibleCue: ForkSide | null; delayStepsRemaining: number; action: Action; reward: number; energy: number; branchChoice: ForkSide | null; features: number[]; membranePotentialsMv: number[]; spikes: boolean[]; actionProbabilities: number[] }
interface GateETrace { label: string; delaySteps: number; controller: GateEController; target: ForkSide; frames: GateEFrame[]; correctChoice: boolean; foodEaten: boolean; finalEnergy: number }
interface GateEDataset { version: string; config: { modelSeedCount: number; evaluationEpisodes: number; memoryDelays: number[]; damageFraction: number; lifMembraneTimeConstantMs: number; lifSpikeTraceDecay: number; controller: { hiddenLeak: number } }; controllerBudgets: { controller: GateEController; conceptualStateBytes: number; implementationDynamicStateBytes: number; trainableActionWeightCount: number }[]; reports: GateEReport[]; pairedEffects: { id: string; effect: { correctChoiceFraction: Interval } }[]; reliableMemoryBoundarySteps: [GateEController, number | null][]; aggregateCosts: [GateEController, { environmentSteps: number; inputEvents: number; emittedSpikes: number; denseInputMultiplyAccumulates: number; denseReadoutMultiplyAccumulates: number }][]; traces: GateETrace[]; acceptance: Record<string, boolean> & { passed: boolean } }
interface GateERuntime { controller: GateEController; repetitions: number; environmentSteps: number; elapsedSeconds: number; nanosecondsPerEnvironmentStep: number }

const byId = <T extends HTMLElement>(id: string): T => {
  const found = document.getElementById(id);
  if (!found) throw new Error(`missing #${id}`);
  return found as T;
};

const worldCanvas = byId<HTMLCanvasElement>("world");
const curveCanvas = byId<HTMLCanvasElement>("learning-curve");
const gateACurveCanvas = byId<HTMLCanvasElement>("gate-a-curve");
const gateASelect = byId<HTMLSelectElement>("gate-a-select");
const gateBWorldCanvas = byId<HTMLCanvasElement>("gate-b-world");
const gateBCurveCanvas = byId<HTMLCanvasElement>("gate-b-curve");
const gateBTraceSelect = byId<HTMLSelectElement>("gate-b-trace-select");
const gateBRange = byId<HTMLInputElement>("gate-b-range");
const gateBPlay = byId<HTMLButtonElement>("gate-b-play");
const gateCTraceSelect = byId<HTMLSelectElement>("gate-c-trace-select");
const gateCRange = byId<HTMLInputElement>("gate-c-range");
const gateDBoundaryCanvas = byId<HTMLCanvasElement>("gate-d-boundary");
const gateDTraceSelect = byId<HTMLSelectElement>("gate-d-trace-select");
const gateDRange = byId<HTMLInputElement>("gate-d-range");
const gateEBoundaryCanvas = byId<HTMLCanvasElement>("gate-e-boundary");
const gateERasterCanvas = byId<HTMLCanvasElement>("gate-e-raster");
const gateETraceSelect = byId<HTMLSelectElement>("gate-e-trace-select");
const gateERange = byId<HTMLInputElement>("gate-e-range");
const traceSelect = byId<HTMLSelectElement>("trace-select");
const timeline = byId<HTMLInputElement>("timeline");
const play = byId<HTMLButtonElement>("play");
const prev = byId<HTMLButtonElement>("prev");
const next = byId<HTMLButtonElement>("next");
const speed = byId<HTMLSelectElement>("speed");

let dataset: Dataset;
let gateA: GateADataset;
let gateB: GateBDataset;
let robustness: RobustnessDataset;
let gateC: GateCDataset;
let gateCTrace: GateCTrace;
let gateCFrameIndex = 0;
let gateD: GateDDataset;
let gateDTrace: GateDTrace;
let gateDFrameIndex = 0;
let gateE: GateEDataset;
let gateERuntime: GateERuntime[];
let gateETrace: GateETrace;
let gateEFrameIndex = 0;
let gateBTrace: GateBTrace;
let gateBFrameIndex = 0;
let gateBPlaying = false;
let gateBLastTick = performance.now();
let trace: Trace;
let frameIndex = 0;
let playing = false;
let lastTick = performance.now();
let ready = false;

const labels: Record<string, string> = {
  untrained: "未经训练", learned: "学习后", shuffled: "打乱突触", lesioned: "内部单元消融",
  "learning-disabled": "关闭学习", forward: "前进", "turn-left": "左转", "turn-right": "右转", eat: "进食",
  north: "北", east: "东", south: "南", west: "西",
  left: "左", right: "右", stateless: "无状态", "state-reset": "每步清空", "leaky-state": "泄漏状态",
  "delayed-cue": "延迟线索", "cue-visible-at-fork": "岔路可见", "cue-randomized": "随机线索", "history-shuffled": "历史置乱",
  "no-recurrence": "无循环", "shuffled-recurrence": "置乱循环", "structured-recurrence": "结构化循环",
  "continuous-state": "连续状态", "lif-spiking": "LIF 脉冲",
};

const fitCanvas = (canvas: HTMLCanvasElement): CanvasRenderingContext2D => {
  const ratio = Math.min(devicePixelRatio, 2);
  const rect = canvas.getBoundingClientRect();
  const width = Math.max(1, Math.floor(rect.width * ratio));
  const height = Math.max(1, Math.floor(rect.height * ratio));
  if (canvas.width !== width || canvas.height !== height) { canvas.width = width; canvas.height = height; }
  const context = canvas.getContext("2d");
  if (!context) throw new Error("2D canvas unavailable");
  context.setTransform(ratio, 0, 0, ratio, 0, 0);
  return context;
};

const drawWorld = () => {
  const ctx = fitCanvas(worldCanvas);
  const { width, height } = dataset.config.arena;
  const rect = worldCanvas.getBoundingClientRect();
  const padding = 22;
  const cell = Math.min((rect.width - padding * 2) / width, (rect.height - padding * 2) / height);
  const ox = (rect.width - cell * width) / 2;
  const oy = (rect.height - cell * height) / 2;
  ctx.clearRect(0, 0, rect.width, rect.height);
  ctx.fillStyle = "#f7faf7"; ctx.fillRect(0, 0, rect.width, rect.height);
  ctx.strokeStyle = "#dce6dc"; ctx.lineWidth = 1;
  for (let x = 0; x <= width; x++) { ctx.beginPath(); ctx.moveTo(ox + x * cell, oy); ctx.lineTo(ox + x * cell, oy + height * cell); ctx.stroke(); }
  for (let y = 0; y <= height; y++) { ctx.beginPath(); ctx.moveTo(ox, oy + y * cell); ctx.lineTo(ox + width * cell, oy + y * cell); ctx.stroke(); }
  const center = (point: Position): [number, number] => [ox + (point.x + .5) * cell, oy + (point.y + .5) * cell];
  for (const hazard of trace.hazards) { const [x,y] = center(hazard); ctx.fillStyle = "#ec6a5e"; ctx.beginPath(); ctx.arc(x,y,cell*.28,0,Math.PI*2); ctx.fill(); }
  const current = trace.frames[Math.min(frameIndex, trace.frames.length - 1)];
  for (const food of current.remainingFood) { const [x,y] = center(food); ctx.fillStyle = "#4db878"; ctx.beginPath(); ctx.arc(x,y,cell*.25,0,Math.PI*2); ctx.fill(); ctx.strokeStyle="#207447";ctx.stroke(); }
  if (frameIndex > 0) {
    ctx.strokeStyle = "rgba(47, 125, 98, .32)"; ctx.lineWidth = Math.max(2, cell*.12); ctx.lineCap="round"; ctx.beginPath();
    trace.frames.slice(0, frameIndex + 1).forEach((f,i) => { const [x,y]=center(f.position); i ? ctx.lineTo(x,y) : ctx.moveTo(x,y); }); ctx.stroke();
  }
  const [ax,ay] = center(current.position); const angle = {north:-Math.PI/2,east:0,south:Math.PI/2,west:Math.PI}[current.heading];
  ctx.save(); ctx.translate(ax,ay); ctx.rotate(angle); ctx.fillStyle="#174f45"; ctx.beginPath(); ctx.moveTo(cell*.38,0); ctx.lineTo(-cell*.28,-cell*.27); ctx.lineTo(-cell*.28,cell*.27); ctx.closePath(); ctx.fill(); ctx.restore();
};

const makeBar = (label: string, value: number, signed = false) => {
  const row = document.createElement("div"); row.className = "bar-row";
  const normalized = signed ? (value + 1) / 2 : Math.max(0, Math.min(1, value));
  row.innerHTML = `<span>${label}</span><div><i style="width:${normalized*100}%"></i></div><strong>${value.toFixed(2)}</strong>`;
  return row;
};

const renderFrame = () => {
  timeline.value = String(frameIndex);
  const frame = trace.frames[frameIndex];
  byId("step-label").textContent = `step ${frame.step} / ${trace.frames.length}`;
  byId("action-name").textContent = labels[frame.action] ?? frame.action;
  byId("energy-value").textContent = frame.energy.toFixed(1);
  byId<HTMLElement>("energy-bar").style.width = `${frame.energy / dataset.config.arena.maximumEnergy * 100}%`;
  byId("food-value").textContent = `${frame.foodsEaten} / ${dataset.config.arena.foodCount}`;
  byId("reward-value").textContent = `${frame.reward >= 0 ? "+" : ""}${frame.reward.toFixed(3)}`;
  byId("reward-value").className = frame.reward >= 0 ? "positive" : "negative";
  byId("energy-reward-value").textContent = `${frame.rewardBreakdown.energyDelta >= 0 ? "+" : ""}${frame.rewardBreakdown.energyDelta.toFixed(3)}`;
  byId("energy-reward-value").className = frame.rewardBreakdown.energyDelta >= 0 ? "positive" : "negative";
  byId("distance-reward-value").textContent = `${frame.rewardBreakdown.distanceProgress >= 0 ? "+" : ""}${frame.rewardBreakdown.distanceProgress.toFixed(3)}`;
  byId("distance-reward-value").className = frame.rewardBreakdown.distanceProgress >= 0 ? "positive" : "negative";
  byId("position-value").textContent = `${frame.position.x}, ${frame.position.y}`;
  byId("heading-value").textContent = labels[frame.heading];
  const actionBars = byId("action-bars"); actionBars.replaceChildren(); frame.actionProbabilities.forEach((value,i) => actionBars.append(makeBar(labels[dataset.actionLabels[i]] ?? dataset.actionLabels[i], value)));
  const sensorBars = byId("sensor-bars"); sensorBars.replaceChildren(); frame.sensors.forEach((value,i) => sensorBars.append(makeBar(dataset.sensorLabels[i], value, i === 11)));
  const hidden = byId("hidden-grid"); hidden.replaceChildren(); frame.hiddenActivity.forEach((value,i) => { const cell=document.createElement("i"); cell.title=`H${i}: ${value.toFixed(3)}`; cell.style.setProperty("--activity", String(Math.abs(value))); cell.className=value>=0?"excited":"suppressed"; hidden.append(cell); });
  drawWorld();
};

const drawCurve = () => {
  const ctx = fitCanvas(curveCanvas); const rect=curveCanvas.getBoundingClientRect(); ctx.clearRect(0,0,rect.width,rect.height);
  const pad={l:44,r:18,t:16,b:30}; const w=rect.width-pad.l-pad.r,h=rect.height-pad.t-pad.b; const maxFood=dataset.config.arena.foodCount;
  ctx.strokeStyle="#d8e3db";ctx.fillStyle="#718078";ctx.font="11px system-ui";ctx.textAlign="right";
  for(let i=0;i<=maxFood;i++){const y=pad.t+h-i/maxFood*h;ctx.beginPath();ctx.moveTo(pad.l,y);ctx.lineTo(pad.l+w,y);ctx.stroke();if(i%2===0)ctx.fillText(String(i),pad.l-8,y+4);}
  ctx.strokeStyle="#23845d";ctx.lineWidth=3;ctx.beginPath();dataset.trainingCurve.forEach((p,i)=>{const x=pad.l+p.episode/dataset.config.trainingEpisodes*w;const y=pad.t+h-p.meanFoodsEaten/maxFood*h;i?ctx.lineTo(x,y):ctx.moveTo(x,y);});ctx.stroke();
  ctx.fillStyle="#718078";ctx.textAlign="center";ctx.fillText("训练回合",pad.l+w/2,rect.height-6);ctx.textAlign="left";ctx.fillStyle="#23845d";ctx.fillText("平均获取食物",pad.l+8,pad.t+14);
};

const intervalText = (interval: Interval, digits = 2) =>
  `${interval.mean.toFixed(digits)} [${interval.lower95.toFixed(digits)}, ${interval.upper95.toFixed(digits)}]`;

const auditConfigParts = (variant: AuditVariant) => {
  const parts = [`距离奖励 ${variant.reward.distanceProgressWeight.toFixed(3)}`];
  if (!variant.sensors.foodDirectionEnabled) parts.push("食物方向关闭");
  else {
    if (variant.sensors.foodDirectionPrecision < 1) parts.push(`方向精度 ${(variant.sensors.foodDirectionPrecision * 100).toFixed(0)}%`);
    if (variant.sensors.foodDirectionNoise > 0) parts.push(`噪声 ${variant.sensors.foodDirectionNoise.toFixed(2)}`);
    if (variant.sensors.foodDirectionDropout > 0) parts.push(`遮挡 ${(variant.sensors.foodDirectionDropout * 100).toFixed(0)}%`);
  }
  if (parts.length === 1 && variant.sensors.foodDirectionEnabled) parts.push("完整方向感觉");
  return parts;
};

const drawGateACurve = () => {
  const variant = gateA.variants.find(item => item.id === gateASelect.value) ?? gateA.variants[0];
  const ctx = fitCanvas(gateACurveCanvas); const rect = gateACurveCanvas.getBoundingClientRect(); ctx.clearRect(0, 0, rect.width, rect.height);
  const pad = {l: 48, r: 20, t: 18, b: 32}; const w = rect.width - pad.l - pad.r, h = rect.height - pad.t - pad.b;
  const maxFood = gateA.config.arena.foodCount; const xOf = (episode: number) => pad.l + episode / gateA.config.trainingEpisodes * w;
  const yOf = (food: number) => pad.t + h - Math.max(0, Math.min(maxFood, food)) / maxFood * h;
  ctx.strokeStyle = "#d8e3db"; ctx.fillStyle = "#718078"; ctx.font = "11px system-ui"; ctx.textAlign = "right";
  for (let i = 0; i <= maxFood; i++) { const y = yOf(i); ctx.beginPath(); ctx.moveTo(pad.l, y); ctx.lineTo(pad.l + w, y); ctx.stroke(); if (i % 2 === 0) ctx.fillText(String(i), pad.l - 8, y + 4); }
  const points = variant.trainingCurve;
  ctx.fillStyle = "rgba(43, 153, 105, .18)"; ctx.beginPath();
  points.forEach((point, index) => { const x = xOf(point.episode), y = yOf(point.foodsEaten.upper95); index ? ctx.lineTo(x, y) : ctx.moveTo(x, y); });
  [...points].reverse().forEach(point => ctx.lineTo(xOf(point.episode), yOf(point.foodsEaten.lower95))); ctx.closePath(); ctx.fill();
  ctx.strokeStyle = "#23845d"; ctx.lineWidth = 3; ctx.beginPath();
  points.forEach((point, index) => { const x = xOf(point.episode), y = yOf(point.foodsEaten.mean); index ? ctx.lineTo(x, y) : ctx.moveTo(x, y); }); ctx.stroke();
  ctx.fillStyle = "#718078"; ctx.textAlign = "center"; ctx.fillText("训练回合", pad.l + w / 2, rect.height - 6);
  ctx.textAlign = "left"; ctx.fillStyle = "#23845d"; ctx.fillText("均值与 95% CI", pad.l + 8, pad.t + 14);
  byId("gate-a-config").replaceChildren(...auditConfigParts(variant).map(text => { const chip = document.createElement("span"); chip.textContent = text; return chip; }));
};

const renderGateA = () => {
  byId("gate-a-protocol").textContent = `${gateA.config.modelSeedCount} 模型种子 × ${gateA.config.evaluationEpisodes} 未见地图`;
  const acceptanceLabels: Record<string, string> = {
    canonicalVariantsComplete: "9项审计完整", multipleModelSeeds: "独立模型种子",
    rewardComponentsRecorded: "奖励分量可核对", confidenceIntervalsReported: "报告95%区间",
    noShapingBeatsLearningDisabled: "无塑形仍胜过对照",
  };
  const acceptance = byId("gate-a-acceptance"); acceptance.replaceChildren();
  for (const [key, label] of Object.entries(acceptanceLabels)) {
    const pass = gateA.acceptance[key]; const item = document.createElement("div"); item.className = pass ? "pass" : "fail";
    item.innerHTML = `<i>${pass ? "✓" : "×"}</i><span>${label}</span>`; acceptance.append(item);
  }
  const variants = byId("gate-a-variants"); variants.replaceChildren();
  for (const variant of gateA.variants) {
    const card = document.createElement("article"); card.className = `audit-variant ${variant.reward.distanceProgressWeight === 0 ? "no-shaping" : ""}`;
    const significant = variant.pairedEffect.foodsEaten.lower95 > 0;
    const threshold = variant.sampleEfficiency.meanEpisodesWhenReached;
    const thresholdText = threshold ? `${threshold.mean.toFixed(0)} 回合 · ${(variant.sampleEfficiency.reachedSeedFraction * 100).toFixed(0)}%种子` : "未达到";
    card.innerHTML = `<header><div><span>${variant.id}</span><h3>${variant.label}</h3></div><i class="${significant ? "significant" : "uncertain"}">${significant ? "显著" : "不确定"}</i></header><div class="config-chips">${auditConfigParts(variant).map(text => `<span>${text}</span>`).join("")}</div><dl><div><dt>学习后</dt><dd>${intervalText(variant.learned.foodsEaten)}</dd></div><div><dt>关闭学习</dt><dd>${intervalText(variant.learningDisabled.foodsEaten)}</dd></div><div><dt>达到 ${variant.sampleEfficiency.foodThreshold.toFixed(0)} 食物</dt><dd>${thresholdText}</dd></div><div class="effect"><dt>配对增益 Δ</dt><dd>${intervalText(variant.pairedEffect.foodsEaten)}</dd></div></dl>`;
    variants.append(card);
  }
  gateASelect.replaceChildren(...gateA.variants.map(variant => { const option = document.createElement("option"); option.value = variant.id; option.textContent = variant.label; if (variant.id === "no-distance-shaping") option.selected = true; return option; }));
  drawGateACurve();
};

const gateBPhase = (frame: GateBFrame) => {
  if (frame.visibleCue && frame.step <= gateB.config.cueSteps) return "线索写入";
  if (frame.positionBefore.y > 2) return "无信息延迟";
  if (frame.positionBefore.y === 2 && frame.positionBefore.x === 4) return frame.branchChoice ? "岔路选择" : "岔路决策";
  return frame.reward > 0 ? "获得食物" : "分支结果";
};

const drawGateBWorld = () => {
  const ctx = fitCanvas(gateBWorldCanvas); const rect = gateBWorldCanvas.getBoundingClientRect();
  ctx.clearRect(0, 0, rect.width, rect.height); ctx.fillStyle = "#f7faf7"; ctx.fillRect(0, 0, rect.width, rect.height);
  const cell = Math.min((rect.width - 70) / 5, (rect.height - 55) / 8); const ox = rect.width / 2 - 2.5 * cell; const oy = (rect.height - 8 * cell) / 2;
  const center = (point: Position): [number, number] => [ox + (point.x - 2 + .5) * cell, oy + (point.y - 1 + .5) * cell];
  const open = [{x:3,y:2},{x:5,y:2}, ...Array.from({length:7}, (_,i)=>({x:4,y:i+2}))];
  for (const point of open) { const [x,y]=center(point); ctx.fillStyle="#edf4ef";ctx.strokeStyle="#cbd9cf";ctx.lineWidth=1;ctx.beginPath();ctx.roundRect(x-cell*.45,y-cell*.45,cell*.9,cell*.9,7);ctx.fill();ctx.stroke(); }
  const target = gateBTrace.target === "left" ? {x:3,y:2} : {x:5,y:2}; const other = gateBTrace.target === "left" ? {x:5,y:2} : {x:3,y:2};
  let [tx,ty]=center(target); ctx.fillStyle="#dff3e7";ctx.strokeStyle="#35a46e";ctx.lineWidth=2;ctx.beginPath();ctx.roundRect(tx-cell*.43,ty-cell*.43,cell*.86,cell*.86,7);ctx.fill();ctx.stroke();
  ctx.fillStyle="#25835a";ctx.font="600 10px system-ui";ctx.textAlign="center";ctx.fillText("食物",tx,ty+4);
  const [wx,wy]=center(other);ctx.fillStyle="#87958e";ctx.font="10px system-ui";ctx.fillText("无奖励",wx,wy+4);
  const frames=gateBTrace.frames.slice(0,gateBFrameIndex+1);if(frames.length){ctx.strokeStyle="rgba(37,139,94,.38)";ctx.lineWidth=4;ctx.lineCap="round";ctx.beginPath();frames.forEach((frame,index)=>{const [x,y]=center(frame.position);index?ctx.lineTo(x,y):ctx.moveTo(x,y);});ctx.stroke();}
  const frame=gateBTrace.frames[gateBFrameIndex];const [ax,ay]=center(frame.position);ctx.fillStyle="#174f45";ctx.beginPath();ctx.arc(ax,ay,Math.max(7,cell*.17),0,Math.PI*2);ctx.fill();
  if(frame.visibleCue){ctx.fillStyle="#f2b84b";ctx.font="700 12px system-ui";ctx.fillText(`线索：${labels[frame.visibleCue]}`,rect.width/2,18);}
  ctx.fillStyle="#718078";ctx.font="10px system-ui";ctx.fillText("观察画面标出真实目标；控制器输入不包含目标位置",rect.width/2,rect.height-8);
};

const renderGateBFrame = () => {
  const frame=gateBTrace.frames[gateBFrameIndex];gateBRange.value=String(gateBFrameIndex);
  byId("gate-b-step").textContent=`step ${frame.step} / ${gateBTrace.frames.length}`;
  byId("gate-b-phase").textContent=gateBPhase(frame);byId("gate-b-cue").textContent=frame.visibleCue?labels[frame.visibleCue]:"关闭";
  byId("gate-b-target").textContent=labels[gateBTrace.target];byId("gate-b-choice").textContent=frame.branchChoice?labels[frame.branchChoice]:"未选择";
  byId("gate-b-action").textContent=labels[frame.action];byId("gate-b-reward").textContent=`${frame.reward>=0?"+":""}${frame.reward.toFixed(3)}`;
  const actions=byId("gate-b-action-bars");actions.replaceChildren();frame.actionProbabilities.forEach((value,index)=>actions.append(makeBar(labels[["forward","turn-left","turn-right","eat"][index]],value)));
  const hidden=byId("gate-b-hidden");hidden.replaceChildren();frame.hiddenActivity.forEach((value,index)=>{const cell=document.createElement("i");cell.title=`H${index}: ${value.toFixed(3)}`;cell.style.setProperty("--activity",String(Math.abs(value)));cell.className=value>=0?"excited":"suppressed";hidden.append(cell);});
  [...byId("gate-b-events").children].forEach((item,index)=>item.classList.toggle("active",index===gateBFrameIndex));drawGateBWorld();
};

const selectGateBTrace = () => {
  gateBTrace=gateB.traces.find(item=>item.label===gateBTraceSelect.value)??gateB.traces[0];gateBFrameIndex=0;gateBPlaying=false;gateBPlay.textContent="▶";gateBRange.max=String(gateBTrace.frames.length-1);
  const events=byId("gate-b-events");events.replaceChildren(...gateBTrace.frames.map((frame,index)=>{const button=document.createElement("button");button.className=frame.visibleCue?"cue":frame.branchChoice?"decision":"delay";button.innerHTML=`<i>${index+1}</i><span>${gateBPhase(frame)}</span>`;button.addEventListener("click",()=>setGateBFrame(index));return button;}));renderGateBFrame();
};

const setGateBFrame = (value:number) => { gateBFrameIndex=Math.max(0,Math.min(gateBTrace.frames.length-1,value));renderGateBFrame(); };

const drawGateBCurve = () => {
  const reports=gateB.reports.filter(report=>report.condition==="delayed-cue");const ctx=fitCanvas(gateBCurveCanvas);const rect=gateBCurveCanvas.getBoundingClientRect();ctx.clearRect(0,0,rect.width,rect.height);
  const pad={l:48,r:20,t:20,b:32},w=rect.width-pad.l-pad.r,h=rect.height-pad.t-pad.b;const xOf=(episode:number)=>pad.l+episode/gateB.config.trainingEpisodes*w;const yOf=(value:number)=>pad.t+h-Math.max(0,Math.min(1,value))*h;
  ctx.strokeStyle="#d8e3db";ctx.fillStyle="#718078";ctx.font="11px system-ui";ctx.textAlign="right";for(let i=0;i<=4;i++){const value=i/4,y=yOf(value);ctx.beginPath();ctx.moveTo(pad.l,y);ctx.lineTo(pad.l+w,y);ctx.stroke();ctx.fillText(`${value*100}%`,pad.l-8,y+4);}
  const colors:Record<GateBController,string>={stateless:"#8a9891","state-reset":"#d08a43","leaky-state":"#23845d"};for(const report of reports){const points=report.trainingCurve,color=colors[report.controller];ctx.fillStyle=`${color}20`;ctx.beginPath();points.forEach((point,index)=>{const x=xOf(point.episode),y=yOf(point.correctChoiceFraction.upper95);index?ctx.lineTo(x,y):ctx.moveTo(x,y);});[...points].reverse().forEach(point=>ctx.lineTo(xOf(point.episode),yOf(point.correctChoiceFraction.lower95)));ctx.closePath();ctx.fill();ctx.strokeStyle=color;ctx.lineWidth=2.5;ctx.beginPath();points.forEach((point,index)=>{const x=xOf(point.episode),y=yOf(point.correctChoiceFraction.mean);index?ctx.lineTo(x,y):ctx.moveTo(x,y);});ctx.stroke();}
  ctx.textAlign="left";let lx=pad.l+8;for(const report of reports){ctx.fillStyle=colors[report.controller];ctx.fillRect(lx,pad.t+4,14,3);ctx.fillText(labels[report.controller],lx+19,pad.t+8);lx+=92;}ctx.fillStyle="#718078";ctx.textAlign="center";ctx.fillText("训练回合",pad.l+w/2,rect.height-6);
};

const renderGateB = () => {
  byId("gate-b-protocol").textContent=`${gateB.config.modelSeedCount} 模型种子 × ${gateB.config.evaluationEpisodes} 均衡测试 · leak ${gateB.config.controller.hiddenLeak.toFixed(2)}`;
  const acceptanceLabels:Record<string,string>={visibleControlLearnable:"岔路可见均可学",randomizedControlAtChance:"随机线索为机会水平",leakyBeatsStateReset:"泄漏状态显著胜出",leakyReachesSeventyPercent:"延迟正确率 ≥ 70%",historyShuffleHurts:"置乱历史显著退化",leftRightConsistent:"左右镜像一致",deterministic:"完全确定性"};const grid=byId("gate-b-acceptance");grid.replaceChildren();for(const [key,label] of Object.entries(acceptanceLabels)){const pass=gateB.acceptance[key];const item=document.createElement("div");item.className=pass?"pass":"fail";item.innerHTML=`<i>${pass?"✓":"×"}</i><span>${label}</span>`;grid.append(item);}
  gateBTraceSelect.replaceChildren(...gateB.traces.map(trace=>{const option=document.createElement("option");option.value=trace.label;option.textContent=`${labels[trace.condition]} · ${labels[trace.controller]}`;if(trace.condition==="delayed-cue"&&trace.controller==="leaky-state")option.selected=true;return option;}));
  const comparison=byId("gate-b-comparison");comparison.replaceChildren(...gateB.reports.filter(report=>report.condition==="delayed-cue").map(report=>{const card=document.createElement("article");const metric=report.metrics.correctChoiceFraction;card.className=report.controller==="leaky-state"?"memory":"";card.innerHTML=`<span>${labels[report.controller]}</span><strong>${(metric.mean*100).toFixed(1)}%</strong><small>95% CI ${(metric.lower95*100).toFixed(1)}–${(metric.upper95*100).toFixed(1)}%</small><dl><div><dt>左目标</dt><dd>${(report.metrics.leftTargetAccuracy.mean*100).toFixed(1)}%</dd></div><div><dt>右目标</dt><dd>${(report.metrics.rightTargetAccuracy.mean*100).toFixed(1)}%</dd></div><div><dt>到达岔路</dt><dd>${(report.metrics.branchChoiceFraction.mean*100).toFixed(0)}%</dd></div></dl>`;return card;}));
  const effects=byId("gate-b-effects");effects.replaceChildren(...gateB.pairedEffects.map(effect=>{const card=document.createElement("article");const value=effect.effect.correctChoiceFraction;card.innerHTML=`<span>${effect.id==="leaky-vs-state-reset"?"泄漏状态 − 每步清空":"原历史 − 置乱历史"}</span><strong>+${(value.mean*100).toFixed(1)} pp</strong><small>配对 95% CI ${(value.lower95*100).toFixed(1)}–${(value.upper95*100).toFixed(1)} pp</small>`;return card;}));
  selectGateBTrace();drawGateBCurve();
};

const renderRobustness = () => {
  byId("robustness-boundary").textContent = robustness.reliableMemoryBoundarySteps === null ? "未找到可靠边界" : `可靠记忆边界 ${robustness.reliableMemoryBoundarySteps} 步`;
  const definitions: { factor: RobustnessFactor; title: string; unit: string }[] = [
    { factor: "delay-steps", title: "无信息延迟", unit: "步" },
    { factor: "cue-input-scale", title: "线索投影", unit: "×" },
    { factor: "persistent-input-scale", title: "持续输入干扰", unit: "×" },
  ];
  const groups = byId("robustness-groups");
  groups.replaceChildren(...definitions.map(definition => {
    const article = document.createElement("article");
    const points = robustness.points.filter(point => point.factor === definition.factor).sort((a,b) => a.value-b.value);
    article.innerHTML = `<h3>${definition.title}</h3><div class="boundary-bars">${points.map(point => {
      const metric = point.report.metrics.correctChoiceFraction;
      const reliable = metric.mean >= .7 && metric.lower95 > .5;
      const baseline = point.delaySteps === robustness.baselineDelaySteps && point.cueInputScale === robustness.baselineCueInputScale && point.persistentInputScale === robustness.baselinePersistentInputScale;
      return `<div class="boundary-row ${reliable ? "reliable" : "limited"} ${baseline ? "baseline" : ""}"><span>${point.value.toFixed(definition.factor === "delay-steps" ? 0 : 2)} ${definition.unit}</span><div><i style="width:${Math.max(0,Math.min(100,metric.mean*100))}%"></i><b style="left:${Math.max(0,Math.min(100,metric.lower95*100))}%;width:${Math.max(0,(Math.min(1,metric.upper95)-Math.max(0,metric.lower95))*100)}%"></b></div><strong>${(metric.mean*100).toFixed(1)}%</strong></div>`;
    }).join("")}</div>`;
    return article;
  }));
};

const renderGateCFrame = () => {
  const frame = gateCTrace.frames[gateCFrameIndex];
  gateCRange.value = String(gateCFrameIndex);
  const phase = frame.phaseBefore === "junction" ? "选择分支" : frame.energyPaidOut ? "真实能量到账" : `强制等待 · 还剩 ${frame.waitingStepsRemainingBefore} 步`;
  byId("gate-c-phase").textContent = phase;
  byId("gate-c-reward").textContent = frame.reward > 0 ? `+${frame.reward.toFixed(3)} reward` : "无能量后果";
  byId("gate-c-reward").className = frame.reward > 0 ? "positive" : "";
  [...byId("gate-c-events").children].forEach((item,index) => item.classList.toggle("active", index === gateCFrameIndex));
  byId("gate-c-trace-note").textContent = `目标 ${labels[gateCTrace.target]} · 线索 ${frame.visibleCue ? labels[frame.visibleCue] : "已关闭"} · 动作 ${labels[frame.action]} · 能量 ${frame.energy.toFixed(3)}`;
};

const selectGateCTrace = () => {
  gateCTrace = gateC.traces.find(item => item.label === gateCTraceSelect.value) ?? gateC.traces[0];
  gateCFrameIndex = 0; gateCRange.max = String(gateCTrace.frames.length - 1);
  const events = byId("gate-c-events");
  events.replaceChildren(...gateCTrace.frames.map((frame,index) => {
    const button = document.createElement("button");
    button.className = frame.phaseBefore === "junction" ? "choice" : frame.energyPaidOut ? "payout" : "wait";
    button.innerHTML = `<i>${frame.step}</i><span>${frame.phaseBefore === "junction" ? "选择" : frame.energyPaidOut ? "到账" : "等待"}</span>`;
    button.addEventListener("click", () => { gateCFrameIndex=index; renderGateCFrame(); });
    return button;
  }));
  renderGateCFrame();
};

const renderGateC = () => {
  byId("gate-c-protocol").textContent = `${gateC.config.modelSeedCount} 模型种子 × ${gateC.config.evaluationEpisodes} 均衡测试`;
  const acceptanceLabels: Record<string,string> = { immediateRewardLearnable:"即时奖励可学习",currentTraceBeatsZeroAtDelayEight:"8 步资格迹显著胜出",currentTraceReachesSeventyPercent:"8 步正确率 ≥ 70%",zeroTraceDegradesWithDelay:"无轨迹随延迟退化",randomizedControlAtChance:"随机线索为机会水平",leftRightAndBranchConsistent:"左右与提交一致",deterministic:"完全确定性" };
  const acceptance = byId("gate-c-acceptance"); acceptance.replaceChildren();
  for (const [key,label] of Object.entries(acceptanceLabels)) { const pass=gateC.acceptance[key]; const item=document.createElement("div"); item.className=pass?"pass":"fail"; item.innerHTML=`<i>${pass?"✓":"×"}</i><span>${label}</span>`; acceptance.append(item); }
  const matrix = byId("gate-c-matrix");
  const cells = [`<div class="matrix-head">延迟 \ trace</div>`, ...gateC.config.eligibilityDecays.map(value=>`<div class="matrix-head">${value.toFixed(2)}</div>`)];
  for (const delay of gateC.config.rewardDelays) {
    cells.push(`<div class="matrix-head">${delay} 步</div>`);
    for (const decay of gateC.config.eligibilityDecays) {
      const report=gateC.reports.find(item=>!item.cueRandomized&&item.rewardDelaySteps===delay&&Math.abs(item.eligibilityDecay-decay)<1e-9);
      if (!report) { cells.push("<div>—</div>"); continue; }
      const value=report.metrics.correctChoiceFraction; const strength=Math.max(0,Math.min(1,(value.mean-.5)/.5));
      cells.push(`<div class="matrix-cell" style="--strength:${strength}"><strong>${(value.mean*100).toFixed(1)}%</strong><small>${(value.lower95*100).toFixed(1)}–${(value.upper95*100).toFixed(1)}</small></div>`);
    }
  }
  matrix.innerHTML=cells.join("");
  const effects=byId("gate-c-effects"); effects.replaceChildren(...gateC.pairedEffects.map(effect=>{const card=document.createElement("article");const value=effect.effect.correctChoiceFraction;const title=effect.id==="delay-8-current-vs-zero"?"延迟 8：trace 0.88 − 0":"trace 0：即时 − 延迟 8";card.innerHTML=`<span>${title}</span><strong>${(value.mean*100).toFixed(1)} pp</strong><small>配对 95% CI ${(value.lower95*100).toFixed(1)}–${(value.upper95*100).toFixed(1)} pp</small>`;return card;}));
  gateCTraceSelect.replaceChildren(...gateC.traces.map(trace=>{const option=document.createElement("option");option.value=trace.label;option.textContent=`延迟 ${trace.rewardDelaySteps} · trace ${trace.eligibilityDecay.toFixed(2)}${trace.cueRandomized?" · 随机线索":""}`;if(trace.rewardDelaySteps===8&&Math.abs(trace.eligibilityDecay-.88)<1e-9)option.selected=true;return option;}));
  selectGateCTrace();
};

const drawGateDBoundary = () => {
  const ctx=fitCanvas(gateDBoundaryCanvas);const rect=gateDBoundaryCanvas.getBoundingClientRect();ctx.clearRect(0,0,rect.width,rect.height);
  const pad={l:48,r:18,t:26,b:34},w=rect.width-pad.l-pad.r,h=rect.height-pad.t-pad.b;
  const delays=gateD.config.memoryDelays;const xOf=(delay:number)=>pad.l+delays.indexOf(delay)/Math.max(1,delays.length-1)*w;const yOf=(value:number)=>pad.t+h-Math.max(0,Math.min(1,value))*h;
  ctx.font="10px system-ui";ctx.strokeStyle="#d8e3db";ctx.fillStyle="#718078";ctx.textAlign="right";
  for(let i=0;i<=4;i++){const value=i/4,y=yOf(value);ctx.beginPath();ctx.moveTo(pad.l,y);ctx.lineTo(pad.l+w,y);ctx.stroke();ctx.fillText(`${value*100}%`,pad.l-7,y+3);}
  ctx.setLineDash([5,4]);ctx.strokeStyle="#b79548";ctx.beginPath();ctx.moveTo(pad.l,yOf(.7));ctx.lineTo(pad.l+w,yOf(.7));ctx.stroke();ctx.setLineDash([]);
  const colors:Record<GateDController,string>={"no-recurrence":"#8a9891","shuffled-recurrence":"#d08a43","structured-recurrence":"#23845d"};
  for(const controller of Object.keys(colors) as GateDController[]){const reports=gateD.reports.filter(report=>report.controller===controller).sort((a,b)=>a.delaySteps-b.delaySteps);ctx.strokeStyle=colors[controller];ctx.lineWidth=2.5;ctx.beginPath();reports.forEach((report,index)=>{const x=xOf(report.delaySteps),y=yOf(report.metrics.correctChoiceFraction.mean);index?ctx.lineTo(x,y):ctx.moveTo(x,y);});ctx.stroke();for(const report of reports){const metric=report.metrics.correctChoiceFraction,x=xOf(report.delaySteps);ctx.strokeStyle=colors[controller];ctx.lineWidth=1.5;ctx.beginPath();ctx.moveTo(x,yOf(metric.lower95));ctx.lineTo(x,yOf(metric.upper95));ctx.stroke();ctx.fillStyle=colors[controller];ctx.beginPath();ctx.arc(x,yOf(metric.mean),4,0,Math.PI*2);ctx.fill();}}
  ctx.fillStyle="#718078";ctx.textAlign="center";delays.forEach(delay=>ctx.fillText(`${delay} 步`,xOf(delay),rect.height-9));
  let legendX=pad.l+8;ctx.textAlign="left";for(const controller of Object.keys(colors) as GateDController[]){ctx.fillStyle=colors[controller];ctx.fillRect(legendX,9,14,3);ctx.fillText(labels[controller],legendX+19,13);legendX+=105;}
};

const renderGateDFrame = () => {
  const frame=gateDTrace.frames[gateDFrameIndex];gateDRange.value=String(gateDFrameIndex);
  const previous=gateDFrameIndex>0?gateDTrace.frames[gateDFrameIndex-1]:null;
  const phase=frame.reward>0?"获得真实食物能量":frame.visibleCue?"线索写入":frame.branchChoice&&!previous?.branchChoice?"岔路选择":frame.branchChoice?"已选分支":frame.delayStepsRemaining>0?`无信息延迟 · 剩 ${frame.delayStepsRemaining} 步`:"岔路决策";
  byId("gate-d-phase").textContent=phase;byId("gate-d-cue").textContent=frame.visibleCue?`线索 ${labels[frame.visibleCue]}`:"线索关闭";
  const hidden=byId("gate-d-hidden");hidden.replaceChildren();frame.hiddenActivity.forEach((value,index)=>{const cell=document.createElement("i");cell.title=`H${index}: ${value.toFixed(3)}`;cell.style.setProperty("--activity",String(Math.abs(value)));cell.className=value>=0?"excited":"suppressed";hidden.append(cell);});
  [...byId("gate-d-events").children].forEach((item,index)=>item.classList.toggle("active",index===gateDFrameIndex));
  byId("gate-d-trace-note").textContent=`${labels[gateDTrace.controller]} · 目标 ${labels[gateDTrace.target]} · 动作 ${labels[frame.action]} · 能量 ${frame.energy.toFixed(3)}`;
};

const selectGateDTrace = () => {
  gateDTrace=gateD.traces.find(item=>item.label===gateDTraceSelect.value)??gateD.traces[0];gateDFrameIndex=0;gateDRange.max=String(gateDTrace.frames.length-1);
  const events=byId("gate-d-events");events.replaceChildren(...gateDTrace.frames.map((frame,index)=>{const previous=index>0?gateDTrace.frames[index-1]:null;const chose=frame.branchChoice&&!previous?.branchChoice;const label=frame.reward>0?"食物":frame.visibleCue?"线索":chose?"选择":frame.branchChoice?"分支":"延迟";const button=document.createElement("button");button.className=frame.reward>0?"payout":frame.visibleCue?"choice":"wait";button.innerHTML=`<i>${frame.step}</i><span>${label}</span>`;button.addEventListener("click",()=>{gateDFrameIndex=index;renderGateDFrame();});return button;}));renderGateDFrame();
};

const renderGateD = () => {
  byId("gate-d-protocol").textContent=`${gateD.config.modelSeedCount} 模型种子 · ${gateD.config.recurrentInDegree} 入边/单元 · 谱 ${gateD.config.recurrentSpectralRadius.toFixed(2)}`;
  const acceptanceLabels:Record<string,string>={structuredBeatsShuffledAtDelayEight:"结构显著胜过置乱",structuredDelayEightLearnable:"8 步正确率 ≥ 70%",structuredExtendsReliableBoundary:"可靠边界得到延长",shuffledDoesNotMatchPrimaryGain:"任意循环不足以解释",structuredStateStable:"内部状态稳定",budgetsAndRecurrentControlsMatched:"预算与权重控制匹配",deterministic:"完全确定性"};const acceptance=byId("gate-d-acceptance");acceptance.replaceChildren();for(const [key,label] of Object.entries(acceptanceLabels)){const pass=gateD.acceptance[key];const item=document.createElement("div");item.className=pass?"pass":"fail";item.innerHTML=`<i>${pass?"✓":"×"}</i><span>${label}</span>`;acceptance.append(item);}
  const boundary=new Map(gateD.reliableMemoryBoundarySteps);const comparison=byId("gate-d-comparison");comparison.replaceChildren(...(["no-recurrence","shuffled-recurrence","structured-recurrence"] as GateDController[]).map(controller=>{const report=gateD.reports.find(item=>item.delaySteps===8&&item.controller===controller)!;const metric=report.metrics.correctChoiceFraction;const card=document.createElement("article");if(controller==="structured-recurrence")card.className="memory";card.innerHTML=`<span>${labels[controller]}</span><strong>${(metric.mean*100).toFixed(1)}%</strong><small>延迟 8 · 95% CI ${(metric.lower95*100).toFixed(1)}–${(metric.upper95*100).toFixed(1)}%</small><dl><div><dt>可靠边界</dt><dd>${boundary.get(controller)??"无"} 步</dd></div><div><dt>左目标</dt><dd>${(report.metrics.leftTargetAccuracy.mean*100).toFixed(1)}%</dd></div><div><dt>右目标</dt><dd>${(report.metrics.rightTargetAccuracy.mean*100).toFixed(1)}%</dd></div></dl>`;return card;}));
  const effects=byId("gate-d-effects");effects.replaceChildren(...gateD.pairedEffects.map(effect=>{const value=effect.effect.correctChoiceFraction;const card=document.createElement("article");card.innerHTML=`<span>${effect.id.includes("structured")?"结构化 − 等权重置乱":"置乱循环 − 无循环"}</span><strong>${value.mean>=0?"+":""}${(value.mean*100).toFixed(1)} pp</strong><small>配对 95% CI ${(value.lower95*100).toFixed(1)}–${(value.upper95*100).toFixed(1)} pp</small>`;return card;}));
  const structured=gateD.reports.find(item=>item.delaySteps===8&&item.controller==="structured-recurrence")!;const stabilityDefinitions:[string,Interval,number,string][]=[["平均绝对活动",structured.stability.meanAbsoluteActivity,.9,""],["饱和比例",structured.stability.saturatedUnitFraction,.25,"%"],["静默比例",structured.stability.silentUnitFraction,.6,"%"],["群体同步",structured.stability.populationSynchrony,.9,""]];const stability=byId("gate-d-stability");stability.replaceChildren(...stabilityDefinitions.map(([label,metric,limit,unit])=>{const row=document.createElement("div");row.innerHTML=`<span>${label}</span><div><i style="width:${Math.min(100,metric.mean/limit*100)}%"></i></div><strong>${unit?(metric.mean*100).toFixed(1)+unit:metric.mean.toFixed(3)}</strong>`;return row;}));
  gateDTraceSelect.replaceChildren(...gateD.traces.map(trace=>{const option=document.createElement("option");option.value=trace.label;option.textContent=labels[trace.controller];if(trace.controller==="structured-recurrence")option.selected=true;return option;}));selectGateDTrace();drawGateDBoundary();
};

const drawGateEBoundary = () => {
  const ctx=fitCanvas(gateEBoundaryCanvas);const rect=gateEBoundaryCanvas.getBoundingClientRect();ctx.clearRect(0,0,rect.width,rect.height);
  const pad={l:48,r:18,t:28,b:34},w=rect.width-pad.l-pad.r,h=rect.height-pad.t-pad.b;
  const delays=gateE.config.memoryDelays;const xOf=(delay:number)=>pad.l+delays.indexOf(delay)/Math.max(1,delays.length-1)*w;const yOf=(value:number)=>pad.t+h-Math.max(0,Math.min(1,value))*h;
  ctx.font="10px system-ui";ctx.strokeStyle="#d8e3db";ctx.fillStyle="#718078";ctx.textAlign="right";
  for(let i=0;i<=4;i++){const value=i/4,y=yOf(value);ctx.beginPath();ctx.moveTo(pad.l,y);ctx.lineTo(pad.l+w,y);ctx.stroke();ctx.fillText(`${value*100}%`,pad.l-7,y+3);}
  ctx.setLineDash([5,4]);ctx.strokeStyle="#b79548";ctx.beginPath();ctx.moveTo(pad.l,yOf(.7));ctx.lineTo(pad.l+w,yOf(.7));ctx.stroke();ctx.setLineDash([]);
  const colors:Record<GateEController,string>={stateless:"#8a9891","continuous-state":"#3b82a0","lif-spiking":"#d9822b"};
  for(const controller of Object.keys(colors) as GateEController[]){const reports=gateE.reports.filter(report=>report.controller===controller).sort((a,b)=>a.delaySteps-b.delaySteps);ctx.strokeStyle=colors[controller];ctx.lineWidth=2.5;ctx.beginPath();reports.forEach((report,index)=>{const x=xOf(report.delaySteps),y=yOf(report.metrics.correctChoiceFraction.mean);index?ctx.lineTo(x,y):ctx.moveTo(x,y);});ctx.stroke();for(const report of reports){const metric=report.metrics.correctChoiceFraction,x=xOf(report.delaySteps);ctx.strokeStyle=colors[controller];ctx.lineWidth=1.5;ctx.beginPath();ctx.moveTo(x,yOf(metric.lower95));ctx.lineTo(x,yOf(metric.upper95));ctx.stroke();ctx.fillStyle=colors[controller];ctx.beginPath();ctx.arc(x,yOf(metric.mean),4,0,Math.PI*2);ctx.fill();}}
  ctx.fillStyle="#718078";ctx.textAlign="center";delays.forEach(delay=>ctx.fillText(`${delay} 步`,xOf(delay),rect.height-9));
  let legendX=pad.l+5;ctx.textAlign="left";for(const controller of Object.keys(colors) as GateEController[]){ctx.fillStyle=colors[controller];ctx.fillRect(legendX,10,14,3);ctx.fillText(labels[controller],legendX+19,14);legendX+=112;}
};

const drawGateERaster = () => {
  const ctx=fitCanvas(gateERasterCanvas);const rect=gateERasterCanvas.getBoundingClientRect();ctx.clearRect(0,0,rect.width,rect.height);
  const frames=gateETrace.frames,rows=24,pad={l:28,r:8,t:18,b:19},w=rect.width-pad.l-pad.r,h=rect.height-pad.t-pad.b,cellW=w/frames.length,rowH=h/rows;
  ctx.fillStyle="#6d7d75";ctx.font="8px ui-monospace,monospace";ctx.textAlign="right";for(let row=0;row<rows;row+=4)ctx.fillText(`H${row}`,pad.l-5,pad.t+(row+.8)*rowH);
  frames.forEach((frame,column)=>{for(let row=0;row<rows;row++){const value=Math.min(1,Math.abs(frame.features[row]));ctx.fillStyle=`rgba(39,151,103,${.04+value*.32})`;ctx.fillRect(pad.l+column*cellW,pad.t+row*rowH,Math.max(1,cellW),Math.max(1,rowH));if(frame.spikes[row]){ctx.fillStyle="#e38b21";ctx.fillRect(pad.l+column*cellW,pad.t+row*rowH,Math.max(2,cellW),Math.max(1.5,rowH));}}});
  const markerX=pad.l+(gateEFrameIndex+.5)*cellW;ctx.strokeStyle="#174f3e";ctx.lineWidth=1.5;ctx.beginPath();ctx.moveTo(markerX,pad.t-4);ctx.lineTo(markerX,pad.t+h);ctx.stroke();
  ctx.fillStyle="#718078";ctx.textAlign="center";ctx.fillText("时间 →",pad.l+w/2,rect.height-5);
};

const renderGateEFrame = () => {
  const frame=gateETrace.frames[gateEFrameIndex];gateERange.value=String(gateEFrameIndex);const previous=gateEFrameIndex>0?gateETrace.frames[gateEFrameIndex-1]:null;
  const phase=frame.reward>0?"获得真实食物能量":frame.visibleCue?"线索写入":frame.branchChoice&&!previous?.branchChoice?"岔路选择":frame.branchChoice?"已选分支":frame.delayStepsRemaining>0?`无信息延迟 · 剩 ${frame.delayStepsRemaining} 步`:"岔路决策";
  byId("gate-e-phase").textContent=phase;byId("gate-e-cue").textContent=frame.visibleCue?`线索 ${labels[frame.visibleCue]}`:"线索关闭";
  const neurons=byId("gate-e-neurons");neurons.replaceChildren();frame.features.forEach((value,index)=>{const cell=document.createElement("i");cell.title=`H${index}: trace ${value.toFixed(3)} · V ${frame.membranePotentialsMv[index].toFixed(3)} mV${frame.spikes[index]?" · spike":""}`;cell.style.setProperty("--activity",String(Math.min(1,Math.abs(value))));cell.className=frame.spikes[index]?"spike":"excited";neurons.append(cell);});
  byId("gate-e-trace-note").textContent=`${labels[gateETrace.controller]} · 目标 ${labels[gateETrace.target]} · 动作 ${labels[frame.action]} · 本步 ${frame.spikes.filter(Boolean).length} 个脉冲 · 能量 ${frame.energy.toFixed(3)}`;drawGateERaster();
};

const selectGateETrace = () => {
  gateETrace=gateE.traces.find(item=>item.label===gateETraceSelect.value)??gateE.traces[0];gateEFrameIndex=0;gateERange.max=String(gateETrace.frames.length-1);renderGateEFrame();
};

const renderGateE = () => {
  byId("gate-e-protocol").textContent=`${gateE.config.modelSeedCount} 模型种子 · 衰减 ${gateE.config.lifSpikeTraceDecay.toFixed(2)} · 25% 损伤`;
  const acceptanceLabels:Record<string,string>={budgetsMatched:"连接与状态预算匹配",lifBehaviorLearnable:"LIF 8 步可学习",lifImprovesAtLeastOneAxis:"至少一项功能收益",lifActivityValidAndSparse:"脉冲有限且稀疏",damageProtocolComplete:"损伤对照完整",deterministic:"完全确定性"};const acceptance=byId("gate-e-acceptance");acceptance.replaceChildren();for(const [key,label] of Object.entries(acceptanceLabels)){const pass=gateE.acceptance[key];const item=document.createElement("div");item.className=pass?"pass":"fail";item.innerHTML=`<i>${pass?"✓":"×"}</i><span>${label}</span>`;acceptance.append(item);}
  const boundary=new Map(gateE.reliableMemoryBoundarySteps);const comparison=byId("gate-e-comparison");comparison.replaceChildren(...(["stateless","continuous-state","lif-spiking"] as GateEController[]).map(controller=>{const report=gateE.reports.find(item=>item.delaySteps===8&&item.controller===controller)!;const metric=report.metrics.correctChoiceFraction;const card=document.createElement("article");if(controller==="lif-spiking")card.className="memory";card.innerHTML=`<span>${labels[controller]}</span><strong>${(metric.mean*100).toFixed(1)}%</strong><small>延迟 8 · 95% CI ${(metric.lower95*100).toFixed(1)}–${(metric.upper95*100).toFixed(1)}%</small><dl><div><dt>可靠边界</dt><dd>${boundary.get(controller)??"无"}${boundary.get(controller)?" 步":""}</dd></div><div><dt>左目标</dt><dd>${(report.metrics.leftTargetAccuracy.mean*100).toFixed(1)}%</dd></div><div><dt>右目标</dt><dd>${(report.metrics.rightTargetAccuracy.mean*100).toFixed(1)}%</dd></div></dl>`;return card;}));
  const effects=byId("gate-e-effects");effects.replaceChildren(...gateE.pairedEffects.map(effect=>{const value=effect.effect.correctChoiceFraction;const card=document.createElement("article");card.innerHTML=`<span>${effect.id.includes("lif")?"LIF − 匹配衰减连续状态":"连续状态 − 无状态"}</span><strong>+${(value.mean*100).toFixed(1)} pp</strong><small>配对 95% CI ${(value.lower95*100).toFixed(1)}–${(value.upper95*100).toFixed(1)} pp</small>`;return card;}));
  const damage=byId("gate-e-damage");damage.replaceChildren(...(["stateless","continuous-state","lif-spiking"] as GateEController[]).map(controller=>{const report=gateE.reports.find(item=>item.delaySteps===8&&item.controller===controller)!;const drop=report.damageAccuracyDrop!;const row=document.createElement("div");row.innerHTML=`<span>${labels[controller]}</span><div><i style="width:${Math.min(100,Math.max(0,drop.mean)*500)}%"></i></div><strong>${drop.mean>=0?"−":"+"}${Math.abs(drop.mean*100).toFixed(1)} pp</strong>`;return row;}));
  const costMap=new Map(gateE.aggregateCosts),budgetMap=new Map(gateE.controllerBudgets.map(item=>[item.controller,item]));const runtimeMap=new Map(gateERuntime.map(item=>[item.controller,item]));const costs=byId("gate-e-costs");costs.replaceChildren(...(["stateless","continuous-state","lif-spiking"] as GateEController[]).map(controller=>{const cost=costMap.get(controller)!,runtime=runtimeMap.get(controller)!,budget=budgetMap.get(controller)!;const steps=cost.environmentSteps;const card=document.createElement("article");if(controller==="lif-spiking")card.className="warning";card.innerHTML=`<span>${labels[controller]} · 当前 CPU</span><strong>${runtime.nanosecondsPerEnvironmentStep.toFixed(0)} ns/步</strong><small>概念/实现状态 ${budget.conceptualStateBytes}/${budget.implementationDynamicStateBytes} B · 输入事件 ${(cost.inputEvents/steps).toFixed(1)}/步 · 脉冲 ${(cost.emittedSpikes/steps).toFixed(3)}/步<br>输入/读出仍为 288 + 96 稠密 MAC</small>`;return card;}));
  gateETraceSelect.replaceChildren(...gateE.traces.map(trace=>{const option=document.createElement("option");option.value=trace.label;option.textContent=labels[trace.controller];if(trace.controller==="lif-spiking")option.selected=true;return option;}));selectGateETrace();drawGateEBoundary();
};

const renderEvidence = () => {
  const cards=byId("evaluation-cards");cards.replaceChildren();
  for(const item of dataset.evaluations){const card=document.createElement("article");card.className=`evaluation ${item.label}`;card.innerHTML=`<span>${labels[item.label]??item.label}</span><strong>${item.meanFoodsEaten.toFixed(2)}</strong><small>平均食物 / ${dataset.config.arena.foodCount}</small><dl><div><dt>完成率</dt><dd>${(item.completionFraction*100).toFixed(0)}%</dd></div><div><dt>最终能量</dt><dd>${item.meanFinalEnergy.toFixed(1)}</dd></div><div><dt>危险接触</dt><dd>${item.meanHazardContacts.toFixed(1)}</dd></div></dl>`;cards.append(card);}
  byId("evaluation-count").textContent=`${dataset.config.evaluationEpisodes} 张未见地图`;
  const acceptanceLabels:Record<string,string>={weightsChanged:"突触产生持久变化",learnedBeatsLearningDisabled:"胜过关闭学习",shuffleHurtsPerformance:"打乱连接后退化",lesionHurtsPerformance:"内部单元消融后退化",deterministic:"完全确定性"};
  const grid=byId("acceptance-grid");grid.replaceChildren();for(const [key,label] of Object.entries(acceptanceLabels)){const pass=dataset.acceptance[key];const item=document.createElement("div");item.className=pass?"pass":"fail";item.innerHTML=`<i>${pass?"✓":"×"}</i><span>${label}</span>`;grid.append(item);}
  byId("plasticity-summary").textContent=`${dataset.plasticity.changedWeightCount}/${dataset.plasticity.totalWeightCount} 动作突触改变 · RMS ${dataset.plasticity.rootMeanSquareChange.toFixed(3)}`;
};

const selectTrace = () => { trace=dataset.traces.find(item=>item.label===traceSelect.value)??dataset.traces[0];frameIndex=0;timeline.max=String(trace.frames.length-1);playing=false;play.textContent="▶";renderFrame(); };
const setFrame = (value:number) => { frameIndex=Math.max(0,Math.min(trace.frames.length-1,value));renderFrame(); };
play.addEventListener("click",()=>{playing=!playing;play.textContent=playing?"Ⅱ":"▶";lastTick=performance.now();});
prev.addEventListener("click",()=>setFrame(frameIndex-1)); next.addEventListener("click",()=>setFrame(frameIndex+1));
timeline.addEventListener("input",()=>setFrame(Number(timeline.value))); traceSelect.addEventListener("change",selectTrace);
gateASelect.addEventListener("change", drawGateACurve);
gateBTraceSelect.addEventListener("change",selectGateBTrace);
gateCTraceSelect.addEventListener("change",selectGateCTrace);
gateCRange.addEventListener("input",()=>{gateCFrameIndex=Number(gateCRange.value);renderGateCFrame();});
gateDTraceSelect.addEventListener("change",selectGateDTrace);
gateDRange.addEventListener("input",()=>{gateDFrameIndex=Number(gateDRange.value);renderGateDFrame();});
gateETraceSelect.addEventListener("change",selectGateETrace);
gateERange.addEventListener("input",()=>{gateEFrameIndex=Number(gateERange.value);renderGateEFrame();});
gateBPlay.addEventListener("click",()=>{gateBPlaying=!gateBPlaying;gateBPlay.textContent=gateBPlaying?"Ⅱ":"▶";gateBLastTick=performance.now();});
byId("gate-b-prev").addEventListener("click",()=>setGateBFrame(gateBFrameIndex-1));byId("gate-b-next").addEventListener("click",()=>setGateBFrame(gateBFrameIndex+1));gateBRange.addEventListener("input",()=>setGateBFrame(Number(gateBRange.value)));
window.addEventListener("resize",()=>{if(ready){drawWorld();drawCurve();drawGateACurve();drawGateBWorld();drawGateBCurve();drawGateDBoundary();drawGateEBoundary();drawGateERaster();}});

const animate = (now:number) => { if(playing && now-lastTick>=1000/Number(speed.value)){lastTick=now;if(frameIndex>=trace.frames.length-1){playing=false;play.textContent="▶";}else setFrame(frameIndex+1);}if(gateBPlaying&&now-gateBLastTick>=650){gateBLastTick=now;if(gateBFrameIndex>=gateBTrace.frames.length-1){gateBPlaying=false;gateBPlay.textContent="▶";}else setGateBFrame(gateBFrameIndex+1);}requestAnimationFrame(animate); };

const start = async () => {
  const [behaviorResponse,gateAResponse,gateBResponse,robustnessResponse,gateCResponse,gateDResponse,gateEResponse,gateERuntimeResponse]=await Promise.all([fetch("/embodied-v1.json"),fetch("/gate-a-v1.1.json"),fetch("/gate-b-v1.2.json"),fetch("/gate-b-robustness-v1.2b.json"),fetch("/gate-c-v1.3.json"),fetch("/gate-d-v1.4.json"),fetch("/gate-e-v1.5.json"),fetch("/gate-e-runtime-windows-x86_64.json")]);
  if(!behaviorResponse.ok)throw new Error(`behavior dataset ${behaviorResponse.status}`);if(!gateAResponse.ok)throw new Error(`Gate A dataset ${gateAResponse.status}`);if(!gateBResponse.ok)throw new Error(`Gate B dataset ${gateBResponse.status}`);if(!robustnessResponse.ok)throw new Error(`Gate B robustness dataset ${robustnessResponse.status}`);if(!gateCResponse.ok)throw new Error(`Gate C dataset ${gateCResponse.status}`);if(!gateDResponse.ok)throw new Error(`Gate D dataset ${gateDResponse.status}`);if(!gateEResponse.ok)throw new Error(`Gate E dataset ${gateEResponse.status}`);if(!gateERuntimeResponse.ok)throw new Error(`Gate E runtime dataset ${gateERuntimeResponse.status}`);
  dataset=await behaviorResponse.json() as Dataset;gateA=await gateAResponse.json() as GateADataset;gateB=await gateBResponse.json() as GateBDataset;robustness=await robustnessResponse.json() as RobustnessDataset;gateC=await gateCResponse.json() as GateCDataset;gateD=await gateDResponse.json() as GateDDataset;gateE=await gateEResponse.json() as GateEDataset;gateERuntime=await gateERuntimeResponse.json() as GateERuntime[];
  ready=true;
  byId("version").textContent=gateE.version;byId("acceptance-label").textContent=gateE.acceptance.passed?"Gate E 验收通过":"Gate E 验收失败";byId("acceptance-dot").className=gateE.acceptance.passed?"pass":"fail";
  traceSelect.replaceChildren(...dataset.traces.map(item=>{const option=document.createElement("option");option.value=item.label;option.textContent=labels[item.label]??item.label;if(item.label==="learned")option.selected=true;return option;}));
  renderEvidence();renderGateA();renderGateB();renderRobustness();renderGateC();renderGateD();renderGateE();drawCurve();selectTrace();requestAnimationFrame(animate);
};

start().catch(error=>{document.body.innerHTML=`<pre class="fatal">无法载入具身实验：${String(error)}</pre>`;console.error(error);});
