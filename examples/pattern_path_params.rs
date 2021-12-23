use std::path;

use regex::Regex;

fn main() {
    let re_sep = if cfg!(windows) {
        String::from(r"\\")
    } else {
        String::from(path::MAIN_SEPARATOR)
    };

    let pattern = format!(r"{}=[^{}]*{}?", re_sep, re_sep, re_sep);

    let pattern = format!(r"{}=[^{}]*{}", r"\\", r"\\", r"\\");

    println!(">>> pattern = {}", pattern);

    let re = &Regex::new(&pattern).unwrap();

    let unix_success_tests = vec![
        r"/=",
        r"/=/",
        r"/=name/",
        r"/Users/=name",
        r"/Users/=name/",
        r"/Users/=name/test",
        r"/=name/test",
        r"/Users/test/=",
    ];

    let unix_fail_tests = vec![
        r"=",
        r"/name=/",
        r"/Users/name=",
        r"/Users\=name\",
        r"/Users/name=/test",
    ];

    run_tests(re, unix_success_tests, "Success");
    run_tests(re, unix_fail_tests, "Fail");

    let win_success_tests = vec![
        r"C:\\=",
        r"C:\\=\",
        r"C:\\=name\",
        r"C:\\Users\=name",
        r"C:\\Users\=name\",
        r"C:\\Users\=name\test",
        r"C:\=name\test",
        r"C:\Users\test\=",
    ];

    let win_fail_tests = vec![
        r"C:=",
        r"C:\\name=\",
        r"C:\\Users\name=",
        r"C:\\Users/=name/",
        r"C:\\Users\name=\test",
    ];

    run_tests(re, win_success_tests, "Success");
    run_tests(re, win_fail_tests, "Fail");
}

fn run_tests(re: &Regex, tests: Vec<&str>, kind: &str) {
    println!("\n===============( {} )===============", kind);
    for test in tests.iter() {
        let result = if let Some(m) = re.find(test) {
            m.as_str()
        } else {
            ""
        };

        println!(r#"match on "{}", found "{}""#, test, result);

        for i in re.split(test) {
            println!(r#"\tsplit = "{}""#, i);
        }
    }
}
