use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Script {
    pub global_settings: Vec<GlobalSetting>,
    pub macros: Vec<Macro>,
    pub blocks: Vec<Block>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum GlobalSetting {
    Cli(String),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Macro {
    pub name: String,
    pub body: Vec<Statement>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Block {
    pub triggers: Vec<TriggerCombinations>,
    pub body: Vec<Statement>,
}

// A trigger can be a single key or a combination (e.g. #0x01 + Code_A)
// In design.md: #0x02 + Code_A
// We can represent this as a list of keys required to be active.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct TriggerCombinations(pub Vec<TriggerKey>);

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum TriggerKey {
    Physical(u16),         // #0x...
    ExtendedPhysical(u16), // #E0/0x... (e.g. #E0/0x2E)
    Virtual(String),       // Code_... or plain name
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Statement {
    Run(String),
    Execute(String),
    TryRun {
        command: String,
        failure: Option<Box<Statement>>, // Can be FailRun or FailExecute
    },
    TryExecute {
        command: String,
        failure: Option<Box<Statement>>,
    },
    Send(Vec<SendExpression>),
    Wait(u64),
    If {
        condition: Condition,
        then_branch: Vec<Statement>,
        else_if_branches: Vec<(Condition, Vec<Statement>)>,
        else_branch: Option<Vec<Statement>>,
    },
    Loop {
        count: usize,
        body: Vec<Statement>,
    },
    MacroCall(String),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Condition {
    WaitInput(Vec<TriggerCombinations>),
    WaitInputTime(Vec<TriggerCombinations>, u64),
    NowInput(Vec<TriggerCombinations>),
    WaitReleased(Vec<TriggerCombinations>),
    WaitReleasedTime(Vec<TriggerCombinations>, u64),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum SendExpression {
    Key(TriggerKey),
    String(String),
    Combo(Vec<TriggerKey>), // Key + Key
}
