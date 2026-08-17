import * as THREE from "three";
import "./style.css";

type Polarity = "excitatory" | "inhibitory";

interface CompactBundle {
  version: number;
  topology: { neurons: number[][]; synapses: number[][] };
  branches: CompactBranch[];
}
interface CompactBranch {
  label: string;
  startMs: number;
  endMs: number;
  eventDigest: string;
  pacemakerNeuronIds: number[];
  spikes: number[][];
  arrivals: number[][];
  samples: number[][];
  metrics: number[][];
  flights: number[][];
}
interface PlaybackDataset {
  version: number;
  label: string;
  startMs: number;
  endMs: number;
  eventDigest: string;
  neurons: PlaybackNeuron[];
  synapses: PlaybackSynapse[];
  pacemakerNeuronIds: number[];
  chunks: PlaybackChunk[];
}
interface PlaybackNeuron {
  id: number;
  polarity: Polarity;
  position: [number, number, number];
  restPotentialMv: number;
  resetPotentialMv: number;
  thresholdMv: number;
  membraneTimeConstantMs: number;
  refractoryPeriodMs: number;
}
interface PlaybackSynapse { id: number; source: number; target: number; magnitudeMv: number; delayMs: number }
interface PlaybackSpike { id: number; neuronId: number; timeMs: number }
type ArrivalOrigin =
  | { kind: "initialization"; eventId: number }
  | { kind: "pacemaker"; eventId: number }
  | { kind: "stimulus"; eventId: number }
  | { kind: "synaptic"; spikeId: number; synapseId: number; source: number };
interface PlaybackArrival {
  sequence: number;
  target: number;
  timeMs: number;
  polarity: Polarity;
  magnitudeMv: number;
  origin: ArrivalOrigin;
  ignoredDuringRefractory: boolean;
}
interface PlaybackNeuronSample { neuronId: number; timeMs: number; membranePotentialMv: number; refractoryUntilMs: number | null }
interface PlaybackMetricSample { timeMs: number; spikeCount: number; activeNeuronCount: number }
interface PlaybackInFlight {
  spikeId: number;
  synapseId: number;
  source: number;
  target: number;
  sendTimeMs: number;
  arrivalTimeMs: number;
  polarity: Polarity;
}
interface PlaybackChunk {
  startMs: number;
  endMs: number;
  spikeEvents: PlaybackSpike[];
  arrivalEvents: PlaybackArrival[];
  neuronSamples: PlaybackNeuronSample[];
  metricSamples: PlaybackMetricSample[];
  inFlightIntervals: PlaybackInFlight[];
}
interface FlatPlayback {
  spikes: PlaybackSpike[];
  arrivals: PlaybackArrival[];
  samples: PlaybackNeuronSample[];
  metrics: PlaybackMetricSample[];
  flights: PlaybackInFlight[];
}

const element = <T extends HTMLElement>(id: string): T => {
  const value = document.getElementById(id);
  if (!value) throw new Error(`missing element #${id}`);
  return value as T;
};

const canvas = element<HTMLCanvasElement>("viewport");
const branch = element<HTMLSelectElement>("branch");
const timeline = element<HTMLInputElement>("timeline");
const playButton = element<HTMLButtonElement>("play");
const prevButton = element<HTMLButtonElement>("prev");
const nextButton = element<HTMLButtonElement>("next");
const speedSelect = element<HTMLSelectElement>("speed");
const timeLabel = element<HTMLElement>("time");
const digest = element<HTMLElement>("digest");
const perf = element<HTMLElement>("perf");
const comparison = element<HTMLElement>("comparison");
const loading = element<HTMLElement>("loading");
const search = element<HTMLInputElement>("search");
const searchGo = element<HTMLButtonElement>("search-go");
const backButton = element<HTMLButtonElement>("back");
const focusButton = element<HTMLButtonElement>("focus");
const upstreamButton = element<HTMLButtonElement>("upstream");
const downstreamButton = element<HTMLButtonElement>("downstream");
const isolate = element<HTMLInputElement>("isolate");
const neuronTitle = element<HTMLElement>("neuron-title");
const details = element<HTMLElement>("details");
const events = element<HTMLElement>("events");
const raster = element<HTMLCanvasElement>("raster");
const activity = element<HTMLCanvasElement>("activity");

const flatten = (dataset: PlaybackDataset): FlatPlayback => {
  const flights = new Map<string, PlaybackInFlight>();
  for (const flight of dataset.chunks.flatMap((chunk) => chunk.inFlightIntervals)) {
    flights.set(`${flight.spikeId}:${flight.synapseId}`, flight);
  }
  return {
    spikes: dataset.chunks.flatMap((chunk) => chunk.spikeEvents).sort((a, b) => a.timeMs - b.timeMs),
    arrivals: dataset.chunks.flatMap((chunk) => chunk.arrivalEvents).sort((a, b) => a.timeMs - b.timeMs),
    samples: dataset.chunks.flatMap((chunk) => chunk.neuronSamples).sort((a, b) => a.timeMs - b.timeMs),
    metrics: dataset.chunks.flatMap((chunk) => chunk.metricSamples).sort((a, b) => a.timeMs - b.timeMs),
    flights: [...flights.values()].sort((a, b) => a.sendTimeMs - b.sendTimeMs),
  };
};

const potentialAt = (neuron: PlaybackNeuron, sample: PlaybackNeuronSample | undefined, timeMs: number): number => {
  if (!sample) return neuron.restPotentialMv;
  if (sample.refractoryUntilMs !== null && timeMs < sample.refractoryUntilMs) {
    return sample.membranePotentialMv;
  }
  const decayStart = Math.max(sample.timeMs, sample.refractoryUntilMs ?? sample.timeMs);
  const elapsed = Math.max(0, timeMs - decayStart);
  const decay = Math.exp(-elapsed / neuron.membraneTimeConstantMs);
  return neuron.restPotentialMv + (sample.membranePotentialMv - neuron.restPotentialMv) * decay;
};

class NeuralScene {
  private readonly renderer = new THREE.WebGLRenderer({ canvas, antialias: true, alpha: true, powerPreference: "high-performance" });
  private readonly scene = new THREE.Scene();
  private readonly camera = new THREE.PerspectiveCamera(55, 1, 0.05, 500);
  private readonly raycaster = new THREE.Raycaster();
  private readonly pointer = new THREE.Vector2();
  private readonly keys = new Set<string>();
  private readonly baseMatrix = new THREE.Matrix4();
  private readonly workMatrix = new THREE.Matrix4();
  private readonly workColor = new THREE.Color();
  private readonly excitatory = new THREE.Color(0xffa44d);
  private readonly inhibitory = new THREE.Color(0x49c9f2);
  private readonly hot = new THREE.Color(0xffffff);
  private readonly comparisonColor = new THREE.Color(0xc492ff);
  private neurons?: THREE.InstancedMesh;
  private pacemakerRings?: THREE.InstancedMesh;
  private pulses?: THREE.InstancedMesh;
  private connectionLines?: THREE.LineSegments;
  private dataset?: PlaybackDataset;
  private flat?: FlatPlayback;
  private samplesByNeuron = new Map<number, PlaybackNeuronSample[]>();
  private spikesByNeuron = new Map<number, PlaybackSpike[]>();
  private indexById = new Map<number, number>();
  private selectedId: number | null = null;
  private isolated = false;
  private comparisonIds = new Set<number>();
  private comparisonStartMs = Number.POSITIVE_INFINITY;
  private yaw = -Math.PI * 0.72;
  private pitch = -0.34;
  private dragging = false;
  private dragged = false;
  private lastPointer = { x: 0, y: 0 };
  private frameCounter = 0;
  private frameTime = 0;
  private currentFps = 0;
  readonly cameraHistory: Array<{ position: THREE.Vector3; yaw: number; pitch: number }> = [];
  onSelect: (id: number) => void = () => undefined;

  constructor() {
    this.camera.up.set(0, 0, 1);
    this.camera.position.set(18, 18, 13);
    this.scene.add(new THREE.HemisphereLight(0x8ad9ff, 0x071015, 1.35));
    const light = new THREE.DirectionalLight(0xffffff, 1.6);
    light.position.set(8, -5, 13);
    this.scene.add(light);
    const grid = new THREE.GridHelper(36, 18, 0x244652, 0x142c35);
    grid.rotation.x = Math.PI / 2;
    grid.position.z = -8;
    this.scene.add(grid);
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    this.renderer.setClearColor(0x000000, 0);
    new ResizeObserver(() => this.resize()).observe(canvas.parentElement!);
    this.resize();
    this.bindControls();
  }

  setDataset(dataset: PlaybackDataset, flat: FlatPlayback): void {
    this.dataset = dataset;
    this.flat = flat;
    this.samplesByNeuron.clear();
    this.spikesByNeuron.clear();
    this.indexById.clear();
    dataset.neurons.forEach((neuron, index) => this.indexById.set(neuron.id, index));
    for (const sample of flat.samples) {
      const list = this.samplesByNeuron.get(sample.neuronId) ?? [];
      list.push(sample);
      this.samplesByNeuron.set(sample.neuronId, list);
    }
    for (const spike of flat.spikes) {
      const list = this.spikesByNeuron.get(spike.neuronId) ?? [];
      list.push(spike);
      this.spikesByNeuron.set(spike.neuronId, list);
    }
    this.buildNeurons();
    this.buildPacemakerRings();
    this.buildPulses();
    this.rebuildConnections();
  }

  setSelected(id: number | null): void {
    this.selectedId = id;
    this.rebuildConnections();
  }

  setIsolated(value: boolean): void { this.isolated = value; }

  setComparison(ids: Set<number>, startMs: number): void {
    this.comparisonIds = ids;
    this.comparisonStartMs = startMs;
  }

  focusNeuron(id: number, remember = true): void {
    if (!this.dataset) return;
    const neuron = this.dataset.neurons.find((item) => item.id === id);
    if (!neuron) return;
    if (remember) {
      this.cameraHistory.push({ position: this.camera.position.clone(), yaw: this.yaw, pitch: this.pitch });
    }
    const target = this.worldPosition(neuron.position);
    const forward = this.forwardVector();
    this.camera.position.copy(target).addScaledVector(forward, -6);
    this.updateCameraRotation();
  }

  restoreCamera(): boolean {
    const state = this.cameraHistory.pop();
    if (!state) return false;
    this.camera.position.copy(state.position);
    this.yaw = state.yaw;
    this.pitch = state.pitch;
    this.updateCameraRotation();
    return true;
  }

  update(timeMs: number, deltaSeconds: number): void {
    this.updateMovement(deltaSeconds);
    this.updateNeuronInstances(timeMs);
    this.updatePulseInstances(timeMs);
    this.renderer.render(this.scene, this.camera);
    this.frameCounter++;
    this.frameTime += deltaSeconds;
    if (this.frameTime >= 0.5) {
      this.currentFps = this.frameCounter / this.frameTime;
      this.frameCounter = 0;
      this.frameTime = 0;
    }
  }

  performanceText(): string {
    const positionAttribute = this.connectionLines?.geometry.getAttribute("position");
    const visibleConnections = positionAttribute ? positionAttribute.count / 2 : 0;
    const activeFlights = this.pulses?.count ?? 0;
    return `${this.currentFps.toFixed(0)} FPS · ${this.dataset?.neurons.length ?? 0} nodes · ${visibleConnections} links · ${activeFlights} pulses`;
  }

  private bindControls(): void {
    window.addEventListener("keydown", (event) => {
      if (event.target instanceof HTMLInputElement || event.target instanceof HTMLSelectElement) return;
      this.keys.add(event.key.toLowerCase());
    });
    window.addEventListener("keyup", (event) => this.keys.delete(event.key.toLowerCase()));
    window.addEventListener("blur", () => this.keys.clear());
    canvas.addEventListener("pointerdown", (event) => {
      this.dragging = true;
      this.dragged = false;
      this.lastPointer = { x: event.clientX, y: event.clientY };
      canvas.setPointerCapture(event.pointerId);
      canvas.classList.add("dragging");
    });
    canvas.addEventListener("pointermove", (event) => {
      if (!this.dragging) return;
      const dx = event.clientX - this.lastPointer.x;
      const dy = event.clientY - this.lastPointer.y;
      if (Math.abs(dx) + Math.abs(dy) > 1) this.dragged = true;
      this.yaw -= dx * 0.004;
      this.pitch = THREE.MathUtils.clamp(this.pitch - dy * 0.004, -1.45, 1.45);
      this.lastPointer = { x: event.clientX, y: event.clientY };
      this.updateCameraRotation();
    });
    canvas.addEventListener("pointerup", (event) => {
      this.dragging = false;
      canvas.classList.remove("dragging");
      if (!this.dragged) this.pick(event);
    });
    canvas.addEventListener("wheel", (event) => {
      event.preventDefault();
      this.camera.position.addScaledVector(this.forwardVector(), event.deltaY * 0.008);
    }, { passive: false });
  }

  private resize(): void {
    const width = canvas.clientWidth;
    const height = canvas.clientHeight;
    if (!width || !height) return;
    this.camera.aspect = width / height;
    this.camera.updateProjectionMatrix();
    this.renderer.setSize(width, height, false);
  }

  private buildNeurons(): void {
    if (this.neurons) {
      this.scene.remove(this.neurons);
      this.neurons.geometry.dispose();
      (this.neurons.material as THREE.Material).dispose();
    }
    const count = this.dataset!.neurons.length;
    this.neurons = new THREE.InstancedMesh(
      new THREE.IcosahedronGeometry(0.32, 2),
      new THREE.MeshStandardMaterial({ roughness: 0.42, metalness: 0.05, vertexColors: true, emissive: 0x0b1a20, emissiveIntensity: 0.6 }),
      count,
    );
    this.neurons.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    this.neurons.instanceColor = new THREE.InstancedBufferAttribute(new Float32Array(count * 3), 3);
    this.scene.add(this.neurons);
  }

  private buildPacemakerRings(): void {
    if (this.pacemakerRings) {
      this.scene.remove(this.pacemakerRings);
      this.pacemakerRings.geometry.dispose();
      (this.pacemakerRings.material as THREE.Material).dispose();
    }
    const ids = this.dataset!.pacemakerNeuronIds;
    this.pacemakerRings = new THREE.InstancedMesh(
      new THREE.TorusGeometry(0.52, 0.035, 8, 28),
      new THREE.MeshBasicMaterial({ color: 0xf4df70, transparent: true, opacity: 0.9 }),
      ids.length,
    );
    ids.forEach((id, index) => {
      const neuron = this.dataset!.neurons[this.indexById.get(id)!];
      this.baseMatrix.makeTranslation(...this.worldPosition(neuron.position).toArray());
      this.pacemakerRings!.setMatrixAt(index, this.baseMatrix);
    });
    this.scene.add(this.pacemakerRings);
  }

  private buildPulses(): void {
    if (this.pulses) {
      this.scene.remove(this.pulses);
      this.pulses.geometry.dispose();
      (this.pulses.material as THREE.Material).dispose();
    }
    const capacity = Math.min(Math.max(this.flat!.flights.length, 1), 1024);
    this.pulses = new THREE.InstancedMesh(
      new THREE.SphereGeometry(0.105, 8, 8),
      new THREE.MeshBasicMaterial({ color: 0xffffff, vertexColors: true }),
      capacity,
    );
    this.pulses.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    this.pulses.instanceColor = new THREE.InstancedBufferAttribute(new Float32Array(capacity * 3), 3);
    this.pulses.count = 0;
    this.scene.add(this.pulses);
  }

  private rebuildConnections(): void {
    if (this.connectionLines) {
      this.scene.remove(this.connectionLines);
      this.connectionLines.geometry.dispose();
      (this.connectionLines.material as THREE.Material).dispose();
      this.connectionLines = undefined;
    }
    if (!this.dataset || this.selectedId === null) return;
    const related = this.dataset.synapses.filter((synapse) => synapse.source === this.selectedId || synapse.target === this.selectedId);
    const positions: number[] = [];
    const colors: number[] = [];
    for (const synapse of related) {
      const source = this.dataset.neurons[this.indexById.get(synapse.source)!];
      const target = this.dataset.neurons[this.indexById.get(synapse.target)!];
      positions.push(...this.worldPosition(source.position).toArray(), ...this.worldPosition(target.position).toArray());
      const color = source.polarity === "excitatory" ? this.excitatory : this.inhibitory;
      colors.push(color.r, color.g, color.b, color.r, color.g, color.b);
    }
    const geometry = new THREE.BufferGeometry();
    geometry.setAttribute("position", new THREE.Float32BufferAttribute(positions, 3));
    geometry.setAttribute("color", new THREE.Float32BufferAttribute(colors, 3));
    this.connectionLines = new THREE.LineSegments(
      geometry,
      new THREE.LineBasicMaterial({ vertexColors: true, transparent: true, opacity: 0.34, depthWrite: false }),
    );
    this.scene.add(this.connectionLines);
  }

  private updateNeuronInstances(timeMs: number): void {
    if (!this.neurons || !this.dataset) return;
    const visible = this.visibleNeuronIds();
    for (let index = 0; index < this.dataset.neurons.length; index++) {
      const neuron = this.dataset.neurons[index];
      const sample = this.lastAtOrBefore(this.samplesByNeuron.get(neuron.id) ?? [], timeMs, (item) => item.timeMs);
      const potential = potentialAt(neuron, sample, timeMs);
      const activation = THREE.MathUtils.clamp((potential - neuron.restPotentialMv) / (neuron.thresholdMv - neuron.restPotentialMv), 0, 1);
      const latestSpike = this.lastAtOrBefore(this.spikesByNeuron.get(neuron.id) ?? [], timeMs, (item) => item.timeMs);
      const flash = latestSpike ? Math.max(0, 1 - (timeMs - latestSpike.timeMs) / 36) : 0;
      const base = neuron.polarity === "excitatory" ? this.excitatory : this.inhibitory;
      this.workColor.copy(base).multiplyScalar(0.26 + activation * 0.74).lerp(this.hot, flash);
      if (timeMs >= this.comparisonStartMs && this.comparisonIds.has(neuron.id)) {
        this.workColor.lerp(this.comparisonColor, 0.42);
      }
      this.neurons.setColorAt(index, this.workColor);
      const selectedScale = this.selectedId === neuron.id ? 1.65 : 1;
      const scale = visible.has(neuron.id) ? selectedScale : 0;
      this.workMatrix.compose(
        this.worldPosition(neuron.position),
        new THREE.Quaternion(),
        new THREE.Vector3(scale, scale, scale),
      );
      this.neurons.setMatrixAt(index, this.workMatrix);
    }
    this.neurons.instanceMatrix.needsUpdate = true;
    if (this.neurons.instanceColor) this.neurons.instanceColor.needsUpdate = true;
  }

  private updatePulseInstances(timeMs: number): void {
    if (!this.pulses || !this.flat || !this.dataset) return;
    let index = 0;
    for (const flight of this.flat.flights) {
      if (index >= 1024) break;
      if (timeMs < flight.sendTimeMs || timeMs > flight.arrivalTimeMs) continue;
      const source = this.dataset.neurons[this.indexById.get(flight.source)!];
      const target = this.dataset.neurons[this.indexById.get(flight.target)!];
      const progress = (timeMs - flight.sendTimeMs) / (flight.arrivalTimeMs - flight.sendTimeMs);
      const position = this.worldPosition(source.position).lerp(this.worldPosition(target.position), progress);
      this.workMatrix.makeTranslation(...position.toArray());
      this.pulses.setMatrixAt(index, this.workMatrix);
      this.pulses.setColorAt(index, flight.polarity === "excitatory" ? this.excitatory : this.inhibitory);
      index++;
    }
    this.pulses.count = index;
    this.pulses.instanceMatrix.needsUpdate = true;
    if (this.pulses.instanceColor) this.pulses.instanceColor.needsUpdate = true;
  }

  private visibleNeuronIds(): Set<number> {
    if (!this.dataset || !this.isolated || this.selectedId === null) {
      return new Set(this.dataset?.neurons.map((neuron) => neuron.id) ?? []);
    }
    const ids = new Set<number>([this.selectedId]);
    for (const synapse of this.dataset.synapses) {
      if (synapse.source === this.selectedId) ids.add(synapse.target);
      if (synapse.target === this.selectedId) ids.add(synapse.source);
    }
    return ids;
  }

  private pick(event: PointerEvent): void {
    if (!this.neurons || !this.dataset) return;
    const rect = canvas.getBoundingClientRect();
    this.pointer.set(((event.clientX - rect.left) / rect.width) * 2 - 1, -((event.clientY - rect.top) / rect.height) * 2 + 1);
    this.raycaster.setFromCamera(this.pointer, this.camera);
    const hit = this.raycaster.intersectObject(this.neurons, false)[0];
    if (hit?.instanceId === undefined) return;
    const id = this.dataset.neurons[hit.instanceId].id;
    this.onSelect(id);
  }

  private updateMovement(deltaSeconds: number): void {
    const speed = (this.keys.has("shift") ? 14 : 7) * deltaSeconds;
    const horizontalForward = new THREE.Vector3(Math.cos(this.yaw), Math.sin(this.yaw), 0);
    const right = new THREE.Vector3(-Math.sin(this.yaw), Math.cos(this.yaw), 0);
    if (this.keys.has("w")) this.camera.position.addScaledVector(horizontalForward, speed);
    if (this.keys.has("s")) this.camera.position.addScaledVector(horizontalForward, -speed);
    if (this.keys.has("a")) this.camera.position.addScaledVector(right, -speed);
    if (this.keys.has("d")) this.camera.position.addScaledVector(right, speed);
    if (this.keys.has("q")) this.camera.position.z -= speed;
    if (this.keys.has("e")) this.camera.position.z += speed;
    this.updateCameraRotation();
  }

  private updateCameraRotation(): void {
    const forward = this.forwardVector();
    this.camera.lookAt(this.camera.position.clone().add(forward));
  }

  private forwardVector(): THREE.Vector3 {
    const cosPitch = Math.cos(this.pitch);
    return new THREE.Vector3(cosPitch * Math.cos(this.yaw), cosPitch * Math.sin(this.yaw), Math.sin(this.pitch)).normalize();
  }

  private worldPosition(position: [number, number, number]): THREE.Vector3 {
    return new THREE.Vector3((position[0] - 0.5) * 22, (position[1] - 0.5) * 22, (position[2] - 0.5) * 16);
  }

  private lastAtOrBefore<T>(items: T[], time: number, getTime: (item: T) => number): T | undefined {
    let low = 0;
    let high = items.length - 1;
    let found: T | undefined;
    while (low <= high) {
      const middle = (low + high) >> 1;
      if (getTime(items[middle]) <= time) {
        found = items[middle];
        low = middle + 1;
      } else {
        high = middle - 1;
      }
    }
    return found;
  }
}

let compactBundle: CompactBundle;
let dataset: PlaybackDataset;
let flat: FlatPlayback;
let displayTime = 0;
let playing = false;
let selectedId: number | null = null;
let lastFrame = performance.now();
let lastUiUpdate = 0;
const neuralScene = new NeuralScene();

const setSelection = (id: number): void => {
  if (!dataset.neurons.some((neuron) => neuron.id === id)) return;
  selectedId = id;
  neuralScene.setSelected(id);
  isolate.disabled = false;
  focusButton.disabled = false;
  upstreamButton.disabled = !dataset.synapses.some((synapse) => synapse.target === id);
  downstreamButton.disabled = !dataset.synapses.some((synapse) => synapse.source === id);
  updateInspector();
};
neuralScene.onSelect = setSelection;

const setDataset = (index: number): void => {
  dataset = decodeDataset(compactBundle, compactBundle.branches[index]);
  flat = flatten(dataset);
  displayTime = dataset.startMs;
  playing = false;
  playButton.textContent = "▶";
  timeline.min = String(dataset.startMs);
  timeline.max = String(dataset.endMs);
  timeline.value = String(displayTime);
  digest.textContent = `digest ${dataset.eventDigest}`;
  neuralScene.setDataset(dataset, flat);
  updateComparison(index);
  if (selectedId !== null) neuralScene.setSelected(selectedId);
  drawCharts();
  updateUi();
};

const updateComparison = (index: number): void => {
  if (index !== 1) {
    comparison.hidden = true;
    neuralScene.setComparison(new Set(), Number.POSITIVE_INFINITY);
    return;
  }
  const control = flatten(decodeDataset(compactBundle, compactBundle.branches[0]));
  const counts = (spikes: PlaybackSpike[]): Map<string, number> => {
    const result = new Map<string, number>();
    for (const spike of spikes) {
      const key = `${Math.floor(spike.timeMs / 10)}:${spike.neuronId}`;
      result.set(key, (result.get(key) ?? 0) + 1);
    }
    return result;
  };
  const left = counts(control.spikes);
  const right = counts(flat.spikes);
  const keys = new Set([...left.keys(), ...right.keys()]);
  let firstBin = Number.POSITIVE_INFINITY;
  const affected = new Set<number>();
  for (const key of keys) {
    if ((left.get(key) ?? 0) === (right.get(key) ?? 0)) continue;
    const [bin, neuron] = key.split(":").map(Number);
    firstBin = Math.min(firstBin, bin);
    affected.add(neuron);
  }
  const firstMs = firstBin * 10;
  comparison.textContent = `对照首次分歧 ${firstMs.toFixed(0)} ms · ${affected.size} 个受影响神经元`;
  comparison.hidden = false;
  neuralScene.setComparison(affected, firstMs);
};

const updateInspector = (): void => {
  if (selectedId === null) return;
  const neuron = dataset.neurons.find((item) => item.id === selectedId)!;
  const samples = flat.samples.filter((sample) => sample.neuronId === selectedId && sample.timeMs <= displayTime);
  const sample = samples.at(-1);
  const neuronSpikes = flat.spikes.filter((spike) => spike.neuronId === selectedId && spike.timeMs <= displayTime);
  const recentArrivals = flat.arrivals.filter((arrival) => arrival.target === selectedId && arrival.timeMs <= displayTime).slice(-4).reverse();
  const recentFlights = flat.flights.filter((flight) => flight.target === selectedId && flight.arrivalTimeMs <= displayTime).slice(-4).reverse();
  const inputs = dataset.synapses.filter((synapse) => synapse.target === selectedId).length;
  const outputs = dataset.synapses.filter((synapse) => synapse.source === selectedId).length;
  const pacemaker = dataset.pacemakerNeuronIds.includes(selectedId);
  neuronTitle.textContent = `神经元 ${selectedId}${pacemaker ? " · 起搏源" : ""}`;
  details.innerHTML = [
    ["类型", neuron.polarity === "excitatory" ? "兴奋性" : "抑制性"],
    ["膜电位", `${potentialAt(neuron, sample, displayTime).toFixed(2)} mV`],
    ["阈值", `${neuron.thresholdMv.toFixed(2)} mV`],
    ["状态", sample?.refractoryUntilMs && sample.refractoryUntilMs > displayTime ? "绝对不应期" : "可响应"],
    ["最近放电", neuronSpikes.at(-1) ? `${neuronSpikes.at(-1)!.timeMs.toFixed(3)} ms` : "—"],
    ["连接", `${inputs} 入 / ${outputs} 出`],
    ["坐标", neuron.position.map((value) => value.toFixed(3)).join(", ")],
  ].map(([key, value]) => `<div><dt>${key}</dt><dd>${value}</dd></div>`).join("");
  const latestSpike = neuronSpikes.at(-1);
  const eventRows = [
    ...(latestSpike ? [`<div class="event"><b>放电</b> · ${latestSpike.timeMs.toFixed(3)} ms</div>`] : []),
    ...recentArrivals.map((arrival) => {
      const source = arrival.origin.kind === "synaptic" ? `神经元 ${arrival.origin.source}` : originLabel(arrival.origin.kind);
      const ignored = arrival.ignoredDuringRefractory ? " · 不应期忽略" : "";
      return `<div class="event"><b>${source}</b> · ${arrival.magnitudeMv.toFixed(2)} mV<br>${arrival.timeMs.toFixed(3)} ms${ignored}</div>`;
    }),
    ...recentFlights.map((flight) => {
      const magnitude = dataset.synapses.find((synapse) => synapse.id === flight.synapseId)?.magnitudeMv ?? 0;
      return `<div class="event"><b>神经元 ${flight.source}</b> · ${magnitude.toFixed(2)} mV<br>${flight.arrivalTimeMs.toFixed(3)} ms</div>`;
    }),
  ];
  events.innerHTML = eventRows.join("") || '<span class="muted">当前时间之前没有事件</span>';
};

const originLabel = (kind: ArrivalOrigin["kind"]): string => ({
  initialization: "初始化",
  pacemaker: "起搏器",
  stimulus: "局部刺激",
  synaptic: "突触输入",
}[kind]);

const updateUi = (): void => {
  timeline.value = String(displayTime);
  timeLabel.textContent = `${displayTime.toFixed(3)} ms`;
  perf.textContent = neuralScene.performanceText();
  updateInspector();
  drawTimeMarkers();
};

const canvasContext = (target: HTMLCanvasElement): CanvasRenderingContext2D => {
  const dpr = Math.min(window.devicePixelRatio, 2);
  const width = Math.max(1, Math.floor(target.clientWidth * dpr));
  const height = Math.max(1, Math.floor(target.clientHeight * dpr));
  if (target.width !== width || target.height !== height) {
    target.width = width;
    target.height = height;
  }
  return target.getContext("2d")!;
};

const drawCharts = (): void => {
  const rasterContext = canvasContext(raster);
  const activityContext = canvasContext(activity);
  const width = raster.width;
  const height = raster.height;
  rasterContext.clearRect(0, 0, width, height);
  for (const spike of flat.spikes) {
    const x = (spike.timeMs - dataset.startMs) / (dataset.endMs - dataset.startMs) * width;
    const y = (spike.neuronId + 0.5) / dataset.neurons.length * height;
    const neuron = dataset.neurons[spike.neuronId];
    rasterContext.fillStyle = neuron?.polarity === "inhibitory" ? "#4acbf0" : "#f6a24e";
    rasterContext.fillRect(x, y, Math.max(1, window.devicePixelRatio), Math.max(1, window.devicePixelRatio));
  }
  activityContext.clearRect(0, 0, activity.width, activity.height);
  const maxSpikes = Math.max(1, ...flat.metrics.map((metric) => metric.spikeCount));
  activityContext.beginPath();
  flat.metrics.forEach((metric, index) => {
    const x = (metric.timeMs - dataset.startMs) / (dataset.endMs - dataset.startMs) * activity.width;
    const y = activity.height - (metric.spikeCount / maxSpikes) * (activity.height - 8) - 3;
    if (index === 0) activityContext.moveTo(x, y); else activityContext.lineTo(x, y);
  });
  activityContext.strokeStyle = "#69d4ff";
  activityContext.lineWidth = Math.max(1, window.devicePixelRatio);
  activityContext.stroke();
};

const drawTimeMarkers = (): void => {
  drawCharts();
  const fraction = (displayTime - dataset.startMs) / (dataset.endMs - dataset.startMs);
  for (const target of [raster, activity]) {
    const context = target.getContext("2d")!;
    const x = fraction * target.width;
    context.strokeStyle = "rgba(255,255,255,.72)";
    context.lineWidth = Math.max(1, window.devicePixelRatio);
    context.beginPath();
    context.moveTo(x, 0);
    context.lineTo(x, target.height);
    context.stroke();
  }
};

const jumpSpike = (direction: -1 | 1): void => {
  const relevant = selectedId === null ? flat.spikes : flat.spikes.filter((spike) => spike.neuronId === selectedId);
  const target = direction > 0
    ? relevant.find((spike) => spike.timeMs > displayTime + 0.0001)
    : [...relevant].reverse().find((spike) => spike.timeMs < displayTime - 0.0001);
  if (target) displayTime = target.timeMs;
  playing = false;
  playButton.textContent = "▶";
  updateUi();
};

playButton.addEventListener("click", () => {
  if (displayTime >= dataset.endMs) displayTime = dataset.startMs;
  playing = !playing;
  playButton.textContent = playing ? "❚❚" : "▶";
});
prevButton.addEventListener("click", () => jumpSpike(-1));
nextButton.addEventListener("click", () => jumpSpike(1));
timeline.addEventListener("input", () => {
  displayTime = Number(timeline.value);
  playing = false;
  playButton.textContent = "▶";
  updateUi();
});
branch.addEventListener("change", () => setDataset(Number(branch.value)));
searchGo.addEventListener("click", () => {
  const id = Number(search.value);
  if (Number.isInteger(id) && dataset.neurons.some((neuron) => neuron.id === id)) {
    setSelection(id);
    neuralScene.focusNeuron(id);
    backButton.disabled = false;
  }
});
search.addEventListener("keydown", (event) => { if (event.key === "Enter") searchGo.click(); });
focusButton.addEventListener("click", () => {
  if (selectedId !== null) {
    neuralScene.focusNeuron(selectedId);
    backButton.disabled = false;
  }
});
backButton.addEventListener("click", () => { neuralScene.restoreCamera(); backButton.disabled = neuralScene.cameraHistory.length === 0; });
isolate.addEventListener("change", () => neuralScene.setIsolated(isolate.checked));
upstreamButton.addEventListener("click", () => {
  const synapse = dataset.synapses.find((item) => item.target === selectedId);
  if (synapse) { setSelection(synapse.source); neuralScene.focusNeuron(synapse.source); backButton.disabled = false; }
});
downstreamButton.addEventListener("click", () => {
  const synapse = dataset.synapses.find((item) => item.source === selectedId);
  if (synapse) { setSelection(synapse.target); neuralScene.focusNeuron(synapse.target); backButton.disabled = false; }
});
window.addEventListener("resize", drawCharts);

const animate = (now: number): void => {
  const delta = Math.min((now - lastFrame) / 1000, 0.05);
  lastFrame = now;
  if (playing) {
    displayTime += delta * 1000 * Number(speedSelect.value);
    if (displayTime >= dataset.endMs) {
      displayTime = dataset.endMs;
      playing = false;
      playButton.textContent = "▶";
    }
  }
  neuralScene.update(displayTime, delta);
  if (now - lastUiUpdate > 80) {
    updateUi();
    lastUiUpdate = now;
  }
  requestAnimationFrame(animate);
};

const start = async (): Promise<void> => {
  const response = await fetch("./experiment-001-v1.json");
  if (!response.ok) throw new Error(`failed to load experiment: ${response.status}`);
  compactBundle = await response.json() as CompactBundle;
  if (compactBundle.version !== 1 || compactBundle.branches.length === 0) throw new Error("unsupported or empty playback bundle");
  compactBundle.branches.forEach((item, index) => {
    const option = document.createElement("option");
    option.value = String(index);
    option.textContent = item.label;
    branch.append(option);
  });
  setDataset(0);
  loading.classList.add("hidden");
  requestAnimationFrame((now) => { lastFrame = now; animate(now); });
};

const decodeDataset = (bundleValue: CompactBundle, value: CompactBranch): PlaybackDataset => {
  const neurons = bundleValue.topology.neurons.map((item): PlaybackNeuron => ({
    id: item[0],
    polarity: item[1] === 1 ? "inhibitory" : "excitatory",
    position: [item[2], item[3], item[4]],
    restPotentialMv: item[5],
    resetPotentialMv: item[6],
    thresholdMv: item[7],
    refractoryPeriodMs: item[8],
    membraneTimeConstantMs: item[9],
  }));
  const synapses = bundleValue.topology.synapses.map((item): PlaybackSynapse => ({
    id: item[0], source: item[1], target: item[2], magnitudeMv: item[3], delayMs: item[4],
  }));
  const spikeEvents = value.spikes.map((item): PlaybackSpike => ({ id: item[0], neuronId: item[1], timeMs: item[2] }));
  const arrivalEvents = value.arrivals.map((item): PlaybackArrival => {
    const kind = item[5];
    let origin: ArrivalOrigin;
    if (kind === 0) origin = { kind: "initialization", eventId: item[6] };
    else if (kind === 1) origin = { kind: "pacemaker", eventId: item[6] };
    else if (kind === 2) origin = { kind: "stimulus", eventId: item[6] };
    else {
      const packed = item[7];
      origin = { kind: "synaptic", spikeId: item[6], synapseId: Math.floor(packed / 1000), source: packed % 1000 };
    }
    return {
      sequence: item[0], target: item[1], timeMs: item[2],
      polarity: item[3] === 1 ? "inhibitory" : "excitatory",
      magnitudeMv: item[4], origin, ignoredDuringRefractory: item[8] === 1,
    };
  });
  const neuronSamples = value.samples.map((item): PlaybackNeuronSample => ({
    neuronId: item[0], timeMs: item[1], membranePotentialMv: item[2], refractoryUntilMs: item[3] < 0 ? null : item[3],
  }));
  const metricSamples = value.metrics.map((item): PlaybackMetricSample => ({
    timeMs: item[0], spikeCount: item[1], activeNeuronCount: item[2],
  }));
  const inFlightIntervals = value.flights.map((item): PlaybackInFlight => ({
    spikeId: item[0], synapseId: item[1], source: item[2], target: item[3], sendTimeMs: item[4], arrivalTimeMs: item[5],
    polarity: item[6] === 1 ? "inhibitory" : "excitatory",
  }));
  return {
    version: 1, label: value.label, startMs: value.startMs, endMs: value.endMs,
    eventDigest: value.eventDigest, neurons, synapses, pacemakerNeuronIds: value.pacemakerNeuronIds,
    chunks: [{
      startMs: value.startMs, endMs: value.endMs, spikeEvents, arrivalEvents,
      neuronSamples, metricSamples, inFlightIntervals,
    }],
  };
};

start().catch((error: unknown) => {
  loading.textContent = error instanceof Error ? error.message : String(error);
});
