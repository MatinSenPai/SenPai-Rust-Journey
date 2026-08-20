//! Locale parsing and every piece of interface copy.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    Fa,
    En,
}

impl Locale {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "fa" => Some(Self::Fa),
            "en" => Some(Self::En),
            _ => None,
        }
    }

    pub const fn code(self) -> &'static str {
        match self {
            Self::Fa => "fa",
            Self::En => "en",
        }
    }

    pub const fn dir(self) -> &'static str {
        match self {
            Self::Fa => "rtl",
            Self::En => "ltr",
        }
    }

    pub const fn other(self) -> Self {
        match self {
            Self::Fa => Self::En,
            Self::En => Self::Fa,
        }
    }

    pub const fn is_fa(self) -> bool {
        matches!(self, Self::Fa)
    }

    pub const fn home(self) -> &'static str {
        if self.is_fa() {
            "خانه"
        } else {
            "Home"
        }
    }

    pub const fn search(self) -> &'static str {
        if self.is_fa() {
            "جست‌وجو"
        } else {
            "Search"
        }
    }

    pub const fn search_placeholder(self) -> &'static str {
        if self.is_fa() {
            "جست‌وجوی درس، مالکیت، async…"
        } else {
            "Search lessons, ownership, async…"
        }
    }

    pub const fn skip(self) -> &'static str {
        if self.is_fa() {
            "پرش به محتوای اصلی"
        } else {
            "Skip to main content"
        }
    }

    pub const fn course_map(self) -> &'static str {
        if self.is_fa() {
            "نقشه‌ی دوره"
        } else {
            "Course map"
        }
    }

    pub const fn contents(self) -> &'static str {
        if self.is_fa() {
            "محتوا"
        } else {
            "Contents"
        }
    }

    pub const fn solution(self) -> &'static str {
        if self.is_fa() {
            "دیدن راه‌حل مرجع — اول خودت واقعاً تلاش کن"
        } else {
            "Show the reference solution — have a real attempt first"
        }
    }

    pub const fn mark_complete(self) -> &'static str {
        if self.is_fa() {
            "ثبت پایان درس"
        } else {
            "Mark complete"
        }
    }

    pub const fn mark_incomplete(self) -> &'static str {
        if self.is_fa() {
            "برگرداندن به ناتمام"
        } else {
            "Mark incomplete"
        }
    }

    pub const fn complete_next(self) -> &'static str {
        if self.is_fa() {
            "تمام شد؛ درس بعدی"
        } else {
            "Complete & next lesson"
        }
    }

    pub const fn completed(self) -> &'static str {
        if self.is_fa() {
            "تکمیل‌شده"
        } else {
            "Completed"
        }
    }

    pub const fn previous(self) -> &'static str {
        if self.is_fa() {
            "درس قبلی"
        } else {
            "Previous lesson"
        }
    }

    pub const fn next(self) -> &'static str {
        if self.is_fa() {
            "درس بعدی"
        } else {
            "Next lesson"
        }
    }

    // ---------------------------------------------------------- progress

    pub const fn progress(self) -> &'static str {
        if self.is_fa() {
            "پیشرفت"
        } else {
            "Progress"
        }
    }

    pub const fn theme_toggle(self) -> &'static str {
        if self.is_fa() {
            "جابه‌جایی بین تم روشن و تاریک"
        } else {
            "Toggle light and dark theme"
        }
    }

    pub const fn progress_title(self) -> &'static str {
        if self.is_fa() {
            "تا کجا آمده‌ای"
        } else {
            "How far you've come"
        }
    }

    pub const fn progress_intro(self) -> &'static str {
        if self.is_fa() {
            "این صفحه برای این نیست که بهت نمره بده. برای اینه که ببینی کدام مفهوم‌ها واقعاً مالِ تو شدن و کدام‌ها هنوز جا دارن."
        } else {
            "This page isn't scoring you. It's here so you can see which concepts are genuinely yours and which still have room."
        }
    }

    pub const fn stat_complete(self) -> &'static str {
        if self.is_fa() {
            "تکمیل‌شده"
        } else {
            "Complete"
        }
    }

    pub const fn stat_lessons(self) -> &'static str {
        if self.is_fa() {
            "درس‌ها"
        } else {
            "Lessons"
        }
    }

    pub const fn stat_streak(self) -> &'static str {
        if self.is_fa() {
            "روزهای پشت‌سرهم"
        } else {
            "Day streak"
        }
    }

    pub const fn stat_time(self) -> &'static str {
        if self.is_fa() {
            "زمان تقریبی"
        } else {
            "Estimated time"
        }
    }

    pub const fn hours(self) -> &'static str {
        if self.is_fa() {
            "ساعت"
        } else {
            "h"
        }
    }

    pub const fn days(self) -> &'static str {
        if self.is_fa() {
            "روز"
        } else {
            "days"
        }
    }

    pub const fn by_phase(self) -> &'static str {
        if self.is_fa() {
            "فاز به فاز"
        } else {
            "Phase by phase"
        }
    }

    pub const fn mastery(self) -> &'static str {
        if self.is_fa() {
            "مفهوم‌هایی که دیده‌ای"
        } else {
            "Concepts you've met"
        }
    }

    pub const fn mastery_note(self) -> &'static str {
        if self.is_fa() {
            "از روی docs/concept-map.toml ساخته می‌شود: هر مفهوم دقیقاً همان‌جایی علامت می‌خورد که درسِ معرفی‌کننده‌اش را تمام کرده‌ای — نه جایی که فقط اسمش را شنیده‌ای."
        } else {
            "Built from docs/concept-map.toml: a concept lights up when you finish the lesson that teaches it, not when you first hear the word."
        }
    }

    /// `{n}` is substituted with a count at render time.
    pub const fn mastery_unwritten(self) -> &'static str {
        if self.is_fa() {
            "({n} مفهومِ دیگر هم در نقشه هست که درسِ معرفی‌کننده‌شان هنوز نوشته نشده.)"
        } else {
            "({n} more concepts are in the map but their lesson isn't written yet.)"
        }
    }

    pub const fn recent_activity(self) -> &'static str {
        if self.is_fa() {
            "تازه‌ترین‌ها"
        } else {
            "Recent activity"
        }
    }

    pub const fn nothing_yet(self) -> &'static str {
        if self.is_fa() {
            "هنوز چیزی ثبت نشده. اولین درس را تمام کن تا اینجا پر شود."
        } else {
            "Nothing recorded yet. Finish your first lesson and this fills in."
        }
    }

    // -------------------------------------------------------- self-check

    pub const fn self_check(self) -> &'static str {
        if self.is_fa() {
            "یادداشت خودت"
        } else {
            "Your own notes"
        }
    }

    pub const fn exercise_progress(self) -> &'static str {
        if self.is_fa() {
            "پله‌های تمرین"
        } else {
            "Exercise rungs"
        }
    }

    pub const fn confidence(self) -> &'static str {
        if self.is_fa() {
            "چقدر برایت جا افتاد؟"
        } else {
            "How settled does it feel?"
        }
    }

    pub const fn confidence_levels(self) -> [&'static str; 3] {
        if self.is_fa() {
            ["باید برگردم", "تقریباً", "محکم"]
        } else {
            ["Revisit", "Almost", "Solid"]
        }
    }

    pub const fn note_label(self) -> &'static str {
        if self.is_fa() {
            "یادداشت (فقط برای خودت)"
        } else {
            "Note to yourself"
        }
    }

    pub const fn note_placeholder(self) -> &'static str {
        if self.is_fa() {
            "چه چیزی گیرت انداخت؟ چه چیزی بالاخره کلیک کرد؟"
        } else {
            "What tripped you up? What finally clicked?"
        }
    }

    pub const fn save(self) -> &'static str {
        if self.is_fa() {
            "ذخیره"
        } else {
            "Save"
        }
    }

    pub const fn translation_missing(self) -> &'static str {
        if self.is_fa() {
            "ترجمه‌ی این بخش هنوز آماده نیست؛ فعلاً متن انگلیسی را می‌بینی."
        } else {
            "This section is not available in English."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_only_supported_locales() {
        assert_eq!(Locale::parse("fa"), Some(Locale::Fa));
        assert_eq!(Locale::parse("en"), Some(Locale::En));
        assert_eq!(Locale::parse("de"), None);
    }
}
