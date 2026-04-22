with open('/Users/nikhil/ChronoSentiment_MEGA_FINAL/core/src/ga.rs', 'r') as f:
    opened = 0
    closed = 0
    for i, line in enumerate(f, 1):
        clean = line.split('//')[0]
        opened += clean.count('{')
        closed += clean.count('}')
        if closed > opened:
            print(f"Balance broken at line {i}: {{={opened}, }}={closed}")
            print(f"Line content: {line.strip()}")
            break
    else:
        print(f"End of file reached. Totals: {{={opened}, }}={closed}")
