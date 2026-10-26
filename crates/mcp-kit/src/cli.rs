//! Shared command-line handling: every server answers `--help` and
//! `--version` the same way before it starts speaking MCP on stdio.

/// What a server says about itself on the command line.
pub struct About<'a> {
    pub name: &'a str,
    pub version: &'a str,
    /// One or two sentences: what the server does.
    pub summary: &'a str,
    /// `(VARIABLE, meaning)` pairs the server reads.
    pub env: &'a [(&'a str, &'a str)],
}

impl About<'_> {
    pub fn help(&self) -> String {
        let mut out = format!("{}-mcp {}\n\n{}\n", self.name, self.version, self.summary);
        if !self.env.is_empty() {
            out.push_str("\nEnvironment:\n");
            let width = self.env.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
            for (k, v) in self.env {
                out.push_str(&format!("  {k:width$}  {v}\n"));
            }
        }
        out.push_str(&format!(
            "\nAdd to your client:\n  {{\"mcpServers\": {{\"{n}\": {{\"command\": \"{n}-mcp\"}}}}}}\n\nWith no arguments the server speaks MCP over stdio.\n",
            n = self.name
        ));
        out
    }
}

/// If `args` asks for help or the version, return the text to print.
pub fn handle(about: &About, args: impl IntoIterator<Item = String>) -> Option<String> {
    for a in args {
        match a.as_str() {
            "-h" | "--help" => return Some(about.help()),
            "-V" | "--version" => return Some(format!("{}-mcp {}\n", about.name, about.version)),
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const ABOUT: About = About {
        name: "demo",
        version: "1.2.3",
        summary: "Does one thing well.",
        env: &[("DEMO_ROOT", "only read files under this directory"), ("DEMO_X", "x")],
    };

    #[test]
    fn version_and_help() {
        assert_eq!(handle(&ABOUT, ["--version".to_string()]).unwrap(), "demo-mcp 1.2.3\n");
        let help = handle(&ABOUT, ["-h".to_string()]).unwrap();
        assert!(help.starts_with("demo-mcp 1.2.3\n\nDoes one thing well."));
        assert!(help.contains("  DEMO_ROOT  only read files"));
        assert!(help.contains("  DEMO_X     x"));
        assert!(help.contains("\"command\": \"demo-mcp\""));
    }

    #[test]
    fn other_arguments_fall_through() {
        assert!(handle(&ABOUT, ["audit".to_string()]).is_none());
        assert!(handle(&ABOUT, Vec::<String>::new()).is_none());
    }
}
