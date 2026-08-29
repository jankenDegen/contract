use crate::processor::Processor;
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    Processor::process(program_id, accounts, instruction_data)
}

#[cfg(not(feature = "no-entrypoint"))]
use solana_security_txt::security_txt;

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {

    // Required fields
    name: "JANKEN-DEGEN",
    project_url: "https://jankendegen.xyz/",
    contacts: "email:talkingcat@jankendegen.xyz",
    policy: "Don't ask why I can talk ",

    // Optional Fields
    preferred_languages: "english"
}
