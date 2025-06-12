#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum PossibleScore {
    Yes(u8),
    Likely(u8),
    Maybe(u8),
    Unsure(u8),
    No(u8),
    Num(u8),
}

impl PossibleScore {
    fn f32_score(&self) -> f32 {
        match self {
            PossibleScore::Yes(i) => 9_f32 + Self::little_score(i),
            PossibleScore::Likely(i) => 8_f32 + Self::little_score(i),
            PossibleScore::Maybe(i) => 5_f32 + Self::little_score(i),
            PossibleScore::Unsure(i) => 2_f32 + Self::little_score(i),
            PossibleScore::No(i) => 0 as f32 + Self::little_score(i),
            PossibleScore::Num(i) => i.to_owned() as f32 / 256.0 * 10.0,
        }
    }

    fn little_score(i: &u8) -> f32 {
        i.to_owned() as f32 / 256_f32
    }

    pub fn merge(self, score: u8) -> Self {
        PossibleScore::Num(self.f32_score() as u8 * 25 / 2 + score)
    }
}

impl PartialOrd for PossibleScore {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.f32_score().partial_cmp(&other.f32_score())
    }
}