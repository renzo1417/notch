use serde::Deserialize;

pub(crate) fn parse_result(bytes: &[u8], code: u32) -> Result<Vec<String>, String> {
    #[derive(Deserialize)]
    struct File {
        state: String,
        outputs: Vec<String>,
        error: Option<String>,
        #[serde(rename = "needsEngines", default)]
        needs_engines: Vec<String>,
    }
    #[derive(Deserialize)]
    struct ResultDocument {
        converted: u32,
        files: Vec<File>,
    }
    let document: ResultDocument = serde_json::from_slice(bytes).map_err(|_| {
        "Filee stopped without a valid result. Check its engines or try a smaller file.".to_string()
    })?;
    if let Some(file) = document.files.first() {
        if code == 0
            && document.converted == 1
            && document.files.len() == 1
            && file.state == "done"
            && !file.outputs.is_empty()
        {
            return Ok(file.outputs.clone());
        }
        if !file.needs_engines.is_empty() {
            return Err(format!(
                "Install the required engines in Filee: {}. Then choose Check again.",
                file.needs_engines.join(", ")
            ));
        }
        if let Some(error) = &file.error {
            return Err(error.chars().take(300).collect());
        }
    }
    Err("Filee did not convert this file. Check its engines and try again.".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn success_requires_a_matching_exit_code_and_real_output_list() {
        let done = br#"{"converted":1,"files":[{"state":"done","outputs":["result.png"]}]}"#;
        assert_eq!(parse_result(done, 0).unwrap(), ["result.png"]);
        assert!(parse_result(done, 1).is_err());
        assert!(parse_result(br#"{"converted":0,"files":[]}"#, 0).is_err());
        assert!(parse_result(b"garbage", 0).is_err());
        let missing = br#"{"converted":0,"files":[{"state":"failed","outputs":[],"needsEngines":["libreoffice"]}]}"#;
        assert!(parse_result(missing, 1)
            .unwrap_err()
            .contains("libreoffice"));
    }
}
