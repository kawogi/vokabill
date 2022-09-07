use std::{io, fs::{read_to_string, self}};

use rand::{thread_rng, Rng, seq::SliceRandom};
use serde_derive::{Deserialize, Serialize};

fn input() -> String {
    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();
    line.trim().to_string()
}

const LEVEL_COUNT: u32 = 5;
const LEARNING_EXPONENT: i32 = 3;

// TODO
// line edit (cursor)
// colorize results
// different error levels (missing "to …", wrong capitalization, missing punctuation)
// show errors
// clear screen
// unique IDs to avoid duplicates
// bigger exponent for better learning of new words
// prevent crash when 0/1 words are in the dictionary
// randomize within level


#[derive(Deserialize, Serialize, Debug, Default)]
struct Vocabulary {
    description: String,
    words: Vec<Word>,
}

impl Vocabulary {

    fn add(&mut self, b: impl ToString, a: impl ToString) {
        self.words.push(Word {
            level: 0,
            ok: 0,
            warn: 0,
            minor: 0,
            fail: 0,
            variants: vec![Variant::new_simple(a, b)],
        });
    }

    fn get_random_word(&mut self) -> &mut Word {
        self.words.sort_unstable_by_key(|word| word.level);

        let mut rng = thread_rng();
        let x: f32 = rng.gen::<f32>().powi(LEARNING_EXPONENT);
        let index = (x * self.words.len() as f32).floor() as usize;

        let level = self.words[index].level;

        let mut start_index = index;
        while start_index > 0 && self.words[start_index - 1].level == level {
            start_index -= 1;
        }

        let mut end_index = index + 1;
        while end_index < self.words.len() && self.words[end_index].level == level {
            end_index += 1;
        }

        assert_eq!(self.words[start_index].level, level);
        if start_index > 0 {
            assert_ne!(self.words[start_index - 1].level, level);
        }
        assert_eq!(self.words[end_index - 1].level, level);
        if end_index < self.words.len() {
            assert_ne!(self.words[end_index].level, level);
        }

        let index = rng.gen_range(start_index..end_index);
        &mut self.words[index]
        
        // let mut level = 0;
        // while index >= vocabulary[level].len() {
        //     index -= vocabulary[level].len();
        //     level += 1;
        // }
        // let (de, en) = &vocabulary[level][index];
    }

    pub(crate) fn avg_level(&self, level_count: u32) -> Option<f32> {
        let (count, sum) = self.words.iter()
                .filter(|word| word.has_been_asked())
                .fold((0_u32, 0), |(count, sum), word| (count + 1, sum + word.level.min(level_count - 1) ));
        (count > 0).then(|| sum as f32 / count as f32)
    }

    pub(crate) fn print_stats(&self) {
        
    }

}

#[derive(Deserialize, Serialize, Debug)]
struct Word {
    level: u32,
    ok: u32,
    warn: u32,
    minor: u32,
    fail: u32,
    variants: Vec<Variant>,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
enum Result {
    Fail,
    Minor,
    Warn,
    Ok,
}

// impl Result {
//     fn rating(&self) -> u32 {
//         match self {
//             Result::Ok => 3,
//             Result::Warn => 2,
//             Result::Minor => 1,
//             Result::Fail => 0,
//         }
//     }
// }

// impl PartialEq for Result {
//     fn eq(&self, other: &Self) -> bool {
//         match (self, other) {
//             (Self::Ok(_), Self::Ok(_)) |
//             (Self::Warn(_), Self::Warn(_)) |
//             (Self::Minor(_), Self::Minor(_)) |
//             (Self::Fail(_), Self::Fail(_)) => true,
//             _ => false,
//         }
//     }
// }

// impl Eq for Result {
// }

// impl PartialOrd for Result {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         Some(self.cmp(&other))
//     }
// }

// impl Ord for Result {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         self.rating().cmp(&other.rating())
//     }
// }

impl Word {

    fn variant(&self) -> &Variant {
        let mut rng = thread_rng();
        self.variants.choose(&mut rng).unwrap()
    }

    fn apply_result(&mut self, result: Result) {
        match result {
            Result::Ok => {
                self.ok += 1;
                self.level += 1;
            }
            Result::Warn => {
                self.warn += 1;
            },
            Result::Minor => {
                self.minor += 1;
                if self.level > 0 {
                    self.level -= 1;
                }
            },
            Result::Fail => {
                self.fail += 1;
                self.level = 0;
            },
        }
    }

    fn has_been_asked(&self) -> bool {
        self.ok > 0 || self.warn > 0 || self.minor > 0 || self.fail > 0
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct Variant {
    a: Vec<String>,
    b: Vec<String>,
}

impl Variant {

    fn new_simple(a: impl ToString, b: impl ToString) -> Self {
        Self {
            a: vec![a.to_string()],
            b: vec![b.to_string()],
        }
    }

    fn query_b(&self) -> Query {
        let mut rng = thread_rng();

        Query {
            query: &self.b, // [rng.gen_range(0..self.b.len())]
            expected: &self.a
        }
    }

}


struct Query<'a> {
    query: &'a [String],
    expected: &'a [String],
}

impl<'a> Query<'a> {

    fn compare_all(&self, input: &str) -> Result {
        let mut best_result = None;
        for expected in self.expected {
            let result = Self::compare(expected, input);
            if best_result.as_ref().map_or(true, |best_result| result > *best_result) {
                best_result = Some(result);
            }
        }

        best_result.unwrap_or(Result::Fail)
    }

    fn compare(expected: &str, input: &str) -> Result {

        if input.eq(expected) {
            return Result::Ok;
        }

        let input = Self::fuzzy1(input);
        let expected = Self::fuzzy1(expected);

        if input == expected {
            return Result::Warn;
        }

        let input = Self::fuzzy2(&input);
        let expected = Self::fuzzy2(&expected);

        
        if input == expected {
            return Result::Minor;
        }
        
        Result::Fail
    }

    fn fuzzy1(s: &str) -> String {
        let mut fuzzed = s.trim_matches(|c| [' ', '_', '?', '.', '!'].contains(&c));
        if fuzzed.starts_with("to ") {
            fuzzed = &fuzzed[3..];
        }
        fuzzed.to_owned()
    }

    fn fuzzy2(s: &str) -> String {
        let mut fuzzed = s.to_ascii_lowercase();
        fuzzed.retain(|c| ![' ', '-', ',', '_', '\''].contains(&c));
        fuzzed
    }

}

fn main() {

    let path = "vokabeln2.json"; //std::env::args().nth(1).unwrap();

    //let mut vocabulary = Vocabulary::default();

    let json_str = read_to_string(path).unwrap();
    let mut vocabulary = serde_json::from_str::<Vocabulary>(&json_str).unwrap();

    // vocabulary.add("Zoo", "zoo");
    // vocabulary.add("Bär", "bear");
    // vocabulary.add("Elefant", "elephant");
    // vocabulary.add("Giraffe", "giraffe");
    // vocabulary.add("Löwe", "lion");
    // vocabulary.add("Affe", "monkey");
    // vocabulary.add("Schwein", "pig");
    // vocabulary.add("Schlange", "snake");
    // vocabulary.add("Wal", "whale");
    // vocabulary.add("gut", "good");
    // vocabulary.add("Haustier", "pet");
    // vocabulary.add("Ameise", "ant");
    // vocabulary.add("Vogel", "bird");
    // vocabulary.add("Schmetterling", "butterfly");
    // vocabulary.add("Katze", "cat");
    // vocabulary.add("Hund", "dog");
    // vocabulary.add("Frosch", "frog");
    // vocabulary.add("Meerschweinchen", "guinea pig");
    // vocabulary.add("Pferd", "horse");
    // vocabulary.add("Kaninchen", "rabbit");
    // vocabulary.add("Ratte", "rat");
    // vocabulary.add("ein Elefant", "an elephant");
    // vocabulary.add("denken, glauben", "to think");
    // vocabulary.add("oder", "or");
    // vocabulary.add("aber", "but");
    // vocabulary.add("_ ist kein gutes Haustier", "_ isn't a good pet");
    // vocabulary.add("seltsam, komisch", "strange");
    // vocabulary.add("Spiel", "game");
    // vocabulary.add("sagen", "to say");
    // vocabulary.add("neu", "new");
    // vocabulary.add("(Schul-)Klasse", "class");
    // vocabulary.add("Lass mich dir _ zeigen.", "Let me show you _.");
    // vocabulary.add("mich; mir", "me");
    // vocabulary.add("zeigen", "to show");
    // vocabulary.add("unser/e", "our");
    // vocabulary.add("Klassenzimmer", "classroom");
    // vocabulary.add("Tafel", "board");
    // vocabulary.add("Schrank", "cupboard");
    // vocabulary.add("Stuhl", "chair");
    // vocabulary.add("(Wand-,Stand-,Turm-)Uhr", "clock");
    // vocabulary.add("Armbanduhr", "watch");
    // vocabulary.add("Tür", "door");
    // vocabulary.add("Schreibtisch", "desk");
    // vocabulary.add("Fenster", "window");
    // vocabulary.add("dies ist _ / das ist _", "this is _");
    // vocabulary.add("kommen", "to come");
    // vocabulary.add("Englisch; englisch", "English");
    // vocabulary.add("(Unterrichts-)Stunde", "lesson");
    // vocabulary.add("Schultasche", "school bag");
    // vocabulary.add("Buch", "book");
    // vocabulary.add("Schulheft, Übungsheft", "exercise book");
    // vocabulary.add("Lineal", "ruler");
    // vocabulary.add("Kugelschreiber, Stift, Füller", "pen");
    // vocabulary.add("Bleistift", "pencil");
    // vocabulary.add("Federmäppchen", "pencil case");
    // vocabulary.add("Klebestift", "glue stick");
    // vocabulary.add("Klebstoff", "glue");
    // vocabulary.add("Radiergummi", "rubber");
    // vocabulary.add("Anspitzer", "sharpener");
    // vocabulary.add("nun, jetzt", "now");
    // vocabulary.add("Simon sagt _", "Simon says _");
    // vocabulary.add("legen, stellen, (etwas wohin) tun", "to put");
    // vocabulary.add("reden (mit), sich unterhalten (mit)", "to talk");
    // vocabulary.add("Lehrer/in", "teacher");
    // vocabulary.add("öffnen, aufmachen", "to open");
    // vocabulary.add("berühren, anfassen", "to touch");
    // vocabulary.add("geben", "to give");

    // vocabulary.add("der erste Tag", "the first day");
    // vocabulary.add("in der Schule", "at school");
    // vocabulary.add("vor der Schule (vor Schulbeginn)", "before school");
    // vocabulary.add("vorm Unterricht", "before lessons");
    // vocabulary.add("ihr bester Freund / ihre beste Freundin", "her best friend");
    // vocabulary.add("sie sind _", "they are _"); // they're
    // vocabulary.add("Wohnung", "flat");
    // vocabulary.add("ihr erster Tag", "their first day");
    // vocabulary.add("(Schul-)Uniform", "uniform");
    // vocabulary.add("nett, schön", "nice");
    // vocabulary.add("wissen; kennen", "to know");
    // vocabulary.add("Bruder", "brother");
    // vocabulary.add("Schwester", "sister");
    // vocabulary.add("Schüler/in, Student/in", "student");
    // vocabulary.add("er/sie/es ist nicht _ / ist keine _", "he/she/it is not _"); // he/she/it isn't _
    // vocabulary.add("seine Mama", "his mum");
    // vocabulary.add("Mama, Mutti", "mum");
    // vocabulary.add("Mutter", "mother");
    // vocabulary.add("Papa, Vati", "dad");
    // vocabulary.add("Vater", "father");
    // vocabulary.add("Marine", "navy");
    // vocabulary.add("dunkel", "dark");
    // vocabulary.add("Danke.", "Thank you.");
    // vocabulary.add("mit", "with");
    // vocabulary.add("ohne", "without");
    // vocabulary.add("Du bist spät dran. / Du bist zu spät.", "You're late.");
    // vocabulary.add("warten auf", "to wait for");
    // vocabulary.add("Minute", "minute");
    // vocabulary.add("Wie spät ist es?", "What time is it?");
    // vocabulary.add("Zeit; Uhrzeit", "time");
    // vocabulary.add("Bis gleich. / Bis bald.", "See you.");
    // vocabulary.add("Profil; Beschreibung, Porträt", "profile");
    // vocabulary.add("Bibliothek, Bücherei", "library");
    // vocabulary.add("Heim, Zuhause", "home");
  
    // vocabulary.add("auf dem Weg zu/nach _", "on the way to _");
    // vocabulary.add("du bist nicht/keine _ / ihr seid nicht/keine _", "you are not _"); // you aren't _
    // vocabulary.add("da, dort; dahin, dorthin", "there");
    // vocabulary.add("traurig", "sad");
    // vocabulary.add("Geh nicht.", "Don't go.");
    // vocabulary.add("Tschüs.", "Bye.");
    // vocabulary.add("Pass auf! Vorsicht!", "Watch out!");
    // vocabulary.add("Tut mir leid / Entschuldigung", "I'm sorry");
    // vocabulary.add("Und? / Na und?", "So?");
    // vocabulary.add("Freut mich/dich/euch Sie kennenzulernen.", "Nice to meet you.");
    // vocabulary.add("sich beeilen", "to hurry up");
    // vocabulary.add("freundlich", "friendly");
    // vocabulary.add("Schuh", "shoe");
    // vocabulary.add("Turnschuh", "trainer");
    // vocabulary.add("vergessen", "to forget");
    // vocabulary.add("tragen (Kleidung)", "to wear");
    // vocabulary.add("britisch", "British");
    // vocabulary.add("verschieden; anders", "different");
    // vocabulary.add("haben", "to have");
    // vocabulary.add("Ich mag Grün nicht. / Ich mag kein Grün.", "I don't like green.");
    // vocabulary.add("Versuch's mal.", "Have a go.");

    //fs::write(path, serde_json::to_string_pretty(&vocabulary).unwrap()).unwrap();

    loop {
        let avg = vocabulary.avg_level(LEVEL_COUNT);
        let word = vocabulary.get_random_word();
        let variant = word.variant();
        let query = variant.query_b();

        println!();
        print!("Vokabelnote {}", LEVEL_COUNT - word.level.min(LEVEL_COUNT - 1));

        if let Some(avg) = avg {
            print!(" / Gesamtnote {}", LEVEL_COUNT as f32 - avg);
        }
        println!();
        println!();

        for query in query.query {
            println!("\t{query}");
        }

        let line = input();
        if &line == "X" {
            break;
        }

        let result = query.compare_all(line.trim());

        match result {
            Result::Ok => {
                println!("Richtig!");
            },
            Result::Warn => {
                println!("Fast richtig:");
                for expected in query.expected {
                    println!("\t{expected}");
                }
            },
            Result::Minor => {
                println!("Nicht ganz richtig:");
                for expected in query.expected {
                    println!("\t{expected}");
                }
            },
            Result::Fail => {
                println!("Fehler:");
                for expected in query.expected {
                    println!("\t{expected}");
                }
            },
        }

        word.apply_result(result);
        vocabulary.print_stats();


        fs::write(path, serde_json::to_string_pretty(&vocabulary).unwrap()).unwrap();
    }
}
