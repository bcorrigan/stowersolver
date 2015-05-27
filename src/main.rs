#![feature(collections)]

use std::collections::HashMap;
use std::collections::hash_map::Entry::Vacant;
use std::collections::hash_map::Entry::Occupied;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::cmp::max;
use std::cmp::min;
use std::cmp::Ordering;


struct WordState {
    is_word: bool,
    is_prefix: bool
}

struct Words {
    wordmap:HashMap<String, WordState>
}

impl Words {

    //is this a known word OW a prefix of a known word
    fn has(&self, chars:Vec<char>) -> bool {
        //might need to be chars.to_owned() ??? ie a copy. Or, just iter() ?
        let str = chars.into_iter().collect::<String>();
        self.wordmap.contains_key(&str)
    }

    //does it exist AND is a word
    fn is_word(&self, chars:Vec<char>) -> bool {
        if chars.len()<3 {
            return false;
        }

        let str = chars.into_iter().collect::<String>();
        if self.wordmap.contains_key(&str) {
            return self.wordmap[&str].is_word
        }

        false
    }

    //does it exist AND is a prefix
    fn is_prefix(&self, chars:Vec<char>) -> bool {
        let str = chars.into_iter().collect::<String>();
        if self.wordmap.contains_key(&str) {
            return self.wordmap[&str].is_prefix
        }

        false
    }

    fn read_words() -> Words {
        let mut map:HashMap<String,WordState> = HashMap::new();
        let file = match File::open("dict") {
            Ok(file) => file,
            Err(..)  => panic!("No dict file found."),
        };
        let reader = BufReader::new(&file);

        let mut lines=0;
        for line in reader.lines() {
            add_all(&mut map, line.unwrap());
            lines+=1;
        }

        println!("Read {} words from dict file. {} items in map.", lines, map.len());

        Words{ wordmap:map }
    }
}

struct Board {
    board: Vec<Vec<char>>,
    width: i16,
    height: i16
}

//position in the board
#[derive(PartialEq, Eq, Clone)]
struct Point {
    x: i16,
    y: i16
}

//sequence of points - a path through the board
type Path = Vec<Point>;

impl Board {
    fn read_board() -> Board {
        let mut board:Vec<Vec<char>> = Vec::new();

        let file = match File::open("board") {
            Ok(file) => file,
            Err(..) => panic!("You need to write a board file.")
        };

        let reader = BufReader::new(&file);
        let mut width=0;
        let mut length:i16=0;
        for lopt in reader.lines() {
            let line = lopt.unwrap();
            if width==0 {
                width=line.len();
            }
            if width!=line.len() {
                panic!("The board file must have lines all the same length. But \"{}\" is not the same length as previous lines.", line);
            }
            let v:Vec<char> = line.chars().collect();
            board.push(v);
            length+=1;
        }

        Board{ board:board, width:width as i16, height:length }
    }

    //all the neighbours not in the path
    //always searches entire path for all neighbours, so that could be optimised
    fn neighbours(&self, point:&Point, path:&Path) -> Vec<Point> {
        let mut neighbours:Vec<Point> = Vec::new();
        let xfloor = max(point.x-1, 0);
        let xceil = min(point.x+1, self.width-1)+1;
        let yfloor = max(point.y-1, 0);
        let yceil = min(point.y+1, self.height-1)+1;

        for x in xfloor..xceil {
            for y in yfloor..yceil {
                let p:Point = Point{x:x, y:y};
                if !path.contains(&p) {
                    neighbours.push(p);
                }
            }
        }

        neighbours
    }

    fn to_chars(&self, path:&Path) -> Vec<char> {
        path.into_iter().map(|p| self.char_at(p)).collect()
    }

    fn char_at(&self, point:&Point) -> char {
        self.board[point.y as usize][point.x as usize]
    }

    /*
     * path = the path walked up to now - including this point
     * point = the additional point we are considering
     * 1) Get all the neighbours that don't intersect the path
     * 2) Is each neighbour, added onto path, a valid prefix/word
     * 3)  Yes, a word => add onto known paths
     * 4)  Yes, a prefix => recurse and ponder it too.
     * 5) When all neighbours considered, return the paths
     */
    fn word_walk(&self, mut path:&mut Path, mut known_paths:&mut Vec<Path>, point:&Point, words:&Words) {
        if words.is_word(self.to_chars(&path)) {
            known_paths.push(path.clone());
        }

        let nbours = self.neighbours(point,path);
        for nbour in nbours {
            path.push(nbour.clone());
            if words.has(self.to_chars(&path)) {
                self.word_walk(path,known_paths,&nbour,words);
            }
            path.pop();
        }
    }

    fn to_str(&self, path:&Path) -> String {
        let mut s = String::new();
        for y in 0 .. self.height {
            for x in 0 .. self.width {
                let point = Point{x:x, y:y};
                if path.iter().any( |p| *p == point) {
                    s.push(self.char_at(&point));
                } else {
                    s.push(' ');
                }
            }
            s.push('\n');
        }

        s
    }
}

fn add_all(map: &mut HashMap<String, WordState>, line: String) {
    for i in 1..(line.len()) {
        //let s = String::from_str(line.slice_chars(0,i));
        match map.entry(String::from_str(line.slice_chars(0,i))) {
            Vacant(entry) => {entry.insert(WordState{is_word:false, is_prefix:true}); } ,
            Occupied(mut entry) => {entry.get_mut().is_prefix=true; }
        }
    }
    //let s = line.clone();
    match map.entry(line) {
        Vacant(entry) => { entry.insert(WordState{is_word:true, is_prefix:false}) ; },
        Occupied(mut entry) => { entry.get_mut().is_word=true; }
    }
}

fn main() {
    let words = Words::read_words();

    //can now ask if some letters are a prefix / a word in themselves
    println!("dog is_word:{} is_prefix:{}", words.wordmap["dog"].is_word, words.wordmap["dog"].is_prefix);

    let board = Board::read_board();

    println!("board loaded, 1,1 is: {}", board.board[2][5]);

    let mut word_paths:Vec<Path> = Vec::new();

    for x in 0..board.width {
        for y in 0..board.height {
            let mut first_path:Path = Vec::new();
            let first_point = Point{x:x, y:y};
            first_path.push(first_point);
            board.word_walk(&mut first_path, &mut word_paths, &Point{x:x, y:y}, &words);
        }
    }
    println!("Ran paths on all.. Found {} words.", word_paths.len());

    word_paths.sort_by(|a, b| if a.len()<b.len() {
        return Ordering::Greater;
    } else if a.len()>b.len() {
        return Ordering::Less;
    } else {
        return Ordering::Equal;
    });

    let mut i=0;
    for path in &word_paths {
        let str = board.to_chars(&path).into_iter().collect::<String>();
        i+=1;
        if i==10 {
            std::process::exit(0);
        }
        println!("Path: {}", str);
        println!("MAP:");
        println!("{}", board.to_str(&path));
    }
}
