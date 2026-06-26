use std::sync::{Arc, Mutex};

use pcmanx_rs::script::{ScriptEngine, ScriptHandler};

struct RecordingHandler {
    calls: Arc<Mutex<Vec<(usize, String)>>>,
}

impl ScriptHandler for RecordingHandler {
    fn on_new_incoming_message(&self, session_id: usize, text: &str) {
        self.calls.lock().unwrap().push((session_id, text.to_string()));
    }
}

#[test]
fn script_engine_dispatches_to_handler() {
    let calls: Arc<Mutex<Vec<(usize, String)>>> = Arc::new(Mutex::new(Vec::new()));
    let handler = RecordingHandler { calls: calls.clone() };

    let mut engine = ScriptEngine::new();
    engine.register(Box::new(handler));
    engine.on_new_incoming_message(42, "hello bbs");

    let recorded = calls.lock().unwrap();
    assert_eq!(recorded.len(), 1);
    assert_eq!(recorded[0].0, 42);
    assert_eq!(recorded[0].1, "hello bbs");
}

#[test]
fn script_engine_dispatches_to_multiple_handlers() {
    let calls1: Arc<Mutex<Vec<(usize, String)>>> = Arc::new(Mutex::new(Vec::new()));
    let calls2: Arc<Mutex<Vec<(usize, String)>>> = Arc::new(Mutex::new(Vec::new()));

    let mut engine = ScriptEngine::new();
    engine.register(Box::new(RecordingHandler { calls: calls1.clone() }));
    engine.register(Box::new(RecordingHandler { calls: calls2.clone() }));
    engine.on_new_incoming_message(1, "test");

    assert_eq!(calls1.lock().unwrap().len(), 1);
    assert_eq!(calls2.lock().unwrap().len(), 1);
}

#[test]
fn script_engine_no_handlers_does_not_panic() {
    let engine = ScriptEngine::new();
    engine.on_new_incoming_message(0, "no handlers");
}
