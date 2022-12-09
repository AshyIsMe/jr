gen-smoke:
  PATH=$PATH:~/ins/j903 cargo run -q --example gen-runlist tests/smoke.ijs tests/smoke.toml
  PATH=$PATH:~/ins/j903 cargo run -q --example gen-runlist tests/snippets/ tests/snippets.toml
