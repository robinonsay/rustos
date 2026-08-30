pub struct PinHandle<const N: usize>
{
    _private: ()
}

impl<const N: usize> PinHandle<N>
{
    pub const unsafe fn new() -> Self
    {
        Self { _private: () }
    }
}
#[macro_export]
macro_rules! define_board {
    (
        $board:ident{
            $pins:ident {
                $($pin_name:ident: $n:literal),+ $(,)?
            }
        }
    ) =>
    {
        static BOARD_TAKEN:core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);
        pub struct $pins {
            $(pub $pin_name: $crate::device::PinHandle<$n>,)+
        }
        pub struct $board {
            pub pins: $pins
        }

        impl $board {
            const fn new() -> Self {
                unsafe{Self{
                    pins: $pins{$($pin_name: $crate::device::PinHandle::new(), )+}
                }}
            }
            /// Claim the board, once per boot.
            ///
            /// The pin handles inside are zero-sized proofs of ownership, so the only
            /// invariant that needs runtime enforcement is that they are minted once.
            /// This compare-exchange is that enforcement: the first caller gets the
            /// board, every later caller gets `None`.
            pub fn take() -> Option<Self> {
                return BOARD_TAKEN
                    .compare_exchange(false, true, Acquire, Acquire)
                    .ok()
                    .map(|_| Self::new());
            }
        }
    };
}

