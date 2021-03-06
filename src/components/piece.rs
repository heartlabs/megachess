use amethyst::ecs::{Component, DenseVecStorage, Entity};

use std::iter::successors;
use crate::resources::board::Board;
use crate::components::bounded::PowerAnimation;
use crate::states::load::Sprites;


#[derive(Debug,PartialEq)]
pub enum EffectKind {
    Protection,
}

impl Move {
    pub fn new(direction: Direction, steps: u8) -> Move {
        Move {
            range: Range::new(direction, steps)
        }
    }
}

#[derive(Debug,Copy,Clone)]
pub struct ActivatablePower {
    pub kind: Power,
    pub range: Range,
}

pub struct Move {
    pub range: Range,
}

#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct TurnInto {
    pub kind: PieceKind,
}

#[derive(Debug,PartialEq,Copy,Clone)]
pub enum Direction {
    Vertical,
    Horizontal,
    Diagonal,
    Straight,
    Star,
    Anywhere
}

#[derive(Debug,Copy,Clone)]
pub enum Power {
    Blast,
    TargetedShoot
}

#[derive(Clone, Copy, Debug)]
pub enum PieceKind {
    Simple,
    HorizontalBar,
    VerticalBar,
    Cross,
    Queen,
    Castle,
    Sniper
}

impl Piece {
    pub fn new(team_id: usize) -> Piece {
        Piece {
            attack: true,
            pierce: true,
            shield: false,
            movement: None,
            activatable: None,
            dying: false,
            exhaustion: Exhaustion::new_exhausted(ExhaustionStrategy::Either),
            team_id,
            exists: true
        }
    }
}

#[derive(Component)]
#[storage(DenseVecStorage)]
pub struct Piece {
    pub attack: bool,
    pub pierce: bool,
    pub shield: bool,
    pub movement: Option<Move>,
    pub activatable: Option<ActivatablePower>,
    pub dying: bool,
    pub exhaustion: Exhaustion,
    pub team_id: usize,
    pub exists: bool // To mark pieces that don't exist in the game but are still stored in the history
}

pub enum ExhaustionStrategy {
    Either,
    Both,
    Move,
    Special
}

pub struct Exhaustion {
    moved: bool,
    used_special: bool,
    strategy: ExhaustionStrategy
}

impl Exhaustion {
    pub(crate) fn new_exhausted(strategy: ExhaustionStrategy) -> Exhaustion {
      Exhaustion {
          moved: true,
          used_special: true,
          strategy
      }
    }

    pub(crate) fn new_rested(strategy: ExhaustionStrategy) -> Exhaustion {
        Exhaustion {
            moved: false,
            used_special: false,
            strategy
        }
    }

    pub fn can_move(&self) -> bool {
        match &self.strategy {
            ExhaustionStrategy::Special => false,
            ExhaustionStrategy::Move | ExhaustionStrategy::Both => !self.moved,
            ExhaustionStrategy::Either => !self.moved && !self.used_special,
        }
    }

    pub fn can_attack(&self) -> bool {
        match &self.strategy {
            ExhaustionStrategy::Move => false,
            ExhaustionStrategy::Special | ExhaustionStrategy::Both => !self.used_special,
            ExhaustionStrategy::Either => !self.moved && !self.used_special,
        }
    }

    pub fn is_done(&self) -> bool {
        return !self.can_move() && !self.can_attack();
    }

    pub fn on_move(&mut self) {
        self.moved = true;
    }

    pub fn undo_move(&mut self) {
        self.moved = false;
    }

    pub fn on_attack(&mut self) {
        self.used_special = true;
    }

    pub fn undo_attack(&mut self) {
        self.used_special = false;
    }

    pub fn reset(&mut self) {
        self.moved = false;
        self.used_special = false;
    }
}

#[derive(Debug,Copy,Clone)]
pub struct Range {
    pub direction: Direction,
    pub steps: u8,
    pub jumps: bool,
    pub include_self: bool,
}

impl Direction {
    fn reaches(&self, from_x: u8, from_y: u8, to_x: u8, to_y: u8) -> bool{
        match &self {
            Direction::Vertical => from_x == to_x,
            Direction::Horizontal => from_y == to_y,
            Direction::Diagonal => (from_y as i16 - to_y as i16).abs() == (from_x as i16 - to_x as i16).abs(),
            Direction::Straight => from_x == to_x || from_y == to_y,
            Direction::Star =>(from_y as i16 - to_y as i16).abs() == (from_x as i16 - to_x as i16).abs() || from_x == to_x || from_y == to_y,
            Direction::Anywhere => true
        }
    }

    fn paths(&self) -> Vec<Box<dyn Fn((i16,i16)) -> (i16,i16)>>{
        match &self {
            Direction::Vertical => vec![
                Box::new(|(x,y)| (x,y+1)),
                Box::new(|(x,y)| (x,y-1))
            ],
            Direction::Horizontal => vec![
                Box::new(|(x,y)| (x+1,y)),
                Box::new(|(x,y)| (x-1,y))
            ],
            Direction::Diagonal => vec![
                Box::new(|(x,y)| (x+1,y+1)),
                Box::new(|(x,y)| (x-1,y-1)),
                Box::new(|(x,y)| (x+1,y-1)),
                Box::new(|(x,y)| (x-1,y+1)),
            ],
            Direction::Straight => vec![
                Box::new(|(x,y)| (x+1,y)),
                Box::new(|(x,y)| (x-1,y)),
                Box::new(|(x,y)| (x,y+1)),
                Box::new(|(x,y)| (x,y-1))
            ],
            Direction::Star => vec![
                Box::new(|(x,y)| (x+1,y+1)),
                Box::new(|(x,y)| (x-1,y-1)),
                Box::new(|(x,y)| (x+1,y-1)),
                Box::new(|(x,y)| (x-1,y+1)),
                Box::new(|(x,y)| (x+1,y)),
                Box::new(|(x,y)| (x-1,y)),
                Box::new(|(x,y)| (x,y+1)),
                Box::new(|(x,y)| (x,y-1))
            ],
            Direction::Anywhere => Vec::new()
        }
    }
}

#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct Effect {
    pub kind: EffectKind,
    pub range: Range,
}

impl Range {
    pub fn new_unlimited(direction: Direction) -> Range {
        Range {direction, steps: 255, jumps: false, include_self: false}
    }

    pub fn new(direction: Direction, steps: u8) -> Range {
        Range {direction, steps, jumps: false, include_self: false}
    }

    pub fn anywhere() -> Range {
        Range {direction: Direction::Anywhere, steps: 255, jumps: true, include_self: false}
    }

    pub fn reaches(&self, from_x: u8, from_y: u8, to_x: u8, to_y: u8) -> bool {
        self.direction.reaches(from_x, from_y, to_x, to_y)
            && (from_y as i16 - to_y as i16).abs() as u8 <= self.steps
            && (from_x as i16 - to_x as i16).abs() as u8 <= self.steps
    }

    pub fn paths(&self, from_x: u8, from_y: u8) -> Box<dyn Iterator<Item=Box<dyn Iterator<Item=(i16,i16)>>>> {
        if self.direction == Direction::Anywhere { // TODO: This only works if jump=true and steps=max
            let row_iter = (0..255 as i16).map(move |x| {
                Box::new((0..255 as i16).map(move |y| (x, y))) as Box<Iterator<Item=(i16, i16)>>
            });
            return Box::new(row_iter);
        }

        let mut vec = self.direction.paths().into_iter().map(move |i| {
            let x: Box<dyn Iterator<Item=(i16,i16)>> = Box::new(successors(
                Some((from_x as i16, from_y as i16)),
                move |&x| Some(i(x)))
                .skip(1)
                .take(self.steps as usize));
            x
        }).collect::<Vec<Box<dyn Iterator<Item=(i16,i16)>>>>();

        if self.include_self {
            vec.push(Box::new(Some((from_x as i16,from_y as i16)).into_iter()));
        }

        Box::new(vec.into_iter())
    }

    pub fn for_each<F>(&self, from_x: u8, from_y: u8, board: &Board, mut perform: F) where F: FnMut(u8,u8,Entity) -> bool {
        for direction in self.paths(from_x, from_y) {
            for (x_i16,y_i16) in direction {
                let (x,y) = (x_i16 as u8, y_i16 as u8);
                if let Some(cell) = board.get_cell_safely(x_i16, y_i16) {
                    let proceed = perform(x,y,cell);
                    if !self.jumps && (board.get_piece(x, y).is_some() || !proceed) {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
    }
}


