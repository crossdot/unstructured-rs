use crate::*;
use pest::Parser;
use pest_derive::*;

#[derive(Parser)]
#[grammar = "jq/grammar/jq.pest"]
struct JqParser;

impl Document {
    pub fn jq(self: &Document, sel: &str) -> Result<Document, String> {
        let mut current = self.clone();

        let selection = JqParser::parse(Rule::query, sel).map_err(|e| e.to_string())?;
        for selector in selection {
            match selector.as_rule() {
                Rule::number => {
                    let index = selector.as_str().parse::<usize>()
                        .map_err(|e| format!("Parse failure: {}!", e))?;
                    current = current[index].clone();
                }
                Rule::string | Rule::identifier => {
                    let index = selector.as_str();
                    current = current[index].clone();
                }
                Rule::function_length => {
                    match current {
                        Document::Seq(l) => {
                            current = l.len().into();
                        }
                        _ => {}
                    }
                }
                Rule::EOI => {
                    return Ok(current)
                }
                _ => return Err(format!("Invalid selector {}", selector)),
            }
        }
        Ok(current)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Serialize)]
    struct TestStruct {
        val: String,
        vals: Vec<usize>,
        child: ChildStruct,
    }

    #[derive(Serialize)]
    struct ChildStruct {
        val: String,
        vals: Vec<usize>,
    }

    #[test]
    fn test_path() {
        let doc = Document::new(vec![1, 2, 3]).unwrap();
        assert_eq!(doc.jq(".[0]").unwrap(), 1);

        let doc = Document::new(TestStruct {
            val: "some_val".to_string(),
            vals: vec![1, 2, 3],
            child: ChildStruct {
                val: "ccc".to_string(),
                vals: vec![4, 5, 6]
            }
        }).unwrap();
        assert_eq!(doc.jq(".val").unwrap().clone(), "some_val".to_string());
        assert_eq!(doc.jq(".child.vals[0]").unwrap().clone(), 4 as u64);
        assert_eq!(doc.jq(".child | .vals[0]").unwrap().clone(), 4 as u64);
        assert_eq!(doc.jq(".child | .vals | .[0]").unwrap().clone(), 4 as u64);
        assert_eq!(doc.jq(".child | .vals | length").unwrap().clone(), 3 as u64);
    }
}