#![warn(missing_debug_implementations, missing_copy_implementations)]
//! Provides a html downloader as well as an xpah based extractor

pub mod html_loader;
use libxml::{parser::Parser, xpath::Context};

#[derive(Debug, Clone, Copy)]
pub enum FletcherError {
    /// The site could not be fetched within the specified limit
    FailedTooOften,
    /// The request limit of the site has been reached while trying to fetch
    /// the page. Waiting and trying again, or outright stopping are sensable
    RateLimitReached,
    /// We are not allowed to load the requested page. Either the ressource is
    /// actually restricted, or we are ip blocked/timed out
    Forbidden,
    /// The site could not be found
    NotFound,
    /// The response from the server was invalid
    InvalidResponse,
    /// The HTML body was received, but could not be parsed into a DOM
    DomParsingError,
}

/// Downloads the requested site and applies all xpaths on it. The results of
/// each XPath will be in a seperate Vec
pub fn fetch_extract<const COUNT: usize>(
    page: &str,
    xpaths: &[&str; COUNT],
) -> Result<[Vec<String>; COUNT], FletcherError> {
    let page = html_loader::load_page(page)?;

    let doc = match Parser::default_html().parse_string(page) {
        Ok(d) => d,
        Err(_) => return Err(FletcherError::DomParsingError),
    };
    let context = match Context::new(&doc) {
        Ok(c) => c,
        Err(_) => return Err(FletcherError::DomParsingError),
    };

    let results: [Vec<String>; COUNT] = xpaths
        .iter()
        .map(|xp| {
            let root = if xp.starts_with('/') { "" } else { "/" };
            let xp = &(root.to_string() + xp);
            match context.evaluate(xp) {
                Ok(res) => {
                    let nodes = res.get_nodes_as_vec();
                    let texts: Vec<_> =
                        nodes.into_iter().map(|n| n.get_content()).collect();
                    texts
                }
                Err(_) => Vec::new(),
            }
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
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
        assert!(res[0][0] == "Example Domain");
    }
}
