import "./style.css";

type Heading = "north" | "east" | "south" | "west";
type Action = "forward" | "turn-left" | "turn-right" | "eat";
interface Position { x: number; y: number }
interface Frame {
  step: number; position: Position; heading: Heading; energy: number; foodsEaten: number;
  action: Action; reward: number; actionProbabilities: number[]; sensors: number[];
  hiddenActivity: number[]; remainingFood: Position[];
}
interface Summary { stepsSurvived: number; foodsEaten: number; finalEnergy: number; collisions: number; hazardContacts: number; completed: boolean; totalReward: number }
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

const byId = <T extends HTMLElement>(id: string): T => {
  const found = document.getElementById(id);
  if (!found) throw new Error(`missing #${id}`);
  return found as T;
};

const worldCanvas = byId<HTMLCanvasElement>("world");
const curveCanvas = byId<HTMLCanvasElement>("learning-curve");
const traceSelect = byId<HTMLSelectElement>("trace-select");
const timeline = byId<HTMLInputElement>("timeline");
const play = byId<HTMLButtonElement>("play");
const prev = byId<HTMLButtonElement>("prev");
const next = byId<HTMLButtonElement>("next");
const speed = byId<HTMLSelectElement>("speed");

let dataset: Dataset;
let trace: Trace;
let frameIndex = 0;
let playing = false;
let lastTick = performance.now();

const labels: Record<string, string> = {
  untrained: "未经训练", learned: "学习后", shuffled: "打乱突触", lesioned: "内部单元消融",
  "learning-disabled": "关闭学习", forward: "前进", "turn-left": "左转", "turn-right": "右转", eat: "进食",
  north: "北", east: "东", south: "南", west: "西",
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
window.addEventListener("resize",()=>{drawWorld();drawCurve();});

const animate = (now:number) => { if(playing && now-lastTick>=1000/Number(speed.value)){lastTick=now;if(frameIndex>=trace.frames.length-1){playing=false;play.textContent="▶";}else setFrame(frameIndex+1);}requestAnimationFrame(animate); };

const start = async () => {
  const response=await fetch("/embodied-v1.json");if(!response.ok)throw new Error(`dataset ${response.status}`);dataset=await response.json() as Dataset;
  byId("version").textContent=dataset.version;byId("acceptance-label").textContent=dataset.acceptance.passed?"行为验收通过":"行为验收失败";byId("acceptance-dot").className=dataset.acceptance.passed?"pass":"fail";
  traceSelect.replaceChildren(...dataset.traces.map(item=>{const option=document.createElement("option");option.value=item.label;option.textContent=labels[item.label]??item.label;if(item.label==="learned")option.selected=true;return option;}));
  renderEvidence();drawCurve();selectTrace();requestAnimationFrame(animate);
};

start().catch(error=>{document.body.innerHTML=`<pre class="fatal">无法载入具身实验：${String(error)}</pre>`;console.error(error);});
