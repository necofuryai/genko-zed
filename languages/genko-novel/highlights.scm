(heading_marker) @punctuation.special
(heading_text) @title

(primary_dialogue
  "「" @punctuation.bracket
  "」" @punctuation.bracket)
(primary_dialogue_text) @string

(secondary_dialogue
  "『" @punctuation.bracket
  "』" @punctuation.bracket)
(secondary_dialogue_text) @string.special

(explicit_ruby
  "｜" @punctuation.special
  base: (ruby_base) @string
  "《" @punctuation.bracket
  reading: (ruby_reading) @string.special
  "》" @punctuation.bracket)

(kakuyomu_emphasis
  "《《" @punctuation.special
  text: (emphasis_text) @emphasis
  "》》" @punctuation.special)

(aozora_annotation) @comment
(html_comment) @comment
