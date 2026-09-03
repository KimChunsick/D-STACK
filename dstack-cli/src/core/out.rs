// core/out.rs
// The output sink of every verb: say to stdout, warn to stderr, and a capture mode for self-calls.

use std::io::Write;

struct Capture {
    out: String,
    err: String,
}

/// say(), warn() and the raw writer of common.sh. Verbs never print directly, so an in-process
/// self-call (Context::call) can buffer exactly what the shell captured from a subprocess.
pub struct Out {
    captures: Vec<Capture>,
}

impl Out {
    pub fn new() -> Out {
        Out {
            captures: Vec::new(),
        }
    }

    /// say(): one line on stdout.
    pub fn say(&mut self, line: &str) {
        match self.captures.last_mut() {
            Some(c) => {
                c.out.push_str(line);
                c.out.push('\n');
            }
            None => {
                let mut stdout = std::io::stdout();
                let _ = writeln!(stdout, "{line}");
            }
        }
    }

    /// Text written to stdout as it is (no newline added).
    pub fn raw(&mut self, text: &str) {
        match self.captures.last_mut() {
            Some(c) => c.out.push_str(text),
            None => {
                let mut stdout = std::io::stdout();
                let _ = write!(stdout, "{text}");
            }
        }
    }

    /// warn(): a note on stderr that exits nothing.
    pub fn warn(&mut self, message: &str) {
        self.err_line(&format!("dstack: warning: {message}"));
    }

    /// One line on stderr (the error line of main lands here).
    pub fn err_line(&mut self, line: &str) {
        match self.captures.last_mut() {
            Some(c) => {
                c.err.push_str(line);
                c.err.push('\n');
            }
            None => {
                let mut stderr = std::io::stderr();
                let _ = writeln!(stderr, "{line}");
            }
        }
    }

    pub fn flush(&mut self) {
        let _ = std::io::stdout().flush();
        let _ = std::io::stderr().flush();
    }

    pub fn begin_capture(&mut self) {
        self.captures.push(Capture {
            out: String::new(),
            err: String::new(),
        });
    }

    /// The buffered (stdout, stderr) of the innermost capture.
    pub fn end_capture(&mut self) -> (String, String) {
        match self.captures.pop() {
            Some(c) => (c.out, c.err),
            None => (String::new(), String::new()),
        }
    }
}

impl Default for Out {
    fn default() -> Out {
        Out::new()
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn r13__capture_keeps_the_streams_apart() {
        let mut out = Out::new();
        out.begin_capture();
        out.say("one");
        out.warn("two");
        out.raw("three");
        let (stdout, stderr) = out.end_capture();
        assert_eq!(stdout, "one\nthree");
        assert_eq!(stderr, "dstack: warning: two\n");
    }
}
