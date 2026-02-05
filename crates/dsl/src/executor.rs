use crate::ast::*;
use async_trait::async_trait;
use futures::FutureExt;
use futures::future::BoxFuture;
use std::collections::HashMap;
use std::fmt;
use std::process::Command;
use std::sync::Arc;
use tokio::time::{Duration, sleep};

#[async_trait]
pub trait InputSimulator: Send + Sync + fmt::Debug {
    async fn send_keys(&self, expressions: &[SendExpression]);
}

#[async_trait]
pub trait ConditionEvaluator: Send + Sync + fmt::Debug {
    async fn evaluate(&self, condition: &Condition) -> bool;
}

pub struct Executor {
    global_settings: Vec<GlobalSetting>,
    macros: HashMap<String, Vec<Statement>>,
    input_sim: Arc<dyn InputSimulator>,
    cond_eval: Arc<dyn ConditionEvaluator>,
}

impl fmt::Debug for Executor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Executor")
            .field("global_settings", &self.global_settings)
            .field("macros", &self.macros)
            .field("input_sim", &self.input_sim)
            .field("cond_eval", &self.cond_eval)
            .finish()
    }
}

impl Executor {
    pub fn new(
        script: Script,
        input_sim: Arc<dyn InputSimulator>,
        cond_eval: Arc<dyn ConditionEvaluator>,
    ) -> Self {
        let mut macros = HashMap::new();
        for m in script.macros {
            macros.insert(m.name, m.body);
        }
        Self {
            global_settings: script.global_settings,
            macros,
            input_sim,
            cond_eval,
        }
    }

    pub async fn execute_block(&self, block: &Block) {
        self.execute_statements(&block.body).await;
    }

    pub async fn execute_statements(&self, statements: &[Statement]) {
        for stmt in statements {
            self.execute_statement(stmt).await;
        }
    }

    pub fn execute_statement<'a>(&'a self, stmt: &'a Statement) -> BoxFuture<'a, ()> {
        async move {
            match stmt {
                Statement::Run(cmd) => {
                    let _ = Command::new("cmd").args(["/C", cmd]).spawn();
                }
                Statement::Execute(cmd) => {
                    let _ = Command::new("powershell").args(["-Command", cmd]).spawn();
                }
                Statement::TryRun { command, failure } => {
                    match Command::new("cmd").args(["/C", command]).status() {
                        Ok(status) if status.success() => {}
                        _ => {
                            if let Some(f) = failure {
                                self.execute_statement(f).await;
                            }
                        }
                    }
                }
                Statement::TryExecute { command, failure } => {
                    match Command::new("powershell")
                        .args(["-Command", command])
                        .status()
                    {
                        Ok(status) if status.success() => {}
                        _ => {
                            if let Some(f) = failure {
                                self.execute_statement(f).await;
                            }
                        }
                    }
                }
                Statement::Send(exprs) => {
                    self.input_sim.send_keys(exprs).await;
                }
                Statement::Wait(ms) => {
                    sleep(Duration::from_millis(*ms)).await;
                }
                Statement::If {
                    condition,
                    then_branch,
                    else_if_branches,
                    else_branch,
                } => {
                    if self.cond_eval.evaluate(condition).await {
                        self.execute_statements(then_branch).await;
                    } else {
                        let mut matched = false;
                        for (c, b) in else_if_branches {
                            if self.cond_eval.evaluate(c).await {
                                self.execute_statements(b).await;
                                matched = true;
                                break;
                            }
                        }
                        if !matched {
                            if let Some(b) = else_branch {
                                self.execute_statements(b).await;
                            }
                        }
                    }
                }
                Statement::Loop { count, body } => {
                    for _ in 0..*count {
                        self.execute_statements(body).await;
                    }
                }
                Statement::MacroCall(name) => {
                    if let Some(body) = self.macros.get(name) {
                        self.execute_statements(body).await;
                    }
                }
            }
        }
        .boxed()
    }
}
