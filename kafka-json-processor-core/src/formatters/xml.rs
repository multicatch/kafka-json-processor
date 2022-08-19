use crate::formatters::PREPARED_INDENTS;

#[derive(Eq, PartialEq, Clone)]
enum XmlSymbol {
    ElementStart,
    SameLevelTag,
    ElementEnd,
    TagEnd,
    NotATag,
    Whitespace,
}

/// Naive implementation of pretty XML. Not so accurate, but readable and fast.
pub fn pretty_xml(source: String) -> String {
    if !source.contains("</") {
        return source;
    }

    let capacity = source.len() as f64 * 2f64;
    let capacity = capacity.ceil() as usize;

    let source_bytes = source.as_bytes();
    let mut result = Vec::with_capacity(capacity);

    let mut next_indent: usize = 0;
    let mut last_symbol = XmlSymbol::NotATag;
    let mut source_rewrite_pos: usize = 0;
    let mut xml_started = false;
    let mut last_character = None;

    for (i, current_char) in source_bytes.iter().enumerate() {
        let symbol = detect_last_xml_symbol(last_character, *current_char);
        last_character = Some(current_char);
        if symbol == XmlSymbol::Whitespace {
            continue;
        }

        if symbol == XmlSymbol::ElementStart {
            if xml_started {
                next_indent += 1;
            }
            xml_started = true;
        }

        if should_append_indent(&symbol, &last_symbol) {
            result.extend_from_slice(&source_bytes[source_rewrite_pos..i-1]);
            result.extend_from_slice(PREPARED_INDENTS.get(next_indent)
                .unwrap_or_else(|| PREPARED_INDENTS.last().unwrap())
            );
            source_rewrite_pos = i - 1;
        }

        if symbol == XmlSymbol::ElementEnd && next_indent > 0 {
            next_indent -= 1;
        }

        last_symbol = symbol;
    }

    result.extend_from_slice(&source_bytes[source_rewrite_pos..]);
    String::from_utf8(result).unwrap()
}

/// Detect xml symbol based on most common character collocations in XML.
fn detect_last_xml_symbol(last_character: Option<&u8>, current_char: u8) -> XmlSymbol {
    match last_character {
        Some(&b'<') =>
            match current_char {
                b'/' =>
                    XmlSymbol::ElementEnd,
                b'!' | b'?' =>
                    XmlSymbol::SameLevelTag,
                b' ' =>
                    XmlSymbol::NotATag,
                _ =>
                    XmlSymbol::ElementStart
            },
        Some(&b'>') =>
            XmlSymbol::TagEnd,
        Some(&b' ') | Some(&b'\t') =>
            XmlSymbol::Whitespace,
        _ =>
            XmlSymbol::NotATag
    }
}

fn should_append_indent(symbol: &XmlSymbol, last_symbol: &XmlSymbol) -> bool {
    symbol != &XmlSymbol::NotATag && symbol != &XmlSymbol::TagEnd
        && last_symbol != &XmlSymbol::NotATag
}

#[cfg(test)]
mod tests {
    use crate::formatters::xml::pretty_xml;

    #[test]
    fn should_format_xml_as_pretty() {
        let original = r##"[INFO] This is a sample log message. Body: <?xml version="1.0" encoding="UTF-8"?><breakfast_menu><!-- comment --><!-- comment after comment --><food>  <name>Belgian Waffles</name><!-- comment 2 -->    <price>$5.95</price><description>Two of our famous Belgian Waffles with plenty of real maple syrup</description><calories>650</calories></food><food><name>Strawberry Belgian Waffles</name><price>$7.95</price><description>Light Belgian waffles covered with strawberries and whipped cream</description><calories>900</calories></food><food><name>Berry-Berry Belgian Waffles</name><price>$8.95</price><description>Light Belgian waffles covered with an assortment of fresh berries and whipped cream</description><calories>900</calories></food><food><name>French Toast</name><price>$4.50</price><description>Thick slices made from our homemade sourdough bread</description><calories>600</calories></food><food><name>Homestyle Breakfast</name><price>$6.95</price><description>Two eggs, bacon or sausage, toast, and our ever-popular hash browns</description><calories>950</calories></food></breakfast_menu>"##;

        let expected = format!(r##"[INFO] This is a sample log message. Body: <?xml version="1.0" encoding="UTF-8"?>
<breakfast_menu>
<!-- comment -->
<!-- comment after comment -->
  <food>{}
    <name>Belgian Waffles</name>
  <!-- comment 2 -->{}
    <price>$5.95</price>
    <description>Two of our famous Belgian Waffles with plenty of real maple syrup</description>
    <calories>650</calories>
  </food>
  <food>
    <name>Strawberry Belgian Waffles</name>
    <price>$7.95</price>
    <description>Light Belgian waffles covered with strawberries and whipped cream</description>
    <calories>900</calories>
  </food>
  <food>
    <name>Berry-Berry Belgian Waffles</name>
    <price>$8.95</price>
    <description>Light Belgian waffles covered with an assortment of fresh berries and whipped cream</description>
    <calories>900</calories>
  </food>
  <food>
    <name>French Toast</name>
    <price>$4.50</price>
    <description>Thick slices made from our homemade sourdough bread</description>
    <calories>600</calories>
  </food>
  <food>
    <name>Homestyle Breakfast</name>
    <price>$6.95</price>
    <description>Two eggs, bacon or sausage, toast, and our ever-popular hash browns</description>
    <calories>950</calories>
  </food>
</breakfast_menu>"##, "  ", "    ");

        assert_eq!(pretty_xml(original.to_string()), expected);
    }

    #[test]
    fn should_format_xml_partially_formatted_xml() {
        let original = r##"[INFO] This is a sample log message. Body: <?xml version="1.0" encoding="UTF-8"?>
<breakfast_menu>
<!-- multiline
comment -->
  <food>
    <name>Belgian Waffles</name>
    <price>$5.95</price><description>Two of our famous Belgian Waffles with plenty of real maple syrup</description><calories>650</calories></food></breakfast_menu>"##;

        let expected = r##"[INFO] This is a sample log message. Body: <?xml version="1.0" encoding="UTF-8"?>
<breakfast_menu>
<!-- multiline
comment -->
  <food>
    <name>Belgian Waffles</name>
    <price>$5.95</price>
    <description>Two of our famous Belgian Waffles with plenty of real maple syrup</description>
    <calories>650</calories>
  </food>
</breakfast_menu>"##;

        assert_eq!(pretty_xml(original.to_string()), expected);
    }

    #[test]
    fn should_leave_pretty_xml_intact() {
        let original = r##"[INFO] This is a sample log message. Body: <?xml version="1.0" encoding="UTF-8"?>
<breakfast_menu>
<!-- comment -->
  <food>
    <name>Belgian Waffles</name>
  <!-- comment 2 -->
    <price>$5.95</price>
    <description>Two of our famous Belgian Waffles with plenty of real maple syrup</description>
    <calories>650</calories>
  </food>
  <food>
    <name>Strawberry Belgian Waffles</name>
    <price>$7.95</price>
    <description>Light Belgian waffles covered with strawberries and whipped cream</description>
    <calories>900</calories>
  </food>
  <food>
    <name>Berry-Berry Belgian Waffles</name>
    <price>$8.95</price>
    <description>Light Belgian waffles covered with an assortment of fresh berries and whipped cream</description>
    <calories>900</calories>
  </food>
  <food>
    <name>French Toast</name>
    <price>$4.50</price>
    <description>Thick slices made from our homemade sourdough bread</description>
    <calories>600</calories>
  </food>
  <food>
    <name>Homestyle Breakfast</name>
    <price>$6.95</price>
    <description>Two eggs, bacon or sausage, toast, and our ever-popular hash browns</description>
    <calories>950</calories>
  </food>
</breakfast_menu>"##;

        assert_eq!(pretty_xml(original.to_string()), original);
    }
}