use crate::keyboard::resolve_trigger_key;
use async_trait::async_trait;
use dsl::{Condition, ConditionEvaluator};
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct KeyConditionEvaluator {
    pub held_keys: Arc<Mutex<BTreeSet<u16>>>,
}

#[async_trait]
impl ConditionEvaluator for KeyConditionEvaluator {
    async fn evaluate(&self, condition: &Condition) -> bool {
        match condition {
            Condition::NowInput(combos) => {
                let held = self.held_keys.lock().unwrap();
                for combo in combos {
                    let mut all_match = true;
                    for tk in &combo.0 {
                        if let Some(sc) = resolve_trigger_key(tk) {
                            if !held.contains(&sc) {
                                all_match = false;
                                break;
                            }
                        } else {
                            all_match = false;
                            break;
                        }
                    }
                    if all_match {
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }
}
