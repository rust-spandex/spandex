# todo

## single level unordered list

Storing a paragraph inside each list item is probably easier, and the right thing to do

add to pub enum Ast in parser/ast.rs
 [x] Add UnorderedList<ListItem>
 [x] and UnorderedListItem<Vec<AST::paragraph>>
 [x] add to impl children for new enums
 [x] add to print_debug
 [ ] (not sure needs doing) add to fmt::Display

Parse
 - add parse_unordered_list, similar to parse_title
   - first char should be a dash for parsing
   - if other lines start with a dash, then they become new list items
   - if other lines don't start with a dash, then they become part of the previous item
   - don't worry about nested lists for now
   - parse_single_line already exists, parse_title is a good template
 - add parse_unordered_list to parse_block_content

Print
- also want to do the typesetting stuff, but maybe that is for part 2
