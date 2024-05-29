// generate passwords

use rand::{
    seq::{IteratorRandom, SliceRandom},
    thread_rng,
};

use crate::cli::{Generate, PasswordSpec};

pub struct Password {
    choices: Vec<Choice>,
    length: usize,
}

impl Default for Password {
    fn default() -> Self {
        Password {
            choices: vec![
                Choice::at_least(1, CharStyle::Upper),
                Choice::at_least(1, CharStyle::Lower),
                Choice::at_least(1, CharStyle::Number),
                Choice::at_least(1, CharStyle::Symbol),
            ],
            length: 32,
        }
    }
}

impl Password {
    pub fn new(choices: Vec<Choice>, length: usize) -> Self {
        Self { choices, length }
    }
    pub fn generate(&self) -> Option<String> {
        if self.check() {
            let mut characters = vec![];
            let mut active = self.choices.clone();
            for choice in &mut active {
                characters.extend(choice.get_required());
            }
            let remaining = self.length - characters.len();
            let mut active: Vec<_> = active.into_iter().filter(|x| x.active()).collect();

            for _ in 0..remaining {
                if let Some(index) = (0..active.len()).choose(&mut thread_rng()) {
                    let c = active[index].step().unwrap();
                    characters.push(c);
                    if !active[index].active() {
                        active.remove(index);
                    }
                }
            }

            characters.shuffle(&mut thread_rng());
            Some(characters.into_iter().collect())
        } else {
            None
        }
    }

    fn check(&self) -> bool {
        let mut min_length: usize = 0;
        let mut max_length: usize = 0;
        for choice in &self.choices {
            min_length = min_length.checked_add(choice.min).unwrap_or(usize::MAX);
            max_length = max_length.checked_add(choice.max).unwrap_or(usize::MAX);
        }
        min_length <= self.length && self.length <= max_length
    }
    // TODO: builder style interface
    // need to ensure that only have one of each charstyle in the choices vector when building
}

// TODO: generic character sets value
#[derive(Debug, Clone)]
pub enum CharStyle {
    Upper,
    Lower,
    Number,
    Symbol,
}

impl CharStyle {
    fn to_charset(&self) -> Vec<char> {
        match self {
            Self::Upper => ('A'..='Z').collect(),
            Self::Lower => ('a'..='z').collect(),
            Self::Number => ('1'..='9').collect(),
            Self::Symbol => {
                // no real standard for allowed character sets for symbols, but I have some suspicions
                // about disallowed ones
                // for now not including quotes and backslash even though I think others could be
                // troublesome
                vec![
                    '!', '@', '#', '%', '^', '&', '*', '(', ')', '-', '_', '=', '+', '[', '{', ']',
                    '}', '|', ':', ';', ',', '.', '?', '<', '>', '~',
                ]
            }
        }
    }

    pub fn at_least(self, size: usize) -> Choice {
        Choice::at_least(size, self)
    }

    pub fn at_most(self, size: usize) -> Choice {
        Choice::at_most(size, self)
    }

    pub fn exactly(self, size: usize) -> Choice {
        Choice::exactly(size, self)
    }

    pub fn as_choice(self, min: usize, max: usize) -> Option<Choice> {
        Choice::new(min, max, self)
    }
}
#[derive(Debug, Clone)]
pub struct Choice {
    min: usize,
    max: usize,
    chars: CharStyle,
}

impl Choice {
    fn new(min: usize, max: usize, chars: CharStyle) -> Option<Self> {
        if max >= min {
            Some(Self { min, max, chars })
        } else {
            None
        }
    }

    fn exactly(count: usize, chars: CharStyle) -> Self {
        Self {
            min: count,
            max: count,
            chars,
        }
    }

    fn at_least(count: usize, chars: CharStyle) -> Self {
        Self {
            min: count,
            max: usize::MAX,
            chars,
        }
    }

    fn at_most(count: usize, chars: CharStyle) -> Self {
        Self {
            min: usize::MIN,
            max: count,
            chars,
        }
    }

    fn active(&self) -> bool {
        self.max > 0
    }

    fn required(&self) -> bool {
        self.min > 0
    }

    fn decrement(&mut self) {
        if self.min > 0 {
            self.min -= 1;
        }
        if self.max > 0 {
            self.max -= 1;
        }
    }

    fn step(&mut self) -> Option<char> {
        if self.active() {
            self.decrement();
            self.chars.to_charset().choose(&mut thread_rng()).copied()
        } else {
            None
        }
    }

    fn get_required(&mut self) -> Vec<char> {
        let mut res = vec![];
        while self.required() {
            if let Some(c) = self.step() {
                res.push(c);
            }
        }
        res
    }
}

impl From<PasswordSpec> for Password {
    fn from(value: PasswordSpec) -> Self {
        Password::new(
            vec![
                CharStyle::Upper.at_least(value.upper),
                CharStyle::Lower.at_least(value.lower),
                CharStyle::Number.at_least(value.numbers),
                CharStyle::Symbol.at_least(value.symbols),
            ],
            value.length,
        )
    }
}

impl From<Generate> for Password {
    fn from(value: Generate) -> Self {
        let Generate::Generate(spec) = value;
        Password::from(spec)
    }
}
