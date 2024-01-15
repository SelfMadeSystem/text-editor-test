use std::{cell::RefCell, str::FromStr};

#[derive(Debug, Clone)]
pub struct EditedText {
    original: String,
    added: String,
    pieces: Vec<Piece>,
    cached: RefCell<Option<(String, Vec<String>)>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Which {
    Original,
    Added,
}

#[derive(Debug, Clone)]
pub struct Piece {
    which: Which,
    start: usize,
    end: usize,
}

impl Piece {
    fn len(&self) -> usize {
        self.end - self.start
    }

    fn split(&self, index: usize) -> (Option<Piece>, Option<Piece>) {
        if index == 0 {
            (None, Some(self.clone()))
        } else if index == self.len() {
            (Some(self.clone()), None)
        } else {
            assert!(index < self.len());
            (
                Some(Piece {
                    which: self.which.clone(),
                    start: self.start,
                    end: self.start + index,
                }),
                Some(Piece {
                    which: self.which.clone(),
                    start: self.start + index,
                    end: self.end,
                }),
            )
        }
    }

    fn remove_char(&self, index: usize) -> (Option<Piece>, Option<Piece>) {
        if index == 0 {
            if self.len() == 1 {
                (None, None)
            } else {
                (
                    None,
                    Some(Piece {
                        which: self.which.clone(),
                        start: self.start + 1,
                        end: self.end,
                    }),
                )
            }
        } else if index == self.len() {
            (Some(self.clone()), None)
        } else {
            assert!(index < self.len());
            (
                Some(Piece {
                    which: self.which.clone(),
                    start: self.start,
                    end: self.start + index,
                }),
                Some(Piece {
                    which: self.which.clone(),
                    start: self.start + index + 1,
                    end: self.end,
                }),
            )
        }
    }

    fn try_merge(&self, other: &Piece) -> Option<Piece> {
        if self.which == other.which && self.end == other.start {
            Some(Piece {
                which: self.which.clone(),
                start: self.start,
                end: other.end,
            })
        } else {
            None
        }
    }
}

impl FromStr for EditedText {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(EditedText::new(s))
    }
}

impl Into<String> for EditedText {
    fn into(self) -> String {
        self.to_string()
    }
}

impl From<String> for EditedText {
    fn from(s: String) -> Self {
        EditedText::new(&s)
    }
}

impl From<&str> for EditedText {
    fn from(s: &str) -> Self {
        EditedText::new(s)
    }
}

impl ToString for EditedText {
    fn to_string(&self) -> String {
        if let Some((str, _)) = &*self.cached.borrow() {
            return str.clone();
        }
        let mut result = String::new();
        for piece in &self.pieces {
            match piece.which {
                Which::Original => {
                    result.push_str(&self.original[piece.start..piece.end]);
                }
                Which::Added => {
                    result.push_str(&self.added[piece.start..piece.end]);
                }
            }
        }
        *self.cached.borrow_mut() = Some((
            result.clone(),
            result.split('\n').map(|x| x.to_owned()).collect(),
        ));
        result
    }
}

impl EditedText {
    pub fn new(original: &str) -> Self {
        EditedText {
            original: original.to_owned(),
            added: String::new(),
            pieces: vec![Piece {
                which: Which::Original,
                start: 0,
                end: original.len(),
            }],
            cached: RefCell::new(None),
        }
    }

    pub fn get_lines(&self) -> Vec<String> {
        if let Some((_, lines)) = &*self.cached.borrow() {
            return lines.clone();
        }
        self.to_string();
        self.get_lines()
    }

    fn get_piece(&self, index: usize) -> Option<(&Piece, usize, usize)> {
        let mut index = index;
        let mut i = 0;
        for piece in &self.pieces {
            let len = piece.len();
            if index < len {
                return Some((piece, index, i));
            }
            index -= len;
            i += 1;
        }
        None
    }

    fn replace_piece(&mut self, og_piece_i: usize, pieces: Vec<Option<Piece>>) {
        self.cached.replace(None);

        let mut pieces = pieces.into_iter().filter_map(|x| x).collect::<Vec<_>>();

        if pieces.len() == 0 {
            self.pieces.remove(og_piece_i);

            let prev = self.pieces.get(og_piece_i - 1);
            let next = self.pieces.get(og_piece_i);

            if let (Some(prev), Some(next)) = (prev, next) {
                if let Some(merged_piece) = prev.try_merge(next) {
                    self.pieces[og_piece_i - 1] = merged_piece;
                    self.pieces.remove(og_piece_i);
                    return;
                }
            }
            return;
        }

        let mut i = pieces.len();

        while i > 1 {
            let prev = &pieces[i - 2];
            let piece = &pieces[i - 1];

            if let Some(merged_piece) = prev.try_merge(piece) {
                pieces[i - 2] = merged_piece;
                pieces.remove(i - 1);
                i -= 1;
            } else {
                i -= 1;
            }
        }

        if og_piece_i > 0 {
            let prev = &self.pieces[og_piece_i - 1];
            if let Some(piece) = pieces.first() {
                if let Some(merged_piece) = prev.try_merge(piece) {
                    self.pieces[og_piece_i - 1] = merged_piece;
                    pieces.remove(0);
                }
            }
        }
        if og_piece_i + 1 < self.pieces.len() {
            let next = &self.pieces[og_piece_i + 1];
            if let Some(piece) = pieces.last() {
                if let Some(merged_piece) = piece.try_merge(next) {
                    self.pieces[og_piece_i] = merged_piece;
                    pieces.pop();
                }
            }
        }

        if pieces.len() == 0 {
            return;
        }

        if og_piece_i == self.pieces.len() {
            self.pieces.extend(pieces);
            return;
        }
        self.pieces.splice(og_piece_i..og_piece_i + 1, pieces);
    }

    fn add_piece(&mut self, piece: Piece, index: usize) {
        let Some((old_piece, old_piece_index, old_piece_i)) = self.get_piece(index) else {
            self.replace_piece(self.pieces.len(), vec![Some(piece)]);
            return;
        };
        let (left, right) = old_piece.split(old_piece_index);
        self.replace_piece(old_piece_i, vec![left, Some(piece), right]);
    }

    pub fn remove_char(&mut self, index: usize) {
        let Some((old_piece, old_piece_index, old_piece_i)) = self.get_piece(index) else {
            return;
        };
        let (left, right) = old_piece.remove_char(old_piece_index);
        println!(
            "old_piece: {:?}, left: {:?}, right: {:?}",
            old_piece, left, right
        );
        self.replace_piece(old_piece_i, vec![left, right]);
    }

    pub fn add_char(&mut self, c: char, index: usize) {
        let piece = Piece {
            which: Which::Added,
            start: self.added.len(),
            end: self.added.len() + c.len_utf8(),
        };
        self.added.push(c);
        self.add_piece(piece, index);
    }
}
