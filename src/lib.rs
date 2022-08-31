#![warn(missing_debug_implementations, missing_copy_implementations)]
//! Provides a multithreaded html downloader, as well as a regex based crawler

pub mod html_loader;

use libxml::{parser::Parser, xpath::Context};
use std::error::Error;

pub fn fetch_extract(
    page: &str,
    xpaths: &[&str],
) -> Result<Vec<Option<Vec<String>>>, Box<dyn Error>> {
    let page = html_loader::load_page(page)?;

    let doc = Parser::default_html().parse_string(page)?;
    let context = Context::new(&doc).unwrap();

    let mut results = Vec::new();
    for xp in xpaths {
        let root = if xp.starts_with('/') { "" } else { "/" };
        let xp = &(root.to_string() + xp);
        match context.evaluate(xp) {
            Ok(res) => {
                let nodes = res.get_nodes_as_vec();
                let texts: Vec<_> =
                    nodes.into_iter().map(|n| n.get_content()).collect();
                results.push(Some(texts))
            }
            Err(_) => results.push(None),
        }
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    #[test]
    fn basic() {
        let res = crate::fetch_extract(
            "https://www.example.com",
            &["descendant::h1"],
        )
        .unwrap();
        assert!(
            res.first().unwrap().as_ref().unwrap().first().unwrap()
                == "Example Domain"
        );
    }
}
