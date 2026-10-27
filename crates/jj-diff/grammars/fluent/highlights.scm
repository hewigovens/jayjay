(comment_block) @comment

(file_comment) @comment.file

(group_comment) @comment.group

(doc_comment_block) @comment.documentation

(number_literal) @number

(string_literal) @string

(escaped_literal) @string.escape

(message
  id: (identifier) @tag)

(message_reference
  id: (identifier) @tag)

(message_reference
  attribute: (identifier) @tag.attribute)

(message
  (attributes
    (attribute
      id: (identifier) @tag.attribute)))

(term_reference
  attribute: (identifier) @function.macro)

(term_identifier) @function.macro

(term
  (attributes
    (attribute
      id: (identifier) @function.macro)))

(variable) @variable.parameter

(selector_variant
  key: (identifier) @property)

(default_variant
  key: (identifier) @property)

(default_variant
  "*" @punctuation.special)

(function_reference
  id: (function_name) @function)

((function_reference
  id: (function_name) @function.builtin)
  (#match? @function.builtin "^(NUMBER|DATETIME)$"))

(named_argument
  id: (identifier) @variable.parameter)

[
  "["
  "]"
  "("
  ")"
  "{"
  "}"
] @punctuation.bracket

(placeable
  "{" @punctuation.special
  "}" @punctuation.special)

[
  "="
  "->"
  ":"
] @operator

[
  "."
  ","
] @punctuation.delimiter
