use crate::*;
use pest::Parser;
use pest_derive::*;
use pest::iterators::Pair;

#[derive(Parser)]
#[grammar = "jq/grammar/jq.pest"]
struct JqParser;

fn jq_find(doc: &Document, selection: &[Pair<Rule>]) -> Result<Vec<Document>, String> {
    let mut current = doc;
    let mut current_index = 0;
    for selector in selection {
        match selector.as_rule() {
            Rule::number => {
                let index = selector.as_str().parse::<usize>()
                    .map_err(|e| format!("Parse failure: {}!", e))?;
                current = &current[index];
            }
            Rule::string | Rule::identifier => {
                let index = selector.as_str();
                current = &current[index];
            }
            Rule::function_length => {
                match current {
                    Document::Seq(l) => {
                        return jq_find(&l.len().into(), &selection[current_index..])
                    }
                    _ => {}
                }
            }
            Rule::EOI => {
                return Ok(vec![current.clone()])
            }
            _ => return Err(format!("Invalid selector {}", selector)),
        }
        current_index = current_index + 1;
    }
    Ok(vec![current.clone()])
}

fn jq_find_all(doc_list: &[&Document], selection: &[Pair<Rule>]) -> Result<Vec<Document>, String> {
    let mut result : Vec<Document> = vec![];
    for doc in doc_list {
        let mut piece = jq_find(doc, selection)?;
        result.append(&mut piece);
    }
    Ok(result)
}

impl Document {
    pub fn jq(self: &Document, sel: &str) -> Result<Vec<Document>, String> {
        let selection = JqParser::parse(Rule::query, sel).map_err(|e| e.to_string())?;
        let selection_vec : Vec<_> = selection.map(|p| p).collect();
        let v : Vec<&Document> = vec![self];
        jq_find_all(&v, &selection_vec[..])
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
        assert_eq!(doc.jq(".[0]").unwrap(), vec![1]);

        let doc = Document::new(TestStruct {
            val: "some_val".to_string(),
            vals: vec![1, 2, 3],
            child: ChildStruct {
                val: "ccc".to_string(),
                vals: vec![4, 5, 6]
            }
        }).unwrap();
        assert_eq!(doc.jq(".val").unwrap().clone(), vec!["some_val".to_string()]);
        assert_eq!(doc.jq(".child.vals[0]").unwrap().clone(), vec![4 as u64]);
        assert_eq!(doc.jq(".child | .vals[0]").unwrap().clone(), vec![4 as u64]);
        assert_eq!(doc.jq(".child | .vals | .[0]").unwrap().clone(), vec![4 as u64]);
        assert_eq!(doc.jq(".child | .vals | length").unwrap().clone(), vec![3 as u64]);
    }
}