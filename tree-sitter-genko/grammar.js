/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: "genko",

  extras: () => [],

  rules: {
    source_file: ($) =>
      seq(
        repeat(
          choice(
            seq(choice($.heading, $.paragraph), $._newline),
            $._newline,
          ),
        ),
        optional(choice($.heading, $.paragraph)),
      ),

    heading: ($) =>
      prec(
        2,
        seq(
          field("marker", $.heading_marker),
          optional($._horizontal_space),
          optional(field("title", $.heading_text)),
        ),
      ),

    heading_marker: () => token(/#{1,6}/),

    heading_text: () => token(/[^ \t\r\n][^\r\n]*/),

    paragraph: ($) => repeat1($._inline),

    _inline: ($) =>
      choice(
        $.primary_dialogue,
        $.secondary_dialogue,
        $._common_markup,
        $.plain_text,
      ),

    _common_markup: ($) =>
      choice(
        $.explicit_ruby,
        $.kakuyomu_emphasis,
        $.aozora_annotation,
        $.html_comment,
      ),

    primary_dialogue: ($) =>
      seq("「", optional($.primary_dialogue_content), "」"),

    primary_dialogue_content: ($) =>
      repeat1(
        choice(
          $.secondary_dialogue,
          $._common_markup,
          $.primary_dialogue_text,
        ),
      ),

    primary_dialogue_text: () =>
      token(
        choice(
          /[^『｜《［<」\r\n]+/,
          /[『｜《［<]/,
        ),
      ),

    secondary_dialogue: ($) =>
      seq("『", optional($.secondary_dialogue_content), "』"),

    secondary_dialogue_content: ($) =>
      repeat1(
        choice(
          $._common_markup,
          $.secondary_dialogue_text,
        ),
      ),

    secondary_dialogue_text: () =>
      token(
        choice(
          /[^｜《［<』\r\n]+/,
          /[｜《［<]/,
        ),
      ),

    explicit_ruby: ($) =>
      seq(
        "｜",
        field("base", $.ruby_base),
        "《",
        optional(field("reading", $.ruby_reading)),
        "》",
      ),

    ruby_base: () => token(/[^《》\r\n]+/),

    ruby_reading: () => token(/[^《》\r\n]+/),

    kakuyomu_emphasis: ($) =>
      seq("《《", field("text", $.emphasis_text), "》》"),

    emphasis_text: () => token(/[^《》\r\n]+/),

    aozora_annotation: ($) =>
      seq("［＃", optional(field("text", $.annotation_text)), "］"),

    annotation_text: () => token(/[^］\r\n]+/),

    html_comment: () =>
      token(seq("<!--", repeat(choice(/[^-]/, /-[^-]/)), "-->")),

    plain_text: () =>
      token(
        choice(
          /[^#「『｜《［<\r\n]+/,
          /[#「『｜《［<]/,
        ),
      ),

    _horizontal_space: () => token(/[ \t]+/),

    _newline: () => token(/\r?\n/),
  },
});
