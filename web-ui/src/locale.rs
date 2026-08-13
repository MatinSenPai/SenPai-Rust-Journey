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
            "دیدن راه‌حل مرجع — اول ایست بازرسی را جواب بده"
        } else {
            "Show the reference solution — answer the checkpoint first"
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

    pub const fn answers_title(self) -> &'static str {
        if self.is_fa() {
            "جواب‌های تو"
        } else {
            "Your answers"
        }
    }

    pub const fn answers_intro(self) -> &'static str {
        if self.is_fa() {
            "جواب‌ها رو همین‌جا بنویس. رو دیسکِ خودت ذخیره می‌شن، پس هر وقت خواستی برگرد و بخونشون."
        } else {
            "Write your answers here. They're saved to your own disk, so you can come back and reread them any time."
        }
    }

    pub const fn answers_placeholder(self) -> &'static str {
        if self.is_fa() {
            "۱. …&#10;۲. …"
        } else {
            "1. …&#10;2. …"
        }
    }

    pub const fn answers_save(self) -> &'static str {
        if self.is_fa() {
            "ذخیره‌ی جواب‌ها"
        } else {
            "Save answers"
        }
    }

    pub const fn answers_saved(self) -> &'static str {
        if self.is_fa() {
            "ذخیره شد در"
        } else {
            "Saved to"
        }
    }

    pub const fn answers_empty(self) -> &'static str {
        if self.is_fa() {
            "هنوز چیزی ننوشتی."
        } else {
            "Nothing written yet."
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
