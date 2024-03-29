//use nom::types::CompleteStr;
use regex::{Regex, RegexBuilder, escape};
use crate::types::{State, Group, Tab};


/*named!(atom<CompleteStr, Parsed>,
    do_parse!(
        many0!(char!(' ')) >>
        // TODO better literal test
        literal: is_not!(" ") >>
        (Parsed::Literal(RegExp::new(&RegExp::escape(&literal), "i")))
    )
);

named!(parse<CompleteStr, Parsed>,
    do_parse!(
        init: atom >>
        res: fold_many0!(
            tuple!(
                many1!(char!(' ')),
                atom
            ),
            init,
            |acc, v: (_, _)| {
                Parsed::And(Box::new(acc), Box::new(v.1))
            }
        ) >>
        (res)
    )
);*/


impl Tab {
    pub(crate) fn set_matches_search(&self, matches: bool) {
        self.matches_search.set_neq(matches);
    }
}


impl State {
    pub(crate) fn search_tab(&self, tab: &Tab) {
        let tab_matches = {
            let search_parser = self.search_parser.lock_ref();
            search_parser.matches_tab(tab)
        };

        tab.set_matches_search(tab_matches);
    }
}


#[derive(Debug)]
pub(crate) enum Parsed {
    True,
    Literal(Regex),
    And(Box<Parsed>, Box<Parsed>),
    IsLoaded,
}

impl Parsed {
    // TODO proper parser
    pub(crate) fn new(input: &str) -> Self {
        // TODO a bit hacky
        //parse(CompleteStr(input)).unwrap().1

        input.split(" ")
            .filter(|x| *x != "")
            .map(|x| {
                // TODO make this faster
                match x.splitn(2, ":").collect::<Vec<_>>().as_slice() {
                    [x] => {
                        Parsed::Literal(RegexBuilder::new(&escape(x))
                            .case_insensitive(true)
                            .unicode(false)
                            .build()
                            .unwrap())
                    },
                    [x, y] => {
                        if *x == "is" && *y == "loaded" {
                            Parsed::IsLoaded

                        } else {
                            // TODO error on invalid input
                            Parsed::True
                        }
                    },
                    _ => unreachable!(),
                }
            })
            .fold(Parsed::True, |old, new| {
                if let Parsed::True = old {
                    new

                } else {
                    Parsed::And(Box::new(old), Box::new(new))
                }
            })
    }

    pub(crate) fn matches_tab(&self, tab: &Tab) -> bool {
        match self {
            Parsed::True => true,

            Parsed::Literal(regexp) => {
                let title = tab.title.lock_ref();
                let url = tab.url.lock_ref();

                // TODO make this more efficient ?
                let title = title.as_ref().map(|x| x.as_str()).unwrap_or("");
                let url = url.as_ref().map(|x| x.as_str()).unwrap_or("");

                regexp.is_match(title) || regexp.is_match(url)
            },

            Parsed::And(left, right) => left.matches_tab(tab) && right.matches_tab(tab),

            Parsed::IsLoaded => !tab.state.status.get().is_unloaded(),
        }
    }
}


/*#[cfg(test)]
mod tests {
    #[test]
    fn parse() {
        let parsed = super::Parsed::new("foo.\\d\\D\\w\\W\\s\\S\\t\\r\\n\\v\\f[\\b]\\0\\cM\\x00\\u0000\\u{0000}\\u{00000}\\\\[xyz][a-c][^xyz][^a-c]x|y^$\\b\\B(x)\\1(?:x)x*x+x?x{5}x{5,}x{5,6}x*?x+?x??x{5}?x{5,}?x{5,6}?x(?=y)x(?!y)bar");

        //assert!(parsed.pattern.len() == 1);
        assert!(parsed.matches("foo.\\d\\D\\w\\W\\s\\S\\t\\r\\n\\v\\f[\\b]\\0\\cM\\x00\\u0000\\u{0000}\\u{00000}\\\\[xyz][a-c][^xyz][^a-c]x|y^$\\b\\B(x)\\1(?:x)x*x+x?x{5}x{5,}x{5,6}x*?x+?x??x{5}?x{5,}?x{5,6}?x(?=y)x(?!y)bar"));
    }

    #[test]
    fn and() {
        let parsed = super::Parsed::new("   foo   bar     ");

        //assert!(parsed.pattern.len() == 2);
        assert!(parsed.matches("foo"));
        assert!(parsed.matches("bar"));
        assert!(parsed.matches("Foo"));
        assert!(parsed.matches("bAR"));
    }
}*/
