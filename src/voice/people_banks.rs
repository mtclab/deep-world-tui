use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct PeopleVoiceBank {
    pub greetings: Vec<String>,
    pub farewells: Vec<String>,
    pub neutral: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PeopleBanks {
    pub metsik: PeopleVoiceBank,
    pub arkit: PeopleVoiceBank,
    pub vayla: PeopleVoiceBank,
    pub laakso: PeopleVoiceBank,
    pub sepat: PeopleVoiceBank,
    pub ahjo: PeopleVoiceBank,
    pub varhaiset: PeopleVoiceBank,
    pub metsareunat: PeopleVoiceBank,
    pub porokansa: PeopleVoiceBank,
    pub koskimetsa: PeopleVoiceBank,
    pub muistikansa: PeopleVoiceBank,
    pub taulukansa: PeopleVoiceBank,
    pub kirjakansa: PeopleVoiceBank,
    pub takovaki: PeopleVoiceBank,
    pub rantavaki: PeopleVoiceBank,
    pub saarivaki: PeopleVoiceBank,
    pub hiekkakavelijat: PeopleVoiceBank,
    pub haramaki: PeopleVoiceBank,
    pub jamavaki: PeopleVoiceBank,
    pub pohjavaki: PeopleVoiceBank,
    pub tzakhar: PeopleVoiceBank,
    pub merak: PeopleVoiceBank,
    pub shear: PeopleVoiceBank,
    pub hal: PeopleVoiceBank,
    pub khor: PeopleVoiceBank,
}

impl PeopleBanks {
    pub fn load(path: &str) -> Result<Self, String> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read people banks: {}", e))?;
        ron::from_str(&contents).map_err(|e| format!("Failed to parse people banks: {}", e))
    }

    pub fn bank_for(&self, people: &crate::model::PeopleKind) -> &PeopleVoiceBank {
        use crate::model::PeopleKind as P;
        match people {
            P::Metsik => &self.metsik,
            P::Arkit => &self.arkit,
            P::Vayla => &self.vayla,
            P::Laakso => &self.laakso,
            P::Sepat => &self.sepat,
            P::Ahjo => &self.ahjo,
            P::Varhaiset => &self.varhaiset,
            P::Metsareunat => &self.metsareunat,
            P::Porokansa => &self.porokansa,
            P::Koskimetsa => &self.koskimetsa,
            P::Muistikansa => &self.muistikansa,
            P::Taulukansa => &self.taulukansa,
            P::Kirjakansa => &self.kirjakansa,
            P::Takovaki => &self.takovaki,
            P::Rantavaki => &self.rantavaki,
            P::Saarivaki => &self.saarivaki,
            P::Hiekkakavelijat => &self.hiekkakavelijat,
            P::Haramaki => &self.haramaki,
            P::Jamavaki => &self.jamavaki,
            P::Pohjavaki => &self.pohjavaki,
            P::Tzakhar => &self.tzakhar,
            P::Merak => &self.merak,
            P::Shear => &self.shear,
            P::Hal => &self.hal,
            P::Khor => &self.khor,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_people_banks() {
        let banks = PeopleBanks::load("data/voice/people_banks.ron").expect("should load");
        assert!(
            banks.metsik.greetings.len() >= 5,
            "Metsik should have >= 5 greetings"
        );
        assert!(
            banks.arkit.farewells.len() >= 5,
            "Arkit should have >= 5 farewells"
        );
        assert!(
            banks.khor.neutral.len() >= 5,
            "Khor should have >= 5 neutral lines"
        );
    }

    #[test]
    fn bank_for_each_people() {
        let banks = PeopleBanks::load("data/voice/people_banks.ron").unwrap();
        use crate::model::PeopleKind as P;
        for people in [
            P::Metsik,
            P::Arkit,
            P::Vayla,
            P::Laakso,
            P::Sepat,
            P::Ahjo,
            P::Varhaiset,
            P::Metsareunat,
            P::Porokansa,
            P::Koskimetsa,
            P::Muistikansa,
            P::Taulukansa,
            P::Kirjakansa,
            P::Takovaki,
            P::Rantavaki,
            P::Saarivaki,
            P::Hiekkakavelijat,
            P::Haramaki,
            P::Jamavaki,
            P::Pohjavaki,
            P::Tzakhar,
            P::Merak,
            P::Shear,
            P::Hal,
            P::Khor,
        ] {
            let bank = banks.bank_for(&people);
            assert!(
                bank.greetings.len() >= 5,
                "{:?} should have >= 5 greetings",
                people
            );
            assert!(
                bank.farewells.len() >= 5,
                "{:?} should have >= 5 farewells",
                people
            );
            assert!(
                bank.neutral.len() >= 5,
                "{:?} should have >= 5 neutral lines",
                people
            );
        }
    }
}
