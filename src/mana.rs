#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
    Colorless
}
pub struct Mana(pub Color);

//Add support for hybrid mana later
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ManaCostSymbol {
    White,
    Blue,
    Black,
    Red,
    Green,
    Colorless,
    Generic,
}

pub fn mana_cost_string(coststr: &str) -> Vec<ManaCostSymbol> {
    let mut generic: i32 = 0;
    let mut res=Vec::new();
    for letter in coststr.chars() {
        if letter.is_digit(10) {
            generic *= 10;
            //This should be safe bc/ these are hardcoded within the code
            generic += i32::try_from(letter.to_digit(10).unwrap()).unwrap();
        }
        if letter == 'W' {
            res.push(ManaCostSymbol::White);
        }
        if letter == 'U' {
            res.push(ManaCostSymbol::Blue);
        }
        if letter == 'B' {
            res.push(ManaCostSymbol::Black);
        }
        if letter == 'R' {
            res.push(ManaCostSymbol::Red);
        }
        if letter == 'G' {
            res.push(ManaCostSymbol::Green);
        }
    }
    for _ in 0..generic {
        res.push(ManaCostSymbol::Generic);
    }
    res
}
