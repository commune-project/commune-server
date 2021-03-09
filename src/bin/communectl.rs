use commune::db::establish_connection;
use commune::db::actions::user::create_user;

use getopts::Options;
use rpassword;
use std::env;

fn help(subcmd: &str) {
    match subcmd {
        "user" => {
            println!(
                r#"Usage:
            communectl user create ,,,
            "#
            );
        }
        _ => {
            println!(
                r#"Usage:
            communectl help <subcommand>
            communectl user ...
            "#
            );
        }
    }
}

fn help_opts(args: &Vec<String>, opts: &Options) -> String {
    let brief = format!("Usage: {} [options]", args.join(" "));
    opts.usage(&brief)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).as_ref().map(|s| &s[..]) {
        Some("user") => match args.get(2).as_ref().map(|s| &s[..]) {
            Some("create") => subcmd_user_create(args),
            Some(_) => help("user"),
            None => help("user"),
        },
        _ => help(""),
    }
}

fn subcmd_user_create(args: Vec<String>) {
    let mut opts = Options::new();
    opts.reqopt("u", "username", "username (without domain)", "USERNAME");
    opts.reqopt("d", "domain", "user domain", "DOMAIN");
    let args_usage = args[0..3].to_vec();
    let matches = match opts.parse(&args[3..]) {
        Ok(m) => m,
        Err(_f) => {
            println!("{}", help_opts(&args_usage, &opts));
            panic!("required opt");
        },
    };
    let username = matches.opt_str("u").expect(&help_opts(&args_usage, &opts));
    let domain = matches.opt_str("d").expect(&help_opts(&args_usage, &opts));
    let password = rpassword::prompt_password_stdout("Password: ").ok();

    if let Some(password) = password {
        let conn = establish_connection();
        match create_user(
            &conn,
            username.as_str(),
            domain.as_str(),
            password.as_str(),
            "",
            "und",
            None,
        ) {
            Ok(_) => (),
            Err(e) => eprintln!("{}", e),
        };
    }
}
