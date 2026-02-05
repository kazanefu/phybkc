use crate::keyboard::{resolve_trigger_key, send_key_event, send_unicode_char};
use async_trait::async_trait;
use dsl::{InputSimulator, SendExpression};
use std::collections::BTreeSet;

#[derive(Debug)]
pub struct WindowsInputSimulator;

#[async_trait]
impl InputSimulator for WindowsInputSimulator {
    async fn send_keys(&self, expressions: &[SendExpression]) {
        let mut to_release = BTreeSet::new();
        for expr in expressions {
            match expr {
                SendExpression::Key(k) => {
                    if let Some(sc) = resolve_trigger_key(k) {
                        unsafe {
                            send_key_event(sc, true, false);
                            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                            send_key_event(sc, false, false);
                        }
                    }
                }
                SendExpression::Hold(k) => {
                    if let Some(sc) = resolve_trigger_key(k) {
                        unsafe {
                            send_key_event(sc, true, false);
                        }
                        to_release.insert(sc);
                    }
                }
                SendExpression::Release(k) => {
                    if let Some(sc) = resolve_trigger_key(k) {
                        unsafe {
                            send_key_event(sc, false, false);
                        }
                        to_release.remove(&sc);
                    }
                }
                SendExpression::String(s) => {
                    for c in s.chars() {
                        unsafe {
                            send_unicode_char(c);
                        }
                    }
                }
                SendExpression::Combo(keys) => {
                    let mut scancodes = Vec::new();
                    for k in keys {
                        if let Some(sc) = resolve_trigger_key(k) {
                            scancodes.push(sc);
                            unsafe {
                                send_key_event(sc, true, false);
                            }
                            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                        }
                    }
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    for sc in scancodes.into_iter().rev() {
                        unsafe {
                            send_key_event(sc, false, false);
                        }
                        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    }
                }
            }
        }
        // Automatic release at the end of the Send statement (at ;)
        for sc in to_release {
            unsafe {
                send_key_event(sc, false, false);
            }
        }
    }
}
