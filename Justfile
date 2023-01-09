gen-smoke:
  PATH=$PATH:~/ins/j903 cargo run -q --example gen-runlist tests/smoke.ijs tests/smoke.toml
  PATH=$PATH:~/ins/j903 cargo run -q --example gen-runlist tests/snippets/ tests/snippets.toml

regen: clean-smoke gen-smoke

clean-smoke:
  rm -f tests/smoke.toml tests/snippets.toml

gen-impl-status:
  echo "# Implementation Status" > STATUS.md
  # echo "This file auto-generated: just get-impl-status\n" >> STATUS.md

  # echo "\n## Implemented Verbs" >> STATUS.md
  # grep '=> primitive(' src/lib.rs >> STATUS.md

  # echo "\n## Implemented Adverbs" >> STATUS.md
  # grep -e 'adverb(' src/lib.rs | grep -v 'not_impl' >> STATUS.md

  # echo "\n## Implemented Conjunctions" >> STATUS.md
  # grep -e 'conj(' src/lib.rs | grep -v 'not_impl' >> STATUS.md
  # grep -e 'WordyConjunction' src/lib.rs | grep -v 'not_impl' >> STATUS.md

  echo "\n## Not Implemented Yet" >> STATUS.md
  grep 'not_impl(' src/lib.rs >> STATUS.md
  grep 'a_not_implemented' src/lib.rs >> STATUS.md
  grep 'c_not_implemented' src/lib.rs >> STATUS.md

  echo "\n\`\`\`" >> STATUS.md
  grep -h -R -C 3 NonceError src/verbs/*.rs | grep 'pub fn' | grep -v 'Vocabulary.html' >> STATUS.md
  echo "\`\`\`" >> STATUS.md
