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
type GateFController = "fixed-internal" | "plastic-sensory" | "plastic-recurrent" | "plastic-recurrent-no-homeostasis";
type GateFPhase = "before-change" | "reversal" | "restoration";
interface GateFReport { controller: GateFController; phase: GateFPhase; rule: "original" | "reversed"; checkpointEpisode: number; metrics: { accuracy: Interval; leftTargetAccuracy: Interval; rightTargetAccuracy: Interval; rightChoiceFraction: Interval }; weights: { rmsChange: Interval; maximumAbsoluteWeight: Interval; finiteWeightFraction: Interval; meanRelativeGroupNormDrift: Interval }; modelSeedCount: number }
interface GateFTraceFrame { step: number; cueVisible: boolean; cueRight: boolean; hiddenActivity: number[] }
interface GateFTrace { label: string; controller: GateFController; phase: GateFPhase; rule: "original" | "reversed"; cueRight: boolean; targetRight: boolean; chosenRight: boolean; reward: number; actionProbabilities: number[]; frames: GateFTraceFrame[] }
interface GateFDataset { version: string; task: string; config: { modelSeedCount: number; pretrainingEpisodes: number; adaptationEpisodes: number; evaluationEpisodes: number; checkpoints: number[]; adaptationExploration: number }; reports: GateFReport[]; pairedEffects: { id: string; effect: { accuracy: Interval } }[]; adaptation: { controller: GateFController; reversalEpisodesTo75Percent: number | null; restorationEpisodesTo75Percent: number | null }[]; traces: GateFTrace[]; acceptance: Record<string, boolean> & { passed: boolean } }
type Map0Class = "rigid" | "learnable-stable" | "task-specialized" | "unstable";
type Map0Control = "baseline" | "reference-norm-homeostasis" | "frozen-plasticity" | "shuffled-structure" | "no-homeostasis" | "no-exploration" | "random-reward" | "additive-plasticity" | "no-resource-accounting" | "no-resource-supply" | "reset-between-trials";
type Map0Axis = "recurrentGain" | "internalLearningRate" | "homeostasisStrength" | "explorationRate";
interface Map0Parameters { id: number; recurrentGain: number; internalLearningRate: number; homeostasisStrength: number; explorationRate: number }
interface Map0Summary { parameters: Map0Parameters; control: Map0Control; seedCount: number; formationProbability: Interval; meanProbeScore: number; meanMemoryAccuracy: number; meanDelayedCreditAccuracy: number; meanReversalAccuracy: number; meanRestorationAccuracy: number; meanRepeatedReversalAccuracy: number; meanPerturbationRecoveryAccuracy: number; meanPerturbationInitialAccuracy: number; meanPerturbationDamageDrop: number; meanPerturbationRecoveryGain: number; meanSaturationFraction: number; meanSynchronyFraction: number; meanRelativeWeightDrift: number; meanResourceLevel?: number; dominantClass: Map0Class }
interface Map0Dataset { version: string; config: { developmentConfigCount: number; developmentSeedCount: number; confirmationSeedCount: number; recurrentGainRange: number[]; internalLearningRateRange: number[]; homeostasisStrengthRange: number[]; explorationRateRange: number[] }; developmentSummaries: Map0Summary[]; confirmationSummaries: Map0Summary[]; confirmationParameterIds: number[]; controlSummaries: { control: Map0Control; parameterId: number; formationProbability: Interval; meanProbeScore: number; formationProbabilityChangeFromBaseline: number; meanProbeScoreChangeFromBaseline: number }[]; stableRegionParameterIds: number[]; conclusions: string[]; acceptance: Record<string, boolean> & { passed: boolean; stableRegionFound: boolean } }
interface Map1Comparison { parameterId: number; dualTimescaleFormationProbability: Interval; referenceNormFormationProbability: Interval; formationProbabilityChange: number; meanProbeScoreChange: number; repeatedReversalAccuracyChange: number; perturbationRecoveryGainChange: number; meanWeightDriftChange: number }
interface Map1Dataset extends Omit<Map0Dataset,"config"> { config: Map0Dataset["config"] & { activityTarget: number; activityEmaRate: number; excitabilityAdjustmentRate: number; weightNormRelaxationRate: number; minimumExcitabilityGain: number; maximumExcitabilityGain: number }; referenceNormSummaries: Map0Summary[]; mechanismComparisons: Map1Comparison[]; acceptance: Record<string, boolean> & { passed: boolean; stableRegionFound: boolean; mechanismImprovesTradeoff: boolean } }
interface Map2ADataset { version: string; confirmationSummaries: Map0Summary[]; mechanismComparisons: { meanProbeScoreChange: number; repeatedReversalAccuracyChange: number; perturbationRecoveryGainChange: number; meanWeightDriftChange: number }[]; stableRegionParameterIds: number[]; conclusions: string[]; acceptance: Record<string, boolean> & { passed: boolean; stagePassed: boolean; stableRegionFound: boolean } }
interface Map2BDataset { version: string; confirmationSummaries: Map0Summary[]; noSupplySummaries: Map0Summary[]; mechanismComparisons: { meanProbeScoreChange: number; meanWeightDriftChange: number; meanResourceLevelChange: number }[]; stableRegionParameterIds: number[]; conclusions: string[]; acceptance: Record<string, boolean> & { passed: boolean; stagePassed: boolean; stableRegionFound: boolean } }
interface Map2CSummary { parameters: Map0Parameters; formationProbability: Interval; meanOverallAccuracy: number; meanPostChangeAccuracy: number; meanSettledAccuracy: number; meanRecoveryGain: number; meanResourceLevel: number; meanRelativeWeightDrift: number }
interface Map2CDataset { version: string; confirmationSummaries: Map2CSummary[]; resetSummaries: Map2CSummary[]; noSupplySummaries: Map2CSummary[]; frozenSummaries: Map2CSummary[]; stableRegionParameterIds: number[]; conclusions: string[]; acceptance: Record<string, boolean> & { passed: boolean; stagePassed: boolean; stableRegionFound: boolean } }
interface M0Dataset {
  version: string;
  carrierState: { version: string; hiddenUnitCount: number; sensoryChannelCount: number; actionCount: number; plasticRecurrentConnectionsPerUnit: number; adjustableVariables: { variable: string; readScope: string; updatePermission: string; valueRange: string; updateBudget: string; freezeMode: string }[]; forbiddenObservations: string[]; forbiddenWrites: string[] };
  adjustmentMechanism: string;
  interfaceAudit: Record<string, boolean>;
  frozenMap2Artifacts: { file: string; sha256: string }[];
  capabilityResult: { map2cContinuousAccuracy: number; resetControlAccuracy: number; noSupplyAccuracy: number; frozenPlasticityAccuracy: number; stableRegionParameterIds: number[]; interpretation: string };
}
interface M1RetentionSummary { parameters: Map0Parameters; meanDepartureAAccuracy: number; meanReturnAInitialAccuracy: number; meanReturnAFinalAccuracy: number; meanRetentionDrop: number; meanNovelRuleFinalAccuracy: number; meanMinimumSingleRuleFinalAccuracy: number; meanResourceLevel: number; meanRelativeWeightDrift: number; retentionProbability: Interval; failureClass: string }
interface M1RetentionDataset {
  version: string; sequence: string[];
  confirmationSeedResults: { phaseResults: { phaseIndex: number; rule: string; initialAccuracy: number; departureAccuracy: number; trialsToThreshold: number | null }[] }[];
  confirmationSummaries: M1RetentionSummary[]; frozenSummaries: M1RetentionSummary[]; randomConsequenceSummaries: M1RetentionSummary[]; resetSummaries: M1RetentionSummary[]; singleRuleSummaries: M1RetentionSummary[];
  dominantFailureClass: string; acceptance: Record<string,boolean> & { passed:boolean; retentionBoundaryPassed:boolean }; conclusions: string[];
}
interface M2CStructureSummary { control:string; runCount:number; meanMinimumSingleRuleAccuracy:number; meanNovelRuleAccuracy:number; meanReturnAInitialAccuracy:number; meanRetentionDrop:number; meanResourceLevel:number; meanRelativeWeightDrift:number; meanSequenceRewireCount:number; connectionBudgetPreservedFraction:number; readoutFrozenFraction:number; finiteFraction:number }
interface M2CStructureDataset {
  version:string; selectedMechanismParameters:{id:number;rewiringInterval:number;evidenceDecay:number};
  confirmationSummaries:M2CStructureSummary[]; pairedEffects:{metric:string;comparison:string;interval:Interval}[];
  decision:string; acceptance:Record<string,boolean>&{passed:boolean;stagePassed:boolean;mechanismAccepted:boolean}; conclusions:string[];
}
interface StructuralDiagnosticSummary { seedRunCount:number;checkpointCount:number;candidateEvaluationCount:number;meanSpearmanCorrelation:Interval;meanShuffledSpearmanCorrelation:Interval;meanCorrelationAdvantage:Interval;meanSelectedBenefit:Interval;meanRandomBenefit:Interval;meanSelectedBenefitAdvantage:Interval;meanOracleBenefit:Interval;meanSelectedRegret:Interval;meanTopQuartileHitRate:Interval;finiteFraction:number }
interface StructuralDiagnosticPhase { phaseIndex:number;rule:string;checkpointCount:number;meanSpearmanCorrelation:Interval;meanCorrelationAdvantage:Interval;meanSelectedBenefit:Interval;meanRandomBenefit:Interval;meanOracleBenefit:Interval;meanSelectedRegret:Interval;topQuartileHitRate:Interval;topQuartileChanceLevel:number }
interface StructuralDiagnosticDataset { version:string;confirmationSummary:StructuralDiagnosticSummary;confirmationPhaseSummaries:StructuralDiagnosticPhase[];decision:string;acceptance:Record<string,boolean>&{passed:boolean;stagePassed:boolean;evidenceInformative:boolean;usefulCounterfactualSwapsExist:boolean};conclusions:string[] }
interface StructuralTimescaleSummary extends StructuralDiagnosticSummary { trainingHorizon:number }
interface StructuralTimescalePhase extends StructuralDiagnosticPhase { trainingHorizon:number;seedRunCount:number }
interface StructuralTimescaleComparison { fromTrainingHorizon:number;toTrainingHorizon:number;oracleBenefitChange:Interval;selectedBenefitChange:Interval;correlationChange:Interval }
interface StructuralTimescaleDataset { version:string;confirmationHorizonSummaries:StructuralTimescaleSummary[];confirmationPhaseSummaries:StructuralTimescalePhase[];confirmationHorizonComparisons:StructuralTimescaleComparison[];decision:string;acceptance:Record<string,boolean>&{passed:boolean;stagePassed:boolean;horizon32ReplicatesV04:boolean;usefulLongHorizonSwapsExist:boolean;longHorizonEvidenceInformative:boolean};conclusions:string[] }
interface StructuralGroupSummary { edgeCount:number;seedRunCount:number;checkpointCount:number;bundleEvaluationCount:number;componentControlEvaluationCount:number;meanSpearmanCorrelation:Interval;meanShuffledSpearmanCorrelation:Interval;meanCorrelationAdvantage:Interval;meanSelectedBenefit:Interval;meanRandomBenefit:Interval;meanSelectedBenefitAdvantage:Interval;meanBudgetedOracleBenefit:Interval;meanSelectedRegret:Interval;meanTopQuartileHitRate:Interval;meanInteractionOverAdditive:Interval;meanSelectedInteractionOverAdditive:Interval;meanOracleInteractionOverAdditive:Interval;finiteFraction:number }
interface StructuralGroupPhase { edgeCount:number;phaseIndex:number;rule:string;seedRunCount:number;checkpointCount:number;meanSelectedBenefit:Interval;meanRandomBenefit:Interval;meanBudgetedOracleBenefit:Interval;meanSelectedBenefitAdvantage:Interval;meanOracleInteractionOverAdditive:Interval }
interface StructuralGroupDataset { version:string;confirmationSummaries:StructuralGroupSummary[];confirmationPhaseSummaries:StructuralGroupPhase[];confirmationComparisons:{edgeCount:number;oracleAdvantageOverSingle:Interval;selectedAdvantageOverSingle:Interval;randomMeanChangeFromSingle:Interval;oracleInteractionOverAdditive:Interval}[];confirmationCapabilityContrast:{edgeCount:number;meanHistoryPhaseOracleBenefit:Interval;meanNovelRuleOracleBenefit:Interval;historyAdvantageOverNovelRules:Interval};selectedGroupSize:number;decision:string;acceptance:Record<string,boolean>&{passed:boolean;stagePassed:boolean;novelRuleGroupedSwapsUseful:boolean;groupedEffectHistoryDominated:boolean};conclusions:string[] }
interface RepresentationRuleSummary { rule:string;seedRunCount:number;rawSensorProbeAccuracy:Interval;preBehaviorAccuracy:Interval;preHiddenProbeAccuracy:Interval;preShuffledProbeAccuracy:Interval;rewardLocalBehaviorAccuracy:Interval;rewardLocalHiddenProbeAccuracy:Interval;rewardLocalShuffledProbeAccuracy:Interval;rewardLocalProbeGainOverPre:Interval;rewardLocalReadoutRescueGap:Interval;targetDirectedBehaviorAccuracy:Interval;targetDirectedHiddenProbeAccuracy:Interval;targetDirectedShuffledProbeAccuracy:Interval;targetDirectedBehaviorGain:Interval;targetDirectedProbeGainOverPre:Interval;rewardLocalRelativeWeightDrift:Interval;targetDirectedRelativeWeightDrift:Interval;finiteFraction:number }
interface RepresentationNovelSummary { seedRunCount:number;rawSensorProbeAccuracy:Interval;preBehaviorAccuracy:Interval;preHiddenProbeAccuracy:Interval;preShuffledProbeAccuracy:Interval;rewardLocalBehaviorAccuracy:Interval;rewardLocalHiddenProbeAccuracy:Interval;rewardLocalShuffledProbeAccuracy:Interval;rewardLocalProbeGainOverPre:Interval;rewardLocalReadoutRescueGap:Interval;targetDirectedBehaviorAccuracy:Interval;targetDirectedHiddenProbeAccuracy:Interval;targetDirectedShuffledProbeAccuracy:Interval;targetDirectedBehaviorGain:Interval;targetDirectedProbeGainOverPre:Interval }
interface RepresentationCapacityDataset { version:string;confirmationSummaries:RepresentationRuleSummary[];confirmationNovelSummary:RepresentationNovelSummary;decision:string;acceptance:Record<string,boolean>&{passed:boolean;stagePassed:boolean;latentNovelCodeAccessible:boolean;rewardLocalRepresentationFormed:boolean;alternativeReadoutRescuesNovelRules:boolean;targetDirectedCreditRescuesNovelRules:boolean};conclusions:string[] }
interface M1FGainSummary { formationGain:number;runCount:number;meanHistoryFinalAccuracy:Interval;meanNovelFinalAccuracy:Interval;meanNovelTargetProbabilityGain:Interval;meanResourceLevel:Interval;meanRelativeWeightDrift:Interval;finiteFraction:number }
interface M1FControlSummary { control:string;formationGain:number;runCount:number;meanHistoryFinalAccuracy:Interval;meanNovelFinalAccuracy:Interval;meanNovelTargetProbabilityGain:Interval;meanResourceLevel:Interval;meanMinimumResourceLevel:Interval;meanRelativeWeightDrift:Interval;finiteFraction:number }
interface M1FDataset { version:string;developmentGainSummaries:M1FGainSummary[];selectedFormationGain:number;confirmationSummaries:M1FControlSummary[];pairedEffects:{candidateOverBaselineNovelAccuracy:Interval;candidateOverRandomNovelAccuracy:Interval;candidateHistoryAccuracyChange:Interval;candidateNovelTargetProbabilityGain:Interval};decision:string;acceptance:Record<string,boolean>&{passed:boolean;stagePassed:boolean;mechanismAccepted:boolean;consequenceSpecificityPassed:boolean;historyPreserved:boolean};conclusions:string[] }
interface M1XLearningRateSummary { learningRate:number;runCount:number;meanNovelBehaviorAccuracy:Interval;meanNovelArgmaxAccuracy:Interval;meanNovelTargetProbability:Interval;meanMinimumNovelAccuracy:Interval;meanTrainingLossReduction:Interval;meanSaturationFraction:Interval;finiteFraction:number }
interface M1XControlSummary { control:string;learningRate:number;runCount:number;meanHistoryBehaviorAccuracy:Interval;meanNovelBehaviorAccuracy:Interval;meanNovelArgmaxAccuracy:Interval;meanNovelTargetProbability:Interval;meanMinimumNovelAccuracy:Interval;meanTrainingLossReduction:Interval;meanRelativeWeightDrift:Interval;meanSaturationFraction:Interval;finiteFraction:number }
interface M1XDataset { version:string;developmentLearningRateSummaries:M1XLearningRateSummary[];selectedLearningRate:number;confirmationSummaries:M1XControlSummary[];pairedEffects:{absoluteOverFrozenBehavior:Interval;absoluteOverShuffledBehavior:Interval;absoluteOverFrozenTargetProbability:Interval;absoluteOverShuffledTargetProbability:Interval;absoluteOverEnvelopeBehavior:Interval};decision:string;acceptance:Record<string,boolean>&{passed:boolean;stagePassed:boolean;optimizerEffective:boolean;taskSpecificityPassed:boolean;envelopeReachabilityPassed:boolean;absoluteReachabilityPassed:boolean};conclusions:string[] }
interface M1XEScaleSummary { boundKind:string;normMultiplier:number|null;runCount:number;meanHistoryBehaviorAccuracy:Interval;meanNovelBehaviorAccuracy:Interval;meanNovelArgmaxAccuracy:Interval;meanNovelTargetProbability:Interval;meanMinimumNovelAccuracy:Interval;meanTrainingLossReduction:Interval;meanRelativeWeightDrift:Interval;meanSaturationFraction:Interval;finiteFraction:number;capabilityPassed:boolean }
interface M1XEScaleComparison { normMultiplier:number;novelBehaviorChangeFromOneX:Interval;targetProbabilityChangeFromOneX:Interval;minimumNovelAccuracyChangeFromOneX:Interval;historyAccuracyChangeFromOneX:Interval;saturationChangeFromOneX:Interval }
interface M1XEDataset { version:string;developmentSummaries:M1XEScaleSummary[];selectedDevelopmentMultiplier:number|null;confirmationSummaries:M1XEScaleSummary[];confirmedMinimumMultiplier:number|null;passingConfirmationMultipliers:number[];confirmationComparisons:M1XEScaleComparison[];decision:string;acceptance:Record<string,boolean>&{passed:boolean;stagePassed:boolean;absoluteAnchorPassed:boolean;developmentBoundaryConfirmed:boolean};conclusions:string[] }
interface M1NEControlSummary { control:string;normMultiplier:number;runCount:number;meanHistoryInitialAccuracy:Interval;meanHistoryFinalAccuracy:Interval;meanNovelInitialAccuracy:Interval;meanNovelFinalAccuracy:Interval;meanNovelFinalArgmaxAccuracy:Interval;meanNovelInitialTargetProbability:Interval;meanNovelFinalTargetProbability:Interval;meanNovelTargetProbabilityGain:Interval;meanMinimumNovelAccuracy:Interval;meanResourceLevel:Interval;meanMinimumResourceLevel:Interval;meanRelativeWeightDrift:Interval;meanSaturationFraction:Interval;finiteFraction:number;capabilityPassed:boolean }
interface M1NEPairedEffects { candidateOverOneXBehavior:Interval;candidateOverOneXTargetProbability:Interval;candidateOverFrozenBehavior:Interval;candidateOverFrozenTargetProbability:Interval;candidateOverRandomBehavior:Interval;candidateOverRandomTargetProbability:Interval;candidateHistoryChangeFromFrozen:Interval;oracleOverCandidateBehavior:Interval;oracleOverCandidateTargetProbability:Interval }
interface M1NEDataset { version:string;developmentSummaries:M1NEControlSummary[];developmentEffects:M1NEPairedEffects;confirmationSummaries:M1NEControlSummary[];confirmationEffects:M1NEPairedEffects;decision:string;acceptance:Record<string,boolean>&{passed:boolean;stagePassed:boolean;oracleAnchorPassed:boolean;candidateCapabilityPassed:boolean;learnedAdvantagePassed:boolean;consequenceSpecificityPassed:boolean;historyPreserved:boolean;onlineSufficiencyConfirmed:boolean};conclusions:string[] }
interface M1CDComponentSummary { component:string;cosineAlignment:Interval;signAgreement:Interval;normRatio:Interval;productiveProjection:Interval }
interface M1CDControlSummary { control:string;independentRunCount:number;components:M1CDComponentSummary[];rawToAppliedAttenuation:Interval;rawAppliedCosine:Interval;homeostasisProjectionLoss:Interval;finiteFraction:number }
interface M1CDCheckpointSummary { trial:number;trueTotalCosine:Interval;trueTotalProductiveProjection:Interval;trueAppliedProductiveProjection:Interval;trueHomeostasisProjectionLoss:Interval }
interface M1CDDataset { version:string;confirmationOnlineSummary:{independentRunCount:number;historyFinalBehavior:Interval;novelInitialBehavior:Interval;novelFinalBehavior:Interval;novelInitialTargetProbability:Interval;novelFinalTargetProbability:Interval};confirmationSummaries:M1CDControlSummary[];confirmationCheckpointSummaries:M1CDCheckpointSummary[];confirmationEffects:{trueOverRandomTotalCosine:Interval;trueOverRandomTotalProjection:Interval;trueOverShuffledTotalCosine:Interval;trueOverShuffledTotalProjection:Interval};decision:string;acceptance:Record<string,boolean>&{passed:boolean;stagePassed:boolean};conclusions:string[] }

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
const gateFCurveCanvas = byId<HTMLCanvasElement>("gate-f-curve");
const gateFTraceSelect = byId<HTMLSelectElement>("gate-f-trace-select");
const gateFRange = byId<HTMLInputElement>("gate-f-range");
const map0Canvas = byId<HTMLCanvasElement>("map0-phase");
const map0Stage = byId<HTMLSelectElement>("map0-stage");
const map0X = byId<HTMLSelectElement>("map0-x");
const map0Y = byId<HTMLSelectElement>("map0-y");
const map0Parameter = byId<HTMLSelectElement>("map0-parameter");
const map1Canvas = byId<HTMLCanvasElement>("map1-phase");
const map1X = byId<HTMLSelectElement>("map1-x");
const map1Y = byId<HTMLSelectElement>("map1-y");
const map1Parameter = byId<HTMLSelectElement>("map1-parameter");
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
let gateF: GateFDataset;
let gateFTrace: GateFTrace;
let gateFFrameIndex = 0;
let map0: Map0Dataset;
let map0SelectedId = 0;
let map1: Map1Dataset;
let map1SelectedId = 0;
let map2a: Map2ADataset;
let map2b: Map2BDataset;
let map2c: Map2CDataset;
let mechanismM0: M0Dataset;
let mechanismM1: M1RetentionDataset;
let mechanismM2C: M2CStructureDataset;
let structuralDiagnostic: StructuralDiagnosticDataset;
let structuralTimescale: StructuralTimescaleDataset;
let structuralGroup: StructuralGroupDataset;
let representationCapacity: RepresentationCapacityDataset;
let ruleFormation: M1FDataset;
let reachability: M1XDataset;
let reachabilityEnvelope: M1XEDataset;
let normEnabledSufficiency: M1NEDataset;
let creditDecomposition: M1CDDataset;
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
  "fixed-internal": "冻结内部", "plastic-sensory": "感觉可塑", "plastic-recurrent": "循环可塑", "plastic-recurrent-no-homeostasis": "循环可塑 · 无内稳态",
  "before-change": "变化前", reversal: "规则反转", restoration: "恢复原规则",
  "learnable-stable": "稳定可学习", "task-specialized": "任务特化", rigid: "僵硬", unstable: "不稳定",
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

const drawGateFCurve = () => {
  const ctx=fitCanvas(gateFCurveCanvas),rect=gateFCurveCanvas.getBoundingClientRect();ctx.clearRect(0,0,rect.width,rect.height);
  const pad={l:46,r:18,t:32,b:42},w=rect.width-pad.l-pad.r,h=rect.height-pad.t-pad.b;
  const points=[...gateF.config.checkpoints.map(episode=>({phase:"reversal" as GateFPhase,episode})),...gateF.config.checkpoints.map(episode=>({phase:"restoration" as GateFPhase,episode}))];
  const xOf=(index:number)=>pad.l+index/Math.max(1,points.length-1)*w,yOf=(value:number)=>pad.t+h-Math.max(0,Math.min(1,value))*h;
  ctx.fillStyle="#fff7e9";ctx.fillRect(pad.l,pad.t,xOf(4)-pad.l+10,h);ctx.fillStyle="#eef8f2";ctx.fillRect(xOf(5)-10,pad.t,pad.l+w-(xOf(5)-10),h);
  ctx.font="10px system-ui";ctx.textAlign="right";for(let i=0;i<=4;i++){const value=i/4,y=yOf(value);ctx.strokeStyle="#d9e3dc";ctx.beginPath();ctx.moveTo(pad.l,y);ctx.lineTo(pad.l+w,y);ctx.stroke();ctx.fillStyle="#718078";ctx.fillText(`${value*100}%`,pad.l-7,y+3);}
  ctx.setLineDash([5,4]);ctx.strokeStyle="#a38a47";ctx.beginPath();ctx.moveTo(pad.l,yOf(.75));ctx.lineTo(pad.l+w,yOf(.75));ctx.stroke();ctx.setLineDash([]);
  const colors:Record<GateFController,string>={"fixed-internal":"#87938d","plastic-sensory":"#268b65","plastic-recurrent":"#397da0","plastic-recurrent-no-homeostasis":"#d07b32"};
  for(const controller of Object.keys(colors) as GateFController[]){ctx.strokeStyle=colors[controller];ctx.lineWidth=2.4;ctx.beginPath();points.forEach((point,index)=>{const report=gateF.reports.find(item=>item.controller===controller&&item.phase===point.phase&&item.checkpointEpisode===point.episode)!;const x=xOf(index),y=yOf(report.metrics.accuracy.mean);index?ctx.lineTo(x,y):ctx.moveTo(x,y);});ctx.stroke();points.forEach((point,index)=>{const report=gateF.reports.find(item=>item.controller===controller&&item.phase===point.phase&&item.checkpointEpisode===point.episode)!;ctx.fillStyle=colors[controller];ctx.beginPath();ctx.arc(xOf(index),yOf(report.metrics.accuracy.mean),3.5,0,Math.PI*2);ctx.fill();});}
  ctx.fillStyle="#61736a";ctx.textAlign="center";points.forEach((point,index)=>ctx.fillText(String(point.episode),xOf(index),pad.t+h+17));ctx.font="600 10px system-ui";ctx.fillText("规则反转",(xOf(0)+xOf(4))/2,15);ctx.fillText("恢复原规则",(xOf(5)+xOf(9))/2,15);
};

const renderGateFFrame = () => {
  const frame=gateFTrace.frames[gateFFrameIndex];gateFRange.value=String(gateFFrameIndex);
  byId("gate-f-phase").textContent=labels[gateFTrace.phase];byId("gate-f-cue").textContent=frame.cueVisible?`线索 ${frame.cueRight?"右":"左"}`:"线索关闭";
  const neurons=byId("gate-f-neurons");neurons.replaceChildren();frame.hiddenActivity.forEach((value,index)=>{const cell=document.createElement("i");cell.title=`H${index}: ${value.toFixed(3)}`;cell.style.setProperty("--activity",String(Math.min(1,Math.abs(value))));cell.className=value>=0?"excited":"inhibited";neurons.append(cell);});
  const selected=gateFTrace.chosenRight?"右":"左",target=gateFTrace.targetRight?"右":"左";byId("gate-f-trace-note").textContent=`${labels[gateFTrace.controller]} · 目标 ${target} · 选择 ${selected} · 奖励 ${gateFTrace.reward>0?"+1":"−1"} · 右侧概率 ${(gateFTrace.actionProbabilities[1]*100).toFixed(1)}%`;
};
const selectGateFTrace = () => {gateFTrace=gateF.traces.find(trace=>trace.label===gateFTraceSelect.value)??gateF.traces[0];gateFFrameIndex=0;gateFRange.max=String(gateFTrace.frames.length-1);renderGateFFrame();};

const renderGateF = () => {
  byId("gate-f-protocol").textContent=`${gateF.config.modelSeedCount} 模型种子 · ${gateF.config.adaptationEpisodes} 回合/阶段 · ${(gateF.config.adaptationExploration*100).toFixed(0)}% 探索`;
  const acceptanceLabels:Record<string,string>={matchedStartAndPlasticBudgets:"相同起点与48槽位",originalRuleLearned:"原规则已学习",sensoryPlasticityAdapts:"感觉可塑性适应",recurrentPlasticityAdapts:"循环可塑性适应",restoredRuleRelearned:"规则恢复后重学",readoutFrozenAndStatesFinite:"读出冻结且状态有限",homeostasisControlComplete:"内稳态关闭对照完整",deterministic:"完全确定性"};const acceptance=byId("gate-f-acceptance");acceptance.replaceChildren();for(const [key,label] of Object.entries(acceptanceLabels)){const pass=gateF.acceptance[key];const item=document.createElement("div");item.className=pass?"pass":"fail";item.innerHTML=`<i>${pass?"✓":"×"}</i><span>${label}</span>`;acceptance.append(item);}
  const finalCards=byId("gate-f-comparison");finalCards.replaceChildren(...(["fixed-internal","plastic-sensory","plastic-recurrent","plastic-recurrent-no-homeostasis"] as GateFController[]).map(controller=>{const report=gateF.reports.find(item=>item.phase==="reversal"&&item.checkpointEpisode===gateF.config.adaptationEpisodes&&item.controller===controller)!;const timing=gateF.adaptation.find(item=>item.controller===controller)!;const card=document.createElement("article");if(controller==="plastic-sensory"||controller==="plastic-recurrent")card.className="memory";card.innerHTML=`<span>${labels[controller]}</span><strong>${(report.metrics.accuracy.mean*100).toFixed(1)}%</strong><small>反转 ${gateF.config.adaptationEpisodes} · 95% CI ${(report.metrics.accuracy.lower95*100).toFixed(1)}–${(report.metrics.accuracy.upper95*100).toFixed(1)}%</small><dl><div><dt>达到75%</dt><dd>${timing.reversalEpisodesTo75Percent??"未达到"}${timing.reversalEpisodesTo75Percent!==null?" 回合":""}</dd></div><div><dt>左目标</dt><dd>${(report.metrics.leftTargetAccuracy.mean*100).toFixed(1)}%</dd></div><div><dt>右目标</dt><dd>${(report.metrics.rightTargetAccuracy.mean*100).toFixed(1)}%</dd></div></dl>`;return card;}));
  const effects=byId("gate-f-effects");effects.replaceChildren(...gateF.pairedEffects.slice(0,2).map(effect=>{const value=effect.effect.accuracy,card=document.createElement("article");card.innerHTML=`<span>${effect.id.includes("sensory")?"感觉可塑 − 冻结内部":"循环可塑 − 冻结内部"}</span><strong>+${(value.mean*100).toFixed(1)} pp</strong><small>配对 95% CI ${(value.lower95*100).toFixed(1)}–${(value.upper95*100).toFixed(1)} pp</small>`;return card;}));
  const stability=byId("gate-f-stability");stability.replaceChildren(...(["plastic-sensory","plastic-recurrent","plastic-recurrent-no-homeostasis"] as GateFController[]).map(controller=>{const report=gateF.reports.find(item=>item.phase==="reversal"&&item.checkpointEpisode===gateF.config.adaptationEpisodes&&item.controller===controller)!;const drift=report.weights.meanRelativeGroupNormDrift.mean,row=document.createElement("div");row.innerHTML=`<span>${labels[controller]}</span><div><i style="width:${Math.min(100,drift*15)}%"></i></div><strong>${(drift*100).toFixed(0)}%</strong>`;return row;}));
  gateFTraceSelect.replaceChildren(...gateF.traces.map(trace=>{const option=document.createElement("option");option.value=trace.label;option.textContent=`${labels[trace.controller]} · ${labels[trace.phase]}`;if(trace.controller==="plastic-recurrent"&&trace.phase==="reversal")option.selected=true;return option;}));selectGateFTrace();drawGateFCurve();
};

const map0AxisLabels:Record<Map0Axis,string>={recurrentGain:"循环增益",internalLearningRate:"内部可塑率",homeostasisStrength:"内稳态强度",explorationRate:"探索率"};
const map0ClassColors:Record<Map0Class,string>={"learnable-stable":"#23845d","task-specialized":"#d49a35",rigid:"#88958e",unstable:"#cf5b53"};
const map0ControlLabels:Record<Map0Control,string>={baseline:"基线","reference-norm-homeostasis":"旧参考范数","frozen-plasticity":"冻结可塑性","shuffled-structure":"置乱结构","no-homeostasis":"关闭内稳态","no-exploration":"关闭探索","random-reward":"随机后果","additive-plasticity":"加性可塑性","no-resource-accounting":"关闭资源核算","no-resource-supply":"关闭供能","reset-between-trials":"逐试次重置"};
const map0Summaries=()=>map0Stage.value==="confirmation"?map0.confirmationSummaries:map0.developmentSummaries;
const map0Range=(axis:Map0Axis)=>({recurrentGain:map0.config.recurrentGainRange,internalLearningRate:map0.config.internalLearningRateRange,homeostasisStrength:map0.config.homeostasisStrengthRange,explorationRate:map0.config.explorationRateRange}[axis]);

const drawMap0 = () => {
  const summaries=map0Summaries(),xAxis=map0X.value as Map0Axis,yAxis=map0Y.value as Map0Axis,ctx=fitCanvas(map0Canvas),rect=map0Canvas.getBoundingClientRect();ctx.clearRect(0,0,rect.width,rect.height);
  const pad={l:58,r:22,t:22,b:46},w=rect.width-pad.l-pad.r,h=rect.height-pad.t-pad.b,xRange=map0Range(xAxis),yRange=map0Range(yAxis);
  const xOf=(value:number)=>pad.l+(value-xRange[0])/(xRange[1]-xRange[0])*w,yOf=(value:number)=>pad.t+h-(value-yRange[0])/(yRange[1]-yRange[0])*h;
  ctx.font="10px system-ui";ctx.fillStyle="#718078";ctx.strokeStyle="#dce5de";for(let i=0;i<=4;i++){const x=pad.l+i/4*w,y=pad.t+h-i/4*h;ctx.beginPath();ctx.moveTo(x,pad.t);ctx.lineTo(x,pad.t+h);ctx.stroke();ctx.beginPath();ctx.moveTo(pad.l,y);ctx.lineTo(pad.l+w,y);ctx.stroke();ctx.textAlign="center";ctx.fillText((xRange[0]+i/4*(xRange[1]-xRange[0])).toFixed(2),x,pad.t+h+18);ctx.textAlign="right";ctx.fillText((yRange[0]+i/4*(yRange[1]-yRange[0])).toFixed(2),pad.l-8,y+3);}
  for(const summary of summaries){const x=xOf(summary.parameters[xAxis]),y=yOf(summary.parameters[yAxis]),radius=5+summary.formationProbability.mean*10;ctx.fillStyle=map0ClassColors[summary.dominantClass];ctx.globalAlpha=.82;ctx.beginPath();ctx.arc(x,y,radius,0,Math.PI*2);ctx.fill();ctx.globalAlpha=1;if(summary.parameters.id===map0SelectedId){ctx.strokeStyle="#173e34";ctx.lineWidth=2.5;ctx.beginPath();ctx.arc(x,y,radius+4,0,Math.PI*2);ctx.stroke();}ctx.fillStyle="#334f46";ctx.textAlign="center";ctx.fillText(String(summary.parameters.id),x,y-radius-4);}
  ctx.fillStyle="#52675e";ctx.textAlign="center";ctx.fillText(map0AxisLabels[xAxis],pad.l+w/2,rect.height-7);ctx.save();ctx.translate(13,pad.t+h/2);ctx.rotate(-Math.PI/2);ctx.fillText(map0AxisLabels[yAxis],0,0);ctx.restore();
};

const map0Bar=(label:string,value:number,percent=true)=>{const row=document.createElement("div"),shown=percent?`${(value*100).toFixed(1)}%`:value.toFixed(3);row.innerHTML=`<span>${label}</span><div><i style="width:${Math.min(100,Math.max(0,value*100))}%"></i></div><strong>${shown}</strong>`;return row;};
const renderMap0Inspector=()=>{
  const summaries=map0Summaries();let summary=summaries.find(item=>item.parameters.id===map0SelectedId);if(!summary){summary=summaries[0];map0SelectedId=summary.parameters.id;}
  map0Parameter.value=String(map0SelectedId);const p=summary.parameters;byId("map0-parameters").innerHTML=`<div><span>形成概率</span><strong>${(summary.formationProbability.mean*100).toFixed(1)}%</strong></div><div><span>分类</span><strong>${labels[summary.dominantClass]??summary.dominantClass}</strong></div><div><span>循环增益</span><strong>${p.recurrentGain.toFixed(3)}</strong></div><div><span>可塑率</span><strong>${p.internalLearningRate.toFixed(3)}</strong></div><div><span>内稳态</span><strong>${p.homeostasisStrength.toFixed(3)}</strong></div><div><span>探索率</span><strong>${p.explorationRate.toFixed(3)}</strong></div>`;
  byId("map0-probes").replaceChildren(map0Bar("短期记忆",summary.meanMemoryAccuracy),map0Bar("延迟信用",summary.meanDelayedCreditAccuracy),map0Bar("规则反转",summary.meanReversalAccuracy),map0Bar("恢复旧规则",summary.meanRestorationAccuracy),map0Bar("重复反转",summary.meanRepeatedReversalAccuracy),map0Bar("损伤后正确率",summary.meanPerturbationInitialAccuracy),map0Bar("损伤幅度",summary.meanPerturbationDamageDrop),map0Bar("恢复增益",summary.meanPerturbationRecoveryGain));
  byId("map0-dynamics").replaceChildren(map0Bar("饱和比例",summary.meanSaturationFraction),map0Bar("同步比例",summary.meanSynchronyFraction),map0Bar("权重漂移",Math.min(1,summary.meanRelativeWeightDrift),false));
  const controls=map0.controlSummaries.filter(item=>item.parameterId===map0SelectedId);byId("map0-control-grid").replaceChildren(...controls.map(item=>{const card=document.createElement("article");card.innerHTML=`<span>${map0ControlLabels[item.control]}</span><strong>${(item.meanProbeScore*100).toFixed(0)}% 探针</strong><small>得分变化 ${item.meanProbeScoreChangeFromBaseline>=0?"+":""}${(item.meanProbeScoreChangeFromBaseline*100).toFixed(1)} pp · 形成 ${(item.formationProbability.mean*100).toFixed(1)}%</small>`;return card;}));drawMap0();
};
const renderMap0=()=>{
  byId("map0-status").textContent=map0.acceptance.stableRegionFound?`找到 ${map0.stableRegionParameterIds.length} 点候选区域`:"测绘完成 · 未找到稳定区域";
  const acceptanceLabels:Record<string,string>={deterministicSampling:"确定性采样",developmentAndConfirmationSeedsDisjoint:"种子严格隔离",allDevelopmentRunsComplete:"开发扫描完整",confirmationRunsComplete:"独立确认完整",causalControlsComplete:"因果对照完整",causalInterventionDetected:"检测到因果变化",finiteOutputs:"数值有限"};const grid=byId("map0-acceptance");grid.replaceChildren();for(const [key,label] of Object.entries(acceptanceLabels)){const pass=map0.acceptance[key];const item=document.createElement("div");item.className=pass?"pass":"fail";item.innerHTML=`<i>${pass?"✓":"×"}</i><span>${label}</span>`;grid.append(item);}
  map0Parameter.replaceChildren(...map0.developmentSummaries.map(summary=>{const option=document.createElement("option");option.value=String(summary.parameters.id);option.textContent=`配置 ${summary.parameters.id}`;return option;}));map0SelectedId=[...map0.developmentSummaries].sort((a,b)=>(b.formationProbability.mean*2+b.meanProbeScore)-(a.formationProbability.mean*2+a.meanProbeScore))[0].parameters.id;byId("map0-conclusions").replaceChildren(...map0.conclusions.map(text=>{const p=document.createElement("p");p.textContent=text;return p;}));renderMap0Inspector();
};

const map1Range=(axis:Map0Axis)=>({recurrentGain:map1.config.recurrentGainRange,internalLearningRate:map1.config.internalLearningRateRange,homeostasisStrength:map1.config.homeostasisStrengthRange,explorationRate:map1.config.explorationRateRange}[axis]);
const drawMap1=()=>{
  const summaries=map1.confirmationSummaries,xAxis=map1X.value as Map0Axis,yAxis=map1Y.value as Map0Axis,ctx=fitCanvas(map1Canvas),rect=map1Canvas.getBoundingClientRect();ctx.clearRect(0,0,rect.width,rect.height);
  const pad={l:58,r:22,t:22,b:46},w=rect.width-pad.l-pad.r,h=rect.height-pad.t-pad.b,xRange=map1Range(xAxis),yRange=map1Range(yAxis),xOf=(value:number)=>pad.l+(value-xRange[0])/(xRange[1]-xRange[0])*w,yOf=(value:number)=>pad.t+h-(value-yRange[0])/(yRange[1]-yRange[0])*h;
  ctx.font="10px system-ui";ctx.fillStyle="#718078";ctx.strokeStyle="#dce5de";for(let i=0;i<=4;i++){const x=pad.l+i/4*w,y=pad.t+h-i/4*h;ctx.beginPath();ctx.moveTo(x,pad.t);ctx.lineTo(x,pad.t+h);ctx.stroke();ctx.beginPath();ctx.moveTo(pad.l,y);ctx.lineTo(pad.l+w,y);ctx.stroke();ctx.textAlign="center";ctx.fillText((xRange[0]+i/4*(xRange[1]-xRange[0])).toFixed(2),x,pad.t+h+18);ctx.textAlign="right";ctx.fillText((yRange[0]+i/4*(yRange[1]-yRange[0])).toFixed(2),pad.l-8,y+3);}
  for(const summary of summaries){const x=xOf(summary.parameters[xAxis]),y=yOf(summary.parameters[yAxis]),radius=7+summary.meanProbeScore*8;ctx.fillStyle=map0ClassColors[summary.dominantClass];ctx.globalAlpha=.86;ctx.beginPath();ctx.arc(x,y,radius,0,Math.PI*2);ctx.fill();ctx.globalAlpha=1;if(summary.parameters.id===map1SelectedId){ctx.strokeStyle="#173e34";ctx.lineWidth=2.5;ctx.beginPath();ctx.arc(x,y,radius+4,0,Math.PI*2);ctx.stroke();}ctx.fillStyle="#334f46";ctx.textAlign="center";ctx.fillText(String(summary.parameters.id),x,y-radius-4);}
  ctx.fillStyle="#52675e";ctx.textAlign="center";ctx.fillText(map0AxisLabels[xAxis],pad.l+w/2,rect.height-7);ctx.save();ctx.translate(13,pad.t+h/2);ctx.rotate(-Math.PI/2);ctx.fillText(map0AxisLabels[yAxis],0,0);ctx.restore();
};
const signed=(value:number,digits=1)=>`${value>=0?"+":""}${(value*100).toFixed(digits)} pp`;
const renderMap1Inspector=()=>{
  const summary=map1.confirmationSummaries.find(item=>item.parameters.id===map1SelectedId)??map1.confirmationSummaries[0];map1SelectedId=summary.parameters.id;map1Parameter.value=String(map1SelectedId);const reference=map1.referenceNormSummaries.find(item=>item.parameters.id===map1SelectedId)!;const comparison=map1.mechanismComparisons.find(item=>item.parameterId===map1SelectedId)!;const p=summary.parameters;
  byId("map1-parameters").innerHTML=`<div><span>双尺度分类</span><strong>${labels[summary.dominantClass]}</strong></div><div><span>旧机制分类</span><strong>${labels[reference.dominantClass]}</strong></div><div><span>循环增益</span><strong>${p.recurrentGain.toFixed(3)}</strong></div><div><span>可塑率</span><strong>${p.internalLearningRate.toFixed(3)}</strong></div><div><span>内稳态</span><strong>${p.homeostasisStrength.toFixed(3)}</strong></div><div><span>探索率</span><strong>${p.explorationRate.toFixed(3)}</strong></div>`;
  byId("map1-mechanism-grid").innerHTML=`<article><span>平均探针</span><strong>${(summary.meanProbeScore*100).toFixed(1)}%</strong><small>旧机制 ${(reference.meanProbeScore*100).toFixed(1)}% · ${signed(comparison.meanProbeScoreChange)}</small></article><article><span>重复反转</span><strong>${(summary.meanRepeatedReversalAccuracy*100).toFixed(1)}%</strong><small>旧机制 ${(reference.meanRepeatedReversalAccuracy*100).toFixed(1)}% · ${signed(comparison.repeatedReversalAccuracyChange)}</small></article><article><span>损伤恢复增益</span><strong>${signed(summary.meanPerturbationRecoveryGain)}</strong><small>旧机制 ${signed(reference.meanPerturbationRecoveryGain)} · 变化 ${signed(comparison.perturbationRecoveryGainChange)}</small></article><article class="warning"><span>相对权重漂移</span><strong>${summary.meanRelativeWeightDrift.toFixed(2)}</strong><small>旧机制 ${reference.meanRelativeWeightDrift.toFixed(2)} · 变化 ${comparison.meanWeightDriftChange>=0?"+":""}${comparison.meanWeightDriftChange.toFixed(2)}</small></article>`;
  const controls=map1.controlSummaries.filter(item=>item.parameterId===map1SelectedId);byId("map1-control-grid").replaceChildren(...controls.map(item=>{const card=document.createElement("article");card.innerHTML=`<span>${map0ControlLabels[item.control]}</span><strong>${(item.meanProbeScore*100).toFixed(0)}% 探针</strong><small>得分变化 ${signed(item.meanProbeScoreChangeFromBaseline)} · 形成 ${(item.formationProbability.mean*100).toFixed(1)}%</small>`;return card;}));drawMap1();
};
const renderMap1=()=>{
  byId("map1-status").textContent=map1.acceptance.stableRegionFound?`找到 ${map1.stableRegionParameterIds.length} 点候选区域`:"机制被淘汰 · 漂移失控";
  const acceptanceLabels:Record<string,string>={onlyHomeostasisMechanismChanged:"仅更换内稳态",developmentAndConfirmationSeedsDisjoint:"开发/确认隔离",allDevelopmentRunsComplete:"开发扫描完整",confirmationRunsComplete:"独立确认完整",referenceMechanismControlsComplete:"旧机制配对完整",causalControlsComplete:"因果对照完整",causalInterventionDetected:"检测到因果变化",finiteOutputs:"数值有限",mechanismImprovesTradeoff:"改善学习—漂移权衡"};const grid=byId("map1-acceptance");grid.replaceChildren();for(const [key,label] of Object.entries(acceptanceLabels)){const pass=map1.acceptance[key];const item=document.createElement("div");item.className=pass?"pass":"fail";item.innerHTML=`<i>${pass?"✓":"×"}</i><span>${label}</span>`;grid.append(item);}
  byId("map1-protocol").textContent=`活动目标 ${map1.config.activityTarget.toFixed(2)} · EMA ${map1.config.activityEmaRate.toFixed(2)} · 权重慢回拉 ${map1.config.weightNormRelaxationRate.toFixed(2)}`;map1Parameter.replaceChildren(...map1.confirmationSummaries.map(summary=>{const option=document.createElement("option");option.value=String(summary.parameters.id);option.textContent=`配置 ${summary.parameters.id}`;return option;}));map1SelectedId=[...map1.confirmationSummaries].sort((a,b)=>b.meanProbeScore-a.meanProbeScore)[0].parameters.id;byId("map1-conclusions").replaceChildren(...map1.conclusions.map(text=>{const p=document.createElement("p");p.textContent=text;return p;}));renderMap1Inspector();
};

const average=<T>(rows:T[],read:(row:T)=>number)=>rows.reduce((sum,row)=>sum+read(row),0)/Math.max(1,rows.length);
const renderMap2=()=>{
  byId("map2-status").textContent=map2c.acceptance.stagePassed?`Map 2 冻结 · ${map2c.stableRegionParameterIds.length} 点稳定区域`:`Map 2C 负结果 · 停止扩展`;
  const stageGrid=byId("map2-stage-grid");
  const stages=[
    {name:"2A · 内生有界可塑性",result:map2a,metric:`漂移 ${average(map2a.mechanismComparisons,row=>row.meanWeightDriftChange).toFixed(3)}`,note:"对称乘性软边界；相同配置、相同种子配对旧加性更新。"},
    {name:"2B · 持续基础供能",result:map2b,metric:`资源 ${average(map2b.confirmationSummaries,row=>row.meanResourceLevel??0).toFixed(3)}`,note:"资源只调制活动与可塑性，不携带任务信息，不增加睡眠状态机。"},
    {name:"2C · 无重置连续流",result:map2c,metric:`准确率 ${(average(map2c.confirmationSummaries,row=>row.meanOverallAccuracy)*100).toFixed(1)}%`,note:"720 个连续试次；规则自然切换，试次间不清空内部状态或资源。"},
  ];
  stageGrid.replaceChildren(...stages.map(stage=>{const article=document.createElement("article");article.className=stage.result.acceptance.stagePassed?"pass":"fail";article.innerHTML=`<span>${stage.name}</span><strong>${stage.metric}</strong><small>${stage.note}</small><small>阶段判据 ${stage.result.acceptance.stagePassed?"通过":"未通过"} · 程序协议 ${stage.result.acceptance.passed?"完整":"异常"}</small>`;return article;}));
  const continuous=average(map2c.confirmationSummaries,row=>row.meanOverallAccuracy);
  const reset=average(map2c.resetSummaries,row=>row.meanOverallAccuracy);
  const noSupply=average(map2c.noSupplySummaries,row=>row.meanOverallAccuracy);
  const frozen=average(map2c.frozenSummaries,row=>row.meanOverallAccuracy);
  const aDrift=average(map2a.mechanismComparisons,row=>row.meanWeightDriftChange);
  const aReversal=average(map2a.mechanismComparisons,row=>row.repeatedReversalAccuracyChange);
  const cards=[
    ["2A 软边界",`${aDrift.toFixed(3)} 漂移`,`重复反转 ${signed(aReversal)}`],
    ["2B 持续供能",`${(average(map2b.confirmationSummaries,row=>row.meanProbeScore)*100).toFixed(1)}% 探针`,`断供 ${(average(map2b.noSupplySummaries,row=>row.meanProbeScore)*100).toFixed(1)}%`],
    ["2C 连续基线",`${(continuous*100).toFixed(1)}%`,`逐试次重置 ${(reset*100).toFixed(1)}%`],
    ["2C 因果必要性",`${(noSupply*100).toFixed(1)}% 断供`,`冻结可塑性 ${(frozen*100).toFixed(1)}%`],
  ];
  byId("map2-comparison").replaceChildren(...cards.map(([label,value,note])=>{const article=document.createElement("article");article.innerHTML=`<span>${label}</span><strong>${value}</strong><small>${note}</small>`;return article;}));
  byId("map2-region").replaceChildren(...map2c.stableRegionParameterIds.map(id=>{const cell=document.createElement("i");cell.textContent=`P${id}`;return cell;}));
  byId("map2-conclusions").replaceChildren(...map2c.conclusions.map(text=>{const p=document.createElement("p");p.textContent=text;return p;}));
};

const renderMechanismM0=()=>{
  const auditPassed=Object.values(mechanismM0.interfaceAudit).every(Boolean);
  byId("m0-status").textContent=auditPassed?"M0 冻结完成":"M0 审计未通过";
  byId("m0-status").className=auditPassed?"pass":"fail";
  const carrier=mechanismM0.carrierState;
  byId("m0-carrier").innerHTML=`<strong>${carrier.hiddenUnitCount} units</strong><small>${carrier.sensoryChannelCount} sensory · ${carrier.actionCount} actions · ${carrier.plasticRecurrentConnectionsPerUnit} plastic recurrent / unit</small><code>${carrier.version}</code>`;
  byId("m0-actions").innerHTML=`<strong>${carrier.adjustableVariables.length} 类受控变量</strong><small>局部观测 → 有界动作；目标标签、读出状态和输出写入均不在接口类型中</small><code>${mechanismM0.adjustmentMechanism}</code>`;
  const result=mechanismM0.capabilityResult;
  byId("m0-capability").innerHTML=`<strong>${(result.map2cContinuousAccuracy*100).toFixed(1)}% continuous</strong><small>断供 ${(result.noSupplyAccuracy*100).toFixed(1)}% · 冻结可塑性 ${(result.frozenPlasticityAccuracy*100).toFixed(1)}% · 稳定区 ${result.stableRegionParameterIds.length} 点</small><code>不是通用学习结论</code>`;
  const names:Record<string,string>={activityState:"活动状态",sensoryWeights:"感觉权重",recurrentWeights:"循环权重",excitability:"兴奋性",resource:"资源",mechanismState:"机制状态"};
  byId("m0-contracts").replaceChildren(...carrier.adjustableVariables.map(contract=>{const article=document.createElement("article");article.innerHTML=`<header><strong>${names[contract.variable]??contract.variable}</strong><span>${contract.valueRange}</span></header><p>${contract.updatePermission}</p><dl><div><dt>预算</dt><dd>${contract.updateBudget}</dd></div><div><dt>冻结</dt><dd>${contract.freezeMode}</dd></div></dl>`;return article;}));
};

const renderMechanismM1=()=>{
  byId("m1-status").textContent=mechanismM1.acceptance.retentionBoundaryPassed?"保留边界通过":"协议通过 · 保留失败";
  byId("m1-status").className=mechanismM1.acceptance.retentionBoundaryPassed?"pass":"diagnostic";
  const phaseMean=(index:number,key:"initialAccuracy"|"departureAccuracy")=>average(mechanismM1.confirmationSeedResults,row=>row.phaseResults[index][key]);
  byId("m1-sequence").replaceChildren(...mechanismM1.sequence.map((rule,index)=>{const article=document.createElement("article");const initial=phaseMean(index,"initialAccuracy"),departure=phaseMean(index,"departureAccuracy");article.innerHTML=`<span>PHASE ${index+1}</span><strong>${rule.toUpperCase()}</strong><div><b>${(initial*100).toFixed(1)}%</b><i>→</i><b>${(departure*100).toFixed(1)}%</b></div><small>进入 → 离开</small>`;if(index===4)article.className="return";return article;}));
  const summaryMean=(rows:M1RetentionSummary[],key:keyof M1RetentionSummary)=>average(rows,row=>Number(row[key]));
  const controls:[string,number,string][]=[
    ["连续基线",summaryMean(mechanismM1.confirmationSummaries,"meanNovelRuleFinalAccuracy"),"B/C/D 离开表现"],
    ["冻结可塑性",summaryMean(mechanismM1.frozenSummaries,"meanNovelRuleFinalAccuracy"),"验证内部调整贡献"],
    ["随机后果",summaryMean(mechanismM1.randomConsequenceSummaries,"meanNovelRuleFinalAccuracy"),"验证后果信息贡献"],
    ["逐试次重置",summaryMean(mechanismM1.resetSummaries,"meanReturnAInitialAccuracy"),"A 重现初始表现"],
    ["单规则容量",summaryMean(mechanismM1.singleRuleSummaries,"meanMinimumSingleRuleFinalAccuracy"),"四规则中的最低值"],
  ];
  byId("m1-controls").replaceChildren(...controls.map(([label,value,note])=>{const article=document.createElement("article");article.innerHTML=`<span>${label}</span><strong>${(value*100).toFixed(1)}%</strong><small>${note}</small>`;return article;}));
  const diagnosisLabels:Record<string,string>={"capacity-insufficient":"容量不足","overwrite-forgetting":"覆盖性遗忘","credit-routing-error":"信用路由错误","resource-exhaustion":"资源耗尽","dynamics-instability":"动力学失稳","no-dominant-failure":"未发现主导失败"};
  byId("m1-diagnosis").textContent=diagnosisLabels[mechanismM1.dominantFailureClass]??mechanismM1.dominantFailureClass;
  byId("m1-diagnosis-note").textContent=mechanismM1.dominantFailureClass==="capacity-insufficient"?"四条规则单独训练时仍有规则接近机会水平，因此现阶段不能把 A 的下降单独解释为覆盖性遗忘，也不触发元可塑性分支。":"诊断由稳定性、资源、单规则容量、保留下降和新规则学习按预注册顺序决定。";
  byId("m1-parameters").replaceChildren(...mechanismM1.confirmationSummaries.map(row=>{const article=document.createElement("article");article.innerHTML=`<header><strong>P${row.parameters.id}</strong><span>${diagnosisLabels[row.failureClass]??row.failureClass}</span></header><dl><div><dt>A 保留下降</dt><dd>${(row.meanRetentionDrop*100).toFixed(1)} pp</dd></div><div><dt>新规则</dt><dd>${(row.meanNovelRuleFinalAccuracy*100).toFixed(1)}%</dd></div><div><dt>单规则最低</dt><dd>${(row.meanMinimumSingleRuleFinalAccuracy*100).toFixed(1)}%</dd></div><div><dt>权重漂移</dt><dd>${row.meanRelativeWeightDrift.toFixed(2)}</dd></div></dl>`;return article;}));
  byId("m1-conclusions").replaceChildren(...mechanismM1.conclusions.map(text=>{const p=document.createElement("p");p.textContent=text;return p;}));
};

const renderMechanismM2C=()=>{
  const decisionLabels:Record<string,string>={"candidate-accepted":"候选通过","no-benefit-over-weight-only":"未优于仅权重可塑","random-rewiring-equivalent":"与随机重连等价","capacity-still-insufficient":"容量仍不足","retention-still-insufficient":"保留仍不足","resource-exhaustion":"资源耗尽","dynamics-instability":"动力学失稳"};
  byId("m2c-status").textContent=mechanismM2C.acceptance.mechanismAccepted?"机制验收通过":"协议通过 · 候选淘汰";
  byId("m2c-status").className=mechanismM2C.acceptance.mechanismAccepted?"pass":"diagnostic";
  const selected=mechanismM2C.selectedMechanismParameters;
  byId("m2c-selected").textContent=`P${selected.id} · ${selected.rewiringInterval} 试次`;
  byId("m2c-selected-note").textContent=`局部证据 EMA 衰减 ${selected.evidenceDecay.toFixed(2)}；只使用开发种子选择，确认后不回调。`;
  byId("m2c-decision").textContent=decisionLabels[mechanismM2C.decision]??mechanismM2C.decision;
  byId("m2c-decision-note").textContent=mechanismM2C.acceptance.mechanismAccepted?"容量、连续学习、保留及两个结构因果对照均通过。":"新增的局部结构选择没有产生可归因收益，因此不进入 M3，也不据此扩容。";
  const controlLabels:Record<string,string>={"local-evidence-rewiring":"局部证据重连","random-rewiring":"随机重连","weight-only":"仅权重可塑","frozen-adjustment":"完全冻结调整"};
  byId("m2c-controls").replaceChildren(...mechanismM2C.confirmationSummaries.map(row=>{const article=document.createElement("article");article.innerHTML=`<header><span>${controlLabels[row.control]??row.control}</span><b>${row.runCount} runs</b></header><dl><div><dt>单规则最低</dt><dd>${(row.meanMinimumSingleRuleAccuracy*100).toFixed(1)}%</dd></div><div><dt>B/C/D</dt><dd>${(row.meanNovelRuleAccuracy*100).toFixed(1)}%</dd></div><div><dt>A 回归初始</dt><dd>${(row.meanReturnAInitialAccuracy*100).toFixed(1)}%</dd></div><div><dt>保留下降</dt><dd>${(row.meanRetentionDrop*100).toFixed(1)} pp</dd></div><div><dt>结构改写</dt><dd>${row.meanSequenceRewireCount.toFixed(0)}</dd></div><div><dt>权重漂移</dt><dd>${row.meanRelativeWeightDrift.toFixed(3)}</dd></div></dl>`;return article;}));
  const effectLabels:Record<string,string>={"minimum-single-rule-accuracy":"单规则最低","novel-rule-accuracy":"B/C/D 连续学习","return-a-initial-accuracy":"A 回归初始"};
  const comparisonLabels:Record<string,string>={"weight-only":"对仅权重可塑","random-rewiring":"对随机重连","frozen-adjustment":"对完全冻结"};
  byId("m2c-effects").replaceChildren(...mechanismM2C.pairedEffects.map(row=>{const article=document.createElement("article");article.innerHTML=`<span>${effectLabels[row.metric]??row.metric} · ${comparisonLabels[row.comparison]??row.comparison}</span><strong>${row.interval.mean>=0?"+":""}${(row.interval.mean*100).toFixed(1)} pp</strong><small>[${(row.interval.lower95*100).toFixed(1)}, ${(row.interval.upper95*100).toFixed(1)}]</small>`;article.className=row.interval.lower95>0?"positive":row.interval.upper95<0?"negative":"uncertain";return article;}));
  byId("m2c-conclusions").replaceChildren(...mechanismM2C.conclusions.map(text=>{const p=document.createElement("p");p.textContent=text;return p;}));
};

const renderStructuralDiagnostic=()=>{
  const decisionLabels:Record<string,string>={"evidence-informative":"证据具有可用信息","evidence-ranking-weak":"排序微弱但选择失败","evidence-not-informative":"结构信用信号无效","counterfactual-effects-flat":"反事实作用近乎平坦","diagnostic-unstable":"诊断不稳定"};
  byId("sd-status").textContent=structuralDiagnostic.acceptance.evidenceInformative?"结构证据通过":"协议通过 · 证据未通过";
  byId("sd-status").className=structuralDiagnostic.acceptance.evidenceInformative?"pass":"diagnostic";
  byId("sd-decision").textContent=decisionLabels[structuralDiagnostic.decision]??structuralDiagnostic.decision;
  byId("sd-decision-note").textContent=structuralDiagnostic.decision==="counterfactual-effects-flat"?"32 试次窗口内，即使 oracle 单边换边的平均收益也低于 2 pp；应先诊断作用时间尺度，而不是复杂化在线选择公式。":"决策按有限性、oracle 可用性、排名对打乱优势和首选对随机优势依次确定。";
  const s=structuralDiagnostic.confirmationSummary;
  const metrics:[string,string,string][]=[
    ["证据 Spearman",s.meanSpearmanCorrelation.mean.toFixed(3),`[${s.meanSpearmanCorrelation.lower95.toFixed(3)}, ${s.meanSpearmanCorrelation.upper95.toFixed(3)}]`],
    ["相对打乱优势",s.meanCorrelationAdvantage.mean.toFixed(3),`[${s.meanCorrelationAdvantage.lower95.toFixed(3)}, ${s.meanCorrelationAdvantage.upper95.toFixed(3)}]`],
    ["证据首选收益",`${s.meanSelectedBenefit.mean>=0?"+":""}${(s.meanSelectedBenefit.mean*100).toFixed(1)} pp`,`随机 ${(s.meanRandomBenefit.mean*100).toFixed(1)} pp`],
    ["首选对随机",`${s.meanSelectedBenefitAdvantage.mean>=0?"+":""}${(s.meanSelectedBenefitAdvantage.mean*100).toFixed(1)} pp`,`[${(s.meanSelectedBenefitAdvantage.lower95*100).toFixed(1)}, ${(s.meanSelectedBenefitAdvantage.upper95*100).toFixed(1)}]`],
    ["oracle 收益",`+${(s.meanOracleBenefit.mean*100).toFixed(1)} pp`,`门槛 +2.0 pp`],
    ["top-quartile 命中",`${(s.meanTopQuartileHitRate.mean*100).toFixed(1)}%`,`机会水平 29.4%`],
  ];
  byId("sd-metrics").replaceChildren(...metrics.map(([label,value,note])=>{const article=document.createElement("article");article.innerHTML=`<span>${label}</span><strong>${value}</strong><small>${note}</small>`;return article;}));
  byId("sd-phases").replaceChildren(...structuralDiagnostic.confirmationPhaseSummaries.map(row=>{const article=document.createElement("article");article.innerHTML=`<header><span>PHASE ${row.phaseIndex+1}</span><strong>${row.rule.toUpperCase()}</strong></header><dl><div><dt>Spearman</dt><dd>${row.meanSpearmanCorrelation.mean.toFixed(3)}</dd></div><div><dt>首选收益</dt><dd>${row.meanSelectedBenefit.mean>=0?"+":""}${(row.meanSelectedBenefit.mean*100).toFixed(1)} pp</dd></div><div><dt>oracle</dt><dd>+${(row.meanOracleBenefit.mean*100).toFixed(1)} pp</dd></div><div><dt>top 命中</dt><dd>${(row.topQuartileHitRate.mean*100).toFixed(1)}%</dd></div></dl>`;if(row.phaseIndex===4)article.className="return";return article;}));
  byId("sd-conclusions").replaceChildren(...structuralDiagnostic.conclusions.map(text=>{const p=document.createElement("p");p.textContent=text;return p;}));
};

const renderStructuralTimescale=()=>{
  const decisionLabels:Record<string,string>={"delayed-structural-effect":"存在延迟结构作用","credit-signal-insufficient":"结构信用信号不足","recovery-only-signal":"仅有恢复信号","single-edge-freedom-insufficient":"单边自由度不足","diagnostic-unstable":"诊断不稳定"};
  byId("st-status").textContent=structuralTimescale.acceptance.stagePassed?"协议通过 · 时间尺度已定位":"协议未通过";
  byId("st-status").className=structuralTimescale.acceptance.stagePassed?"diagnostic":"fail";
  byId("st-decision").textContent=decisionLabels[structuralTimescale.decision]??structuralTimescale.decision;
  byId("st-decision-note").textContent=structuralTimescale.decision==="single-edge-freedom-insufficient"?"oracle 在 32 试次后没有继续增长，256 试次仍低于 +2 pp。短窗口不是主因；下一步只做固定预算成组离线反事实。":"决策依次检查有限性、长时 oracle、阶段局限与证据信息。";
  byId("st-horizons").replaceChildren(...structuralTimescale.confirmationHorizonSummaries.map(row=>{const article=document.createElement("article");article.innerHTML=`<header><span>TRAINING HORIZON</span><strong>${row.trainingHorizon}</strong></header><dl><div><dt>oracle</dt><dd>+${(row.meanOracleBenefit.mean*100).toFixed(2)} pp</dd></div><div><dt>证据首选</dt><dd>${row.meanSelectedBenefit.mean>=0?"+":""}${(row.meanSelectedBenefit.mean*100).toFixed(2)} pp</dd></div><div><dt>首选对随机</dt><dd>${row.meanSelectedBenefitAdvantage.mean>=0?"+":""}${(row.meanSelectedBenefitAdvantage.mean*100).toFixed(2)} pp</dd></div><div><dt>Spearman</dt><dd>${row.meanSpearmanCorrelation.mean.toFixed(3)}</dd></div></dl>`;if(row.trainingHorizon===32)article.className="reference";return article;}));
  byId("st-comparisons").replaceChildren(...structuralTimescale.confirmationHorizonComparisons.map(row=>{const article=document.createElement("article");const delta=row.oracleBenefitChange;article.innerHTML=`<span>${row.fromTrainingHorizon} → ${row.toTrainingHorizon} · oracle</span><strong>${delta.mean>=0?"+":""}${(delta.mean*100).toFixed(2)} pp</strong><small>95% CI [${(delta.lower95*100).toFixed(2)}, ${(delta.upper95*100).toFixed(2)}] · 首选变化 ${(row.selectedBenefitChange.mean*100).toFixed(2)} pp</small>`;return article;}));
  const longPhases=structuralTimescale.confirmationPhaseSummaries.filter(row=>row.trainingHorizon===256);
  byId("st-phases").replaceChildren(...longPhases.map(row=>{const article=document.createElement("article");article.innerHTML=`<header><span>PHASE ${row.phaseIndex+1}</span><strong>${row.rule.toUpperCase()}</strong></header><dl><div><dt>Spearman</dt><dd>${row.meanSpearmanCorrelation.mean.toFixed(3)}</dd></div><div><dt>首选收益</dt><dd>${row.meanSelectedBenefit.mean>=0?"+":""}${(row.meanSelectedBenefit.mean*100).toFixed(2)} pp</dd></div><div><dt>oracle</dt><dd>+${(row.meanOracleBenefit.mean*100).toFixed(2)} pp</dd></div><div><dt>top 命中</dt><dd>${(row.topQuartileHitRate.mean*100).toFixed(1)}%</dd></div></dl>`;if(row.phaseIndex===4)article.className="return";return article;}));
  byId("st-conclusions").replaceChildren(...structuralTimescale.conclusions.map(text=>{const p=document.createElement("p");p.textContent=text;return p;}));
};

const renderStructuralGroup=()=>{
  const decisionLabels:Record<string,string>={"grouped-freedom-and-credit":"成组自由度与信用均可用","grouped-freedom-without-credit":"有成组作用但信用不足","history-dominated-grouped-effect":"成组作用由历史主导","grouped-effect-not-beyond-single":"成组作用未超越单边","no-useful-grouped-effect":"成组作用仍不足","diagnostic-unstable":"诊断不稳定"};
  byId("sg-status").textContent=structuralGroup.acceptance.stagePassed?"协议通过 · M2C 分支关闭":"协议未通过";
  byId("sg-status").className=structuralGroup.acceptance.stagePassed?"diagnostic":"fail";
  byId("sg-decision").textContent=decisionLabels[structuralGroup.decision]??structuralGroup.decision;
  byId("sg-decision-note").textContent=structuralGroup.decision==="history-dominated-grouped-effect"?"4 边总体 oracle 越过门槛，但主要来自已有 A；B/C/D 新规则容量仍不足，因此不进入在线成组重连。":"决策同时检查总体作用、对单边优势、B/C/D 新规则和局部信用。";
  byId("sg-scales").replaceChildren(...structuralGroup.confirmationSummaries.map(row=>{const article=document.createElement("article");article.innerHTML=`<header><span>STRUCTURAL ACTION</span><strong>${row.edgeCount} 边</strong></header><dl><div><dt>budgeted oracle</dt><dd>+${(row.meanBudgetedOracleBenefit.mean*100).toFixed(2)} pp</dd></div><div><dt>证据首选</dt><dd>${row.meanSelectedBenefit.mean>=0?"+":""}${(row.meanSelectedBenefit.mean*100).toFixed(2)} pp</dd></div><div><dt>首选对随机</dt><dd>${row.meanSelectedBenefitAdvantage.mean>=0?"+":""}${(row.meanSelectedBenefitAdvantage.mean*100).toFixed(2)} pp</dd></div><div><dt>oracle 交互</dt><dd>${row.meanOracleInteractionOverAdditive.mean>=0?"+":""}${(row.meanOracleInteractionOverAdditive.mean*100).toFixed(2)} pp</dd></div><div><dt>证据 Spearman</dt><dd>${row.meanSpearmanCorrelation.mean.toFixed(3)}</dd></div></dl>`;if(row.edgeCount===structuralGroup.selectedGroupSize)article.className="selected";return article;}));
  const c=structuralGroup.confirmationCapabilityContrast;
  const cards:[string,Interval,string,string][]=[
    ["已有 A 阶段 oracle",c.meanHistoryPhaseOracleBenefit,"history","A + A 回归"],
    ["B/C/D 新规则 oracle",c.meanNovelRuleOracleBenefit,"","一般容量对象"],
    ["历史阶段优势",c.historyAdvantageOverNovelRules,"history","历史 − 新规则"],
  ];
  byId("sg-contrast").replaceChildren(...cards.map(([label,value,klass,note])=>{const article=document.createElement("article");article.className=klass;article.innerHTML=`<span>${label}</span><strong>${value.mean>=0?"+":""}${(value.mean*100).toFixed(2)} pp</strong><small>${note} · [${(value.lower95*100).toFixed(2)}, ${(value.upper95*100).toFixed(2)}]</small>`;return article;}));
  const phases=structuralGroup.confirmationPhaseSummaries.filter(row=>row.edgeCount===structuralGroup.selectedGroupSize);
  byId("sg-phases").replaceChildren(...phases.map(row=>{const article=document.createElement("article");article.innerHTML=`<header><span>PHASE ${row.phaseIndex+1}</span><strong>${row.rule.toUpperCase()}</strong></header><dl><div><dt>证据首选</dt><dd>${row.meanSelectedBenefit.mean>=0?"+":""}${(row.meanSelectedBenefit.mean*100).toFixed(2)} pp</dd></div><div><dt>随机 bundle</dt><dd>${row.meanRandomBenefit.mean>=0?"+":""}${(row.meanRandomBenefit.mean*100).toFixed(2)} pp</dd></div><div><dt>oracle</dt><dd>+${(row.meanBudgetedOracleBenefit.mean*100).toFixed(2)} pp</dd></div><div><dt>组合交互</dt><dd>+${(row.meanOracleInteractionOverAdditive.mean*100).toFixed(2)} pp</dd></div></dl>`;if(row.phaseIndex===4)article.className="return";return article;}));
  byId("sg-conclusions").replaceChildren(...structuralGroup.conclusions.map(text=>{const p=document.createElement("p");p.textContent=text;return p;}));
};

const renderRepresentationCapacity=()=>{
  const decisionLabels:Record<string,string>={"baseline-capability-present":"基线能力已存在","credit-routing-bottleneck":"信用路由瓶颈","learned-representation-readout-bottleneck":"已形成表征但读出受限","latent-symbol-code-without-rule-formation":"潜在符号码尚未形成规则","representation-insufficient":"表征容量不足","inconclusive-boundary":"边界不确定","diagnostic-unstable":"诊断不稳定"};
  byId("rc-status").textContent=representationCapacity.acceptance.stagePassed?"协议通过 · 容量解释已收窄":"协议未通过";
  byId("rc-status").className=representationCapacity.acceptance.stagePassed?"diagnostic":"fail";
  byId("rc-decision").textContent=decisionLabels[representationCapacity.decision]??representationCapacity.decision;
  byId("rc-decision-note").textContent=representationCapacity.decision==="latent-symbol-code-without-rule-formation"?"外部探针读取的是训练前已有的符号几何；奖励调整几乎没有形成新的目标相关变化，不能把 100% 探针算作系统学习成功。":"判定依次区分正式行为、目标信用救援、训练形成增益、外部读出救援与表征不足。";
  const n=representationCapacity.confirmationNovelSummary;
  const cards:[string,string,string][]=[
    ["固定读出行为",`${(n.rewardLocalBehaviorAccuracy.mean*100).toFixed(1)}%`,"B/C/D 奖励局部调整"],
    ["训练前隐藏探针",`${(n.preHiddenProbeAccuracy.mean*100).toFixed(1)}%`,"潜在信息 · 不算能力"],
    ["训练形成增益",`${n.rewardLocalProbeGainOverPre.mean>=0?"+":""}${(n.rewardLocalProbeGainOverPre.mean*100).toFixed(2)} pp`,"门槛 +5 pp"],
    ["目标信用行为",`${(n.targetDirectedBehaviorAccuracy.mean*100).toFixed(1)}%`,`配对 ${(n.targetDirectedBehaviorGain.mean*100).toFixed(2)} pp`],
  ];
  byId("rc-novel").replaceChildren(...cards.map(([label,value,note])=>{const article=document.createElement("article");article.innerHTML=`<span>${label}</span><strong>${value}</strong><small>${note}</small>`;return article;}));
  byId("rc-rules").replaceChildren(...representationCapacity.confirmationSummaries.map(row=>{const article=document.createElement("article");article.innerHTML=`<header><span>RULE</span><strong>${row.rule.toUpperCase()}</strong></header><dl><div><dt>固定读出行为</dt><dd>${(row.rewardLocalBehaviorAccuracy.mean*100).toFixed(1)}%</dd></div><div><dt>训练前探针</dt><dd>${(row.preHiddenProbeAccuracy.mean*100).toFixed(1)}%</dd></div><div><dt>训练后探针</dt><dd>${(row.rewardLocalHiddenProbeAccuracy.mean*100).toFixed(1)}%</dd></div><div><dt>目标信用行为</dt><dd>${(row.targetDirectedBehaviorAccuracy.mean*100).toFixed(1)}%</dd></div><div><dt>随机标签</dt><dd>${(row.rewardLocalShuffledProbeAccuracy.mean*100).toFixed(1)}%</dd></div><div><dt>原始输入</dt><dd>${(row.rawSensorProbeAccuracy.mean*100).toFixed(1)}%</dd></div></dl>`;if(row.rule==="a")article.className="history";return article;}));
  byId("rc-conclusions").replaceChildren(...representationCapacity.conclusions.map(text=>{const p=document.createElement("p");p.textContent=text;return p;}));
};

const renderRuleFormation=()=>{
  const decisionLabels:Record<string,string>={"candidate-accepted":"候选机制通过","dynamics-unstable":"动力学不稳定","consequence-independent":"后果不具任务特异性","history-only-formation":"只保留历史能力","behavior-without-formation-evidence":"行为改善但缺少形成证据","history-degraded":"历史能力受损","no-behavior-benefit":"没有行为收益"};
  const controlLabels:Record<string,string>={"reward-local-baseline":"现有奖励局部基线","node-perturbation":"节点扰动候选","random-consequence":"随机后果对照","frozen-adjustment":"冻结调整"};
  byId("m1f-status").textContent=ruleFormation.acceptance.mechanismAccepted?"候选通过":"协议通过 · 候选淘汰";
  byId("m1f-status").className=ruleFormation.acceptance.mechanismAccepted?"pass":"diagnostic";
  byId("m1f-decision").textContent=decisionLabels[ruleFormation.decision]??ruleFormation.decision;
  byId("m1f-decision-note").textContent=ruleFormation.decision==="consequence-independent"?"候选未可靠优于使用同一批微扰的随机后果对照，同时明显损伤已有 A；变化不能归因于任务结果信用。":"判定同时要求新规则行为、目标概率形成、后果特异性、历史保持与稳定性。";
  byId("m1f-gains").replaceChildren(...ruleFormation.developmentGainSummaries.map(row=>{const article=document.createElement("article");article.innerHTML=`<header><span>FORMATION GAIN</span><strong>${row.formationGain.toFixed(3)}</strong></header><dl><div><dt>B/C/D 行为</dt><dd>${(row.meanNovelFinalAccuracy.mean*100).toFixed(1)}%</dd></div><div><dt>A 行为</dt><dd>${(row.meanHistoryFinalAccuracy.mean*100).toFixed(1)}%</dd></div><div><dt>目标概率形成</dt><dd>${row.meanNovelTargetProbabilityGain.mean>=0?"+":""}${(row.meanNovelTargetProbabilityGain.mean*100).toFixed(2)} pp</dd></div><div><dt>权重漂移</dt><dd>${row.meanRelativeWeightDrift.mean.toFixed(3)}</dd></div></dl>`;if(row.formationGain===ruleFormation.selectedFormationGain)article.className="selected";return article;}));
  byId("m1f-controls").replaceChildren(...ruleFormation.confirmationSummaries.map(row=>{const article=document.createElement("article");article.innerHTML=`<header><span>${controlLabels[row.control]??row.control}</span><b>${row.runCount} runs</b></header><dl><div><dt>B/C/D 行为</dt><dd>${(row.meanNovelFinalAccuracy.mean*100).toFixed(1)}%</dd></div><div><dt>A 行为</dt><dd>${(row.meanHistoryFinalAccuracy.mean*100).toFixed(1)}%</dd></div><div><dt>目标概率形成</dt><dd>${row.meanNovelTargetProbabilityGain.mean>=0?"+":""}${(row.meanNovelTargetProbabilityGain.mean*100).toFixed(2)} pp</dd></div><div><dt>资源均值</dt><dd>${row.meanResourceLevel.mean.toFixed(3)}</dd></div><div><dt>权重漂移</dt><dd>${row.meanRelativeWeightDrift.mean.toFixed(3)}</dd></div></dl>`;if(row.control==="node-perturbation")article.className="candidate";return article;}));
  const effects:[string,Interval,string][]=[
    ["候选 − 现有基线",ruleFormation.pairedEffects.candidateOverBaselineNovelAccuracy,"B/C/D 行为"],
    ["候选 − 随机后果",ruleFormation.pairedEffects.candidateOverRandomNovelAccuracy,"后果特异性"],
    ["候选 A 变化",ruleFormation.pairedEffects.candidateHistoryAccuracyChange,"历史保持"],
    ["候选目标概率形成",ruleFormation.pairedEffects.candidateNovelTargetProbabilityGain,"形成证据"],
  ];
  byId("m1f-effects").replaceChildren(...effects.map(([label,value,note])=>{const article=document.createElement("article");article.className=value.upper95<0?"negative":value.lower95>0?"positive":"neutral";article.innerHTML=`<span>${label}</span><strong>${value.mean>=0?"+":""}${(value.mean*100).toFixed(2)} pp</strong><small>${note} · [${(value.lower95*100).toFixed(2)}, ${(value.upper95*100).toFixed(2)}]</small>`;return article;}));
  byId("m1f-conclusions").replaceChildren(...ruleFormation.conclusions.map(text=>{const p=document.createElement("p");p.textContent=text;return p;}));
};

const renderReachability=()=>{
  const decisionLabels:Record<string,string>={"reachable-within-carrier-envelope":"原范数包络内可达","reachable-only-at-absolute-bounds":"仅绝对权重边界可达","fixed-subspace-unreached":"固定子空间未达到","optimization-inconclusive":"优化诊断不足","dynamics-unstable":"诊断不稳定"};
  const controlLabels:Record<string,string>={"frozen-weights":"冻结权重","norm-envelope-oracle":"范数包络 oracle","absolute-bound-oracle":"绝对边界 oracle","shuffled-target-oracle":"乱序目标 oracle"};
  byId("m1x-status").textContent=reachability.acceptance.stagePassed?"协议通过 · 表达边界已定位":"协议未通过";
  byId("m1x-status").className=reachability.acceptance.stagePassed?"diagnostic":"fail";
  byId("m1x-decision").textContent=decisionLabels[reachability.decision]??reachability.decision;
  byId("m1x-decision-note").textContent=reachability.decision==="reachable-only-at-absolute-bounds"?"固定拓扑与读出并非绝对不可达；原稳态范数包络不能在全部载体/规则上保持一致能力。在线信用仍未获验证。":"oracle 是离线表达上界，不计作系统学习机制。";
  byId("m1x-rates").replaceChildren(...reachability.developmentLearningRateSummaries.map(row=>{const article=document.createElement("article");article.innerHTML=`<header><span>ADAM RATE</span><strong>${row.learningRate.toFixed(2)}</strong></header><dl><div><dt>B/C/D 行为</dt><dd>${(row.meanNovelBehaviorAccuracy.mean*100).toFixed(1)}%</dd></div><div><dt>目标概率</dt><dd>${(row.meanNovelTargetProbability.mean*100).toFixed(1)}%</dd></div><div><dt>最低新规则</dt><dd>${(row.meanMinimumNovelAccuracy.mean*100).toFixed(1)}%</dd></div><div><dt>饱和</dt><dd>${(row.meanSaturationFraction.mean*100).toFixed(2)}%</dd></div></dl>`;if(row.learningRate===reachability.selectedLearningRate)article.className="selected";return article;}));
  byId("m1x-controls").replaceChildren(...reachability.confirmationSummaries.map(row=>{const article=document.createElement("article");article.innerHTML=`<header><span>${controlLabels[row.control]??row.control}</span><b>${row.runCount} runs</b></header><dl><div><dt>B/C/D 行为</dt><dd>${(row.meanNovelBehaviorAccuracy.mean*100).toFixed(1)}%</dd></div><div><dt>目标概率</dt><dd>${(row.meanNovelTargetProbability.mean*100).toFixed(1)}%</dd></div><div><dt>最低新规则</dt><dd>${(row.meanMinimumNovelAccuracy.mean*100).toFixed(1)}%</dd></div><div><dt>A 行为</dt><dd>${(row.meanHistoryBehaviorAccuracy.mean*100).toFixed(1)}%</dd></div><div><dt>权重漂移</dt><dd>${row.meanRelativeWeightDrift.mean.toFixed(2)}</dd></div><div><dt>饱和</dt><dd>${(row.meanSaturationFraction.mean*100).toFixed(2)}%</dd></div></dl>`;if(row.control==="absolute-bound-oracle")article.className="candidate";if(row.control==="norm-envelope-oracle")article.classList.add("envelope");return article;}));
  const effects:[string,Interval,string][]=[
    ["绝对边界 − 冻结",reachability.pairedEffects.absoluteOverFrozenBehavior,"正式行为"],
    ["绝对边界 − 乱序目标",reachability.pairedEffects.absoluteOverShuffledBehavior,"任务特异行为"],
    ["绝对边界 − 冻结",reachability.pairedEffects.absoluteOverFrozenTargetProbability,"目标概率"],
    ["绝对边界 − 乱序目标",reachability.pairedEffects.absoluteOverShuffledTargetProbability,"任务特异概率"],
    ["绝对边界 − 范数包络",reachability.pairedEffects.absoluteOverEnvelopeBehavior,"包络限制"],
  ];
  byId("m1x-effects").replaceChildren(...effects.map(([label,value,note])=>{const article=document.createElement("article");article.className=value.lower95>0?"positive":"neutral";article.innerHTML=`<span>${label}</span><strong>${value.mean>=0?"+":""}${(value.mean*100).toFixed(2)} pp</strong><small>${note} · [${(value.lower95*100).toFixed(2)}, ${(value.upper95*100).toFixed(2)}]</small>`;return article;}));
  byId("m1x-conclusions").replaceChildren(...reachability.conclusions.map(text=>{const p=document.createElement("p");p.textContent=text;return p;}));
};

const renderReachabilityEnvelope=()=>{
  const decisionLabels:Record<string,string>={"uniform-envelope-boundary-confirmed":"统一包络边界已确认","boundary-shifted":"开发与确认边界漂移","per-unit-allocation-required":"需要按单元分配范数","absolute-anchor-failed":"绝对边界锚点失败","dynamics-unstable":"诊断不稳定"};
  const makeScale=(row:M1XEScaleSummary,boundary:number|null)=>{const article=document.createElement("article");const label=row.boundKind==="absolute-bound"?"绝对边界":`${row.normMultiplier?.toFixed(1)}×`;article.innerHTML=`<header><span>${row.capabilityPassed?"PASS":"FAIL"}</span><strong>${label}</strong></header><dl><div><dt>B/C/D 行为</dt><dd>${(row.meanNovelBehaviorAccuracy.mean*100).toFixed(1)}%</dd></div><div><dt>目标概率</dt><dd>${(row.meanNovelTargetProbability.mean*100).toFixed(1)}%</dd></div><div><dt>最低新规则</dt><dd>${(row.meanMinimumNovelAccuracy.mean*100).toFixed(1)}%</dd></div><div><dt>A 行为</dt><dd>${(row.meanHistoryBehaviorAccuracy.mean*100).toFixed(1)}%</dd></div><div><dt>饱和</dt><dd>${(row.meanSaturationFraction.mean*100).toFixed(2)}%</dd></div></dl>`;if(row.capabilityPassed)article.classList.add("passed");if(row.normMultiplier===boundary)article.classList.add("boundary");if(row.boundKind==="absolute-bound")article.classList.add("anchor");return article;};
  byId("m1xe-status").textContent=reachabilityEnvelope.acceptance.passed?"通过 · 1.5× 边界独立确认":"协议未通过";
  byId("m1xe-status").className=reachabilityEnvelope.acceptance.passed?"pass":"fail";
  byId("m1xe-decision").textContent=decisionLabels[reachabilityEnvelope.decision]??reachabilityEnvelope.decision;
  byId("m1xe-decision-note").textContent=reachabilityEnvelope.decision==="uniform-envelope-boundary-confirmed"?`首个通过点 ${reachabilityEnvelope.confirmedMinimumMultiplier?.toFixed(1)}×；确认窗口 ${reachabilityEnvelope.passingConfirmationMultipliers.map(value=>`${value.toFixed(1)}×`).join("、")}。更大倍率再次退化，能力窗口并非单调。`:"离线 oracle 只诊断表达自由度，不构成在线学习机制。";
  byId("m1xe-development").replaceChildren(...reachabilityEnvelope.developmentSummaries.map(row=>makeScale(row,reachabilityEnvelope.selectedDevelopmentMultiplier)));
  byId("m1xe-confirmation").replaceChildren(...reachabilityEnvelope.confirmationSummaries.map(row=>makeScale(row,reachabilityEnvelope.confirmedMinimumMultiplier)));
  const selected=reachabilityEnvelope.confirmationComparisons.find(row=>row.normMultiplier===reachabilityEnvelope.confirmedMinimumMultiplier);
  const effects:[string,Interval,string][] = selected?[
    ["B/C/D 行为",selected.novelBehaviorChangeFromOneX,"1.5× − 1.0×"],
    ["目标概率",selected.targetProbabilityChangeFromOneX,"真实目标信用"],
    ["最低新规则",selected.minimumNovelAccuracyChangeFromOneX,"一致能力"],
    ["A 行为",selected.historyAccuracyChangeFromOneX,"历史保持"],
    ["饱和变化",selected.saturationChangeFromOneX,"动力学边界"],
  ]:[];
  byId("m1xe-effects").replaceChildren(...effects.map(([label,value,note])=>{const article=document.createElement("article");article.innerHTML=`<span>${label}</span><strong>${value.mean>=0?"+":""}${(value.mean*100).toFixed(2)} pp</strong><small>${note} · [${(value.lower95*100).toFixed(2)}, ${(value.upper95*100).toFixed(2)}]</small>`;return article;}));
  byId("m1xe-conclusions").replaceChildren(...reachabilityEnvelope.conclusions.map(text=>{const p=document.createElement("p");p.textContent=text;return p;}));
};

const renderNormEnabledSufficiency=()=>{
  const decisionLabels:Record<string,string>={"online-sufficiency-confirmed":"在线充分性已确认","amplitude-necessary-but-insufficient":"幅度必要但不足","consequence-independent":"变化与真实后果无关","history-degraded":"已有 A 受损","oracle-anchor-failed":"1.5× oracle 锚点失败","dynamics-unstable":"协议或动力学失败"};
  const controlLabels:Record<string,string>={"reward-local-one-x":"1.0× 奖励局部","reward-local-one-point-five":"1.5× 奖励局部","frozen-one-point-five":"1.5× 冻结","random-consequence-one-point-five":"1.5× 随机后果","oracle-one-point-five":"1.5× oracle"};
  byId("m1ne-status").textContent=normEnabledSufficiency.acceptance.stagePassed?(normEnabledSufficiency.acceptance.onlineSufficiencyConfirmed?"通过 · 在线能力形成":"协议通过 · 在线能力不足"):"协议未通过";
  byId("m1ne-status").className=normEnabledSufficiency.acceptance.onlineSufficiencyConfirmed?"pass":"diagnostic";
  byId("m1ne-decision").textContent=decisionLabels[normEnabledSufficiency.decision]??normEnabledSufficiency.decision;
  byId("m1ne-decision-note").textContent=normEnabledSufficiency.decision==="amplitude-necessary-but-insufficient"?"真实后果更新明显胜过冻结与随机后果，但最终能力仍远低于 1.5× oracle。系统已有部分方向信用，却不足以稳定形成全部新规则。":"判定同时要求绝对能力、范数收益、学习收益、后果特异性、A 保持与稳定性。";
  byId("m1ne-controls").replaceChildren(...normEnabledSufficiency.confirmationSummaries.map(row=>{const article=document.createElement("article");article.innerHTML=`<header><span>${controlLabels[row.control]??row.control}</span><b>${row.runCount} runs</b></header><dl><div><dt>B/C/D 行为</dt><dd>${(row.meanNovelFinalAccuracy.mean*100).toFixed(1)}%</dd></div><div><dt>目标概率</dt><dd>${(row.meanNovelFinalTargetProbability.mean*100).toFixed(1)}%</dd></div><div><dt>最低新规则</dt><dd>${(row.meanMinimumNovelAccuracy.mean*100).toFixed(1)}%</dd></div><div><dt>A 行为</dt><dd>${(row.meanHistoryFinalAccuracy.mean*100).toFixed(1)}%</dd></div><div><dt>目标形成</dt><dd>${row.meanNovelTargetProbabilityGain.mean>=0?"+":""}${(row.meanNovelTargetProbabilityGain.mean*100).toFixed(1)} pp</dd></div><div><dt>资源</dt><dd>${row.meanResourceLevel.mean.toFixed(3)}</dd></div></dl>`;if(row.control==="reward-local-one-point-five")article.classList.add("candidate");if(row.control==="oracle-one-point-five")article.classList.add("oracle");if(row.control==="frozen-one-point-five")article.classList.add("frozen");return article;}));
  const e=normEnabledSufficiency.confirmationEffects;
  const effects:[string,Interval,string][]=[
    ["候选 − 1.0×",e.candidateOverOneXBehavior,"范数空间收益"],
    ["候选 − 1.5× 冻结",e.candidateOverFrozenBehavior,"真实学习收益"],
    ["候选 − 随机后果",e.candidateOverRandomBehavior,"后果特异性"],
    ["候选 A − 冻结 A",e.candidateHistoryChangeFromFrozen,"历史保持"],
    ["oracle − 候选",e.oracleOverCandidateBehavior,"剩余能力缺口"],
  ];
  byId("m1ne-effects").replaceChildren(...effects.map(([label,value,note])=>{const article=document.createElement("article");article.className=value.lower95>0?"positive":value.upper95<0?"negative":"neutral";article.innerHTML=`<span>${label}</span><strong>${value.mean>=0?"+":""}${(value.mean*100).toFixed(2)} pp</strong><small>${note} · [${(value.lower95*100).toFixed(2)}, ${(value.upper95*100).toFixed(2)}]</small>`;return article;}));
  byId("m1ne-conclusions").replaceChildren(...normEnabledSufficiency.conclusions.map(text=>{const p=document.createElement("p");p.textContent=text;return p;}));
};

const renderCreditDecomposition=()=>{
  const decisionLabels:Record<string,string>={"local-credit-direction-adequate":"局部信用方向与强度充分","aligned-but-weak":"方向对齐但有效步长太弱","homeostasis-cancels-credit":"内稳态抵消信用","eligibility-uninformative":"资格连接归属无信息","consequence-signal-uninformative":"后果信号无信息","direction-mostly-misaligned":"局部方向大多失配","dynamics-unstable":"协议或动力学失败"};
  const componentLabels:Record<string,string>={"raw-proposal":"原始提议","applied-plasticity":"约束后可塑性","homeostasis-correction":"内稳态修正","total-update":"最终净更新"};
  const truth=creditDecomposition.confirmationSummaries.find(row=>row.control==="true-consequence")!;
  byId("m1cd-status").textContent=creditDecomposition.acceptance.stagePassed?"协议通过 · 瓶颈已定位":"协议未通过";
  byId("m1cd-status").className=creditDecomposition.acceptance.stagePassed?"diagnostic":"fail";
  byId("m1cd-decision").textContent=decisionLabels[creditDecomposition.decision]??creditDecomposition.decision;
  byId("m1cd-decision-note").textContent=creditDecomposition.decision==="eligibility-uninformative"?"真实后果相对随机后果有部分方向信息，但原资格与具体连接的归属没有胜过保持同分布的置乱对照。高 applied 对齐主要来自软边界几何。":"判定依次隔离后果、资格归属、约束与内稳态。";
  byId("m1cd-components").replaceChildren(...truth.components.map(row=>{const article=document.createElement("article");if(row.component==="applied-plasticity")article.className="applied";if(row.component==="homeostasis-correction")article.className="homeostasis";article.innerHTML=`<header><span>${componentLabels[row.component]??row.component}</span><b>TRUE</b></header><dl><div><dt>oracle 余弦</dt><dd>${row.cosineAlignment.mean.toFixed(3)}</dd></div><div><dt>符号一致</dt><dd>${(row.signAgreement.mean*100).toFixed(1)}%</dd></div><div><dt>范数比</dt><dd>${row.normRatio.mean.toFixed(3)}</dd></div><div><dt>生产性投影</dt><dd>${row.productiveProjection.mean.toFixed(4)}</dd></div></dl>`;return article;}));
  const e=creditDecomposition.confirmationEffects;
  const effects:[string,Interval,string][]=[
    ["真实 − 随机",e.trueOverRandomTotalCosine,"total 余弦"],
    ["真实 − 随机",e.trueOverRandomTotalProjection,"生产性投影"],
    ["真实 − 置乱资格",e.trueOverShuffledTotalCosine,"total 余弦"],
    ["真实 − 置乱资格",e.trueOverShuffledTotalProjection,"生产性投影"],
  ];
  byId("m1cd-effects").replaceChildren(...effects.map(([label,value,note])=>{const article=document.createElement("article");article.className=value.lower95>0?"positive":value.upper95<0?"negative":"neutral";article.innerHTML=`<span>${label}</span><strong>${value.mean>=0?"+":""}${value.mean.toFixed(4)}</strong><small>${note} · [${value.lower95.toFixed(4)}, ${value.upper95.toFixed(4)}]</small>`;return article;}));
  byId("m1cd-checkpoints").replaceChildren(...creditDecomposition.confirmationCheckpointSummaries.map(row=>{const article=document.createElement("article");article.innerHTML=`<header><span>TRIAL ${row.trial}</span><strong>${row.trueTotalCosine.mean.toFixed(3)}</strong></header><dl><div><dt>total 余弦</dt><dd>[${row.trueTotalCosine.lower95.toFixed(3)}, ${row.trueTotalCosine.upper95.toFixed(3)}]</dd></div><div><dt>total 投影</dt><dd>${row.trueTotalProductiveProjection.mean.toFixed(4)}</dd></div><div><dt>applied 投影</dt><dd>${row.trueAppliedProductiveProjection.mean.toFixed(4)}</dd></div><div><dt>稳态损失</dt><dd>${(row.trueHomeostasisProjectionLoss.mean*100).toFixed(1)}%</dd></div></dl>`;return article;}));
  byId("m1cd-conclusions").replaceChildren(...creditDecomposition.conclusions.map(text=>{const p=document.createElement("p");p.textContent=text;return p;}));
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
gateFTraceSelect.addEventListener("change",selectGateFTrace);
gateFRange.addEventListener("input",()=>{gateFFrameIndex=Number(gateFRange.value);renderGateFFrame();});
map0Stage.addEventListener("change",()=>{const summaries=map0Summaries();map0SelectedId=summaries[0].parameters.id;renderMap0Inspector();});
map0X.addEventListener("change",drawMap0);map0Y.addEventListener("change",drawMap0);map0Parameter.addEventListener("change",()=>{map0SelectedId=Number(map0Parameter.value);renderMap0Inspector();});
map1X.addEventListener("change",drawMap1);map1Y.addEventListener("change",drawMap1);map1Parameter.addEventListener("change",()=>{map1SelectedId=Number(map1Parameter.value);renderMap1Inspector();});
gateBPlay.addEventListener("click",()=>{gateBPlaying=!gateBPlaying;gateBPlay.textContent=gateBPlaying?"Ⅱ":"▶";gateBLastTick=performance.now();});
byId("gate-b-prev").addEventListener("click",()=>setGateBFrame(gateBFrameIndex-1));byId("gate-b-next").addEventListener("click",()=>setGateBFrame(gateBFrameIndex+1));gateBRange.addEventListener("input",()=>setGateBFrame(Number(gateBRange.value)));
window.addEventListener("resize",()=>{if(ready){drawWorld();drawCurve();drawGateACurve();drawGateBWorld();drawGateBCurve();drawGateDBoundary();drawGateEBoundary();drawGateERaster();drawGateFCurve();drawMap0();drawMap1();}});

const animate = (now:number) => { if(playing && now-lastTick>=1000/Number(speed.value)){lastTick=now;if(frameIndex>=trace.frames.length-1){playing=false;play.textContent="▶";}else setFrame(frameIndex+1);}if(gateBPlaying&&now-gateBLastTick>=650){gateBLastTick=now;if(gateBFrameIndex>=gateBTrace.frames.length-1){gateBPlaying=false;gateBPlay.textContent="▶";}else setGateBFrame(gateBFrameIndex+1);}requestAnimationFrame(animate); };

const start = async () => {
  const [behaviorResponse,gateAResponse,gateBResponse,robustnessResponse,gateCResponse,gateDResponse,gateEResponse,gateERuntimeResponse,gateFResponse,map0Response,map1Response,map2aResponse,map2bResponse,map2cResponse,m0Response,m1Response,m2cMechanismResponse,structuralDiagnosticResponse,structuralTimescaleResponse,structuralGroupResponse,representationResponse,ruleFormationResponse,reachabilityResponse,reachabilityEnvelopeResponse,normEnabledResponse,creditDecompositionResponse]=await Promise.all([fetch("/embodied-v1.json"),fetch("/gate-a-v1.1.json"),fetch("/gate-b-v1.2.json"),fetch("/gate-b-robustness-v1.2b.json"),fetch("/gate-c-v1.3.json"),fetch("/gate-d-v1.4.json"),fetch("/gate-e-v1.5.json"),fetch("/gate-e-runtime-windows-x86_64.json"),fetch("/gate-f-v1.6.json"),fetch("/map0-v0.1.json"),fetch("/map1-v0.2.json"),fetch("/map2a-v0.3.json"),fetch("/map2b-v0.4.json"),fetch("/map2c-v0.5.json"),fetch("/mechanism-m0-v0.1.json"),fetch("/mechanism-m1-v0.2.json"),fetch("/mechanism-m2c-v0.3.json"),fetch("/structural-diagnostic-v0.4.json"),fetch("/structural-timescale-v0.5.json"),fetch("/structural-group-v0.6.json"),fetch("/representation-capacity-v0.7.json"),fetch("/rule-formation-v0.8.json"),fetch("/reachability-v0.9.json"),fetch("/reachability-envelope-v1.0.json"),fetch("/norm-enabled-sufficiency-v1.1.json"),fetch("/credit-decomposition-v1.2.json")]);
  if(!behaviorResponse.ok)throw new Error(`behavior dataset ${behaviorResponse.status}`);if(!gateAResponse.ok)throw new Error(`Gate A dataset ${gateAResponse.status}`);if(!gateBResponse.ok)throw new Error(`Gate B dataset ${gateBResponse.status}`);if(!robustnessResponse.ok)throw new Error(`Gate B robustness dataset ${robustnessResponse.status}`);if(!gateCResponse.ok)throw new Error(`Gate C dataset ${gateCResponse.status}`);if(!gateDResponse.ok)throw new Error(`Gate D dataset ${gateDResponse.status}`);if(!gateEResponse.ok)throw new Error(`Gate E dataset ${gateEResponse.status}`);if(!gateERuntimeResponse.ok)throw new Error(`Gate E runtime dataset ${gateERuntimeResponse.status}`);if(!gateFResponse.ok)throw new Error(`Gate F dataset ${gateFResponse.status}`);if(!map0Response.ok)throw new Error(`Map 0 dataset ${map0Response.status}`);if(!map1Response.ok)throw new Error(`Map 1 dataset ${map1Response.status}`);if(!map2aResponse.ok)throw new Error(`Map 2A dataset ${map2aResponse.status}`);if(!map2bResponse.ok)throw new Error(`Map 2B dataset ${map2bResponse.status}`);if(!map2cResponse.ok)throw new Error(`Map 2C dataset ${map2cResponse.status}`);if(!m0Response.ok)throw new Error(`M0 dataset ${m0Response.status}`);if(!m1Response.ok)throw new Error(`M1 dataset ${m1Response.status}`);if(!m2cMechanismResponse.ok)throw new Error(`M2C mechanism dataset ${m2cMechanismResponse.status}`);if(!structuralDiagnosticResponse.ok)throw new Error(`structural diagnostic dataset ${structuralDiagnosticResponse.status}`);if(!structuralTimescaleResponse.ok)throw new Error(`structural timescale dataset ${structuralTimescaleResponse.status}`);if(!structuralGroupResponse.ok)throw new Error(`structural group dataset ${structuralGroupResponse.status}`);if(!representationResponse.ok)throw new Error(`representation capacity dataset ${representationResponse.status}`);if(!ruleFormationResponse.ok)throw new Error(`rule formation dataset ${ruleFormationResponse.status}`);if(!reachabilityResponse.ok)throw new Error(`reachability dataset ${reachabilityResponse.status}`);if(!reachabilityEnvelopeResponse.ok)throw new Error(`reachability envelope dataset ${reachabilityEnvelopeResponse.status}`);if(!normEnabledResponse.ok)throw new Error(`norm-enabled sufficiency dataset ${normEnabledResponse.status}`);if(!creditDecompositionResponse.ok)throw new Error(`credit decomposition dataset ${creditDecompositionResponse.status}`);
  dataset=await behaviorResponse.json() as Dataset;gateA=await gateAResponse.json() as GateADataset;gateB=await gateBResponse.json() as GateBDataset;robustness=await robustnessResponse.json() as RobustnessDataset;gateC=await gateCResponse.json() as GateCDataset;gateD=await gateDResponse.json() as GateDDataset;gateE=await gateEResponse.json() as GateEDataset;gateERuntime=await gateERuntimeResponse.json() as GateERuntime[];gateF=await gateFResponse.json() as GateFDataset;map0=await map0Response.json() as Map0Dataset;map1=await map1Response.json() as Map1Dataset;map2a=await map2aResponse.json() as Map2ADataset;map2b=await map2bResponse.json() as Map2BDataset;map2c=await map2cResponse.json() as Map2CDataset;mechanismM0=await m0Response.json() as M0Dataset;mechanismM1=await m1Response.json() as M1RetentionDataset;mechanismM2C=await m2cMechanismResponse.json() as M2CStructureDataset;structuralDiagnostic=await structuralDiagnosticResponse.json() as StructuralDiagnosticDataset;structuralTimescale=await structuralTimescaleResponse.json() as StructuralTimescaleDataset;structuralGroup=await structuralGroupResponse.json() as StructuralGroupDataset;representationCapacity=await representationResponse.json() as RepresentationCapacityDataset;ruleFormation=await ruleFormationResponse.json() as M1FDataset;reachability=await reachabilityResponse.json() as M1XDataset;reachabilityEnvelope=await reachabilityEnvelopeResponse.json() as M1XEDataset;normEnabledSufficiency=await normEnabledResponse.json() as M1NEDataset;creditDecomposition=await creditDecompositionResponse.json() as M1CDDataset;
  ready=true;
  byId("version").textContent=map2c.version;byId("acceptance-label").textContent=map2c.acceptance.stagePassed?"Map 2 完成 · 连续流稳定区域已确认":"Map 2C 完成 · 未达到冻结标准";byId("acceptance-dot").className=map2c.acceptance.passed?"pass":"fail";
  traceSelect.replaceChildren(...dataset.traces.map(item=>{const option=document.createElement("option");option.value=item.label;option.textContent=labels[item.label]??item.label;if(item.label==="learned")option.selected=true;return option;}));
  renderEvidence();renderGateA();renderGateB();renderRobustness();renderGateC();renderGateD();renderGateE();renderGateF();renderMap0();renderMap1();renderMap2();renderMechanismM0();renderMechanismM1();renderMechanismM2C();renderStructuralDiagnostic();renderStructuralTimescale();renderStructuralGroup();renderRepresentationCapacity();renderRuleFormation();renderReachability();renderReachabilityEnvelope();renderNormEnabledSufficiency();renderCreditDecomposition();drawCurve();selectTrace();requestAnimationFrame(animate);
};

start().catch(error=>{document.body.innerHTML=`<pre class="fatal">无法载入具身实验：${String(error)}</pre>`;console.error(error);});
