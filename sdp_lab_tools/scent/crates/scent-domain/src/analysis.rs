//! Enumerations shared across analysis stages.

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Language {
    CSharp,
    Cpp,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Visibility {
    Public,
    Protected,
    Internal,
    Private,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AnalysisScope {
    Project,
    File,
    Type,
    Method,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

/// The design principles the Principle Risk Engine assesses
/// (`docs/prompt.md` §25). Not every variant has a rule feeding it yet —
/// see `scent_rules::principles` for which ones do, and why the rest are
/// deferred rather than fabricated.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Principle {
    Srp,
    Ocp,
    Lsp,
    Isp,
    Dip,
    Dry,
    Kiss,
    Yagni,
    LawOfDemeter,
}
