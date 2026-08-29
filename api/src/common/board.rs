use core::sync::atomic::{AtomicBool, Ordering};

use crate::common::Block;


static BOARD_CREATED: AtomicBool = AtomicBool::new(false);

pub struct Board<'a, const N:usize>
{
    blocks: [&'a mut dyn Block; N]
}

impl<'a, const N:usize> Board<'a, N>
{
    pub fn take(blocks: [&'a mut dyn Block; N]) -> Option<Board<'a, N>>
    {
        match BOARD_CREATED.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        {
            Ok(_) =>
            {
                let mut board = Board{blocks};
                for block in &mut board.blocks 
                {
                    unsafe { block.start(); }
                }
                return Some(board);
            }
            Err(_) => {return None;}
        }
    }
}
