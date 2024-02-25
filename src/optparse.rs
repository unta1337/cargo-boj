use std::error::Error;
use std::fmt;
use std::str::FromStr;

use bpaf::batteries::cargo_helper;
use bpaf::*;

use crate::datastore::Cookies;
use crate::datastore::LanguageTypes;

pub enum Opts {
    Login(Login),
    Test(Test),
    Submit(Submit),
}

#[derive(Clone)]
pub struct Login {
    pub cookies: Option<Cookies>,
}

pub struct Test {
    pub problem_id: String,
    pub bin_or_cmd: Option<BinOrCmd>,
    pub spj_prompt: bool,
    pub refresh: bool,
}

pub enum BinOrCmd {
    Bin(String),
    Cmd(String),
}

pub enum LanguageType {
    Id(usize),
    Name(String),
}

pub fn get_language_id(language: Option<LanguageType>) -> usize {
    match language {
        Some(LanguageType::Id(id)) => id,
        Some(LanguageType::Name(name)) => get_language_id_from_str(&name),
        None => 113,
    }
}

fn get_language_id_from_str(s: &str) -> usize {
    let language_types = LanguageTypes::load();

    match language_types.language_types.get(s) {
        Some(serde_json::Value::Number(id)) => id.as_i64().unwrap() as usize,
        _ => 113,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageTypeError;

impl std::fmt::Display for LanguageTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "provided language id or name is not available".fmt(f)
    }
}

impl Error for LanguageTypeError {
    fn description(&self) -> &str {
        "failed to find matching language id or name"
    }
}

impl FromStr for LanguageType {
    type Err = LanguageTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lt = s.parse::<usize>();

        match lt {
            Ok(id) => Ok(Self::Id(id)),
            Err(_) => Ok(Self::Name(String::from(s))),
        }
    }
}

pub struct Submit {
    pub problem_id: String,
    pub path: Option<String>,
    pub language: Option<LanguageType>,
    pub code_open: Option<CodeOpen>,
}

pub enum CodeOpen {
    Yes,
    No,
    YesOnAc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError;

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "provided string was not `y`, `n`, or `ac`".fmt(f)
    }
}

impl Error for ParseError {
    fn description(&self) -> &str {
        "failed to parse code-open"
    }
}

impl FromStr for CodeOpen {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "y" => Ok(Self::Yes),
            "n" => Ok(Self::No),
            "ac" => Ok(Self::YesOnAc),
            _ => Err(ParseError),
        }
    }
}

impl ToString for CodeOpen {
    fn to_string(&self) -> String {
        match self {
            CodeOpen::Yes => "open",
            CodeOpen::No => "close",
            CodeOpen::YesOnAc => "onlyaccepted",
        }
        .to_string()
    }
}

pub fn cargo_boj_opts() -> Opts {
    let login = construct!(Opts::Login(cargo_boj_login()));
    let test = construct!(Opts::Test(cargo_boj_test()));
    let submit = construct!(Opts::Submit(cargo_boj_submit()));
    cargo_helper("boj", construct!([login, test, submit]))
        .to_options()
        .run()
}

fn cargo_boj_login() -> impl Parser<Login> {
    let bojautologin = long("bojautologin")
        .help("The value of cookie `bojautologin`")
        .argument("str");
    let onlinejudge = long("onlinejudge")
        .help("The value of cookie `OnlineJudge`")
        .argument("str");
    let cookies = construct!(Cookies {
        bojautologin,
        onlinejudge
    })
    .optional();
    construct!(Login { cookies })
        .to_options()
        .descr("Store BOJ login information for submitting solutions.")
        .command("login")
}

fn cargo_boj_test() -> impl Parser<Test> {
    let problem_id = positional("PID").help("Problem ID");
    let bin = short('b')
        .long("bin")
        .help("Bin name in the current Rust crate")
        .argument("BIN");
    let cmd = short('c')
        .long("cmd")
        .help("Command to run a non-Rust program")
        .argument("CMD");
    let spj_prompt = short('p')
        .long("spj-prompt")
        .help("If set, ask the user for confirmation for Special Judge problems")
        .switch();
    let refresh = short('r')
        .long("refresh")
        .help("If set, refresh the cache for the problem")
        .switch();
    let bin = construct!(BinOrCmd::Bin(bin));
    let cmd = construct!(BinOrCmd::Cmd(cmd));
    let bin_or_cmd = construct!([bin, cmd]).optional();
    construct!(Test {
        bin_or_cmd,
        spj_prompt,
        refresh,
        problem_id
    })
    .to_options()
    .descr("Test a solution against example tests.")
    .command("test")
}

fn cargo_boj_submit() -> impl Parser<Submit> {
    let problem_id = positional("PID").help("Problem ID");
    let path = short('p')
        .long("path")
        .help("Path of the file to submit")
        .argument("PATH")
        .optional();
    let language = short('l')
        .long("lang")
        .help("Language ID")
        .argument("LANG")
        .optional();
    let code_open = short('o')
        .long("code-open")
        .help("Whether to open code to public. Options are: y(yes), n(no), ac(yes on AC)")
        .argument("OPT")
        .optional();
    construct!(Submit {
        path,
        language,
        code_open,
        problem_id,
    })
    .to_options()
    .descr("Submit a solution to a BOJ problem.")
    .command("submit")
}
