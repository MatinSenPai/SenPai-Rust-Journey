//! DELIBERATELY BROKEN — expected: E0594.
//!
//!     cargo run -p p2-06-05-refcell-and-interior-mutability --example 05-mutate-through-shared-self --features broken

struct PageViews {
    count: u32,
}

impl PageViews {
    fn record_view(&self) {
        self.count += 1;
    }
}

fn main() {
    let page = PageViews { count: 0 };
    page.record_view();
}
