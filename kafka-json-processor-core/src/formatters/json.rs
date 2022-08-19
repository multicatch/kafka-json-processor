use crate::formatters::PREPARED_INDENTS;

/// Naive implementation of pretty JSON. Not so accurate, but readable and fast.
pub fn pretty_json(source: String) -> String {
    let capacity = source.len() as f64 * 2f64;
    let capacity = capacity.ceil() as usize;

    let source_bytes = source.as_bytes();
    let mut result = Vec::with_capacity(capacity);

    let mut next_indent: usize = 0;
    let mut source_rewrite_pos: usize = 0;
    let mut string_started = false;
    let mut whitespace_started = false;
    let mut last_character = None;
    let mut json_started = false;

    for (i, current_char) in source_bytes.iter().enumerate() {
        let symbol = detect_json_symbol(last_character, *current_char);
        last_character = Some(current_char);

        if !json_started && symbol != JsonSymbol::ObjectOrArrayStart {
            continue;
        }

        json_started = true;

        if symbol == JsonSymbol::Whitespace {
            if !string_started {
                if !whitespace_started {
                    whitespace_started = true;
                    result.extend_from_slice(&source_bytes[source_rewrite_pos..i]);
                }
                source_rewrite_pos = i + 1;
            }
            continue;
        }

        if string_started && whitespace_started {
            // This whitespace was inside a string, so we should leave it as is.
            whitespace_started = false;
        }

        if symbol == JsonSymbol::StringBoundary {
            string_started = !string_started;
        }

        if !string_started {
            if symbol == JsonSymbol::ObjectOrArrayStart {
                next_indent += 1;
            } else if symbol == JsonSymbol::ObjectOrArrayEnd && next_indent > 0 {
                next_indent -= 1;
            }

            if symbol != JsonSymbol::NotAJsonSymbol {
                result.extend_from_slice(&source_bytes[source_rewrite_pos..i + 1]);
                source_rewrite_pos = i + 1;
            }

            if symbol == JsonSymbol::KeyValueSeparator {
                result.extend_from_slice(b" ");
            }

            if symbol == JsonSymbol::ObjectOrArrayStart || symbol == JsonSymbol::ItemSeparator || symbol == JsonSymbol::ObjectOrArrayEnd {
                // fix for closing brackets - they are already appended to result
                let closing_bracket = if symbol == JsonSymbol::ObjectOrArrayEnd {
                    Some(result.remove(result.len() - 1))
                } else {
                    None
                };

                result.extend_from_slice(PREPARED_INDENTS.get(next_indent)
                    .unwrap_or_else(|| PREPARED_INDENTS.last().unwrap())
                );

                if let Some(closing_bracket) = closing_bracket {
                    result.push(closing_bracket);
                }
            }
        }

        if symbol == JsonSymbol::ObjectOrArrayEnd && next_indent == 0 {
            json_started = false;
        }
    }

    result.extend_from_slice(&source_bytes[source_rewrite_pos..]);
    String::from_utf8(result).unwrap()
}

#[derive(Eq, PartialEq, Debug)]
enum JsonSymbol {
    ObjectOrArrayStart,
    ObjectOrArrayEnd,
    Whitespace,
    StringBoundary,
    KeyValueSeparator,
    ItemSeparator,
    EscapedCharacter,
    NotAJsonSymbol,
}

fn detect_json_symbol(last_char: Option<&u8>, current_char: u8) -> JsonSymbol {
    match last_char {
        Some(&b'\\') =>
            JsonSymbol::EscapedCharacter,

        _ => match current_char {
            b'{' | b'[' =>
                JsonSymbol::ObjectOrArrayStart,
            b'}' | b']' =>
                JsonSymbol::ObjectOrArrayEnd,
            b'"' =>
                JsonSymbol::StringBoundary,
            b':' =>
                JsonSymbol::KeyValueSeparator,
            b',' =>
                JsonSymbol::ItemSeparator,
            b' ' | b'\n' | b'\t' =>
                JsonSymbol::Whitespace,
            _ =>
                JsonSymbol::NotAJsonSymbol
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::formatters::json::pretty_json;

    #[test]
    fn should_format_json_as_pretty() {
        let original = r##"{"glossary":[{"title":"example glossary","GlossDiv":{"title":"S","numbers":[1,2,3],"available":true,"number":123.22,"GlossList":{"GlossEntry":{"ID":"SGML","SortAs":"SGML","GlossTerm":"Standard Generalized Markup Language","Acronym":"SGML","Abbrev":"ISO 8879:1986","GlossDef":{"para":"A meta-markup language, used to create markup languages such as \"DocBook\".","GlossSeeAlso":["GML","XML"]},"GlossSee":"markup"}}}}]}"##;

        let expected = r##"{
  "glossary": [
    {
      "title": "example glossary",
      "GlossDiv": {
        "title": "S",
        "numbers": [
          1,
          2,
          3
        ],
        "available": true,
        "number": 123.22,
        "GlossList": {
          "GlossEntry": {
            "ID": "SGML",
            "SortAs": "SGML",
            "GlossTerm": "Standard Generalized Markup Language",
            "Acronym": "SGML",
            "Abbrev": "ISO 8879:1986",
            "GlossDef": {
              "para": "A meta-markup language, used to create markup languages such as \"DocBook\".",
              "GlossSeeAlso": [
                "GML",
                "XML"
              ]
            },
            "GlossSee": "markup"
          }
        }
      }
    }
  ]
}"##;

        assert_eq!(pretty_json(original.to_string()), expected);
    }

    #[test]
    fn should_format_json_partially_formatted_json() {
        let original = r##"{"glossary":[{
        "title": "example glossary",
        "GlossDiv": {"title":"S","available":true,"number":123.22,"GlossList":{"GlossEntry":
        {
           "ID": "SGML",
           "SortAs": "SGML","GlossTerm":"Standard Generalized Markup Language","Acronym":"SGML","Abbrev":"ISO 8879:1986","GlossDef":{"para":"A meta-markup language, used to create markup languages such as \"DocBook\".","GlossSeeAlso":["GML","XML"]},"GlossSee":"markup"}}}}]}"##;

        let expected = r##"{
  "glossary": [
    {
      "title": "example glossary",
      "GlossDiv": {
        "title": "S",
        "available": true,
        "number": 123.22,
        "GlossList": {
          "GlossEntry": {
            "ID": "SGML",
            "SortAs": "SGML",
            "GlossTerm": "Standard Generalized Markup Language",
            "Acronym": "SGML",
            "Abbrev": "ISO 8879:1986",
            "GlossDef": {
              "para": "A meta-markup language, used to create markup languages such as \"DocBook\".",
              "GlossSeeAlso": [
                "GML",
                "XML"
              ]
            },
            "GlossSee": "markup"
          }
        }
      }
    }
  ]
}"##;

        assert_eq!(pretty_json(original.to_string()), expected);
    }
}