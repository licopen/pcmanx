#![allow(dead_code)]

pub trait ScriptHandler: Send + Sync {
    fn on_new_incoming_message(&self, session_id: usize, text: &str);
}

pub struct ScriptEngine {
    handlers: Vec<Box<dyn ScriptHandler>>,
}

impl Default for ScriptEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptEngine {
    pub fn new() -> Self {
        Self { handlers: Vec::new() }
    }

    pub fn register(&mut self, handler: Box<dyn ScriptHandler>) {
        self.handlers.push(handler);
    }

    pub fn on_new_incoming_message(&self, session_id: usize, text: &str) {
        for handler in &self.handlers {
            handler.on_new_incoming_message(session_id, text);
        }
    }
}
