pub struct Question {
    pub text: String,
    pub options: [String; 4],
    /// 0-indexed position of the correct option within `options`.
    pub correct_index: usize,
}

/// The built-in trivia bank. Already implemented — add your own once the
/// rest of this file works.
pub fn question_bank() -> Vec<Question> {
    let raw: [(&str, [&str; 4], usize); 5] = [
        (
            "Who is the captain of the Straw Hat Pirates?",
            ["Zoro", "Sanji", "Luffy", "Nami"],
            2,
        ),
        (
            "What is the name of the Scout Regiment's strongest soldier in Attack on Titan?",
            ["Levi", "Erwin", "Mikasa", "Armin"],
            0,
        ),
        (
            "In Fullmetal Alchemist, what is the First Law of Equivalent Exchange about?",
            [
                "Time travel",
                "To gain something, something of equal value must be lost",
                "Alchemy requires a partner",
                "Only metal can be transmuted",
            ],
            1,
        ),
        (
            "What is Death Note's protagonist's real name?",
            ["L", "Near", "Light Yagami", "Ryuk"],
            2,
        ),
        (
            "In Chainsaw Man, what devil does Denji merge with?",
            ["Gun Devil", "Chainsaw Devil", "Blood Devil", "Fire Devil"],
            1,
        ),
    ];

    raw.into_iter()
        .map(|(text, options, correct_index)| Question {
            text: text.to_string(),
            options: options.map(|s| s.to_string()),
            correct_index,
        })
        .collect()
}

/// Formats a question as `"Q{n}: {text}\n1) ...\n2) ...\n3) ...\n4) ..."`
/// (1-indexed for display, even though `correct_index` is 0-indexed
/// internally).
pub fn format_question(number: usize, question: &Question) -> String {
    todo!(
        "format! \"Q{{number}}: {{text}}\" then one \"{{i}}) {{option}}\" line per option, 1-indexed"
    )
}

pub struct QuizSession {
    questions: Vec<Question>,
    current: usize,
    score: u32,
}

impl QuizSession {
    pub fn new(questions: Vec<Question>) -> Self {
        QuizSession {
            questions,
            current: 0,
            score: 0,
        }
    }

    /// The question currently being asked, or `None` if the quiz is over.
    pub fn current_question(&self) -> Option<&Question> {
        todo!("self.questions.get(self.current)")
    }

    /// The current question's 1-indexed position, for passing to
    /// `format_question`. Already implemented — a thin accessor, not part
    /// of this lesson's exercise.
    pub fn current_question_number(&self) -> usize {
        self.current + 1
    }

    /// Submits an answer (1-indexed, matching `format_question`'s display)
    /// for the current question: increments the score if correct, and
    /// advances to the next question either way. Returns `true` if the
    /// answer was correct. Does nothing (returns `false`) if the quiz is
    /// already finished.
    pub fn answer(&mut self, answer_1_indexed: usize) -> bool {
        todo!(
            "look up current_question(); compare answer_1_indexed - 1 to correct_index; \
             if correct, self.score += 1; either way self.current += 1; return whether it was correct"
        )
    }

    pub fn is_finished(&self) -> bool {
        todo!("self.current >= self.questions.len()")
    }

    pub fn score_summary(&self) -> String {
        todo!("format!(\"You scored {{score}}/{{total}}\", using self.score and self.questions.len())")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_questions() -> Vec<Question> {
        vec![
            Question {
                text: "2 + 2?".to_string(),
                options: [
                    "3".to_string(),
                    "4".to_string(),
                    "5".to_string(),
                    "6".to_string(),
                ],
                correct_index: 1,
            },
            Question {
                text: "Capital of France?".to_string(),
                options: [
                    "Berlin".to_string(),
                    "Madrid".to_string(),
                    "Paris".to_string(),
                    "Rome".to_string(),
                ],
                correct_index: 2,
            },
        ]
    }

    #[test]
    fn formats_a_question_with_one_indexed_options() {
        let q = &sample_questions()[0];
        let formatted = format_question(1, q);
        assert!(formatted.contains("Q1: 2 + 2?"));
        assert!(formatted.contains("1) 3"));
        assert!(formatted.contains("2) 4"));
    }

    #[test]
    fn starts_at_the_first_question_with_zero_score() {
        let session = QuizSession::new(sample_questions());
        assert_eq!(session.current_question().unwrap().text, "2 + 2?");
        assert!(!session.is_finished());
    }

    #[test]
    fn correct_answer_increments_score_and_advances() {
        let mut session = QuizSession::new(sample_questions());
        let was_correct = session.answer(2); // "4" is option 2 (1-indexed)
        assert!(was_correct);
        assert_eq!(
            session.current_question().unwrap().text,
            "Capital of France?"
        );
    }

    #[test]
    fn wrong_answer_does_not_increment_score_but_still_advances() {
        let mut session = QuizSession::new(sample_questions());
        let was_correct = session.answer(1); // "3" is wrong
        assert!(!was_correct);
        assert_eq!(
            session.current_question().unwrap().text,
            "Capital of France?"
        );
    }

    #[test]
    fn finishes_after_the_last_question_and_reports_score() {
        let mut session = QuizSession::new(sample_questions());
        session.answer(2); // correct
        session.answer(3); // correct
        assert!(session.is_finished());
        assert!(session.current_question().is_none());
        assert_eq!(session.score_summary(), "You scored 2/2");
    }

    #[test]
    fn answering_after_finished_does_nothing() {
        let mut session = QuizSession::new(sample_questions());
        session.answer(2);
        session.answer(3);
        assert!(!session.answer(1));
        assert_eq!(session.score_summary(), "You scored 2/2");
    }
}
