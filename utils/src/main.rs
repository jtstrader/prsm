use std::env::args;

use prsm::prelude::*;

mod changelog;
mod constants;

fn main() -> Result<(), String> {
    let script_manager = prsm! {
        [1] "Update CHANGELOG" => changelog::write_tag_action_changes(&args().nth(1).unwrap())
    };

    script_manager.run()?;
    Ok(())
}
