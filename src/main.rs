use graphql_parser::query::{parse_query, Definition, Selection, OperationDefinition};
use regex::Regex;
use std::env;
use std::fs;
use walkdir::WalkDir;

fn extract_graphql_queries(content: &str) -> Vec<String> {
    let re = Regex::new(r"(?:graphql\s*\(|gql\s*)`([\s\S]*?)`").unwrap();
    re.captures_iter(content)
        .map(|cap| cap[1].to_string())
        .collect()
}

fn field_path_exists<'a>(
    selection: &Selection<'a, &'a str>,
    path: &[&str],
) -> bool {
    match selection {
        Selection::Field(field) => {
            if field.name == path[0] {
                if path.len() == 1 {
                    true
                } else {
                    field
                        .selection_set
                        .items
                        .iter()
                        .any(|sel| field_path_exists(sel, &path[1..]))
                }
            } else {
                false
            }
        }
        Selection::InlineFragment(fragment) => fragment
            .selection_set
            .items
            .iter()
            .any(|sel| field_path_exists(sel, path)),
        Selection::FragmentSpread(_) => false, // Ignoring fragment spreads for simplicity
    }
}

fn query_contains_path<'a>(entry: &walkdir::DirEntry, query: &'a str, path: &[&str], verbose: bool) -> bool {
    let ast = match parse_query::<&'a str>(query) {
        Ok(ast) => ast,
        Err(e) => {
            if verbose {
                // Optionally log the error
                eprintln!("Failed to parse query: file={}, error={}", entry.path().display(), e);
            }
            return false; // Skip this query
        }
    };

    for def in ast.definitions {
        if let Definition::Operation(op) = def {
            match op {
                OperationDefinition::SelectionSet(selection_set) => {
                    if selection_set
                        .items
                        .iter()
                        .any(|sel| field_path_exists(sel, path))
                    {
                        return true;
                    }
                }
                OperationDefinition::Query(query) => {
                    if query
                        .selection_set
                        .items
                        .iter()
                        .any(|sel| field_path_exists(sel, path))
                    {
                        return true;
                    }
                }
                OperationDefinition::Mutation(mutation) => {
                    if mutation
                        .selection_set
                        .items
                        .iter()
                        .any(|sel| field_path_exists(sel, path))
                    {
                        return true;
                    }
                }
                OperationDefinition::Subscription(subscription) => {
                    if subscription
                        .selection_set
                        .items
                        .iter()
                        .any(|sel| field_path_exists(sel, path))
                    {
                        return true;
                    }
                }
            }
        } else if let Definition::Fragment(fragment) = def {
            if fragment
                .selection_set
                .items
                .iter()
                .any(|sel| field_path_exists(sel, path))
            {
                return true;
            }
        }
    }
    false
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: graphql-search <field.path.to.search> [--verbose]");
        std::process::exit(1);
    }

    let verbose = args.len() > 2 && args.contains(&"--verbose".to_string());
    let path_to_search: Vec<&str> = args[1].split('.').collect();

    let mut matching_files = Vec::new();

    for entry in WalkDir::new(".")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
    {
        if let Ok(content) = fs::read_to_string(entry.path()) {
            let queries = extract_graphql_queries(&content);
            for query in queries {
                if query_contains_path(&entry, &query, &path_to_search, verbose) {
                    matching_files.push(entry.path().display().to_string());
                    break; // No need to check other queries in this file
                }
            }
        }
    }

    for file in matching_files {
        println!("{}", file);
    }
}