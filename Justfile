gen-smoke:
  rm tests/smoke.toml
  <tests/smoke.ijs PATH=$PATH:~/ins/j903 cargo run -q --example gen-runlist tests/smoke.toml