use exrunner::ExRunner;
use std::cmp::Ordering;
use std::io::BufRead;
use std::collections::HashMap;

type Card = u8;
type CType = u8;

const FIVE_KIND: CType = 10;
const FOUR_KIND: CType = 9;
const FULL_HOUSE: CType = 8;
const THREE_KIND: CType = 7;
const TWO_PAIR: CType = 6;
const ONE_PAIR: CType = 5;
const HIGH_CARD: CType = 4;

#[derive(Debug)]
struct Hand {
    cards: [Card;5],
    cardtype: CType,
    bid: i64,
}

fn parse(input: impl BufRead) -> Vec<Hand> {
    let mut card2value = HashMap::from([
        ('A', 14 as Card),
        ('K', 13 as Card),
        ('Q', 12 as Card),
        ('J', 11 as Card),
        ('T', 10 as Card),
    ]);
    for num in 2..10 {
        card2value.insert(char::from_digit(num, 10).unwrap(), num as Card);
    }
    input.lines().map(|l| {
        let inline = l.expect("Error reading input");
        let mut inwords = inline.split_whitespace();
        let cardstr = inwords.next().expect("Need cards in input line");
        assert_eq!(cardstr.len(), 5, "Expect 5 cards in input");
        let bid = inwords.next().expect("need bid on input line").parse().expect("bid should be numeric");
        let mut cards = [0 as Card;5];
        let mut numvals = HashMap::new();
        for (i, c) in cardstr.as_bytes().into_iter().enumerate() {
            if let Some(cardval) = card2value.get(&(*c as char)) {
                cards[i] = *cardval;
                numvals.entry(cardval).and_modify(|c| *c += 1).or_insert(1);
            } else {
                panic!("Invalid card in input");
            }
        }
        let mut maxnums: Vec<i32> = numvals.into_values().collect();
        maxnums.sort_by(|a, b| b.cmp(a));
        let cardtype = getcardtype(&maxnums);
        Hand{ cards, cardtype, bid }
    }).collect()
}

fn getcardtype(maxnums: &Vec<i32>) -> CType {
    if maxnums[0] == 5 {
        FIVE_KIND
    } else if maxnums[0] == 4 {
        FOUR_KIND
    } else if maxnums[0] == 3 && maxnums[1] == 2 {
        FULL_HOUSE
    } else if maxnums[0] == 3 {
        THREE_KIND
    } else if maxnums[0] == 2 && maxnums[1] == 2 {
        TWO_PAIR
    } else if maxnums[0] == 2 {
        ONE_PAIR
    } else {
        HIGH_CARD
    }
}

pub fn solve(input: impl BufRead, er: &mut ExRunner) {
    let mut hands = parse(input);
    er.parse_done();
    hands.sort_by(|a, b| {
        let ord = a.cardtype.cmp(&b.cardtype);
        if ord == Ordering::Equal {
            a.cards.cmp(&b.cards)
        } else {
            ord
        }
    });
    // er.debugln(&format!("Sorted hands: {:?}", hands));
    er.part1(hands.iter().enumerate().map(|(i, h)| (i as i64 + 1) * h.bid).sum::<i64>(), Some("Total winnings"));
    // change hands to deal with J == Joker
    let mut hands2: Vec<_> = hands.into_iter().map(|h| {
        let mut cards = h.cards;
        let mut numvals = HashMap::new();
        for i in 0..5 {
            if cards[i] == 11 as Card {
                cards[i] = 0;
            }
            numvals.entry(cards[i]).and_modify(|c| *c += 1).or_insert(1);
        }
        let jokers = numvals.remove(&0).unwrap_or(0);
        let mut maxnums: Vec<i32> = numvals.into_values().collect();
        maxnums.sort_by(|a, b| b.cmp(a));
        // use the jokers as the maximum
        if maxnums.len() == 0 {
            // everything is a joker!
            maxnums = vec![jokers];
        } else {
            maxnums[0] += jokers;
        }
        let cardtype = getcardtype(&maxnums);
        Hand{ cards, cardtype, bid: h.bid }
    }).collect();
    hands2.sort_by(|a, b| {
        let ord = a.cardtype.cmp(&b.cardtype);
        if ord == Ordering::Equal {
            a.cards.cmp(&b.cards)
        } else {
            ord
        }
    });
    // er.debugln(&format!("Sorted hands: {:?}", hands2));
    er.part2(hands2.iter().enumerate().map(|(i, h)| (i as i64 + 1) * h.bid).sum::<i64>(), Some("Total winnings via jokers"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    fn test_input() -> BufReader<&'static [u8]> {
        BufReader::new(
"32T3K 765
T55J5 684
KK677 28
KTJJT 220
QQQJA 483
".as_bytes()
        )
    }

    #[test]
    fn test_part12() {
        let er = ExRunner::run("day 7".to_string(), solve, test_input());
        er.print_raw();
        assert_eq!(er.answ()[0], Some("6440".to_string()));
        assert_eq!(er.answ()[1], Some("5905".to_string()));
    }
}
