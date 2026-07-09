use std::cell::RefCell;

thread_local! {
    static DROP_LOG: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

fn log_drop(name: &str) {
    DROP_LOG.with(|log| log.borrow_mut().push(name.to_string()));
}

fn take_log() -> Vec<String> {
    DROP_LOG.with(|log| log.borrow_mut().drain(..).collect())
}

pub struct Tracker {
    name: String,
}

impl Tracker {
    pub fn new(name: &str) -> Self {
        Tracker {
            name: name.to_string(),
        }
    }
}

impl Drop for Tracker {
    fn drop(&mut self) {
        log_drop(&self.name);
    }
}

pub fn drop_order_in_one_scope() -> Vec<String> {
    take_log();
    {
        let _a = Tracker::new("a");
        let _b = Tracker::new("b");
        let _c = Tracker::new("c");
    }
    take_log()
}

pub fn early_drop_demo() -> Vec<String> {
    take_log();
    {
        let first = Tracker::new("first");
        let _second = Tracker::new("second"); // dropped naturally at block end
        drop(first);
    }
    take_log()
}

fn create_tracker(name: &str) -> Tracker {
    Tracker::new(name)
}

pub fn move_extends_lifetime() -> Vec<String> {
    take_log();
    {
        let _t = create_tracker("moved");
    }
    take_log()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_in_reverse_declaration_order() {
        assert_eq!(
            drop_order_in_one_scope(),
            vec!["c".to_string(), "b".to_string(), "a".to_string()]
        );
    }

    #[test]
    fn early_drop_reorders() {
        assert_eq!(
            early_drop_demo(),
            vec!["first".to_string(), "second".to_string()]
        );
    }

    #[test]
    fn move_delays_drop_until_outer_scope_ends() {
        assert_eq!(move_extends_lifetime(), vec!["moved".to_string()]);
    }
}
