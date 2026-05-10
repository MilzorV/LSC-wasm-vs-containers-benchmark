use std::cell::RefCell;

use meili_spin_core::SearchEngine;

thread_local! {
    static ENGINE: RefCell<SearchEngine> = RefCell::new(SearchEngine::new());
}

pub fn with_engine<T>(operation: impl FnOnce(&mut SearchEngine) -> T) -> T {
    ENGINE.with(|engine| operation(&mut engine.borrow_mut()))
}
