use clap::Parser;
use once_cell::sync::Lazy;
use pants_gen::{
    interval::Interval,
    password::{Choice, PasswordSpec},
};

static DEFAULT_SPEC: Lazy<String> = Lazy::new(|| PasswordSpec::default().to_string());

#[derive(Debug, Parser)]
#[command(
    version,
    about,
    long_about = "Generate a password based on a spec. Generally the default spec should cover things\
        with possibly a length adjustment with -l N. If the default symbols cause problems try: -s 0 -c '$_-+|1+' or something similar to substitute in a restricted set of symbols.\
        To fully alter the spec use something like --spec '20//1+|charset1//2|charset2 and customize fairly freely.",
    after_help = "General formatting follows the style of INTERVAL|CHARSET \
        and the overall spec is a combination of these and a length as length//INTERVAL|CHARSET//. \
        A CHARSET is any character and there are special charset patterns\
        (:upper:, :lower:, :number:, :symbol:). An interval follows the form N for\
        exactly N characters, N+ for at least N characters, N- for at most N characters,\
        and A-B for the range of A to B characters. The interface works by having\
        a default password specification and then allowing for modifications as needed."
)]
pub struct CliArgs {
    /// spec string
    #[arg(
        short = 'p',
        long,
        default_value = DEFAULT_SPEC.as_str()
    )]
    spec: PasswordSpec,
    /// length of the generated password
    #[arg(short, long)]
    length: Option<usize>,
    /// constraints on uppercase characters, N|N+|N-|A-B
    #[arg(short, long)]
    upper: Option<Interval>,
    /// constraints on lowercase characters, N|N+|N-|A-B
    #[arg(short = 'd', long)]
    lower: Option<Interval>,
    /// constraints on number characters, N|N+|N-|A-B
    #[arg(short, long)]
    number: Option<Interval>,
    /// constraints on symbols characters, N|N+|N-|A-B
    #[arg(short, long)]
    symbol: Option<Interval>,
    /// constraints on custom characters, INTERVAL|CHARSET
    #[arg(short, long)]
    custom: Vec<Choice>,
}

impl CliArgs {
    pub fn run() -> Option<String> {
        let args = CliArgs::parse();
        args.execute()
    }

    pub fn execute(self) -> Option<String> {
        let mut password_spec = self.spec;
        if let Some(length) = &self.length {
            password_spec = password_spec.length(*length);
        }
        if let Some(upper) = self.upper {
            password_spec = password_spec.upper(upper);
        }
        if let Some(lower) = self.lower {
            password_spec = password_spec.lower(lower);
        }
        if let Some(number) = self.number {
            password_spec = password_spec.number(number);
        }
        if let Some(symbol) = self.symbol {
            password_spec = password_spec.symbol(symbol);
        }

        for c in self.custom {
            password_spec = password_spec.include(c);
        }
        password_spec.generate()
    }
}
