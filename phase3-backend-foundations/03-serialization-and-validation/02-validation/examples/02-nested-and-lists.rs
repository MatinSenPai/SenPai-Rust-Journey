//! Nested structs and lists: the error tree has three kinds of node.
//! Run: `cargo run -p p3-03-02-validation --example 02-nested-and-lists`

use validator::{Validate, ValidationErrors, ValidationErrorsKind};

#[derive(Debug, Validate)]
struct Note {
    #[validate(length(min = 1))]
    text: String,
}

#[derive(Debug, Validate)]
struct Review {
    #[validate(length(min = 1))]
    title: String,
    #[validate(nested)]
    first_note: Note,
    #[validate(nested)]
    notes: Vec<Note>,
}

fn show(errors: &ValidationErrors, depth: usize) {
    let mut entries: Vec<_> = errors.errors().iter().collect();
    entries.sort_by_key(|(name, _)| **name);
    for (name, kind) in entries {
        let pad = "  ".repeat(depth);
        match kind {
            ValidationErrorsKind::Field(list) => println!("{pad}{name}: Field({})", list.len()),
            ValidationErrorsKind::Struct(inner) => {
                println!("{pad}{name}: Struct");
                show(inner, depth + 1);
            }
            ValidationErrorsKind::List(items) => {
                println!("{pad}{name}: List");
                for (i, inner) in items {
                    println!("{pad}  [{i}]");
                    show(inner, depth + 2);
                }
            }
        }
    }
}

fn main() {
    let review = Review {
        title: String::new(),
        first_note: Note {
            text: String::new(),
        },
        notes: vec![
            Note { text: "ok".into() },
            Note {
                text: String::new(),
            },
        ],
    };
    show(&review.validate().unwrap_err(), 0);
}
