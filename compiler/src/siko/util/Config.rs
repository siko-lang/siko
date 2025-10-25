#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildPhase {
    Run,
    BuildSource,
    Build,
}

#[derive(Debug, Clone)]
pub enum OptimizationLevel {
    None,
    O3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetOS {
    Linux,
    MacOS,
    Windows,
}
#[derive(Debug, Clone)]
pub struct DumpConfig {
    pub dumpPreTypecheck: bool,
    pub dumpAfterTypecheck: bool,
    pub borrowCheckerTraceEnabled: bool,
    pub instanceResolverTraceEnabled: bool,
    pub functionProfileBuilderTraceEnabled: bool,
    pub simplifierTraceEnabled: bool,
    pub unusedAssignmentEliminatorTraceEnabled: bool,
    pub dumpAfterDropCheck: bool,
    pub functionCallResolverTraceEnabled: bool,
    pub resolverTraceEnabled: bool,
}

impl DumpConfig {
    pub fn new() -> Self {
        DumpConfig {
            dumpPreTypecheck: false,
            dumpAfterTypecheck: false,
            borrowCheckerTraceEnabled: false,
            instanceResolverTraceEnabled: false,
            functionProfileBuilderTraceEnabled: false,
            simplifierTraceEnabled: false,
            unusedAssignmentEliminatorTraceEnabled: false,
            dumpAfterDropCheck: false,
            functionCallResolverTraceEnabled: false,
            resolverTraceEnabled: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub testOnly: bool,
    pub passDetails: u32,
    pub optimization: OptimizationLevel,
    pub buildPhase: BuildPhase,
    pub sanitized: bool,
    pub inputFiles: Vec<String>,
    pub externalFiles: Vec<String>,
    pub outputFile: String,
    pub keepCSource: bool,
    pub targetOS: TargetOS,
    pub dumpCfg: DumpConfig,
    pub enableInliner: bool,
    pub disableSafetyChecks: bool,
    pub dumpFinalHIR: bool,
}

impl Config {
    pub fn new() -> Self {
        Config {
            testOnly: false,
            passDetails: 0,
            optimization: OptimizationLevel::None,
            buildPhase: BuildPhase::Run,
            sanitized: false,
            inputFiles: Vec::new(),
            externalFiles: Vec::new(),
            outputFile: format!("siko_main"),
            keepCSource: false,
            targetOS: TargetOS::Linux,
            dumpCfg: DumpConfig::new(),
            enableInliner: true,
            disableSafetyChecks: false,
            dumpFinalHIR: false,
        }
    }
}
