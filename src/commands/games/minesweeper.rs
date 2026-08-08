#[ignore]
// TODO: see how to make spoilers and emojis work

use rand::random_range;
use serenity::all::CreateMessage;

use crate::commands::GAMES_CATEGORY;

#[derive(Debug, Clone, Copy)]
enum Tile {
    Bomb,
    Number(u8),
    Empty,
}

impl Tile {
    #[inline]
    pub fn increase_number(&mut self) {
        match self {
            Self::Bomb => (),
            Self::Number(n) => *n += 1,
            Self::Empty => *self = Self::Number(1),
        }
    }

    pub fn as_char(&self) -> &'static str {
        match self {
            Self::Bomb => BOMB,
            Self::Number(n) => NUMBERS[*n as usize],
            Self::Empty => EMPTY,
        }
    }
}

const BOMB: &str = "💥";
const NUMBERS: &[&str] = &["zero","one","two","three","four","five","six","seven","eight"];
const EMPTY: &str = "⬛";

crate::command! {
    names: ["minesweeper","ms"],
    category: GAMES_CATEGORY,
    run: |ctx, msg, _data| {
        let width = random_range(10..=12);
        let height = random_range(10..=12);
        let bombs = width * height / 5;

        let mut board = vec![vec![Tile::Empty; height]; width];

        // populate bombs
        let mut bombs_to_place = bombs;
        let mut max_loops = bombs_to_place * 2; // just for safety
        while bombs_to_place > 0 && max_loops > 0 {
            let x = random_range(0..width);
            let y = random_range(0..height);
            if !matches!(board[x][y], Tile::Bomb) {
                board[x][y] = Tile::Bomb;
                bombs_to_place -= 1;

                // populate numbers
                for rx in -1..=1 {
                    for ry in -1..=1 {
                        if rx == 0 && ry == 0 { continue }
                        if let (Some(nx), Some(ny)) = (x.checked_add_signed(rx), y.checked_add_signed(ry)) && (0..width).contains(&nx) && (0..height).contains(&ny) {
                            board[nx][ny].increase_number();
                        }
                    }
                }
            }
            max_loops -= 1;
        }

        // draw grid
        
        let mut draw = String::with_capacity(47 + width * height * 10); // 5 characters, and numbers can be 5 bytes
        draw.push_str(&format!("Welcome to MineSweeper!\nBoard: {height}x{width}\nBombs: {bombs}\n"));

        for x in 0..width {
            if x != 0 { draw.push('\n'); }
            for y in 0..height {
                draw.push_str("||:");
                draw.push_str(board[x][y].as_char());
                draw.push_str(":||");
            }
        }

        Ok(msg.channel_id.send_message(&ctx.http, CreateMessage::new().content(draw)).await.map(|_| ())?)
    }
}
