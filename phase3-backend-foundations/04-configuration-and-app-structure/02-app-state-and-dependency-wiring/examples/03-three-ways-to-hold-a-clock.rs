//! The same dependency, wired three ways: a concrete type, a generic
//! parameter, and a trait object. The behaviour is identical; what differs
//! is how far the choice spreads through the code.
//!
//!     cargo run -p p3-04-02-app-state-and-dependency-wiring --example 03-three-ways-to-hold-a-clock

use std::sync::Arc;

trait Clock: Send + Sync {
    fn now(&self) -> u64;
}

struct Fixed(u64);

impl Clock for Fixed {
    fn now(&self) -> u64 {
        self.0
    }
}

// 1. Concrete: nothing to choose, nothing to swap.
#[derive(Clone)]
struct ConcreteState {
    clock: Arc<Fixed>,
}

// 2. Generic: the choice is a type parameter on the state, and on every
//    function and handler that names the state.
#[derive(Clone)]
struct GenericState<C> {
    clock: Arc<C>,
}

fn stamp_generic<C: Clock>(state: &GenericState<C>) -> u64 {
    state.clock.now()
}

// 3. Trait object: the choice is made at run time, and the state's type
//    stays a plain `DynState`.
#[derive(Clone)]
struct DynState {
    clock: Arc<dyn Clock>,
}

fn stamp_dyn(state: &DynState) -> u64 {
    state.clock.now()
}

fn main() {
    let concrete = ConcreteState {
        clock: Arc::new(Fixed(100)),
    };
    println!("concrete: {}", concrete.clock.now());
    println!("its type: {}", std::any::type_name_of_val(&concrete));

    let generic = GenericState {
        clock: Arc::new(Fixed(200)),
    };
    println!("generic:  {}", stamp_generic(&generic));
    println!("its type: {}", std::any::type_name_of_val(&generic));

    let dynamic = DynState {
        clock: Arc::new(Fixed(300)),
    };
    println!("dyn:      {}", stamp_dyn(&dynamic));
    println!("its type: {}", std::any::type_name_of_val(&dynamic));
}
