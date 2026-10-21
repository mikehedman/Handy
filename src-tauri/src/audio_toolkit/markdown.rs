//! Markdown voice commands.
//!
//! Every command is the keyword "markdown" followed by a command word. Without
//! the keyword nothing is formatted, so ordinary words like "bold" or "bullet"
//! in a sentence stay as plain text. Punctuation and capitals the speech model
//! attaches to the keyword or command words are ignored.
//!
//! | Say                                 | Markdown            | Slack          |
//! |-------------------------------------|---------------------|----------------|
//! | markdown bold … markdown bold       | `**…**`             | `*…*`          |
//! | markdown italic … markdown italic   | `*…*`               | `_…_`          |
//! | markdown header one / two / three   | `#` / `##` / `###`  | bold line      |
//! | markdown bullet                     | `- ` on a new line  | same           |
//! | markdown numbered bullet            | `1. ` on a new line | same           |
//! | markdown new line                   | line break          | same           |
//!
//! Bold and italic toggle: the same command opens and closes. Anything left
//! open is closed at the end of the line. Accepted spellings and common
//! mishearings live in the word lists below; add to them as new ones turn up.

/// Spellings of the keyword. "mark down" (two words) is handled separately.
const KEYWORD: &[&str] = &["markdown"];
const BOLD: &[&str] = &["bold", "bald", "bolt", "bowled", "bolds"];
const ITALIC: &[&str] = &["italic", "italics"];
const HEADER: &[&str] = &["header", "heading", "headers"];
const HEADER_ONE: &[&str] = &["one", "1", "won"];
const HEADER_TWO: &[&str] = &["two", "2", "to", "too"];
const HEADER_THREE: &[&str] = &["three", "3"];
const BULLET: &[&str] = &["bullet", "bullets"];
/// Optional word after "bullet", as in "bullet point".
const BULLET_POINT: &[&str] = &["point"];
const NUMBERED: &[&str] = &["numbered", "number"];
const NEWLINE: &[&str] = &["newline", "new-line"];
/// "new line" as two words.
const NEW: &[&str] = &["new"];
const LINE: &[&str] = &["line"];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MarkdownMode {
    Markdown, // Standard markdown: **bold**, *italic*
    Slack,    // Slack mrkdwn: *bold*, _italic_
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Command {
    Bold,
    Italic,
    Header(usize),
    Bullet,
    NumberedBullet,
    NewLine,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Style {
    Bold,
    Italic,
}

/// Lowercases a word and strips the punctuation the speech model attaches to it.
fn normalize(word: &str) -> String {
    word.trim_matches(|c: char| c.is_ascii_punctuation() && c != '-')
        .to_lowercase()
}

fn is_one_of(word: Option<&String>, list: &[&str]) -> bool {
    word.is_some_and(|w| list.contains(&w.as_str()))
}

/// If a command starts at `words[i]`, returns it and how many words it spans.
fn parse_command(words: &[String], i: usize) -> Option<(Command, usize)> {
    let at = |k: usize| words.get(i + k);

    let keyword_len = if is_one_of(at(0), KEYWORD) {
        1
    } else if is_one_of(at(0), &["mark"]) && is_one_of(at(1), &["down"]) {
        2
    } else {
        return None;
    };
    let cmd = |k: usize| at(keyword_len + k);

    let (command, len) = if is_one_of(cmd(0), BOLD) {
        (Command::Bold, 1)
    } else if is_one_of(cmd(0), ITALIC) {
        (Command::Italic, 1)
    } else if is_one_of(cmd(0), HEADER) {
        if is_one_of(cmd(1), HEADER_TWO) {
            (Command::Header(2), 2)
        } else if is_one_of(cmd(1), HEADER_THREE) {
            (Command::Header(3), 2)
        } else if is_one_of(cmd(1), HEADER_ONE) {
            (Command::Header(1), 2)
        } else {
            (Command::Header(1), 1)
        }
    } else if is_one_of(cmd(0), BULLET) {
        let point = is_one_of(cmd(1), BULLET_POINT) as usize;
        (Command::Bullet, 1 + point)
    } else if is_one_of(cmd(0), NUMBERED) && is_one_of(cmd(1), BULLET) {
        (Command::NumberedBullet, 2)
    } else if is_one_of(cmd(0), NEWLINE) {
        (Command::NewLine, 1)
    } else if is_one_of(cmd(0), NEW) && is_one_of(cmd(1), LINE) {
        (Command::NewLine, 2)
    } else {
        return None;
    };

    Some((command, keyword_len + len))
}

/// Builds the output line by line while tracking open bold/italic spans.
struct Formatter {
    mode: MarkdownMode,
    lines: Vec<Vec<String>>,
    /// Text placed before the next word: a line prefix like "- " plus the
    /// delimiters of styles opened since the last word.
    line_prefix: String,
    pending_opens: Vec<Style>,
    /// Styles that have been applied to at least one word, in opening order.
    open: Vec<Style>,
    numbered_count: usize,
}

impl Formatter {
    fn new(mode: MarkdownMode) -> Self {
        Self {
            mode,
            lines: vec![Vec::new()],
            line_prefix: String::new(),
            pending_opens: Vec::new(),
            open: Vec::new(),
            numbered_count: 0,
        }
    }

    fn delim(&self, style: Style) -> &'static str {
        match (self.mode, style) {
            (MarkdownMode::Markdown, Style::Bold) => "**",
            (MarkdownMode::Markdown, Style::Italic) => "*",
            (MarkdownMode::Slack, Style::Bold) => "*",
            (MarkdownMode::Slack, Style::Italic) => "_",
        }
    }

    fn current_line(&mut self) -> &mut Vec<String> {
        self.lines.last_mut().expect("lines is never empty")
    }

    fn push_word(&mut self, word: &str) {
        let mut out = std::mem::take(&mut self.line_prefix);
        for style in std::mem::take(&mut self.pending_opens) {
            out.push_str(self.delim(style));
            self.open.push(style);
        }
        out.push_str(word);
        self.current_line().push(out);
    }

    /// Closes `style`, keeping the last word's trailing punctuation outside it.
    fn close(&mut self, style: Style) {
        if let Some(pos) = self.pending_opens.iter().position(|s| *s == style) {
            // Opened and closed with no words in between: drop it.
            self.pending_opens.remove(pos);
            return;
        }
        let Some(pos) = self.open.iter().position(|s| *s == style) else {
            return;
        };
        self.open.remove(pos);
        let delim = self.delim(style);
        if let Some(last) = self.current_line().last_mut() {
            let body_len = last
                .trim_end_matches(|c: char| c.is_ascii_punctuation() && !"*_".contains(c))
                .len();
            last.insert_str(body_len, delim);
        }
    }

    fn toggle(&mut self, style: Style) {
        if self.open.contains(&style) || self.pending_opens.contains(&style) {
            self.close(style);
        } else {
            self.pending_opens.push(style);
        }
    }

    /// Ends the current line, closing anything still open.
    fn end_line(&mut self) {
        for style in self.open.clone().into_iter().rev() {
            self.close(style);
        }
        self.pending_opens.clear();
        self.line_prefix.clear();
    }

    /// Moves to a fresh line for a block element, reusing an empty one.
    fn start_block(&mut self) {
        self.end_line();
        if !self.current_line().is_empty() {
            self.lines.push(Vec::new());
        }
    }

    /// Line break from "new line", allowing at most one blank line in a row.
    fn line_break(&mut self) {
        self.end_line();
        let n = self.lines.len();
        let current_empty = self.lines[n - 1].is_empty();
        let previous_empty = n < 2 || self.lines[n - 2].is_empty();
        if !current_empty || !previous_empty {
            self.lines.push(Vec::new());
        }
    }

    fn apply(&mut self, command: Command) {
        match command {
            Command::Bold => self.toggle(Style::Bold),
            Command::Italic => self.toggle(Style::Italic),
            Command::Header(level) => {
                self.start_block();
                match self.mode {
                    MarkdownMode::Markdown => self.line_prefix = format!("{} ", "#".repeat(level)),
                    // Slack has no headers; a bold line is the closest match.
                    MarkdownMode::Slack => self.pending_opens.push(Style::Bold),
                }
            }
            Command::Bullet => {
                self.start_block();
                self.line_prefix = "- ".to_string();
            }
            Command::NumberedBullet => {
                self.start_block();
                self.numbered_count += 1;
                self.line_prefix = format!("{}. ", self.numbered_count);
            }
            Command::NewLine => self.line_break(),
        }
    }

    fn finish(mut self) -> String {
        self.end_line();
        self.lines
            .iter()
            .map(|line| line.join(" "))
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string()
    }
}

/// Processes dictated text with "markdown …" voice commands into formatted
/// markdown or Slack mrkdwn. See the module docs for the command list.
pub fn process_markdown_commands(text: &str, mode: MarkdownMode) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    let normalized: Vec<String> = words.iter().map(|w| normalize(w)).collect();
    let mut formatter = Formatter::new(mode);

    let mut i = 0;
    while i < words.len() {
        match parse_command(&normalized, i) {
            Some((command, len)) => {
                formatter.apply(command);
                i += len;
                // A sentence-ending mark on the final command belongs to the
                // sentence, e.g. "… markdown bold." → "…**."
                if i == words.len() {
                    keep_final_punctuation(&mut formatter, words[i - 1]);
                }
            }
            None => {
                formatter.push_word(words[i]);
                i += 1;
            }
        }
    }

    formatter.finish()
}

fn keep_final_punctuation(formatter: &mut Formatter, last_command_word: &str) {
    let Some(mark) = last_command_word
        .chars()
        .last()
        .filter(|c| ".!?".contains(*c))
    else {
        return;
    };
    // Close spans first so the mark lands after the delimiters.
    formatter.end_line();
    if let Some(last) = formatter.current_line().last_mut() {
        let ends_in_punctuation = last
            .trim_end_matches(['*', '_'])
            .ends_with(|c: char| c.is_ascii_punctuation());
        if !ends_in_punctuation {
            last.push(mark);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    fn check(input: &str, markdown_expected: &str, slack_expected: &str) {
        let markdown_result = process_markdown_commands(input, MarkdownMode::Markdown);
        assert_eq!(markdown_result, markdown_expected, "Markdown mode failed");

        let slack_result = process_markdown_commands(input, MarkdownMode::Slack);
        assert_eq!(slack_result, slack_expected, "Slack mode failed");
    }

    #[test_case("markdown bold dark text markdown bold", "**dark text**", "*dark text*" ; "bold toggle")]
    #[test_case("Markdown bold dark text. Markdown, bald.", "**dark text**.", "*dark text*." ; "misheard bald with asr punctuation")]
    #[test_case("This is markdown bold important markdown bold.", "This is **important**.", "This is *important*." ; "bold mid sentence keeps final period")]
    #[test_case("mark down bold hi mark down bold", "**hi**", "*hi*" ; "keyword split into two words")]
    #[test_case("markdown bolt one markdown bowled two", "**one** two", "*one* two" ; "bolt and bowled are bold")]
    #[test_case("markdown bold hello world", "**hello world**", "*hello world*" ; "unclosed bold closes at end")]
    #[test_case("hi markdown bold markdown bold there", "hi there", "hi there" ; "empty bold is dropped")]
    #[test_case("markdown bold hello markdown new line world", "**hello**\nworld", "*hello*\nworld" ; "unclosed bold closes at line break")]
    fn bold(input: &str, markdown_expected: &str, slack_expected: &str) {
        check(input, markdown_expected, slack_expected);
    }

    #[test_case("markdown italic soft markdown italics", "*soft*", "_soft_" ; "italic toggle with italics alias")]
    #[test_case("Say it markdown italic gently, markdown italic.", "Say it *gently*,", "Say it _gently_," ; "italic punctuation moves outside")]
    fn italic(input: &str, markdown_expected: &str, slack_expected: &str) {
        check(input, markdown_expected, slack_expected);
    }

    #[test_case("bold move, bullet proof, new line of work", "bold move, bullet proof, new line of work", "bold move, bullet proof, new line of work" ; "bare command words are plain text")]
    #[test_case("I like markdown files", "I like markdown files", "I like markdown files" ; "keyword without command is plain text")]
    #[test_case("Markdown. Pizza", "Markdown. Pizza", "Markdown. Pizza" ; "keyword with unknown word is plain text")]
    fn keyword_required(input: &str, markdown_expected: &str, slack_expected: &str) {
        check(input, markdown_expected, slack_expected);
    }

    #[test_case("markdown header one Meeting Notes markdown new line hello", "# Meeting Notes\nhello", "*Meeting Notes*\nhello" ; "header one")]
    #[test_case("Markdown heading 2. Plan", "## Plan", "*Plan*" ; "heading digit level with punctuation")]
    #[test_case("intro markdown header three Details", "intro\n### Details", "intro\n*Details*" ; "header starts a new line")]
    #[test_case("markdown header Title", "# Title", "*Title*" ; "header without level defaults to one")]
    fn headers(input: &str, markdown_expected: &str, slack_expected: &str) {
        check(input, markdown_expected, slack_expected);
    }

    #[test_case("Items: markdown bullet eggs markdown bullet milk", "Items:\n- eggs\n- milk", "Items:\n- eggs\n- milk" ; "bullets")]
    #[test_case("markdown bullet point eggs", "- eggs", "- eggs" ; "bullet point alias")]
    #[test_case("markdown numbered bullet first markdown number bullet second", "1. first\n2. second", "1. first\n2. second" ; "numbered bullets")]
    fn lists(input: &str, markdown_expected: &str, slack_expected: &str) {
        check(input, markdown_expected, slack_expected);
    }

    #[test_case("First line. Markdown new line. Second line.", "First line.\nSecond line.", "First line.\nSecond line." ; "new line with asr punctuation")]
    #[test_case("a markdown newline b", "a\nb", "a\nb" ; "newline one word")]
    #[test_case("a markdown new line markdown new line markdown new line b", "a\n\nb", "a\n\nb" ; "at most one blank line")]
    #[test_case("a markdown new line markdown new line markdown bullet b", "a\n\n- b", "a\n\n- b" ; "bullet reuses empty line")]
    fn new_lines(input: &str, markdown_expected: &str, slack_expected: &str) {
        check(input, markdown_expected, slack_expected);
    }

    #[test_case("", "", "" ; "empty input")]
    #[test_case("   ", "", "" ; "whitespace only")]
    #[test_case("This is just regular text.", "This is just regular text.", "This is just regular text." ; "no commands")]
    fn edge_cases(input: &str, markdown_expected: &str, slack_expected: &str) {
        check(input, markdown_expected, slack_expected);
    }
}
