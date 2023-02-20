/// Utility functions for common interactions
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use dialoguer::{Confirm, Select};
use log::error;

use crate::{
    colour_output::{ColourOutput, StreamKind, Style},
    config::Config,
    progress_indicator::ProgressIndicator,
};
use cvapi::CheckvistClient;

/// Present the user with a dialogue to select one from their lists
/// msg is any additional message to print before presenting the pick list
/// Returns Ok<Some<list_id, list_name>> on selection, or Ok<None>
/// if the user cancels
pub fn user_select_list(client: &CheckvistClient, msg: &str) -> Result<Option<(u32, String)>> {
    let lists = get_lists(client)?;
    println!("{}", msg);
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
            .append("'", Style::Normal)
            .println()
            .expect("Problem printing colour output");

        list
    })
}

pub fn offer_to_save_new_default_list(config: &Config, path: &PathBuf) -> Result<()> {
    // TODO: confirmation question should follow standard list name colour scheme
    // Don't think ColourOutput can do this, so would need to use term escapes.
    if !Confirm::new()
        .with_prompt(format!(
            "Do you want to save '{}' as your new default list?",
            &config.list_name
        ))
        .interact()?
    {
        println!("Not changing your default list");
        return Ok(());
    }
    config.save(path)?;

    ColourOutput::new(StreamKind::Stdout)
        .append(config.list_name.clone(), Style::ListName)
        .append(" is now your default list", Style::Normal)
        .println()?;

    Ok(())
}
