with open("core/src/ga.rs", "r") as f:
    text = f.read()

text = text.replace("let _evals = vec![make(0, 0.04), make(1, 0.01), make(2, 0.03), make(3, 0.02)];", "let evals = vec![make(0, 0.04), make(1, 0.01), make(2, 0.03), make(3, 0.02)];")

with open("core/src/ga.rs", "w") as f:
    f.write(text)

