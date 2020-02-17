use crate::*;
use pest::Parser;
use pest_derive::*;
use std::collections::BTreeMap;

#[derive(Parser)]
#[grammar = "jq/grammar/jq.pest"]
struct JqParser;

impl Document {
    pub fn jq<'a>(&'a self, sel: &str) -> Result<&'a Document, String> {
        let selection = JqParser::parse(Rule::path, sel)
            .map_err(|e| e.to_string())?;
        let mut result = self;
        for selector in selection {
            match selector.as_rule() {
                Rule::identifier => result = &result[selector.as_str()],
                Rule::string => result = &result[selector.as_str()],
                Rule::number => {
                    let index = selector.as_str().parse::<usize>()
                        .map_err(|e| format!("Parse failure: {}!", e))?;
                    result = &result[index];
                },
                Rule::EOI => return Ok(result),
                _ => return Err(format!("Invalid selector {}", selector)),
            };
        }
        Ok(result)
    }

    pub fn jq_mut<'a>(&'a mut self, sel: &str) -> Result<&'a mut Document, String> {
        let selection = JqParser::parse(Rule::path, sel).map_err(|e| e.to_string())?;
        let mut result = self;
        for selector in selection {
            match selector.as_rule() {
                Rule::identifier => result = &mut result[selector.as_str()],
                Rule::string => result = &mut result[selector.as_str()],
                Rule::number => {
                    let index = selector.as_str().parse::<usize>()
                        .map_err(|e| format!("Parse failure: {}!", e))?;
                    result = &mut result[index];
                },
                Rule::EOI => return Ok(result),
                _ => return Err(format!("Invalid selector {}", selector)),
            };
        }
        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Serialize)]
    struct TestStruct {
        val: String,
        vals: Vec<usize>,
    }

    #[test]
    fn test_path() {
        let doc = Document::new(vec![1, 2, 3]).unwrap();
        assert_eq!(doc.jq(".[0]").unwrap().clone(), Document::I32(1));

        let doc = Document::new(TestStruct {
            val: "some_val".to_string(),
            vals: vec![1, 2, 3],
        }).unwrap();
        dbg!(doc.jq(".val").unwrap().clone());
        assert_eq!(doc.jq(".val").unwrap().clone(), "some_val".to_string());
    }
}