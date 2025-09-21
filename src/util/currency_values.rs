use once_cell::sync::Lazy;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct WordInfo {
    pub value: f64,
    pub is_multiplier: bool,
}

pub static WORD_VALUES: Lazy<HashMap<&'static str, WordInfo>> = Lazy::new(|| {
    [
        // English
        (
            "a",
            WordInfo {
                value: 1.0,
                is_multiplier: false,
            },
        ),
        (
            "one",
            WordInfo {
                value: 1.0,
                is_multiplier: false,
            },
        ),
        (
            "two",
            WordInfo {
                value: 2.0,
                is_multiplier: false,
            },
        ),
        (
            "three",
            WordInfo {
                value: 3.0,
                is_multiplier: false,
            },
        ),
        (
            "four",
            WordInfo {
                value: 4.0,
                is_multiplier: false,
            },
        ),
        (
            "five",
            WordInfo {
                value: 5.0,
                is_multiplier: false,
            },
        ),
        (
            "six",
            WordInfo {
                value: 6.0,
                is_multiplier: false,
            },
        ),
        (
            "seven",
            WordInfo {
                value: 7.0,
                is_multiplier: false,
            },
        ),
        (
            "eight",
            WordInfo {
                value: 8.0,
                is_multiplier: false,
            },
        ),
        (
            "nine",
            WordInfo {
                value: 9.0,
                is_multiplier: false,
            },
        ),
        (
            "ten",
            WordInfo {
                value: 10.0,
                is_multiplier: false,
            },
        ),
        (
            "eleven",
            WordInfo {
                value: 11.0,
                is_multiplier: false,
            },
        ),
        (
            "twelve",
            WordInfo {
                value: 12.0,
                is_multiplier: false,
            },
        ),
        (
            "thirteen",
            WordInfo {
                value: 13.0,
                is_multiplier: false,
            },
        ),
        (
            "fourteen",
            WordInfo {
                value: 14.0,
                is_multiplier: false,
            },
        ),
        (
            "fifteen",
            WordInfo {
                value: 15.0,
                is_multiplier: false,
            },
        ),
        (
            "sixteen",
            WordInfo {
                value: 16.0,
                is_multiplier: false,
            },
        ),
        (
            "seventeen",
            WordInfo {
                value: 17.0,
                is_multiplier: false,
            },
        ),
        (
            "eighteen",
            WordInfo {
                value: 18.0,
                is_multiplier: false,
            },
        ),
        (
            "nineteen",
            WordInfo {
                value: 19.0,
                is_multiplier: false,
            },
        ),
        (
            "twenty",
            WordInfo {
                value: 20.0,
                is_multiplier: false,
            },
        ),
        (
            "thirty",
            WordInfo {
                value: 30.0,
                is_multiplier: false,
            },
        ),
        (
            "forty",
            WordInfo {
                value: 40.0,
                is_multiplier: false,
            },
        ),
        (
            "fifty",
            WordInfo {
                value: 50.0,
                is_multiplier: false,
            },
        ),
        (
            "sixty",
            WordInfo {
                value: 60.0,
                is_multiplier: false,
            },
        ),
        (
            "seventy",
            WordInfo {
                value: 70.0,
                is_multiplier: false,
            },
        ),
        (
            "eighty",
            WordInfo {
                value: 80.0,
                is_multiplier: false,
            },
        ),
        (
            "ninety",
            WordInfo {
                value: 90.0,
                is_multiplier: false,
            },
        ),
        (
            "hundred",
            WordInfo {
                value: 100.0,
                is_multiplier: false,
            },
        ),
        // Russian / Ukrainian
        (
            "ноль",
            WordInfo {
                value: 0.0,
                is_multiplier: false,
            },
        ),
        (
            "нуль",
            WordInfo {
                value: 0.0,
                is_multiplier: false,
            },
        ),
        (
            "один",
            WordInfo {
                value: 1.0,
                is_multiplier: false,
            },
        ),
        (
            "одна",
            WordInfo {
                value: 1.0,
                is_multiplier: false,
            },
        ),
        (
            "одне",
            WordInfo {
                value: 1.0,
                is_multiplier: false,
            },
        ),
        (
            "два",
            WordInfo {
                value: 2.0,
                is_multiplier: false,
            },
        ),
        (
            "две",
            WordInfo {
                value: 2.0,
                is_multiplier: false,
            },
        ),
        (
            "дві",
            WordInfo {
                value: 2.0,
                is_multiplier: false,
            },
        ),
        (
            "три",
            WordInfo {
                value: 3.0,
                is_multiplier: false,
            },
        ),
        (
            "четыре",
            WordInfo {
                value: 4.0,
                is_multiplier: false,
            },
        ),
        (
            "чотири",
            WordInfo {
                value: 4.0,
                is_multiplier: false,
            },
        ),
        (
            "пять",
            WordInfo {
                value: 5.0,
                is_multiplier: false,
            },
        ),
        (
            "п'ять",
            WordInfo {
                value: 5.0,
                is_multiplier: false,
            },
        ),
        (
            "шесть",
            WordInfo {
                value: 6.0,
                is_multiplier: false,
            },
        ),
        (
            "шість",
            WordInfo {
                value: 6.0,
                is_multiplier: false,
            },
        ),
        (
            "семь",
            WordInfo {
                value: 7.0,
                is_multiplier: false,
            },
        ),
        (
            "сім",
            WordInfo {
                value: 7.0,
                is_multiplier: false,
            },
        ),
        (
            "восемь",
            WordInfo {
                value: 8.0,
                is_multiplier: false,
            },
        ),
        (
            "вісім",
            WordInfo {
                value: 8.0,
                is_multiplier: false,
            },
        ),
        (
            "девять",
            WordInfo {
                value: 9.0,
                is_multiplier: false,
            },
        ),
        (
            "дев'ять",
            WordInfo {
                value: 9.0,
                is_multiplier: false,
            },
        ),
        (
            "десять",
            WordInfo {
                value: 10.0,
                is_multiplier: false,
            },
        ),
        (
            "одиннадцать",
            WordInfo {
                value: 11.0,
                is_multiplier: false,
            },
        ),
        (
            "одинадцять",
            WordInfo {
                value: 11.0,
                is_multiplier: false,
            },
        ),
        (
            "двенадцать",
            WordInfo {
                value: 12.0,
                is_multiplier: false,
            },
        ),
        (
            "дванадцять",
            WordInfo {
                value: 12.0,
                is_multiplier: false,
            },
        ),
        (
            "тринадцать",
            WordInfo {
                value: 13.0,
                is_multiplier: false,
            },
        ),
        (
            "тринадцять",
            WordInfo {
                value: 13.0,
                is_multiplier: false,
            },
        ),
        (
            "четырнадцать",
            WordInfo {
                value: 14.0,
                is_multiplier: false,
            },
        ),
        (
            "чотирнадцять",
            WordInfo {
                value: 14.0,
                is_multiplier: false,
            },
        ),
        (
            "пятнадцать",
            WordInfo {
                value: 15.0,
                is_multiplier: false,
            },
        ),
        (
            "п'ятнадцять",
            WordInfo {
                value: 15.0,
                is_multiplier: false,
            },
        ),
        (
            "шестнадцать",
            WordInfo {
                value: 16.0,
                is_multiplier: false,
            },
        ),
        (
            "шістнадцять",
            WordInfo {
                value: 16.0,
                is_multiplier: false,
            },
        ),
        (
            "семнадцать",
            WordInfo {
                value: 17.0,
                is_multiplier: false,
            },
        ),
        (
            "сімнадцять",
            WordInfo {
                value: 17.0,
                is_multiplier: false,
            },
        ),
        (
            "восемнадцать",
            WordInfo {
                value: 18.0,
                is_multiplier: false,
            },
        ),
        (
            "вісімнадцять",
            WordInfo {
                value: 18.0,
                is_multiplier: false,
            },
        ),
        (
            "девятнадцать",
            WordInfo {
                value: 19.0,
                is_multiplier: false,
            },
        ),
        (
            "дев'ятнадцять",
            WordInfo {
                value: 19.0,
                is_multiplier: false,
            },
        ),
        (
            "двадцать",
            WordInfo {
                value: 20.0,
                is_multiplier: false,
            },
        ),
        (
            "двадцять",
            WordInfo {
                value: 20.0,
                is_multiplier: false,
            },
        ),
        (
            "тридцать",
            WordInfo {
                value: 30.0,
                is_multiplier: false,
            },
        ),
        (
            "тридцять",
            WordInfo {
                value: 30.0,
                is_multiplier: false,
            },
        ),
        (
            "сорок",
            WordInfo {
                value: 40.0,
                is_multiplier: false,
            },
        ),
        (
            "пятьдесят",
            WordInfo {
                value: 50.0,
                is_multiplier: false,
            },
        ),
        (
            "п'ятдесят",
            WordInfo {
                value: 50.0,
                is_multiplier: false,
            },
        ),
        (
            "шестьдесят",
            WordInfo {
                value: 60.0,
                is_multiplier: false,
            },
        ),
        (
            "шістдесят",
            WordInfo {
                value: 60.0,
                is_multiplier: false,
            },
        ),
        (
            "семьдесят",
            WordInfo {
                value: 70.0,
                is_multiplier: false,
            },
        ),
        (
            "сімдесят",
            WordInfo {
                value: 70.0,
                is_multiplier: false,
            },
        ),
        (
            "восемьдесят",
            WordInfo {
                value: 80.0,
                is_multiplier: false,
            },
        ),
        (
            "вісімдесят",
            WordInfo {
                value: 80.0,
                is_multiplier: false,
            },
        ),
        (
            "девяносто",
            WordInfo {
                value: 90.0,
                is_multiplier: false,
            },
        ),
        (
            "сто",
            WordInfo {
                value: 100.0,
                is_multiplier: false,
            },
        ),
        (
            "двести",
            WordInfo {
                value: 200.0,
                is_multiplier: false,
            },
        ),
        (
            "двісті",
            WordInfo {
                value: 200.0,
                is_multiplier: false,
            },
        ),
        (
            "триста",
            WordInfo {
                value: 300.0,
                is_multiplier: false,
            },
        ),
        (
            "четыреста",
            WordInfo {
                value: 400.0,
                is_multiplier: false,
            },
        ),
        (
            "чотириста",
            WordInfo {
                value: 400.0,
                is_multiplier: false,
            },
        ),
        (
            "пятьсот",
            WordInfo {
                value: 500.0,
                is_multiplier: false,
            },
        ),
        (
            "п'ятсот",
            WordInfo {
                value: 500.0,
                is_multiplier: false,
            },
        ),
        (
            "шестьсот",
            WordInfo {
                value: 600.0,
                is_multiplier: false,
            },
        ),
        (
            "шістсот",
            WordInfo {
                value: 600.0,
                is_multiplier: false,
            },
        ),
        (
            "семьсот",
            WordInfo {
                value: 700.0,
                is_multiplier: false,
            },
        ),
        (
            "сімсот",
            WordInfo {
                value: 700.0,
                is_multiplier: false,
            },
        ),
        (
            "восемьсот",
            WordInfo {
                value: 800.0,
                is_multiplier: false,
            },
        ),
        (
            "вісімсот",
            WordInfo {
                value: 800.0,
                is_multiplier: false,
            },
        ),
        (
            "девятьсот",
            WordInfo {
                value: 900.0,
                is_multiplier: false,
            },
        ),
        (
            "дев'ятсот",
            WordInfo {
                value: 900.0,
                is_multiplier: false,
            },
        ),
        // Multipliers
        (
            "тысяча",
            WordInfo {
                value: 1_000.0,
                is_multiplier: true,
            },
        ),
        (
            "тысячи",
            WordInfo {
                value: 1_000.0,
                is_multiplier: true,
            },
        ),
        (
            "тысяч",
            WordInfo {
                value: 1_000.0,
                is_multiplier: true,
            },
        ),
        (
            "тыс",
            WordInfo {
                value: 1_000.0,
                is_multiplier: true,
            },
        ),
        (
            "тыщ",
            WordInfo {
                value: 1_000.0,
                is_multiplier: true,
            },
        ),
        (
            "тисяча",
            WordInfo {
                value: 1_000.0,
                is_multiplier: true,
            },
        ),
        (
            "тисячі",
            WordInfo {
                value: 1_000.0,
                is_multiplier: true,
            },
        ),
        (
            "тисяч",
            WordInfo {
                value: 1_000.0,
                is_multiplier: true,
            },
        ),
        (
            "thousand",
            WordInfo {
                value: 1_000.0,
                is_multiplier: true,
            },
        ),
        (
            "миллион",
            WordInfo {
                value: 1_000_000.0,
                is_multiplier: true,
            },
        ),
        (
            "миллиона",
            WordInfo {
                value: 1_000_000.0,
                is_multiplier: true,
            },
        ),
        (
            "миллионов",
            WordInfo {
                value: 1_000_000.0,
                is_multiplier: true,
            },
        ),
        (
            "мільйон",
            WordInfo {
                value: 1_000_000.0,
                is_multiplier: true,
            },
        ),
        (
            "млн",
            WordInfo {
                value: 1_000_000.0,
                is_multiplier: true,
            },
        ),
        (
            "million",
            WordInfo {
                value: 1_000_000.0,
                is_multiplier: true,
            },
        ),
        (
            "миллиард",
            WordInfo {
                value: 1_000_000_000.0,
                is_multiplier: true,
            },
        ),
        (
            "миллиарда",
            WordInfo {
                value: 1_000_000_000.0,
                is_multiplier: true,
            },
        ),
        (
            "миллиардов",
            WordInfo {
                value: 1_000_000_000.0,
                is_multiplier: true,
            },
        ),
        (
            "мільярд",
            WordInfo {
                value: 1_000_000_000.0,
                is_multiplier: true,
            },
        ),
        (
            "млрд",
            WordInfo {
                value: 1_000_000_000.0,
                is_multiplier: true,
            },
        ),
        (
            "billion",
            WordInfo {
                value: 1_000_000_000.0,
                is_multiplier: true,
            },
        ),
        (
            "and",
            WordInfo {
                value: 0.0,
                is_multiplier: false,
            },
        ),
        (
            "и",
            WordInfo {
                value: 0.0,
                is_multiplier: false,
            },
        ),
    ]
    .iter()
    .copied()
    .collect()
});
