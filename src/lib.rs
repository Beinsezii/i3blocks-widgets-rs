pub fn shell(command: &str, args: &[&str]) -> Option<String> {
    let mut cmd = std::process::Command::new(command);
    cmd.args(args);
    let result = match cmd.output() {
        Ok(result) => Some(String::from_utf8(result.stdout).unwrap().trim().to_string()),
        Err(_) => None,
    };
    return if result == Some("".to_string()) {None} else {result};
}

#[cfg(test)]
mod lib_tests {
    use super::*;
    #[test]
    fn good_result() {
        assert_eq!(shell("echo", &["hi"]), Some(String::from("hi")));
    }

    #[test]
    fn no_result() {
        assert_eq!(shell("echo", &[]), None);
    }

    #[test]
    fn trim_no_result() {
        assert_eq!(shell("echo", &[" "]), None);
    }

    #[test]
    fn trim_no_result2() {
        assert_eq!(shell("echo", &["-e", r"\n"]), None);
    }

    #[test]
    fn multiline_result() {
        assert_eq!(shell("echo", &["-e", r"hi\nthere"]), Some(String::from("hi\nthere")));
    }

    #[test]
    fn multiline_trim_result() {
        assert_eq!(shell("echo", &["-e", r"hi\n  "]), Some(String::from("hi")));
    }

    #[test]
    fn command_fails() {
        assert_eq!(shell("echoooo", &["hi"]), None);
    }
}
