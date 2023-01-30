/// Utility functions for common interactions
use anyhow::{anyhow, Result};
use dialoguer::Select;
use log::error;

use crate::{
    colour_output::{ColourOutput, StreamKind, Style},
    progress_indicator::ProgressIndicator,
};
use cvapi::CheckvistClient;

/// Present the user with a dialogue to select one from their lists
/// Returns Ok<Some<list_id, list_name>> on selection, or Ok<None>
/// if the user cancels
pub fn user_select_list(client: &CheckvistClient) -> Result<Option<(u32, String)>> {
    let lists = get_lists(client)?;
    Ok(select_list(lists))
}

fn get_lists(client: &CheckvistClient) -> Result<Vec<(u32, String)>> {
    let before_get_lists = || println!("Fetching lists from checkvist");

    let mut available_lists: Vec<(u32, String)> = Vec::new();
    ProgressIndicator::new('.', Box::new(before_get_lists), 250).run(|| {
        client
            .get_lists()
            .map(|lists| {
                available_lists = lists.into_iter().map(|list| (list.id, list.name)).collect()
            })
            .map_err(|e| anyhow!(e))
    })?;

    Ok(available_lists)
}

fn select_list(lists: Vec<(u32, String)>) -> Option<(u32, String)> {
    println!("Use arrow keys (or j/k) to pick a list. Enter/Space to choose. ESC to cancel\n");
    {
        let lists: &[(u32, String)] = &lists;
        let ids: Vec<&str> = lists.iter().map(|list| list.1.as_str()).collect();
        Select::new()
            .items(&ids)
            .interact_opt()
            // discard error here - nothing we can do so log & continue with None
            .map_err(|e| error!("{:?}", e))
            .ok()
            .flatten()
            // get list id and name as Ok val
            .map(|index| {
                lists
                    .get(index)
                    // if expect isn't safe here it's a lib (dialoguer) bug
                    .expect("Internal error getting list from user")
                    .to_owned()
            })
    }
    .map(|list| {
        ColourOutput::new(StreamKind::Stdout)
            .append("You picked list '", Style::Normal)
            .append(&list.1, Style::ListName)
            .println()
            .expect("Problem printing colour output");

        list
    })
}
