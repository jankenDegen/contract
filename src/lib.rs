pub mod admin;
pub mod constants;
pub mod dice;
pub mod entrypoint;
pub mod error;
pub mod initialize;
pub mod instruction;
pub mod janken;
pub mod magicblock_vrf;
pub mod processor;
pub mod raffle;
pub mod russian_roulette;
pub mod state;
pub mod utils;

#[cfg(target_os = "solana")]
#[no_mangle]
pub extern "C" fn abort() -> ! {
    loop {}
}
