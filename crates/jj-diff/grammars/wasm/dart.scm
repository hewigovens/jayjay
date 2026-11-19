; highlights.scm - Dart syntax highlighting queries

; ============================================================================
; Comments
; ============================================================================

(comment) @comment
(block_comment) @comment.block
(documentation_comment) @comment.documentation
(documentation_block_comment) @comment.block.documentation

; ============================================================================
; Literals
; ============================================================================

(decimal_integer_literal) @number
(hex_integer_literal) @number
(decimal_floating_point_literal) @number.float

(true) @boolean
(false) @boolean
(null_literal) @constant.builtin

(string_literal) @string
(template_chars_single) @string
(template_chars_double) @string
(template_chars_single_single) @string
(template_chars_double_single) @string
(template_chars_raw_slash) @string
(escape_sequence) @string.escape

(symbol_literal) @string.special.symbol

; ============================================================================
; Types
; ============================================================================

(type_identifier) @type
(void_type) @type.builtin

(type_parameter
  (type_identifier) @type.parameter)

"Function" @type.builtin

; Built-in types
((type_identifier) @type.builtin
  (#match? @type.builtin "^(int|double|num|String|bool|List|Set|Map|Runes|Symbol|Future|Stream|Iterable|Never|dynamic|Object)$"))

; ============================================================================
; Functions
; ============================================================================

; Function/getter/setter names (matches at any nesting level)
(function_signature
  name: (identifier) @function)

(getter_signature
  name: (identifier) @function)

(setter_signature
  name: (identifier) @function)

; Methods (inside class/mixin/extension bodies)
(method_signature
  (function_signature
    name: (identifier) @function.method))

(method_signature
  (getter_signature
    name: (identifier) @function.method))

(method_signature
  (setter_signature
    name: (identifier) @function.method))

; Operator methods
(operator_signature
  "operator" @keyword)

; Property access
[
  (member_expression)
  (null_aware_member_expression)
  (cascade_member_expression)
  (cascade_null_aware_member_expression)
] property: (identifier) @property

(static_member_shorthand
  (identifier) @property)

(record_field
  (label
    (identifier) @property))

(object_pattern
  (label
    (identifier) @property))

(record_pattern
  (label
    (identifier) @property))

; Function calls (non-method)
(call_expression
  function: (identifier) @function.call)

; Method calls
(call_expression
  function: [
    (member_expression property: (identifier) @function.method.call)
    (null_aware_member_expression property: (identifier) @function.method.call)
  ])

(cascade_call_expression
  property: (identifier) @function.method.call)

(call_expression
  function: (static_member_shorthand
    (identifier) @function.method.call))

; ============================================================================
; Declarations
; ============================================================================

(class_declaration
  name: (identifier) @type)

(mixin_declaration
  (identifier) @type)

(extension_declaration
  name: (identifier) @type)

(extension_type_declaration
  name: (extension_type_name
    (identifier) @type))

(enum_declaration
  name: (identifier) @type)

(enum_entry
  name: (identifier) @constant)

(type_alias
  (type_identifier) @type.definition)

; Constructors
(constructor_signature
  name: (identifier) @constructor)

(constant_constructor_signature
  name: (identifier) @constructor)

(factory_constructor_signature
  name: (identifier) @constructor)

; name:-qualified so the redirect target (target_constructor:) keeps its
; own capture instead of @constructor.
(redirecting_factory_constructor_signature
  name: (identifier) @constructor)

(primary_constructor
  name: (identifier) @constructor)

(extension_type_representation
  name: (identifier) @property)

; ============================================================================
; Parameters
; ============================================================================

(formal_parameter
  (identifier) @variable.parameter)

(field_formal_parameter
  (identifier) @variable.parameter)

(super_formal_parameter
  (identifier) @variable.parameter)

; Named arguments
(named_argument
  (label
    (identifier) @variable.parameter))

; ============================================================================
; Variables
; ============================================================================

(initialized_variable_declaration
  name: (identifier) @variable)

(initialized_identifier
  (identifier) @variable)

(static_final_declaration
  (identifier) @variable)

(for_statement
  name: (identifier) @variable)

(for_element
  name: (identifier) @variable)

(variable_pattern
  name: (identifier) @variable)

(record_type_field
  name: (identifier) @variable)

(record_type_named_field
  (typed_identifier
    name: (identifier) @variable))

; ============================================================================
; Annotations
; ============================================================================

(annotation
  "@" @attribute
  name: (identifier) @attribute)

(annotation
  "@" @attribute
  name: (qualified_name
    (type_identifier) @attribute
    (identifier) @attribute))

(annotation_open_paren) @punctuation.bracket

; ============================================================================
; Operators
; ============================================================================

(relational_operator) @operator
(prefix_operator) @operator
(negation_operator) @operator
(is_operator) @keyword.operator
(type_cast "as" @keyword.operator)
(cast_pattern "as" @keyword.operator)
(import_specification "as" @keyword)
(binary_operator) @operator
(additive_operator) @operator
(multiplicative_operator) @operator
(shift_operator) @operator
(equality_operator) @operator
(bitwise_operator) @operator
(increment_operator) @operator

[
  "="
  "+="
  "-="
  "*="
  "/="
  "%="
  "~/="
  "<<="
  ">>="
  ">>>="
  "&="
  "^="
  "|="
  "??="
  "=="
  "+"
  "-"
  "*"
  "/"
  "%"
  "~/"
  "<<"
  ">>"
  ">>>"
  "&"
  "^"
  "|"
  "&&"
  "||"
  "??"
  "!"
  "~"
  ".."
  "?.."
  "..."
  "...?"
  "=>"
] @operator

; ============================================================================
; Punctuation
; ============================================================================

["(" ")" "[" "]" "{" "}"] @punctuation.bracket

; Interpolation delimiters override generic brackets (last-wins)
(template_substitution "$" @punctuation.special)
(template_substitution "{" @punctuation.special)
(template_substitution "}" @punctuation.special)
(template_substitution
  (identifier_dollar_escaped) @variable)
(template_substitution
  (identifier) @variable)

(type_arguments
  "<" @punctuation.bracket
  ">" @punctuation.bracket)

(type_parameters
  "<" @punctuation.bracket
  ">" @punctuation.bracket)

["," ";" ":" "." "?." "?."] @punctuation.delimiter

"#" @punctuation.special

; ============================================================================
; Keywords
; ============================================================================

[
  "abstract"
  "assert"
  "async"
  "async*"
  "augment"
  "base"
  "break"
  "case"
  "catch"
  "class"
  "const"
  "continue"
  "covariant"
  "default"
  "deferred"
  "do"
  "else"
  "enum"
  "export"
  "extends"
  "extension"
  "external"
  "factory"
  "final"
  "finally"
  "for"
  "get"
  "hide"
  "if"
  "implements"
  "import"
  "in"
  "inline"
  "interface"
  "late"
  "library"
  "mixin"
  "native"
  "new"
  "on"
  "operator"
  "part"
  "required"
  "rethrow"
  "return"
  "sealed"
  "set"
  "show"
  "static"
  "super"
  "switch"
  "sync*"
  "this"
  "throw"
  "try"
  "type"
  "typedef"
  "var"
  "when"
  "while"
  "with"
] @keyword

; await/yield are keywords only in their expression/statement forms; the
; bare tokens also appear as identifiers in sync contexts and must not be
; painted as keywords there.
(await_expression "await" @keyword)
(yield_statement "yield" @keyword)
(yield_each_statement "yield" @keyword)

; ============================================================================
; Special identifiers
; ============================================================================

((identifier) @variable.builtin
  (#eq? @variable.builtin "this"))

((identifier) @variable.builtin
  (#eq? @variable.builtin "super"))
